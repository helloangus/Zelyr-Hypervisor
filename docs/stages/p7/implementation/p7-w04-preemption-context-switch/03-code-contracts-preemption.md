# P7-W04 Preemption Code Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W04 detailed design](README.md).

Contracts follow the implementation-design checklist §3 template. Names are
internal, stage-local design names. `PcpuId`/`VcpuId` are the predecessor
identity newtypes; monotonic time values come from the P6-W05 time source
(P7-IN-07). Concurrency bounds per
[architecture and state](02-architecture-and-state.md) §4.

## §1 Slice policy

### Contract 1.1 — `TimeSlice`

```text
Name and stability: struct TimeSlice(Ticks) — newtype over the P6 monotonic
  time unit; internal.
Purpose and caller: the quantum granted at dispatch; produced by the policy
  source (1.2), consumed by the deadline discipline (2.1) and switch (2.3).
Inputs / outputs: construction/validation: fn validated(t: Ticks)
  -> Result<TimeSlice, SliceError>; SliceError::Zero | ExceedsMaximum.
Preconditions / postconditions: value > 0 and <= MAX_SLICE (a declared
  boot constant bounding one deadline program); no other invariant.
State and ownership change: none.
Concurrency/allocation context: value type; no allocation.
Errors and failure guarantee: invalid values are configuration errors
  surfaced at policy-source evaluation, never clamped (no silent fallback).
Security/authorization checks: n/a (scheduler-internal).
Logic: none (data + checked bounds).
Validation: boundary tests (0, 1, MAX, MAX+1) (W04-DV02).
```

### Contract 1.2 — slice policy source seam

```text
Name and stability: trait hook (design-level):
  fn slice_for(entity: &SchedulingEntity) -> TimeSlice — owned by the W05
  policy layer; W04 defines only the seam contract and the v0 default rule.
Purpose and caller: lets ADR-017/057 evolve (weighted/RT later) without
  touching the preemption mechanism; called by the control loop at dispatch.
Inputs / outputs: entity -> slice.
Preconditions: returned slice passes 1.1 validation (policy is not above
  the mechanism's bounds).
Postconditions: v0 implementation: uniform DEFAULT_SLICE for all entities;
  DEFAULT_SLICE is a boot constant selected at implementation time per the
  README decision-3 rule and recorded in the W04 record.
State and ownership change: none.
Concurrency/allocation context: called in schedulable context; must be
  O(1) and allocation-free.
Errors and failure guarantee: an invalid policy answer aborts the dispatch
  (candidate requeued) with a recorded error — a policy bug must not
  produce a zero or unbounded quantum.
Security/authorization checks: n/a.
Logic: v0: TimeSlice::validated(DEFAULT_SLICE).
Validation: seam review with W05; default value recorded (W04-DV01/02).
```

## §2 Deadline discipline

### Contract 2.1 — `arm_preemption_deadline`

```text
Name and stability: fn arm_preemption_deadline(pcpu: PcpuId,
  slice: TimeSlice) -> Result<ArmedDeadline, DeadlineError>; internal.
Purpose and caller: arms the single per-pCPU preemption deadline at
  now + slice via the P6-W05 mechanism; called by the control loop at the
  end of a successful dispatch (after `admit_for_entry`, before Guest
  entry) and re-armed by the switch (2.3) for the incoming vCPU.
Inputs / outputs: pCPU + slice -> handle or error.
Preconditions: called in schedulable context on the owning pCPU; any
  previously armed deadline on this pCPU is first cancelled (2.2) or the
  P6 rearm semantics are used per P7-IN-07; scheduler mode Active.
Postconditions: on Ok — exactly one deadline armed on the pCPU, expiry at
  now + slice (P6 monotonic clock); pCPU deadline state == Armed{at}.
  On Err — no deadline armed or prior state preserved (per P6 semantics);
  pCPU state consistent with the error.
State and ownership change: pCPU deadline state (W04-owned interior).
Concurrency/allocation context: schedulable context; allocation-free;
  uses P6's IRQ-safe arm path.
Errors and failure guarantee: TimerUnavailable (P6 mechanism missing or
  failed — a W01 register P7-IN-07 mismatch if persistent);
  ProgramFailed (P6 rejected the arming). Failure guarantee: state
  recorded; caller applies the failure model ([architecture and state]
  §6: shared dispatch aborts, pinned dispatch degrades).
Security/authorization checks: n/a.
Logic:

  at = p6_now(pcpu) + slice.0
  match p6_arm_deadline(pcpu, at):   # P6 contract; may implement rearm
      Ok  -> state.armed = Armed { at }; return Ok(handle)
      Err -> return Err(map(e))
Validation: host tests with a P6-contract-faithful fake; QEMU expiry tests
  (W04-DV03; P7-V08).
```

### Contract 2.2 — `cancel_preemption_deadline`

```text
Name and stability: fn cancel_preemption_deadline(pcpu: PcpuId); internal.
Purpose and caller: disarms the pCPU deadline; called at switch quiesce
  (before the outgoing vCPU leaves) and on transition to idle.
Inputs / outputs: pCPU -> () (idempotent).
Preconditions: owning-pCPU context.
Postconditions: no deadline armed on the pCPU; state == Disarmed; a
  deadline expiring after cancellation is handled by the spurious-expiry
  rule ([architecture and state](02-architecture-and-state.md) §6) —
  recorded, intent set, no error.
State and ownership change: pCPU deadline state.
Concurrency/allocation context: schedulable context; allocation-free.
Errors and failure guarantee: none (P6 cancel is best-effort per its
  contract; spurious expiry is benign by design).
Security/authorization checks: n/a.
Logic: p6_cancel_deadline(pcpu); state.armed = Disarmed.
Validation: host interleaving tests (cancel vs expiry race) (W04-DV03).
```

### Contract 2.3 — `on_preemption_deadline` (IRQ handler)

```text
Name and stability: fn on_preemption_deadline(pcpu: PcpuId); internal;
  the P6 deadline IRQ callback registration target.
Purpose and caller: the bounded IRQ-context response to deadline expiry.
Inputs / outputs: pCPU -> ().
Preconditions: executing in IRQ context on `pcpu` (P6 delivery guarantee).
Postconditions: pCPU intent flag set (2.4); expiry recorded for the loop;
  single trace mark emitted; NOTHING else — no lifecycle calls, no queue
  operations, no allocation, no Guest state access (P4 already owns the
  saved context from the resulting exit).
State and ownership change: intent flag + expiry record.
Concurrency/allocation context: IRQ context; constant work; IRQ-safe
  primitives only.
Errors and failure guarantee: cannot fail; a missing expiry record under
  an armed deadline would be an invariant (checked by sampling audit).
Security/authorization checks: n/a.
Logic:

  intent.set(pcpu);
  record_expiry(pcpu);
  mark_trace(DeadlineExpired { pcpu });
  return   # Guest returns through the P4 exit path; loop decides later
Validation: IRQ-boundedness review + host simulation (W04-DV03); QEMU
  P7-V08 (CPU-bound Guest returns to EL2 after expiry).
```

### Contract 2.4 — reschedule intent

```text
Name and stability: per-pCPU flag with fn set(pcpu), fn take(pcpu) -> bool,
  fn peek(pcpu) -> bool; internal to the intent module; IRQ-safe.
Purpose and caller: coalesces "this pCPU should reconsider soon" from the
  deadline handler and from RescheduleRequest surfacing (W08 semantics over
  the P3 transport); consumed (taken) by the control loop at trigger points.
Inputs / outputs: set/take/peek on the owning pCPU's flag (set may be
  requested cross-CPU only via the P3 transport landing on the owner —
  never by direct cross-CPU writes).
Preconditions / postconditions: set is idempotent; take is atomic
  test-and-clear per the P3 atomic-ordering baseline; after take, the loop
  reconsiders at least once even if no other event occurred.
State and ownership change: flag only.
Concurrency/allocation context: lock-free atomics; allocation-free; safe in
  IRQ context.
Errors and failure guarantee: none.
Security/authorization checks: n/a.
Logic: atomic swap/test-and-clear.
Validation: memory-ordering tests per the P3 baseline; lost-wakeup analysis
  deferred to W06/W08 (their protocols must preserve "set precedes any
  decision that could sleep") (W04-DV03).
```

## §3 Reconsideration triggers

### Contract 3.1 — `PreemptionTrigger` / reconsideration contract

```text
Name and stability: enum PreemptionTrigger { DeadlineExpired,
  GuestExit(ExitKind), RescheduleRequest }; internal.
Purpose and caller: the closed set of events after which the control loop
  must call the reconsideration decision (W05 policy pick); produced by
  2.3, the W02 exit boundary outcomes, and intent takes.
Inputs / outputs: loop observes (trigger, exit-outcome?) and obtains a
  decision { ContinueCurrent, RunNext(candidate), Idle } from W05.
Preconditions / postconditions: every trigger is eventually observed by
  the loop on its pCPU (bounded by the loop's structure, W05); no trigger
  is dropped silently — DeadlineExpired and RescheduleRequest always clear
  the intent; GuestExit always passes through `complete_exit` first (W02).
State and ownership change: none beyond intent consumption.
Concurrency/allocation context: schedulable context.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic: enumeration + loop hook signature only; decision policy is W05.
Validation: review that the set is closed and producers are the three
  named paths (W04-DV03); QEMU P7-V08 (DeadlineExpired path end-to-end).
```

### Contract 3.2 — `DescheduleReason` semantics

```text
Name and stability: enum DescheduleReason { SliceExpired, Voluntary,
  Blocked, Paused, Stopped, Faulted, RescheduleRequest }; internal.
Purpose and caller: the mechanism-side taxonomy carried by every
  deschedule (W02 engine `Deschedule` event context; switch trace); W09
  renders/aggregates; W04 is the canonical producer via the switch.
Inputs / outputs: as above.
Preconditions / postconditions: reason agrees with the post-deschedule
  run state (SliceExpired/Voluntary/RescheduleRequest -> Runnable;
  Blocked -> Blocked; Paused -> Paused; Stopped -> Stopped; Faulted ->
  Faulted); disagreement is an invariant.
State and ownership change: none.
Concurrency/allocation context: plain data.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic: none (data).
Validation: agreement property test (W04-DV03); P7-V20 support via W09.
```
