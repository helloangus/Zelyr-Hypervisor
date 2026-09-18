# P6-W06 Code Contracts — vCPU Timer State, Save/Restore, Entry/Exit

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W06 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (decisions D1–D9),
[02](02-architecture-and-state.md) (ownership, lifecycle), and the W05 time
contracts (`../p6-w05-el2-generic-timer/03-code-contracts-time-core.md` by
slug, cited in prose here as the W05 time-core file).

## 1. Typed counter domains (D9)

### 1.1 `VirtCounter`, `VirtDeadline`, and conversion

```text
Name and stability: VirtCounter(newtype over u64), VirtDeadline(newtype
  over u64); conversions:
    fn virt_counter_read() -> VirtCounter              (Arch-domain;
      ISB + CNTVCT_EL0 read — same ordering discipline as the W05
      clock_read() contract)
    fn virt_deadline_after(base: VirtCounter, delta: TimerDelta)
        -> Result<VirtDeadline, DeadlineOverflowError>
    fn to_phys(&self, offset: VirtualOffset) -> Result<PhysCounter, ...>
    fn virt_deadline_to_phys(&self, offset: VirtualOffset)
        -> Result<TimerDeadline, ...>                  (TimerDeadline is
      the W05 physical-domain deadline type)
  with VirtualOffset(newtype) whose only constructor in P6 is
  VirtualOffset::ZERO. Internal to W06/W07 consumers of Guest-time values;
  stable within P6.
Purpose and caller: keep Guest-domain and Host-domain time numerically
  distinct and convertible only through the explicit offset (D9/D2).
  Callers: exit evaluator, entry restore, diagnostics.
Preconditions / postconditions: conversion with a nonzero offset is
  representable but no W06 path constructs one in P6 (a nonzero offset
  constructor is Reserved, not removed).
Concurrency/allocation context: value semantics; checked arithmetic only.
Errors: overflow on add/conversion; unchanged inputs.
Security/authorization checks: none beyond type discipline (values
  originate in Guest-controlled registers and are treated as untrusted
  data of a trusted type).
Logic: trivial checked arithmetic; the contract's purpose is the domain
  separation, not the math.
Validation: host-side unit tests; reviewed in W06-DV01.
```

### 1.2 `GuestTimerCtl` (typed control register image)

```text
Name and stability: GuestTimerCtl — bitfield-validated image of
  CNTV_CTL_EL0 (enable, imask, istatus as observed). Internal to W06;
  stable within P6.
Purpose and caller: replace raw bit fiddling with validated accessors;
  used by save/restore and the evaluator.
Preconditions: constructed only from a register read; reserved bits
  preserved on restore (read-modify-write with observed reserved bits).
Postconditions: accessors return the validated fields; no accessor can
  produce an out-of-range field.
Concurrency/allocation context: value type.
Errors: constructor cannot fail; inconsistent images (e.g. istatus set
  while disabled) are legal architectural states and modeled as such.
Security/authorization checks: guest-controlled content; validation is
  the field extraction itself.
Validation: host-side unit tests; W06-DV02.
```

## 2. Data structure: `VcpuTimerState`

```text
Name and stability: VcpuTimerState — one per vCPU, held in the vCPU's
  architecture-extension area (assumed P4 contract, [01] §5). Internal to
  W06; stable within P6.
Fields:
  saved_ctl:        GuestTimerCtl
  saved_cval:       Option<VirtDeadline>     (None = Guest never programmed)
  shadow_verdict:   ConditionVerdict         (last evaluation result)
  physical_outstanding: bool                 (D6 single-outstanding rule)
  diagnostics:      saturating counters per [02] §7
Purpose and caller: sole authority for Guest timer state between runs
  (D3); mutated by entry/exit sequences, the PPI handler, the completion
  hook, and initialized at vCPU creation.
Preconditions: exists whenever the vCPU exists; initialized before first
  entry.
Postconditions: after exit, `physical_outstanding == false` and the
  hardware timer is disabled; after entry, registers match the record.
Concurrency: per-vCPU irq-save lock discipline (P3) guards mutations;
  entry/exit are exclusive by the P4 transition contract; see [02] §5.
Errors: invariant violations (flag set at entry, second dispatch while
  held) are counted and recovered per [02] §4.2, never silent.
Security/authorization checks: contents are Guest-influenced and validated
  on use.
Validation: W06-DV01/DV03/DV06.
```

## 3. Entry and exit sequences (Arch-domain)

### 3.1 `vtimer_on_entry(vcpu, record)`

```text
Name and stability: fn vtimer_on_entry(&mut VcpuTimerState) -> () — called
  by the P4 entry path immediately before Guest execution resumes, on the
  hosting pCPU. Arch-domain (system registers). Stable within P6.
Purpose and caller: make the Guest's timer live again from the vCPU
  record (D1/D3).
Inputs / outputs: the vCPU's timer record -> hardware programmed.
Preconditions: pCPU is the vCPU's host; P1 trap baseline active; record
  initialized; entry exclusivity held (P4).
Postconditions: CNTVOFF_EL2 == 0 (D2); CNTV_CVAL_EL0 == saved_cval (if
  any); CNTV_CTL_EL0 == saved_ctl (reserved bits preserved); an invariant
  check confirms `physical_outstanding == false` (else recovery per
  [02] §4.2 and count `timer_impossible_state_count`).
Concurrency: transition context; takes the record lock only for the
  invariant check (O(1)); register writes are exclusive to this context.
Errors: none returned; impossible-state recovery is internal and counted.
Security/authorization checks: restored values were captured by W06 at the
  previous exit (not Guest-supplied memory).
Logic (pseudocode; barrier placement per §3.3):
    lock(record)
    if record.physical_outstanding { recover: release via physical
        lifecycle op; flag=false; count impossible_state }
    unlock(record)
    write CNTVOFF_EL2 = 0                        // D2, per entry
    if let Some(cval) = record.saved_cval { write CNTV_CVAL_EL0 = cval }
    write CNTV_CTL_EL0 = record.saved_ctl        // enable/imask as saved
    dsb_sy()                                     // §3.3 table
Validation: W06-DV03 (restoration equality), DV02.
```

### 3.2 `vtimer_on_exit(vcpu, record)`

```text
Name and stability: fn vtimer_on_exit(&mut VcpuTimerState) ->
  ExitEvaluation — called by the P4 exit path for every controlled exit
  from the Guest (including HVC-controlled exits), before the vCPU is
  regarded as absent. Arch-domain. Stable within P6.
Purpose and caller: save state, silence the hardware, evaluate the
  condition, request deferred delivery, release any held physical
  interrupt (D3/D4/D6/D7).
Inputs / outputs: record -> ExitEvaluation { saved fields,
  verdict, deferred_inject_requested: bool } for diagnostics.
Preconditions: exit exclusivity held; W03 dispatch quiescent for INTID 30
  on this pCPU or safely serialized by the record lock.
Postconditions: record holds the Guest's live register image; hardware
  timer disabled; `physical_outstanding == false`; if the condition was
  met and unmasked, a W07 injection was requested exactly once (dedupe is
  W07's concern); no Host state other than the record and counters
  changed.
Concurrency: transition context; record lock held across
  evaluate-and-release (O(1)).
Errors: none returned; invariant anomalies counted.
Security/authorization checks: all values read from hardware are
  validated images ([1.2]); no Guest memory is dereferenced.
Logic (pseudocode):
    lock(record)
    ctl_raw  = read CNTV_CTL_EL0                  // validated image
    cval_raw = read CNTV_CVAL_EL0
    record.saved_ctl  = GuestTimerCtl::from(ctl_raw)
    record.saved_cval = Some(VirtDeadline::from(cval_raw))
    write CNTV_CTL_EL0 = ctl_raw with ENABLE=0    // silence condition
    dsb_sy()                                      // §3.3
    now = virt_counter_read()                     // ISB + CNTVCT
    verdict = evaluate(record.saved_ctl, record.saved_cval, now)  // [04] §2
    record.shadow_verdict = verdict
    if verdict == ExpiredUnmasked {
        outcome = virq_inject(target vcpu, vintid = TIMER_EVENT_INTID,
                              source = TimerExpiry)
        // Pending or Coalesced are both success; count per outcome
        record.diagnostics.deferred_inject += 1
    }
    if record.physical_outstanding {
        physical_timer_ppi_release()   // safe: condition source disabled
        record.physical_outstanding = false
        count completion_release (exit-path)
    }
    dsb_sy()
    unlock(record)
Validation: W06-DV03 (preservation), DV04 (deferred delivery).
```

### 3.3 Register sequence and barrier contract

| Sequence point | Required ordering | Rationale |
|---|---|---|
| `CNTVOFF_EL2` write before first Guest counter use each entry | program order + DSB SY before ERET (with the P4 entry sequence) | Guest-visible counter zero-offset must be in effect before Guest reads |
| CVAL write before CTL write on restore | program order (system registers) | enabling with a stale compare value would misfire |
| CTL Enable=0 before condition evaluation on exit | program order, then DSB SY | evaluation must observe the silenced condition source; prevents a re-fire between evaluation and release |
| ISB before `CNTVCT_EL0`/`CNTPCT_EL0` decision reads | ISB | speculative counter reads must not hoist above the disable/eval sequence (same rule as W05) |
| Exit sequence completion before the vCPU is marked absent | DSB SY | the absent vCPU's record must be authoritative before any other context reads it |

Exact instruction spellings are fixed against the locked architecture
revision at implementation (task book §8); every accessor is an Arch-domain
`unsafe` inventoried with a `SAFETY` justification. `CNTHCTL_EL2` itself is
never written by W06 at runtime — it is P1 baseline configuration (D1);
W06 only verifies the posture at intake ([04](04-code-contracts-expiry-and-delivery.md) §6).

## 4. Interaction with W05 (Host event basis)

W06 consumes the W05 typed physical-domain types for cross-domain
conversion and diagnostics. W06 does **not** arm W05 host timers for Guest
deadlines in P6 (D4); a future assist (Reserved for P7) would arm a W05
deadline whose expiry record carries the vCPU identity, and would be a
design amendment to this file — not an ad-hoc caller. Guest-domain and
Host-domain values never mix without [1.1] conversion.
