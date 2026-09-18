# P7-W06 Code Contracts — Wakeup Path

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W06 detailed design](README.md).  
Scope: contracts for event recording, eligibility transition, source adapters,
and exclusion rules. Naming authority as stated in
[02-code-contracts-block-path.md](02-code-contracts-block-path.md).

---

## W-1. `post_wake_event`

```text
Name and stability: post_wake_event(entity: VcpuRef, source: WakeEventSource)
                    -> WakeOutcome; internal; the single wake entry point; stable
                    within P7.
Purpose and caller: record an eligible-class event for a vCPU and convert it
                    into eligibility when the vCPU is Blocked. Callers: the
                    host timer-IRQ deadline handler (W-3), the authorized vIRQ
                    injection path (W-4), internal control paths of P7-W07
                    (W-6), and the future Notification producer (W-5).
Inputs / outputs: VcpuRef; WakeEventSource = TimerExpiry | VirtualIrq |
                  Notification | Internal. WakeOutcome = AlreadyRunnable |
                  Woken | EventRecorded | Excluded(state).
Preconditions: entity is a live vCPU (VM/vCPU lifetime owned by the P4/P5
               boundary); source is one of the four; callable from IRQ or
               thread-like scheduler context on any pCPU.
Postconditions: the source flag is pending (coalesced per source) with release
                ordering. If the vCPU was Blocked, its state is now Runnable
                (via L-1) and it is enqueued under placement (S-2/C-1) with a
                reconsideration request (S-3). In every outcome the event is
                not lost: AlreadyRunnable/Excluded leave it pending for a
                defined consumption point (B-1 re-check, Guest re-entry).
State and ownership change: pending-event record of the vCPU; potentially its
                lifecycle state through L-1; runqueue membership through S-2.
Concurrency/allocation context: IRQ-safe; bounded work in IRQ context (set
                flag + short locked section only); allocation-free; ordering
                pairings per [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §5.
Errors and failure guarantee: enqueue failure (S-2 surface error, e.g.
                placement now empty) → the vCPU is Runnable with no queue;
                this is an invariant violation surfaced per the P0-W14
                boundary — placement-empty runnable entities must be
                unrepresentable (C-1 guarantees at least one eligible pCPU);
                never silently dropped.
Security/authorization checks: this entry is hypervisor-internal; Guests
                reach it only through authorized producers (P6-W07 rights
                checks for vIRQ; P5 dispatch for Guest-initiated control);
                no capability is accepted here.
Logic: the waker pseudocode in [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §5.
Validation: P7-V14 all-source wake evidence; lost-wakeup race matrix
            (before/during/after-block); P7-V25 amplification by W11.
```

## W-2. `consume_wake_events`

```text
Name and stability: consume_wake_events(entity: VcpuRef) -> EventSet; internal;
                    stable within P7.
Purpose and caller: atomically clear and return pending events at the two
                    defined consumption points: B-1's re-check and the
                    Guest re-entry eligibility evaluation in the admission
                    path (L-3).
Inputs / outputs: VcpuRef; returns the cleared source set.
Preconditions: caller is one of the two consumption points; vCPU is Running
               (re-entry point) or under B-1's intent (block re-check).
Postconditions: pending set is empty; returned set is complete (no event
                lost between read and clear — swap semantics).
State and ownership change: pending-event record only.
Concurrency/allocation context: IRQ-safe, allocation-free, single atomic swap.
Errors and failure guarantee: none.
Security/authorization checks: none beyond internal callers.
Logic: atomic swap-to-empty of the pending record.
Validation: P7-V13 (a preexisting event prevents the block); P7-V14 (no
            stranded events); unit tests for swap atomicity.
```

## W-3. Host timer-deadline adapter

```text
Name and stability: on_pcpu_deadline_irq(pcpu: PcpuRef) -> WakeScanResult;
                    internal; stable within P7.
Purpose and caller: deadline IRQ handler hook; called by the host per-pCPU
                    timer IRQ path (P6-W05 surface) after the P6/W04 timer
                    ownership recognizes a scheduler-relevant expiry.
Inputs / outputs: PcpuRef; WakeScanResult = { woken: smallvec-like bounded
                  list of (VcpuRef, WakeOutcome), rescheduled_deadline: bool }.
Preconditions: IRQ context of pcpu; the deadline was folded by B-4 or is a
               scheduler tick deadline (P7-W04 surface).
Postconditions: every vCPU whose deadline home is this pCPU and whose deadline
                has passed received post_wake_event(TimerExpiry); the host
                deadline is re-armed to the next earliest obligation or left
                unarmed if none (bounded scan of the home set; the home set is
                the vCPUs whose last-owning pCPU is this one — bounded by VM
                placement, and the scan must have a declared upper bound).
State and ownership change: via W-1 only; deadline re-arm via P6-W05 surface.
Concurrency/allocation context: hard IRQ context: bounded, allocation-free,
                no blocking; long scans are prohibited — the fold (B-4) keeps
                the per-pCPU obligation set bounded.
Errors and failure guarantee: unrecognised expiry (no home vCPU due) →
                diagnostic event (W09), no vCPU affected.
Security/authorization checks: deadline values were validated at fold time.
Logic: scan home set; for each due vCPU call W-1; re-arm earliest remaining.
Validation: P7-V14 timer-source wake; P6-W05 monotonicity unaffected;
            bounded-scan review.
```

## W-4. vIRQ-arrival adapter

```text
Name and stability: on_virq_posted(entity: VcpuRef) -> WakeOutcome; internal;
                    stable within P7.
Purpose and caller: reaction to an authorized vIRQ becoming pending for a
                    target vCPU; called by the P6-W07 injection path after its
                    authorization and target validation.
Inputs / outputs: target VcpuRef (already validated by P6-W07); WakeOutcome.
Preconditions: P6-W07 has accepted and recorded the vIRQ pending state (P-4);
               this adapter adds no authorization of its own.
Postconditions: post_wake_event(entity, VirtualIrq) semantics.
State and ownership change: none beyond W-1.
Concurrency/allocation context: as W-1; callable from IRQ context.
Errors and failure guarantee: W-1's guarantees; P6 unavailable-target outcomes
               are decided by P6-W07 before this adapter runs.
Security/authorization checks: delegated to P6-W07 (rights, target isolation);
               this adapter must not widen access.
Logic: call-through with source VirtualIrq.
Validation: P7-V14 vIRQ-source wake; cross-check with P6-W07 delivery
            evidence.
```

## W-5. Notification-classified seam (Reserved producer)

```text
Name and stability: on_notification_posted(entity: VcpuRef) -> WakeOutcome;
                    internal; producer Reserved (no Notification object exists
                    in P7; ADR-034 object arrives in a later stage).
Purpose and caller: keeps the seam that a future Notification implementation
                    calls, so P7-V14's Notification-source evidence has a
                    defined producer surrogate in tests.
Inputs / outputs / postconditions: as W-1 with source Notification.
Preconditions: in P7, only test harnesses and internal callers may invoke it;
               no Guest or management path reaches it.
Security/authorization checks: when the real Notification object arrives, its
               capability checks precede this seam; P7 adds none.
Logic: call-through with source Notification.
Validation: P7-V14 Notification-source evidence uses internal/test producers;
            the record must state that no real Notification object was
            exercised.
```

## W-6. Internal-control adapter

```text
Name and stability: post_internal_event(entity: VcpuRef, request: ControlKind)
                    -> WakeOutcome; internal; stable within P7.
Purpose and caller: let P7-W07 control paths (pause/stop) make a request
                    observable to a Blocked vCPU's consumption points; called
                    by those paths after L-1 performs whatever transition they
                    choose.
Inputs / outputs: ControlKind is owned by [P7-W07](../p7-w07-pause-stop-fault/README.md);
                  this design treats it as an opaque tag.
Preconditions: the caller performed its L-1 transition or recorded its
               pending-control marker first, so the event cannot be consumed
               before the request exists.
Postconditions: as W-1 with source Internal.
Security/authorization checks: control authorization happened in the P7-W07
               path (P5 capability model); none here.
Logic: call-through with source Internal.
Validation: P7-V14 internal-source wake; P7-V15 pause-of-blocked matrix with
            P7-W07.
```

## W-7. Exclusion rules (invalid-state wakeup)

```text
Name and stability: enforced inside W-1; the decision table is the contract.
Purpose: implement "invalid-state wakeup exclusion" so P7-V14's invalid-wake
         cases behave identically regardless of interleaving.
Decision table (state read under L-1 serialization):
  Blocked    → wake: transition to Runnable via L-1, enqueue, reconsider.  (Woken)
  Running    → no transition; event stays pending.                          (AlreadyRunnable)
  Runnable   → no transition; event stays pending for re-entry.             (AlreadyRunnable)
  Paused     → no transition; event pending; P7-W07 resume will re-evaluate
               eligibility (Paused→Runnable or Paused→Blocked per its
               recorded pre-pause context).                                 (Excluded(Paused))
  Stopped /
  Faulted    → no transition; event pending is recorded but the entity is
               excluded from scheduling (P7-W07 containment); whether the
               event is retained or discarded on stop/fault commit is
               decided by P7-W07 at the same L-1 serialization point.       (Excluded(state))
  Offline    → cannot occur for a scheduled entity; internal-invariant
               diagnostic.
Guarantee: no excluded outcome produces eligibility, capacity, or enqueue;
           no outcome can double-run (L-2); every outcome is accounted
           (P7-W09 hooks).
Validation: P7-V14 invalid-wake cases; P7-V16 with P7-W07; property test:
            for every state, a concurrent wake leaves a consistent state and
            a non-empty event record unless P7-W07 discarded it.
```
