# P7-W05 Shared-Scheduler Code Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W05 detailed design](README.md).

Contracts follow the implementation-design checklist §3 template. Names are
internal, stage-local design names. Seams: W02 gate/exit/engine, W03
`is_eligible`, W04 triggers/`switch_to`/slice source. Concurrency bounds
per [architecture and state](02-architecture-and-state.md) §3.

## §1 Entity

### Contract 1.1 — `SchedulingEntity`

```text
Name and stability: struct SchedulingEntity { vm: VmId, vcpu: VcpuId,
  placement: ResolvedPlacement (frozen ref), entity_id: SchedEntityId };
  internal; ADR names the concept; W05 fixes the view.
Purpose and caller: everything the policy layer may see about a schedulable
  thing; constructed once at vCPU configuration; referenced by queues and
  policy.
Inputs / outputs: construction from the configured vCPU (W03 placement
  already attached; vCPU still Offline).
Preconditions / postconditions: immutable after construction; no policy
  data (weight, class, deadline) exists on the entity in v0 — adding any
  is a Reserved-path design change; VM id is carried for attribution only:
  the scheduler never groups, orders, or prioritizes by VM (VM is not a
  schedulable entity, ADR §5).
State and ownership change: none (immutable view).
Concurrency/allocation context: plain data, shared by reference.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic: none (data).
Validation: construction tests + review that no policy field exists
  (W05-DV02).
```

## §2 Run queue

### Contract 2.1 — `RunQueue`

```text
Name and stability: struct RunQueue { ring: BoundedRing<SchedEntityRef>,
  queued: per-entity presence marks }; owned by exactly one PcpuScheduler;
  internal.
Purpose and caller: the per-pCPU ready set; all rotation happens through
  it. Callers: owner-side loop/policy, owner-side switch requeue,
  owner-side wakeup insert (executed by the requester but only through
  these locked operations).
Inputs / outputs:
  fn enqueue(&self, e: SchedEntityRef, dispatchable: impl Fn() -> bool)
    -> Result<(), EnqueueError>
  fn pop_next(&self) -> Option<SchedEntityRef>
  fn remove(&self, e: SchedEntityRef) -> Result<(), EnqueueError>
    # pause/stop extraction, W07 consumer
  fn len(&self) / fn is_empty(&self) / fn depth_sample(&self)
Preconditions: capacity = configured vCPU population (init-time bound);
  `enqueue` is called only after the engine accepted the entity's
  `Runnable` transition (W02 §5 ordering).
Postconditions: an entity is in at most one queue system-wide (presence
  marks); `pop_next` returns FIFO order (v0 RR cursor semantics); remove
  of an absent entity is an error, not a silent no-op.
State and ownership change: ring + presence marks only.
Concurrency/allocation context: per-queue lock, O(1) hold (ring ops; the
  O(n) duplicate scan is bounded by the configuration-bounded capacity and
  may be replaced by the presence-mark implementation); no allocation on
  the hot path; never called in IRQ context (W06 protocol defers).
Errors and failure guarantee: EnqueueError in { NotDispatchable (state
  re-check failed), AlreadyQueued, CapacityExceeded }; rejected enqueues
  change nothing; the W06 protocol owns retry-on-reject (its race window).
Security/authorization checks: n/a (scheduler-internal refs only).
Logic:

  enqueue(e):
    lock(q)
    guard !q.marked(e) else { unlock; return Err(AlreadyQueued) }
    guard dispatchable()        else { unlock; return Err(NotDispatchable) }
    guard !q.full()             else { unlock; return Err(CapacityExceeded) }
    q.push_tail(e); q.mark(e); unlock
    mark_trace(QueueEnqueue { pcpu, entity, depth })

  pop_next():
    lock(q); e = q.pop_front(); if e { q.unmark(e) }; unlock; return e

Validation: host unit tests (order, duplicates, capacity, remove); model-
  checked-ish property test: random enqueue/pop/remove interleavings keep
  presence marks consistent with ring contents (W05-DV03).
```

## §3 Policy seam and v0 round-robin

### Contract 3.1 — `SchedulingPolicy` seam

```text
Name and stability: design-level trait seam:
  fn pick_next(&mut self, queue: &RunQueue) -> Option<SchedEntityRef>
  fn on_dispatch(&mut self, e: &SchedEntityRef)
  fn on_deschedule(&mut self, e: &SchedEntityRef, reason: DescheduleReason)
  internal; one instance per PcpuScheduler.
Purpose and caller: the ADR-017/057 extension point; the loop calls it,
  never implements rotation itself.
Inputs / outputs: as above; decisions are per-queue (no cross-queue
  balancing in v0).
Preconditions / postconditions: pick_next must return Some for a non-empty
  queue unless every entry fails dispatchability (it does not decide
  idleness — None simply returns the decision to the loop); the policy may
  not touch queue internals beyond the published operations, lifecycle
  state, or timers.
State and ownership change: policy-internal cursor only.
Concurrency/allocation context: schedulable context, O(1) per call,
  allocation-free.
Errors and failure guarantee: no error path; a malformed policy is a code
  defect caught by the discipline checks below.
Security/authorization checks: n/a.
Logic: seam only.
Validation: seam review against Reserved-list (no weights/RT in v0)
  (W05-DV02).
```

### Contract 3.2 — `RoundRobin` v0

```text
Name and stability: struct RoundRobin; implements the seam; internal;
  the replaceable v0 policy (ADR-057 open).
Purpose and caller: first M:N policy; owned by the loop.
Inputs / outputs: seam methods; FIFO order provides rotation.
Preconditions / postconditions: on_deschedule from any reason other than
  terminal/block/pause re-enqueues at tail (via queue.enqueue with the
  dispatchability closure); terminal reasons enqueue nothing (W07 flows
  own extraction); pick_next is pop_next (rotation by construction).
State and ownership change: none beyond the queue.
Concurrency/allocation context: as seam.
Errors and failure guarantee: requeue failures (raced-away entity) are
  counted and dropped from this pass — the entity's own protocol (W06/W07)
  re-enqueues it; no lost vCPU can result because only non-runnable
  entities can fail here (post-transition states).
Security/authorization checks: n/a.
Logic:

  pick_next(q):        return q.pop_next()
  on_dispatch(e):      mark_trace(Dispatched { entity })
  on_deschedule(e, r):
      match r:
        SliceExpired | Voluntary | RescheduleRequest -> q.enqueue(e, dispatchable)
        Blocked | Paused | Stopped | Faulted         -> {}   # terminal flows own requeue

Validation: rotation property test — k continuously-runnable entities on
  one queue receive round-robin turns with no starvation (W05-DV03);
  fairness-window assertion (§5 of architecture and state) passes in
  simulated runs (W05-DV04).
```

### Contract 3.3 — `choose_enqueue_pcpu`

```text
Name and stability: fn choose_enqueue_pcpu(e: &SchedEntityRef,
  event_pcpu: PcpuId, eligible: impl Fn(PcpuId) -> bool) -> EnqueueTarget;
  enum EnqueueTarget { Pcpu(PcpuId), NoneEligible }; internal.
Purpose and caller: the v0 placement-aware enqueue rule; called by wakeup
  paths (W06 protocol) and deschedule requeues.
Inputs / outputs: entity + pCPU where the event occurred + eligibility
  query (W03 `is_eligible` over the online set) -> target.
Preconditions / postconditions: pure; deterministic; the NoneEligible
  result is a defined outcome (not an error): the entity stays undis-
  patched and W08's wake-on-eligibility obligation covers it.
State and ownership change: none.
Concurrency/allocation context: allocation-free; callable from bounded
  post-IRQ steps.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic:

  if eligible(event_pcpu) { return Pcpu(event_pcpu) }
  for p in online_pcpus_in_stable_order():      # lowest-id first
      if eligible(p) { return Pcpu(p) }
  return NoneEligible

Validation: rule tests incl. pinned (only its pCPU), affinity subsets,
  all-offline (NoneEligible) (W05-DV03); review that the "stable order" is
  the pCPU id order — no platform-name or topology-name branching.
```

## §4 Control loop

### Contract 4.1 — `run_scheduler_loop`

```text
Name and stability: fn run_scheduler_loop(sched: &mut PcpuScheduler);
  internal; the per-pCPU main loop entered after scheduler activation
  (W02) on every online pCPU.
Purpose and caller: ties the mechanism together: reconsider -> pick ->
  gate -> switch -> Guest window -> exit -> repeat; the place where M:N
  actually happens.
Inputs / outputs: none (runs for the pCPU's online lifetime); effects via
  Guest execution windows.
Preconditions: scheduler Active (W02); queue/policy/intent initialized;
  P3 per-CPU state attached; P6 deadline machinery available.
Postconditions (per iteration): intent consumed at most once per trigger
  set; every iteration ends in exactly one of { Guest window entered,
  idle hook entered, bounded retry after rejection }; no unbounded
  retry without progress (sustained-rejection diagnostic path).
State and ownership change: queue + policy + slot/deadline via the
  contracted callees only.
Concurrency/allocation context: schedulable context; IRQ state per the P3
  scheduler rules; no allocation per iteration in steady state.
Errors and failure guarantee: no panic paths; rejections are counted,
  traced, and bounded per the loop logic; invariant-grade conditions
  escalate via the W02 audit discipline.
Security/authorization checks: no Guest input; gate enforces all
  admission rules.
Logic:

  loop {
      triggers = drain_local_triggers()      # W04: intent take, exit outcomes
      if triggers.is_empty() {
          idle_hook(sched)                   # W08-owned; must block/yield, not spin
          continue
      }
      match policy.pick_next(&sched.queue) {
          None => { idle_hook(sched); continue }
          Some(cand) => {
              match switch_to(pcpu, from = current_exit_or_idle(),
                              to = cand, reason = triggers.reason()) {
                  Ok(outcome)  => {}         # Guest window happened inside switch
                  Err(GateRejected(_)) => { requeue_tail(cand); continue }  # bounded
                  Err(ActivationFailed(stage)) => { requeue_tail(cand);
                          record_activation_failure(cand, stage);
                          # sustained failures escalate per W04 containment §3
                          continue }
                  Err(DeadlineArmFailed(_)) => { /* per W04 asymmetry:
                          shared -> abort dispatch (requeue); pinned -> degrade */ }
                  Err(InvariantViolation) => fatal()   # P0 policy
              }
          }
      }
  }

Validation: host simulation of the loop against fakes covering every
  branch (W05-DV04); QEMU scenarios S1–S6 (W05-DV05/06; P7-V10–V12).
```

## §5 Scenario and fairness hooks

### Contract 5.1 — scenario/invariant assertion hooks

```text
Name and stability: validation instrumentation, compiled behind the
  test/diagnostic feature surface (P0 profile governance):
  fn assert_no_duplicate_execution()   # INV-1 sample via W02 checker
  fn assert_no_nonrunnable_dispatch()  # INV-3 sample
  fn fairness_window_record(entity, scheduled_ticks)   # accumulators
  fn fairness_window_check(window: W) -> FairnessVerdict
  internal; zero cost when compiled out (P0 profile/feature governance).
Purpose and caller: makes the scenario table (§4 of architecture and
  state) and fairness condition (§5) mechanically checkable by W11/W12
  without re-deriving them.
Inputs / outputs: as above; FairnessVerdict { Satisfied, Violated{entity, window} }.
Preconditions / postconditions: instrumentation never mutates scheduling
  decisions; verdicts are evidence, not enforcement.
State and ownership change: measurement accumulators only.
Concurrency/allocation context: sampling from safe points; host harness
  may allocate.
Errors and failure guarantee: violation verdicts are recorded evidence;
  converting to fatal is the harness/reviewer's call per the P0 policy.
Security/authorization checks: n/a.
Logic: window bookkeeping per the §5 formula; constants recorded at
  implementation.
Validation: W05-DV04 host simulations; W05-DV05/06 target scenarios.
```
