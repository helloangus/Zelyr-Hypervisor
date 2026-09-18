# P3-W08 Code Contracts — Initiator, Collection, Timeout

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths are reserved to the workspace-owning
design.

## 1. `TransportResult` and `TransportError`

```text
Name and stability: TransportResult — enum { Completed { completed:
    TargetMask, excluded: TargetMask }, TimedOut { acked: TargetMask,
    unacked: TargetMask, excluded: TargetMask } } — internal value.
Name and stability: TransportError — enum { NotReady, UnknownTarget,
    EmptyMask } — internal; side-effect-free refusals.
Purpose and caller: the total outcome vocabulary of a transport
    operation; callers: P4 (via W14), W12 assertions, W11 activity.
Preconditions / postconditions: completed ∪ unacked ∪ excluded covers
    every requested id exactly once in either variant (the accounting
    invariant DV06 asserts).
Concurrency/allocation context: plain values; no allocation.
Errors and failure guarantee: errors are returned before any slot write.
Security/authorization checks: targeting refusal = authorization
    boundary at transport level.
Logic: plain types.
Validation: W08-DV04/DV06.
```

## 2. Target selection rules (normative)

- SR-1: The availability gate is W05's `BootPhase::SmpReady`
  (acquire-read); refusal `NotReady`, no slot writes.
- SR-2: Requested ids must exist in W03's registry (`UnknownTarget`
  otherwise) and the effective mask is `requested ∩ OnlineSet`;
  `effective.is_empty()` with non-empty requested is `TargetNotOnline`
  surfaces through the excluded set, not an error, unless *every*
  requested id is offline — then the error row `EmptyMask` after
  reduction (kept distinct from a caller passing an empty mask, which is
  `EmptyMask` before reduction).
- SR-3: The initiator is never a member of the effective mask
  (broadcast_minus_initiator subtracts it; an explicit single-target
  request naming the initiator is refused as `EmptyMask` after reduction —
  the transport does not execute the initiator's local work, README
  decision 3).
- SR-4: Exclusions are reported in every result (`excluded` mask) —
  P3-V08's diagnosable-exclusion requirement.
- SR-5: At P3 there are no post-`SmpReady` offline transitions; the
  reduction is therefore race-free in practice (recorded; the hotplug
  design must revisit SR-2's snapshot semantics).

## 3. `initiate_transport`

```text
Name and stability: initiate_transport(requested: TargetMask, descriptor:
    u64) -> Result<TransportResult, TransportError> — internal; the
    initiator's single entry point.
Purpose and caller: one in-flight transport operation: validate, publish,
    wake, collect; callers: P4's future shootdown code (via W14), W12
    stress, tests.
Inputs / outputs: requested mask + opaque descriptor; result or error.
Preconditions / postconditions: precondition — thread context; no lock
    held on entry (the initiation lock is class Infrastructure, LOL
    rank 3: acquiring it while holding higher-rank locks is a ladder
    violation; nothing lower exists at P3 to hold). Postconditions —
    Completed: every effective target's slot read Completed{seq} (each
    ack acquire-ordered after the target's release-store of completion;
    per README decision 4 the *operation content* ordering obligation
    belongs to P4's binding); TimedOut: unacked named; errors:
    side-effect-free.
State and ownership change: each effective target's slot Empty→Pending
    (and targets →Completed); the boot-global request sequence +1.
Concurrency/allocation context: single-flight `SpinLock` (W06 §2
    contract) held across publish+collect — bounded by the collection
    bound; no allocation; the wait interleaves
    consume_pending_requests() per §5 (BW-4).
Errors and failure guarantee: no partial request survives an error
    (validation precedes all writes; the collection bound converts
    non-response into TimedOut rather than hanging).
Security/authorization checks: SR-1/SR-2 gates; single-flight is also
    the re-entrancy guard (a CPU cannot initiate from inside
    consumption — thread-context rule).
Logic (pseudocode):

    initiate_transport(requested, descriptor):
        if boot_gate.phase().load(Acquire) != SmpReady: return Err(NotReady)
        (effective, excluded) = reduce(requested)?   # SR-2/SR-3
        if effective.is_empty(): return Err(EmptyMask)
        initiation_lock.lock()                        # single-flight
        seq = next_seq()
        for t in effective:
            slot = area_of(t)?.tlb_slot
            slot.descriptor.store(descriptor, Relaxed)   # stable-before-
                                                            pending edge
            slot.control.compare_exchange(pack(Empty, _),
                pack(Pending, seq), Release, Acquire)?           # else fatal
            notify(t, kind 1, 0)?                            # W07 wake
        unacked = effective
        for round in 0..COLLECT_BOUND:
            unacked = effective.filter(|t| slot_of(t).control.state
                                          != Completed{seq})  # acquire
            if unacked.is_empty(): break
            consume_pending_requests()            # BW-4 reactive wait
        drop(initiation_lock)
        if unacked.is_empty():
            return Ok(Completed { completed: effective, excluded })
        return Ok(TimedOut { acked: effective - unacked,
                             unacked, excluded })

Validation: W08-DV03 (happy path, ordering), DV04 (exclusion rows), DV05
    (concurrency), DV06 (timeout).
```

## 4. Timeout bound and recovery

```text
Name and stability: COLLECT_BOUND — stage-local constant (value and
    rationale recorded in the implementation record; revisit at P6);
    bounds the collection loop in acquisitions, per the W02 POLL_BOUND
    pattern.
Purpose and caller: bounds initiate_transport's wait; caller: §3.
Semantics and limitation: "no completion within COLLECT_BOUND
    acquisitions" — a progress bound, not wall-clock time (no timer
    before P6; recorded limitation identical in kind to W02's).
Recovery: after TimedOut the transport remains usable; the next request
    supersedes stale Pending slots by sequence (target §5 of
    [03](03-code-contracts-transport-request.md) matches on seq).
    Repeated never-completing targets are suspect-CPU diagnostics fed to
    W11/W12 surfaces; at P3 they are not lifecycle events (no
    post-admission failure edge in W03) — recorded boundary.
Failure guarantee: no path waits forever; post-timeout state is
    diagnosable (slot states readable).
Validation: W08-DV06 (induced non-consuming target; supersession test).
```

## 5. Concurrency rules (normative)

- CR-1 (single-flight): at most one transport operation in flight
  system-wide; enforced by the initiation lock; concurrent initiators
  serialize with defined order.
- CR-2 (reactive wait): any CPU waiting on the initiation lock or in
  collection interleaves `consume_pending_requests()` (W06 BW-4) — the
  deadlock-safety argument of
  [02 §5](02-architecture-and-state.md) depends on it; a wait loop
  without the interleaving is a contract violation, stop at review.
- CR-3 (no wait under other locks): the initiation lock must not be
  acquired while holding any other lock (ladder: nothing may hold rank
  ≥ 3 and wait; Infrastructure is the highest non-Diagnostics class in
  use at P3), and Diagnostics logging from inside the initiation
  critical section must follow the W06 leaf rule (acquire Diagnostics,
  emit, release — never wait on transport state while holding it).
- CR-4 (target independence): targets take no lock and never wait on the
  initiator; a target's consumption is correct regardless of initiator
  liveness (supersession covers dead initiators).
- CR-5 (context): initiation is thread-context only; consumption is
  thread-context only at P3 (the P1 interrupt posture means no
  interrupt-driven consumption exists; revisit when P6 sources arrive).

## 6. Accounting invariants (the DV06 contract)

For every `initiate_transport` call that passes validation:

- every effective target ends the call in exactly one of {acked,
  unacked} and every requested id in exactly one of {acked, unacked,
  excluded};
- per-target slot state is Completed{seq of the last request} or
  Pending{seq of the last request} — nothing else;
- the target-private `completions` counter increments exactly once per
  consumed request; summed across CPUs it equals total Completed
  transitions (W11's aggregation input).
