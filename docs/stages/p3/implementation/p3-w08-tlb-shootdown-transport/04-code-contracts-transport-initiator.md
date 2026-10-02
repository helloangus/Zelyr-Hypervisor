# P3-W08 Code Contracts — Initiator, Collection, Timeout

**Current admission (2026-10-02):** [Amendment 07](07-timeout-ownership-remediation.md) removes Pending supersession. The owner selected W08-SYNC option A on 2026-10-02: single-attempt protocol admission, TransportBusy on contention, no SpinLock across collection. W06/W08 contracts are reconciled below; final design admission, implementation and execution evidence remain pending.

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
    EmptyMask, TransportBusy { excluded: TargetMask }, TargetsBusy { busy: TargetMask, excluded: TargetMask } }
    — internal; pre-publication refusals. Busy is a preflight observation,
    not an atomic snapshot; a target may complete concurrently.
Purpose and caller: the total outcome vocabulary of a transport
    operation; callers: P4 (via W14), W12 assertions, W11 activity.
Preconditions / postconditions: Completed partitions requested ids into
    completed/excluded; TimedOut partitions them into acked/unacked/excluded
    (the accounting invariant DV06 asserts). Errors publish no new request.
Concurrency/allocation context: plain values; no allocation.
Errors and failure guarantee: errors occur before new-request publication
    or sequence changes. Independent old-reception progress and refusal
    diagnostics may still occur; this is not a frozen global-state snapshot.
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
    held on entry or during collection. Postconditions —
    Completed: every effective target's slot read Completed{seq} (each
    ack acquire-ordered after the target's release-store of completion;
    per README decision 4 the *operation content* ordering obligation
    belongs to P4's binding); TimedOut: unacked named; errors:
    side-effect-free.
State and ownership change: each effective target's slot Empty/Completed→Pending
    (and targets →Completed); the boot-global request sequence +1.
Concurrency/allocation context: one atomic admission attempt; contention
    returns TransportBusy. No allocation, SpinLock or protected-data borrow.
    The private non-Copy admission token serializes publication/collection;
    receivers never acquire it. Collection services own reception per BW-4.

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
        slots = resolve_all_areas_and_notification_prerequisites(effective)?
        token = try_admit_once(current_cpu, Acquire, Relaxed)
        if token is None: return Err(TransportBusy { excluded })
        # No fallible ordinary-return operation after this point without finish(token).
        observed = acquire_load_and_validate_all_controls(slots)
        busy = targets_whose_observed_state_is_Pending(observed)
        if not busy.is_empty():
            finish(token)  # checked owner release; Release success / Relaxed failure
            return Err(TargetsBusy { busy, excluded })  # no publication/seq writes
        seq = next_seq()                   # only now advance modulo-u16 tag
        for (t, slot, old) in slots.zip(observed):
            slot.descriptor.store(descriptor, Relaxed)
            CAS(slot.control, old, pack(Pending, seq), AcqRel, Acquire)
                or fatal_invariant()      # old is exact Empty/Completed
            notify(t, kind 1, 0) or fatal_invariant()    # no ordinary Err after writes
        acked = empty_mask
        unacked = effective
        for round in 0..COLLECT_BOUND:
            acked |= unacked.filter(|t| acquire_control(t) == Completed{seq})
            unacked = effective - acked
            if unacked.is_empty(): break
            consume_pending_requests()            # BW-4 reactive wait
        result = if unacked.is_empty():
            Completed { completed: effective, excluded }
        else: TimedOut { acked, unacked, excluded }
        finish(token)  # checked Active(current_cpu)->Idle, Release/Relaxed
        return Ok(result)  # copied value; no slot reads after release

Admission state: one boot-global atomic Idle / Active(cpu); initialized Idle
    before SmpReady. Use a strong compare_exchange exactly once (no retry,
    no spurious-busy weak CAS), Acquire success / Relaxed failure. Only a
    successful attempt creates the private non-Copy token. Sequence storage
    is atomic and accessed only by its owner; no protected-data borrow is
    exported. All slot publication ordering remains independently specified.
    finish consumes the token and checks its owner; a mismatching release is
    terminal, never a blind store. Tokens have no automatic unlock on Drop
    or unwind. Fatal paths retain admission/slots/resources; no reset/steal.
    Resolve ordinary fallible prerequisites before admission. TargetsBusy,
    Completed and TimedOut are the only ordinary admitted return paths and
    explicitly release exactly once. A halted initiator can strand admission.


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
Recovery: timeout retains every Pending slot and its descriptor. Before any
    subsequent publication, acquire-preflight all effective slots. Pending
    causes TargetsBusy with no publication/sequence writes. A late target
    completes its original operation; reuse only after Completed observation.
    Independent target sets remain usable; stalled targets cannot be recovered
    by overwriting, reset, sequence equality or CPU offlining. Retain referenced
    resources until the owning binding proves completion.
Failure guarantee: collection bounds its number of polls, not scheduling or
    fairness or admission availability. Latched unacked observations may complete before return.
Validation: W08-DV06 pauses at receiver reads/operation, mixed-mask refusal,
    late completion/reuse, sequence wrap and permanently stalled target.
```

## 5. Concurrency rules (normative)

- CR-1 (single-flight): exactly one admitted publication/collection call;
  older timed-out target operations may remain outstanding. Competitors make
  one admission attempt and return TransportBusy immediately on contention.
- CR-2 (reactive wait): collection interleaves consume_pending_requests()
  (BW-4). Caller-owned retry must be bounded and reception-responsive; no
  hidden spin-until-admitted wrapper and no fairness guarantee.
- CR-3 (no wait under locks): entry and collection hold no data-lock guard.
  Admission grants protocol authority only; it is the explicit W06 BW-6
  protocol, not an Infrastructure lock or a BW-2 exception. Diagnostics must
  release any leaf guard before polling or reception.

- CR-4 (target independence): targets take no lock and never wait on the
  initiator; a target's consumption is correct regardless of initiator
  liveness once published; initiator failure never permits Pending reuse.
- CR-5 (context): initiation is thread-context only; consumption is
  thread-context only at P3 (the P1 interrupt posture means no
  interrupt-driven consumption exists; revisit when P6 sources arrive).

## 6. Accounting invariants (the DV06 contract)

For every `initiate_transport` call that passes all preflight and publishes:

- every effective target ends the call in exactly one of {acked,
  unacked} and every requested id in exactly one of {acked, unacked,
  excluded};
- per-target slot state is Completed{seq of the last request} or
  Pending{seq of the last request} — nothing else;
- the target-private `completions` counter increments exactly once per
  consumed request; at quiescence the sum equals Completed transitions.
  Late completions do not rewrite old returned timeout observations.
  Busy refusals publish zero targets and are counted separately by W11.
