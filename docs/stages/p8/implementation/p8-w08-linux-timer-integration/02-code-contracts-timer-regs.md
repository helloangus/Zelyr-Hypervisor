# P8-W08 Code Contracts — Guest Time Source and Timer Registers

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W08 detailed design](README.md).  
**Modules covered:** M1 (time source), M2 (timer registers), M6 (telemetry
hooks). Expiry, WFI/wakeup, and preemption contracts are in
[03](03-code-contracts-expiry-wakeup.md).

All names are logical contract names (README decision D8); pseudocode is an
implementation outline, not runnable production code. Register semantics
follow the AArch64 architected timer (Arm ARM / IHI 0069 PPI positions); the
exposed class and all numeric values are machine-gated
([01 §2](01-architecture-and-state.md)).

## 1. Data contracts

```text
Name and stability: VmTimeFacts — internal; immutable machine-gated data
Purpose and caller: the single source of the Guest-visible time facts; read
  by M1 and validated against W04's DTB timer node at VM creation
Contents:
  frequency:  u32   — <CNTFRQ-VALUE>, frozen by the W02 gate
  offset:     u64   — per-VM virtual-counter offset, machine-gated
  timer_class: enum { VirtualTimer }  — the exposed Guest-programmable class
Preconditions: populated only from approved machine facts
Postconditions: immutable for the VM lifetime (monotonicity invariant I1
  depends on it); VM creation fails if absent
Errors: inconsistency with the DTB timer node facts (W04) fails the
  P8-V05-consistency review — fix is in the machine contract, not in code
Validation: DV01 review; W14 compatibility dimensions
```

```text
Name and stability: GuestTimerCtl — bitfield view of the CTL register (logical)
Fields per the architected timer: ENABLE, IMASK, ISTATUS; reserved bits must
  be written zero and read as zero
ISTATUS note: read-only for the Guest and reports the assertion condition
  (§3 of [02](#3-register-semantics-contracts)), not hardware-pin state
```

## 2. Time-source contracts (M1)

### 2.1 Virtual counter read

```text
Name and stability: guest_read_vcounter(vm) -> u64 — internal
Purpose and caller: the Guest's clocksource (CNTVCT-class access); called
  from the counter-access trap handler or presented untrapped per the W05/
  CNTHCTL policy — this contract fixes the value semantics either way
Inputs: VM (for the immutable offset)
Outputs: virtual counter = physical_counter − vm.facts.offset
Preconditions: facts installed at VM creation
Postconditions: satisfies I1 (monotonic) and I2 (cross-vCPU coherent) because
  the physical counter is monotonic and the offset is immutable; no state
  change
State/ownership change: none
Concurrency/allocation: none (pure read); no locks; no allocation
Errors: none
Security checks: the Guest can never read the physical counter here (the
  physical access path is W05-classified Hidden); Host uptime leaks only the
  offset-subtracted relation, by design
Logic (pseudocode):
  fn guest_read_vcounter(vm) -> u64:
      return p6_w05.read_physical_counter() - vm.time.facts.offset
      # wrapping arithmetic per Coding Guidelines; offset chosen at machine
      # approval such that the subtraction cannot wrap for the VM lifetime
Validation: P8-V11 counter/time rows; I1/I2 checked by the scenario matrix
```

### 2.2 Frequency read

```text
Name and stability: guest_read_freq(vm) -> u32 — internal
Purpose and caller: CNTFRQ-class Guest read; Linux derives ns<->tick math
  from it
Inputs: VM facts
Outputs: vm.facts.frequency (identical on every vCPU of every VM)
Preconditions: facts installed
Postconditions: pure function
State/ownership change: none
Concurrency/allocation: none
Errors: none
Security checks: value is machine-gated, never read from Host hardware state
Logic: return vm.time.facts.frequency
Validation: P8-V11 counter rows; W14 compat dimension
```

## 3. Register-semantics contracts (M2)

The exposed class is the virtual timer (README decision D2): the Guest's
`CNTV_CTL_EL0/1`, `CNTV_CVAL_EL0/1`, `CNTV_TVAL_EL0/1` accesses reach these
handlers (directly or via trap per the W05/CNTHCTL classification). Each
handler translates the access into P6-W06 state operations and an M3
re-arm request; it never stores timer state itself.

### 3.1 CTL write

```text
Name and stability: guest_timer_write_ctl(vcpu, value) -> () — internal
Purpose and caller: Guest enables/disables/masks its timer; the highest-
  frequency Linux timer path (tickless reprogramming writes CTL+CVAL)
Inputs: value (untrusted): ENABLE/IMASK bits; reserved bits must be zero
Outputs: none; effects via P6-W06 + M3
Preconditions: vCPU Running in Guest context; P6-W06 state exists
Postconditions:
  - reserved-bit violation: bits masked; ENABLE/IMASK applied as written
  - state change is committed atomically with the derived re-arm decision
    (one vCPU-timer-lock critical section, [01 §5](01-architecture-and-state.md))
  - if the new condition is asserting and was not: delivery sequence of
    [03 §2](03-code-contracts-expiry-wakeup.md) is triggered
State/ownership change: P6-W06 timer state (owner: P6-W06); armed cache via M3
Concurrency/allocation: vCPU timer lock; bounded; no allocation
Errors and failure guarantee: malformed value cannot corrupt state (masking);
  internal failure escalates via W13; Guest sees only the architectural
  readback consequences
Security/authorization checks: value is data, never control; no hypervisor
  path is selectable by it
Logic (pseudocode):
  fn guest_timer_write_ctl(vcpu, value):
      enable = value.ENABLE; imask = value.IMASK
      lock(vcpu.timer_lock):
          P6_W06.set_ctl(vcpu, enable, imask)
          M3.rearm_request(vcpu)         # derives earliest deadline; may arm/cancel
      if asserting_now(vcpu) and !asserting_before:
          M4.deliver_assertion(vcpu)     # [03 §2]; also covers ENABLE with
                                         # already-elapsed CVAL (immediate path)
      telemetry(program)
Validation: P8-V11 timer-IRQ + high-frequency rows; W18 abuse scenarios
```

### 3.2 CVAL write

```text
Name and stability: guest_timer_write_cval(vcpu, value: u64) -> () — internal
Purpose and caller: Guest sets the absolute compare value (the tickless
  kernel's dominant operation)
Inputs: value (untrusted 64-bit, Guest time base)
Outputs: none; effects via P6-W06 + M3
Preconditions: as §3.1
Postconditions:
  - CVAL committed via P6-W06; a CVAL already in the past with ENABLE=1 and
    IMASK=0 makes the assertion condition true immediately (IHI timer
    semantics) — the immediate path of [03 §2](03-code-contracts-expiry-wakeup.md),
    not a lost deadline
  - re-arm decision atomic with the commit ([01 §5])
State/ownership change: as §3.1
Concurrency/allocation: as §3.1
Errors and failure guarantee: no arithmetic is performed on the Guest value
  on this path (absolute compare), so no overflow surface; readback returns
  the stored value
Security/authorization checks: value never selects hypervisor behavior
Logic (pseudocode):
  lock(vcpu.timer_lock):
      P6_W06.set_cval(vcpu, value)
      M3.rearm_request(vcpu)
  if asserting_now(vcpu): M4.deliver_assertion(vcpu)
  telemetry(program)
Validation: P8-V11 high-frequency rows; immediate-expiry scenario
```

### 3.3 TVAL write/read

```text
Name and stability: guest_timer_write_tval(vcpu, value: u32) -> () — and —
  guest_timer_read_tval(vcpu) -> u32 — internal
Purpose and caller: countdown register form used by older/periodic-style
  Guest code paths
Inputs: 32-bit signed countdown (write); none (read)
Outputs: write — CVAL = counter + sign-extended(value) then §3.2 semantics;
  read — saturating (counter − CVAL) clamped to the 32-bit signed range
Preconditions: as §3.1
Postconditions:
  - TVAL is never stored: reads are always computed from counter and CVAL
    (README decision D4) — repeated reads of an elapsed timer give the
    saturated value, not a drifting one
  - checked arithmetic everywhere; a saturated read is a defined outcome
State/ownership change: CVAL via P6-W06 (through §3.2); no separate TVAL state
Concurrency/allocation: vCPU timer lock (write); none (read)
Errors and failure guarantee: overflow wraps per architected 64-bit addition
  on write ( Guest-controlled, defined); saturation on read (defined)
Security/authorization checks: arithmetic uses checked operations per Coding
  Guidelines; results are Guest-visible data only
Logic (pseudocode):
  write: cval = vcounter(vm).wrapping_add(sign_extend_64(value))
         guest_timer_write_cval(vcpu, cval)
  read:  tval = vcounter(vm).wrapping_sub(P6_W06.cval(vcpu))
         return saturate_to_i32(tval)
Validation: TVAL round-trip scenario; wrap-boundary rows of the matrix
```

### 3.4 Counter and frequency register routing

```text
Name and stability: guest_time_register_access(vcpu, reg, is_write, value) -> AccessOutcome — internal
Purpose and caller: routes CNTVCT/CNTFRQ/timer-register accesses per the W05
  classification table; the single place the Hidden/Reject decisions of W05
  are applied to time registers
Inputs: register identity, direction, value
Outputs: handled (value semantics of §2/§3), or classified outcome
  (Hidden-class read of physical counter -> defined zero-trap or fault per
  W05; writes to read-only regs -> ignored + telemetry)
Preconditions: vCPU Running
Postconditions: exactly one outcome class; one telemetry event
State/ownership change: delegated to §2/§3 handlers
Concurrency/allocation: as delegated; bounded
Errors and failure guarantee: disallowed access is contained (W05 outcome);
  never a Host fault
Security/authorization checks: the classification table here is data derived
  from W05's categories + the machine facts (timer_class); divergence between
  DTB-presented and implemented class fails P8-V05 review
Logic (pseudocode):
  match reg:
      VCOUNTER | FREQ if !is_write => handled(guest_read_*(vcpu.vm))
      VTIMER_CTL/CVAL/TVAL         => handled(§3 handler)
      PCOUNTER | PTIMER_*          => w05_classified_outcome(Hidden)
      _                            => w05_classified_outcome(Reject)
Validation: W18 register-abuse scenarios; DV02 probe matrix
```

## 4. Malformed-programming acceptance scenarios (contracts for W18/W16)

| Scenario | Guest action | Required outcome |
|---|---|---|
| T5a zero/elapsed deadline storm | repeatedly program CVAL ≤ counter | immediate assertion path each time, bounded per-trap work; Host responsive; telemetry shows rate |
| T5b max-frequency reprogram | program CTL+CVAL in a tight loop at fixture-bounded rate | no unbounded host work (I6); Guest time stays monotonic |
| T5c reserved-bit CTL writes | set reserved CTL bits | bits masked; defined readback |
| T5d physical-timer/counter access | read CNTPCT / program CNTP_* (Hidden class) | W05-classified contained outcome; no Host state change |
| T5e TVAL wrap probing | TVAL extremes | defined wrap/saturation; assertion condition still exact |
| T5f deadline while stopped | CPU_OFF with pending deadline, then CPU_ON | no delivery to the stopped vCPU; delivery decision on next ON follows [03 §5](03-code-contracts-expiry-wakeup.md) |
