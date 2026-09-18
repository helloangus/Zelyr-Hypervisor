# P8-W06 Code Contracts — CPU and System Lifecycle

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W06 detailed design](README.md).  
**Modules covered:** M3 (vCPU lifecycle bridge), M4 (VM system lifecycle
bridge). Dispatch, query, and error-mapping contracts are in
[02](02-code-contracts-psci-dispatch.md).

All names are logical contract names (README decision D8); pseudocode is an
implementation outline, not runnable production code. Every handler here
executes in the calling vCPU's VM-exit context and inherits the dispatch
contract's concurrency/allocation rules: bounded locks, no polling, no
dynamic allocation, no waiting on another vCPU.

## 1. Prerequisite seams consumed by this file

| Seam | Contract owner (plan path) | What W06 assumes |
|---|---|---|
| vCPU admission (Offline/Stopped → Runnable with pending entry state) | `docs/stages/p7/plans/p7-w02-scheduler-admission-lifecycle.md` | A lifecycle operation that commits a stopped, pre-provisioned vCPU to Runnable and lets the scheduler place it |
| vCPU stop/pause/fault | `docs/stages/p7/plans/p7-w07-pause-stop-fault.md` | A lifecycle operation that stops the calling vCPU cleanly and reports a stable final state |
| Placement | `docs/stages/p7/plans/p7-w03-placement-configuration.md` | Placement policy is applied by P7 after admission; PSCI never chooses a pCPU |
| Secondary entry state | `docs/stages/p8/plans/p8-w03-linux-boot-contract.md` | Register/PC/context state Linux expects at secondary entry (incl. `context_id` in x0) |
| IPA/Guest-RAM validation | `docs/stages/p4/plans/p4-w09-closeout-p5-handoff.md` facts | Bounds and Stage-2 mappability check for a Guest IPA |
| Cross-CPU kick transport | `docs/stages/p3/plans/p3-w07-cross-cpu-notification.md` | Notification of a remote scheduler domain when needed |

Failure boundaries if these deliver differently are recorded in
[01 §6](01-architecture-and-state.md).

## 2. CPU_ON

```text
Name and stability: psci_cpu_on(caller: &mut Vcpu, target_mpidr: u64,
  entry_ipa: u64, context_id: u64) -> PsciStatus — internal
Purpose and caller: standard firmware CPU-on; the only secondary-start path
  Linux may use (P8-V09: no private HVC dependency); consumed by W10's SMP
  bring-up
Inputs (all untrusted Guest register values):
  target_mpidr — virtual MPIDR of the requested vCPU
  entry_ipa    — Guest IPA at which the secondary starts
  context_id   — opaque Guest value delivered to the secondary
Outputs: PSCI status written to the caller's x0 by the dispatch layer
Preconditions:
  - caller is Running in Guest context
  - VmPsciConfig present; CPU_ON in the frozen subset
Postconditions:
  - SUCCESS: the target vCPU is committed to Runnable (P7 admission) carrying
    pending PSCI entry state {PC = entry_ipa, x0 = context_id, W03 secondary
    state}; the target will execute at entry_ipa exactly once per successful
    ON, and the caller resumes unmodified otherwise
  - error codes: state unchanged, including no partial admission and no
    pending-state residue
State and ownership change: target vCPU lifecycle state (owner: vCPU lifecycle
  per P7); target pending-PSCI-state field (owner: target vCPU); telemetry
  counters. No ownership of memory, capabilities, or devices changes.
Concurrency/allocation:
  - takes the target vCPU's lifecycle lock (bounded); lock order per P3/P7
    rules; caller never holds both caller and target state locks in conflict
  - no allocation; topology, stacks, and context storage are pre-provisioned
  - returns without waiting for the target to be scheduled
Errors and failure guarantee: INVALID_PARAMETERS (target out of topology,
  self-target, reserved bits); ALREADY_ON (target online or pending);
  INVALID_ADDRESS (entry IPA fails P4 bounds/mappability); on any error the
  target's lifecycle state and pending field are untouched
Security/authorization checks: target resolution scoped to calling VM via
  validate_psci_target_mpidr ([02 §3](02-code-contracts-psci-dispatch.md));
  entry IPA validated against GuestAddressSpace (ADR §19); context_id passed
  through as data only
Logic (pseudocode):
  fn psci_cpu_on(caller, target_mpidr, entry_ipa, context_id) -> PsciStatus:
      t = validate_psci_target_mpidr(caller, target_mpidr)?          # [02 §3]
      if t == caller or t.is_online() or t.has_pending_psci_on():
          return if t.is_online() or t == caller { ALREADY_ON }
                 else              { INVALID_PARAMETERS }
      if !guest_ram.contains_ipa(entry_ipa)
         or !stage2.mappable(entry_ipa):          return Err(INVALID_ADDRESS)
      lock(t.lifecycle):
          re-check t stopped/online under lock                   # race-free re-check
          if t.is_online(): return ALREADY_ON
          t.set_pending_psci_entry(entry = W03_secondary_state(
                                     pc = entry_ipa, x0 = context_id))
          scheduler_admit(t)?                                    # P7-W02 seam
          # admission failure is an internal fault path, not a PSCI code (D7)
          notify_placement_domain(t)                             # P3 transport if remote
      telemetry(accepted, CPU_ON)
      return SUCCESS
Validation: P8-V09 S2 (secondary runs Linux secondary init); P8-V14
  enumeration; error rows S5c/S5d/S5e; repeated lifecycle (ON→OFF→ON) in S3b
```

Notes:

- In the P8 model every topology vCPU is pre-provisioned at VM creation (W03
  boot contract), so CPU_ON never creates objects and cannot exhaust resources.
  A future dynamic-vCPU design would have to revisit this contract.
- ON_PENDING: DEN 0022 permits it for asynchronous implementations; this
  design's synchronous model commits the transition before returning, so the
  code exists in the mapping but no reachable path returns it. If P7
  admission is ever made asynchronous, the reachable-status set must be
  re-decided in a revision of this design.

## 3. CPU_OFF

```text
Name and stability: psci_cpu_off(caller: &mut Vcpu) -> PsciStatus — internal
Purpose and caller: standard hotplug offlining; Linux calls it on the CPU it
  wants to take offline and does not expect it to return on success
Inputs: none (the caller is the subject)
Outputs: PSCI status; on SUCCESS the caller never resumes Guest execution
Preconditions: caller is Running in Guest context; CPU_OFF in frozen subset
Postconditions:
  - SUCCESS: caller's vCPU reaches the stopped state (P7-W07) with all
    Guest-architected state preserved for a future CPU_ON (per-vCPU timer,
    vGIC, TLB/VMID obligations handled by their owners — [W08]/[W07]/P4
    lifecycle hooks, not here)
  - error: caller resumes with the status code and its state is unchanged
State and ownership change: caller vCPU lifecycle state only; vCPU remains a
  topology member and may be CPU_ON-ed again (repeated-lifecycle requirement)
Concurrency/allocation: transitions through the P7 stop seam; must complete
  Guest-visible quiescence obligations by invoking the owning subsystems'
  exit hooks (timer cancel [W08], vGIC quiesce [W07], scheduler release) in
  their defined order; no allocation
Errors and failure guarantee: DENIED if the stop seam refuses (e.g. lifecycle
  invariant); the Guest-visible guarantee is "either stopped or unchanged",
  never half-stopped
Security/authorization checks: implicit — a vCPU may always stop itself;
  nothing here can stop another vCPU (CPU_OFF has no target argument by spec)
Logic (pseudocode):
  fn psci_cpu_off(caller) -> PsciStatus:
      # order: cancel vCPU-private wakeup sources before releasing scheduling
      timer_cancel_and_quiesce(caller)            # [W08] hook
      vgic_quiesce(caller)                        # [W07] hook: drop presented state
      match scheduler_stop_self(caller):          # P7-W07 seam
          Ok(stopped) => { telemetry(accepted, CPU_OFF); SUCCESS /* no return to Guest */ }
          Err(_)      => { telemetry(internal-fault, CPU_OFF); DENIED }
Validation: P8-V09 S3 (offline then AFFINITY_INFO reports off); hotplug cycle
  S3b (OFF→ON→OFF stability); W16 repeated-boot/offline regression
```

## 4. AFFINITY_INFO

```text
Name and stability: psci_affinity_info(caller: &Vcpu, target_mpidr: u64,
  pwr_level: u32) -> PsciStatus — internal
Purpose and caller: Linux's psci_cpu_kill waits for a CPU to be really off
  before declaring hotplug complete; answers must reflect virtual state only
Inputs: target_mpidr (untrusted); pwr_level (untrusted; highest level = 0
  expected for this machine profile)
Outputs: SUCCESS with return value OFF (0) / ON (1) / ON_PENDING (2) in x0's
  low bits per DEN 0022, or an error code
Preconditions: target resolution as CPU_ON
Postconditions: answer is a pure function of the target vCPU's virtual
  lifecycle state at read time: stopped/offline → OFF; online/pending → ON;
  no other state influences it
State and ownership change: none (read-only)
Concurrency/allocation: reads target state under its lifecycle lock (bounded);
  no allocation; must not wait for the target to change state (Linux polls)
Errors: INVALID_PARAMETERS for malformed target or non-highest power level
Security checks: VM-scoped resolution; the answer must never be derived from
  pCPU placement, Host power state, or Host PSCI calls (decision D6)
Logic (pseudocode):
  fn psci_affinity_info(caller, target_mpidr, pwr_level) -> PsciStatus:
      if pwr_level != HIGHEST_LEVEL: return INVALID_PARAMETERS
      t = validate_psci_target_mpidr(caller, target_mpidr)?
      lock(t.lifecycle):
          return match t.virtual_power_state():
              Stopped | Offline => SUCCESS_WITH(OFF)
              PendingOn | Online => SUCCESS_WITH(ON_PENDING_OR_ON)
Validation: P8-V09 S3 (post-off read reports OFF); the S3b cycle's
  intermediate reads may report ON or OFF but never a third value; W14
  compat row (affinity semantics)
```

## 5. SYSTEM_OFF and SYSTEM_RESET

```text
Name and stability: psci_system_off(caller: &mut Vcpu) -> PsciStatus — and —
  psci_system_reset(caller: &mut Vcpu) -> PsciStatus — internal
Purpose and caller: Guest-initiated poweroff/reboot of its own virtual
  machine; the "system-off route" of the plan and P8-V09 S4
Inputs: none (subject is the calling VM)
Outputs: SUCCESS is written for observable order, then the calling vCPU never
  returns to Guest execution; on failure the caller resumes with the code
Preconditions: caller Running; function in frozen subset
Postconditions:
  - SYSTEM_OFF: every vCPU of the VM is requested to stop (same seam as
    CPU_OFF); the VM transitions toward Stopped per ADR §4.1; Host and other
    VMs are unaffected; the disposition of the freeing pCPU is a scheduler
    decision (P7-W08 idle), not a PSCI concern
  - SYSTEM_RESET: every vCPU is requested to stop and the VM transitions
    toward a re-loadable state; the Host may re-establish the W03 boot inputs
    and re-enter the VM (fixture relaunch in P8 — mechanism is implementation
    choice, Guest-visible outcome is "execution ceases")
State and ownership change: VM lifecycle state (owner: VM lifecycle); vCPU
  stop requests via the P7 seam; telemetry. No Host power action anywhere.
Concurrency/allocation: vCPU-stop requests are issued under bounded locks and
  may use the P3 cross-CPU transport for vCPUs on other pCPUs; this handler
  must not synchronously join or wait for other vCPUs (no stop-the-world on
  the exit path — the VM lifecycle owner converges them); no allocation
Errors and failure guarantee: a lifecycle refusal surfaces as DENIED with the
  VM unchanged; an internal failure escalates via the W13 fault path; a Guest
  that repeatedly calls SYSTEM_OFF after a refused transition cannot wedge
  the Host (each call is independent and bounded)
Security/authorization checks: subject resolution is the calling VM only —
  there is no parameter that could name another target; this is the core
  containment decision D3 and is exercised by scenario S5f
Logic (pseudocode):
  fn psci_system_off(caller) -> PsciStatus:
      match vm_request_transition(caller.vm, VmTransition::PowerOff):
          Ok(transition_committed) =>
              for v in caller.vm.vcpus(): request_vcpu_stop(v)   # P7 seam, non-joining
              telemetry(accepted, SYSTEM_OFF)
              SUCCESS    # dispatch writes it; control never returns to this Guest
          Err(lifecycle_refusal) => DENIED
  fn psci_system_reset(caller) -> PsciStatus:
      # identical shape with VmTransition::Reset; VM lifecycle owner moves the
      # VM toward the re-loadable state; boot-input re-establishment is Host-owned
Validation: P8-V09 S4 (clean poweroff observed; Host shell/other-VM unaffected);
  S5f multi-vCPU containment; W16 clean-shutdown regression; reboot-via-reset
  exercised by the repeated-boot matrix (P8-V22) when the fixture uses it
```

## 6. Interaction notes for consumers

- **W10:** secondary start = CPU_ON contract here + W03 secondary state + P7
  admission; W10 owns the per-CPU bring-up sequence on top and the 2/4-vCPU
  matrices. W10 must not add any alternative start path.
- **W14:** compatibility dimensions from this file: reported version, subset
  membership (via PSCI_FEATURES probe results), conduit, AFFINITY_INFO
  semantics, and the ALREADY_ON/INVALID_ADDRESS behaviors.
- **W18:** scenario set S5a–S5f ([02 §5](02-code-contracts-psci-dispatch.md))
  plus SYSTEM_OFF containment are the illegal/abnormal PSCI inputs; expected
  results are declared, not negotiated at test time.
- **W13:** internal-failure escalation (decision D7) feeds the fault
  classification inventory; the PSCI layer contributes only the escalation
  call, not a fault taxonomy.
