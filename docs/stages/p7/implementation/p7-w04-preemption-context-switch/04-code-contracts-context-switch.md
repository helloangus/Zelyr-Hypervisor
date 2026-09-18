# P7-W04 Context-Switch Code Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W04 detailed design](README.md).

The switch sequence is the heart of P7-V09. Ownership per step is normative
([architecture and state](02-architecture-and-state.md) §3): a step writing
a class it does not own is a design violation. Names are internal,
stage-local design names; the P4/P6 mechanism calls are the assumed
contracts (P7-IN-05/07/08).

## §1 The switch sequence

### Contract 1.1 — `switch_to`

```text
Name and stability: fn switch_to(pcpu: PcpuId, from: SwitchFrom,
  to: VcpuRef, reason: DescheduleReason)
  -> Result<SwitchOutcome, SwitchError>; internal to the switchseq module;
  called by the control loop (W05) after the picker returns a candidate
  different from (or after) the current entity.
Purpose and caller: the single, ordered, pCPU-local sequence that takes a
  pCPU from "Guest X just exited / idle" to "Guest Y entering", preserving
  every isolation class.
Inputs / outputs:
  SwitchFrom = Idle | Exited { vcpu, outcome: ExitOutcome }
    (the vCPU whose Guest has already exited to EL2, or none if the pCPU
     was idle / boot-strapping its first dispatch)
  -> SwitchOutcome { entered: VcpuId } | SwitchError.
Preconditions:
  - executing on `pcpu`'s control-loop context; scheduler mode Active;
  - `from.Exited`: P4 has already saved the Guest context (the exit
    happened); `pcpu.current_vcpu == Some(from.vcpu)`; run state is still
    `Running` until the sequence changes it;
  - `to` was returned by the W05 picker and is `Runnable` (re-verified by
    the gate);
  - no lifecycle lock held by the caller (W02 leaf rule).
Postconditions (on Ok):
  - `from` (if any) is in its post-outcome state (per `complete_exit`
    semantics applied inside the sequence) and is not current anywhere;
  - `to` is `Running`, is `pcpu.current_vcpu`, has its address space,
    timer, and (per P6) vGIC/LR state active, and the pCPU deadline is
    armed for its slice;
  - Guest entry has been invoked exactly once for `to`.
State and ownership change: see the step table below — every mutation is
  attributed to exactly one owner.
Concurrency/allocation context: schedulable context; IRQ-masked per the P3
  scheduler-critical-section rule for the quiesce..gate window; allocation
  avoided on the hot path (fixed-size records); no cross-CPU operations.
Errors and failure guarantee: named per step below; all-or-nothing for `to`
  — a failed activation leaves `to` undispatched (`Runnable`, requeued) and
  the pCPU idle-capable; `from` handling is never rolled back (it is
  already out of the Guest).
Security/authorization checks: no Guest-influenced input participates; the
  sequence consumes predecessor mechanisms only.
```

Sequence (each line: step — owner — failure):

```text
 0. assert pcpu.current_vcpu matches `from` ................ W02 slot ... invariant (fatal if mismatched)
 1. if from == Exited: complete_exit(pcpu, vcpu, outcome) ... W02 boundary . precondition mismatch = invariant
 2. cancel_preemption_deadline(pcpu) ........................ W04 deadline . benign spurious rule
 3. (from state now != Running; slot cleared by complete_exit)
 4. gate: admit_for_entry(pcpu, to, eligibility) ............ W02 gate ...... NotDispatchable/PlacementIneligible -> requeue candidate, return to picker (recoverable)
 5. activate address space: p4_activate_address_space(to) ... P4 ........... activation error -> containment per §3 below
 6. restore virtual timer: p6_vcpu_timer_restore(to) ........ P6 ........... activation error -> containment
 7. vGIC/LR state active for `to` (per P6 contract; may be lazy/hardware-managed)  P6 ... preservation statement required
 8. arm_preemption_deadline(pcpu, slice_for(to)) ............ W04 deadline . failure model per [architecture and state] §6
 9. invoke P4 Guest entry for `to` .......................... P4 ........... entry error = invariant-grade (state says Running; treat as fatal per P0 policy)
10. mark_trace(SwitchCompleted { pcpu, from?, to, reason }) . W04 semantics (W09 renders)
```

Ordering notes (normative):

- Step 1 precedes step 4: the outgoing vCPU is fully quiesced (state left
  `Running` only until `complete_exit` applies its outcome) before the
  incoming one is gated, so INV-1/INV-2 hold at every intermediate point
  except the deliberate, single-`Running` handover inside steps 1→4 — the
  gate performs `Dispatch` for `to` only after step 1 released `from`.
- Steps 5–7 (activation) happen after gating (`to` is `Running`) and before
  step 9 (entry): an activation failure must be recoverable, which is why
  the sequence treats steps 5–7 errors as candidate containment, and step
  9 errors as invariant-grade (entry after successful activation is
  expected not to fail).
- The deadline (step 8) is armed before entry so the slice covers the full
  Guest execution window; arming after entry would open a monopolization
  window.

### Contract 1.2 — `SwitchFrom` / `SwitchOutcome` / `SwitchError`

```text
Name and stability: as declared in 1.1; internal.
Purpose and caller: input/output data of the sequence.
Inputs / outputs: SwitchError in { GateRejected(AdmissionError),
  ActivationFailed(ActivationStage), DeadlineArmFailed(DeadlineError),
  InvariantViolation } — ActivationStage in { AddressSpace, VirtualTimer,
  VgicState } naming the failing isolation class.
Preconditions / postconditions: error carries enough context for the
  diagnostics record (pcpu, from, to, stage, underlying error).
State and ownership change: per 1.1.
Concurrency/allocation context: stack-borne; no allocation on error paths.
Errors and failure guarantee: see 1.1 and the failure model.
Security/authorization checks: n/a.
Logic: none (data).
Validation: construction/exhaustiveness tests (W04-DV04).
```

## §2 Isolation requirements (P7-V09 basis)

For the rotation A→B→C→A (and any permutation), the sequence must make
these classes observable and correct:

| Class | Requirement at each switch | Evidence hook |
|---|---|---|
| Architectural context | `to`'s GPR/PC/PSTATE/SP and declared sysregs restored from `to`'s own saved context; never another vCPU's | W10 workload checks Guest-observed continuity; W04 trace: activation stage completed |
| Address space | `p4_activate_address_space(to)` before entry; two VMs' vCPUs interleaved never observe each other's IPA space | W10 cross-VM guard pages/canaries; W11 stress |
| Virtual timer | `to`'s timer state restored; deadlines preserved across arbitrary rotation | W10 timer-continuity workload |
| vIRQ/events | `to`'s pending vIRQ/events preserved and delivered to `to` post-rotation; no event delivered to the wrong vCPU | W10 SGI/IRQ workload; W06 wakeup races |
| Scheduler facts | run state, slot agreement, queue membership per W02/W05 invariants at every sampling point | W02 checker hooks; W11 invariant stress |

W04's design obligation is explicit transfer points + trace marks per
class; the evidence itself is W10/W11 scope and P7-V09 is recorded in
verification, not here.

## §3 Failure containment for activation errors

When steps 5–7 fail for candidate `to`:

1. Do not enter the Guest; `to` remains `Runnable` and is requeued
   undispatched-this-round (via W05 operations) with
   `mark_trace(SwitchFailed { vcpu: to, stage, error })`.
2. The loop proceeds to its next decision (next candidate or idle — W05/
   W08); no automatic retry of the same candidate within the same loop
   pass.
3. The failure is recorded with the failing stage; a per-vCPU failure
   counter (W09 rendering) makes repetition visible. A candidate failing
   activation repeatedly is escalated as an invariant investigation record
   (blocked/failed evidence per the W01 §5 discipline) — it is never
   silently retried forever, and never marked `Faulted`, because the
   failure is not Guest-caused.
4. If the failure classification (per the P5/P0 boundary) turns out to be
   a genuine hypervisor invariant violation (e.g., corrupt saved state),
   the fatal path applies instead — the classification is made against the
   predecessor taxonomy, not locally invented.

## §4 What the sequence deliberately does not do

- It does not pick candidates, order queues, or decide idle policy (W05/
  W08).
- It does not implement block/wake sourcing (W06) — a `Block` outcome
  arrives through `ExitOutcome`.
- It does not perform remote operations; cross-CPU work (pause of a remote
  running vCPU, remote reschedule) arrives as reschedule requests and
  control events applied by the owner pCPU (W07/W08).
- It does not touch registers, timer registers, GIC/LR state, or VMID
  allocator internals; predecessor mechanisms do.
- It does not account CPU time (W09 consumes trace points and timestamps
  from the sequence's marks; accumulation policy is W09/W05).
