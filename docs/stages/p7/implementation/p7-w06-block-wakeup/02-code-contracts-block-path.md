# P7-W06 Code Contracts — Blocking Path

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W06 detailed design](README.md).  
Scope: contracts for the block path only. Wake-path contracts are in
[03-code-contracts-wakeup-path.md](03-code-contracts-wakeup-path.md). Names are
stage-local design freedom owned by this design (rationale in the parent
README, decision 3/4/8); they must be reconciled with the W02 module contract
at implementation time without changing W02-owned semantics.

---

## B-1. `try_block_current_vcpu`

```text
Name and stability: try_block_current_vcpu(entity: VcpuRef, exit_hint: BlockExitHint)
                    -> BlockOutcome; internal to the scheduler module; stable within P7.
Purpose and caller: make a WFI/WFE-class Guest exit scheduler-visible as the
                    Blocked state and release the pCPU. Called only from the
                    scheduler exit path at the W02 admission/return seam
                    (assumed contract L-3), on the pCPU currently running the
                    vCPU, after P4 has classified the exit as blocking-class.
Inputs / outputs: VcpuRef — validated scheduler handle to the current vCPU.
                  BlockExitHint — Wfi | Wfe (trace/accounting hint only).
                  BlockOutcome — Blocked | WokeImmediately(EventSet) | NotBlocked(reason).
Preconditions: caller is the scheduler exit path; entity == current vCPU of
               this pCPU (L-3); exit classification already validated (P-1);
               IRQ context of the pCPU, no scheduler lock held by caller.
Postconditions: Blocked → the vCPU lifecycle state is Blocked (via L-1), the
                pCPU no longer registers it as current, the exit hint is
                recorded, and the caller proceeds to the scheduler decision.
                WokeImmediately → state unchanged (still Running), eligible
                events returned to the caller, no capacity released.
                NotBlocked(reason) → state unchanged; reason is diagnostic
                (e.g. pause raced in, exit not blocking-class).
State and ownership change: block-intent marker set and cleared within this
                call; the lifecycle owner of the vCPU state moves Running→Blocked
                through L-1; no memory is allocated or freed.
Concurrency/allocation context: runs in pCPU IRQ/exit context; must be bounded
                and allocation-free; takes the lifecycle lock only inside the
                L-1 transition; spinlocks only per P3-W06 (S-1); never sleeps.
Errors and failure guarantee: unknown or unclassifiable exit → NotBlocked,
                no state change, diagnosed via the P4-W06 boundary; L-1
                rejection (invalid transition, e.g. a pause raced in) →
                NotBlocked with the rejected-transition reason, state
                consistent.
Security/authorization checks: exit classification is a trusted hypervisor
                boundary (P4); a Guest can influence when the path runs but
                can neither forge the classification nor wake itself into
                duplicate execution (L-2).
Logic: see the blocker pseudocode in
       [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §5;
       not runnable production code.
Validation: P7-V13 (no-event block releases pCPU; preexisting event aborts
            the block); unit tests for WokeImmediately on each source; the
            not-a-current-vCPU precondition must be unrepresentable or
            asserted.
```

## B-2. `poll_blocking_eligibility`

```text
Name and stability: poll_blocking_eligibility(entity: VcpuRef) -> Eligibility;
                    internal; stable within P7.
Purpose and caller: evaluate whether any eligible pending event exists; called
                    by B-1 inside its two-phase re-check and by tests.
Inputs / outputs: VcpuRef; Eligibility = { eligible: bool, sources: EventSet }.
Preconditions: caller holds the acquire ordering established by B-1's intent
               store; no locks held that the event producers also take (S-1
               ordering rules apply).
Postconditions: reports a consistent snapshot; true only for sources defined
                eligible in [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §3.
State and ownership change: none (read-only).
Concurrency/allocation context: bounded, allocation-free, IRQ-safe.
Errors and failure guarantee: none; absent events report false.
Security/authorization checks: none beyond the trusted internal callers;
                producer authorization happened at post_wake_event (W-1).
Logic: single acquire-load of the pending set; timer eligibility additionally
       consults the vCPU deadline versus the P6-W05 monotonic now.
Validation: unit tests covering each source alone, combined sources, and
            empty sets; determinism check under P7-V13.
```

## B-3. `BlockExitHint` and `BlockOutcome` types

```text
Name and stability: enum BlockExitHint { Wfi, Wfe }; enum BlockOutcome
                    { Blocked, WokeImmediately(EventSet), NotBlocked(BlockDeclineReason) };
                    enum BlockDeclineReason { NotRunning, TransitionRejected,
                    UnclassifiedExit }; internal; stable within P7.
Purpose and caller: vocabulary for accounting hooks (P7-W09) and for the
                    scheduler decision after a decline; produced by B-1,
                    consumed by the exit path, accounting, and trace.
Inputs / outputs: value types; EventSet is the pending-source set snapshot.
Preconditions / postconditions: values carry no capability; they never gate
               behavior beyond the outcomes defined above (the hint must not
               become a scheduling policy input).
State and ownership change: none.
Concurrency/allocation context: plain values; no allocation.
Errors and failure guarantee: NotBlocked carries the decline reason so callers
               cannot silently re-enter a vCPU whose transition was rejected.
Security/authorization checks: none (internal vocabulary).
Logic: data only.
Validation: enum exhaustiveness in the exit path; accounting regression
            (P7-V19/V20) observes one hint per block.
```

## B-4. Deadline fold at block commit

```text
Name and stability: fold_deadline_on_block(entity: VcpuRef) -> FoldResult;
                    internal; stable within P7.
Purpose and caller: keep a blocked vCPU's guest-timer deadline observable by
                    its home pCPU (README decision 5); called by B-1 after the
                    Blocked transition commits and before the caller may enter
                    idle.
Inputs / outputs: VcpuRef; FoldResult = Folded | NoDeadline | DeadlineUnobservable.
Preconditions: vCPU is Blocked; caller is the home pCPU (the pCPU that last
               ran the vCPU); P-2 (vCPU-owned deadline) and P-3 (per-pCPU host
               timer) hold.
Postconditions: Folded → the home pCPU's host deadline reflects the earliest
                of its existing obligations and this vCPU's deadline;
                NoDeadline → nothing programmed; DeadlineUnobservable → the
                caller must not enter idle (see 01 §7).
State and ownership change: mutates only this pCPU's host-deadline
                programming through the P6-W05 arm/rearm surface; never
                mutates another pCPU's timer.
Concurrency/allocation context: IRQ-safe, bounded, allocation-free; timer
                programming is volatile/ordered per P6-W05's contract and the
                Coding Guidelines MMIO rules.
Errors and failure guarantee: P-3 surface failure → DeadlineUnobservable;
                the block itself is not rolled back (the vCPU is Blocked and
                will be woken by P6-W06's eventual-delivery guarantee through
                some online pCPU), but idle entry is refused until resolved.
Security/authorization checks: deadline values originate from Guest timer
                programming and are already validated by P6-W06; this path
                re-checks sanity (monotonic now ≤ deadline, checked
                arithmetic) before folding.
Logic: read vCPU deadline (P-2); if none → NoDeadline; else request the
       home-pCPU host timer to arm the earlier of (current obligation,
       deadline) via P-3; map the result to FoldResult.
Validation: P7-V14 timer-wake case; unit tests for NoDeadline and earlier-
            obligation folding; idle-entry refusal exercised in P7-V18
            (via P7-W08).
```

## B-5. Accounting and trace hooks (consumed by P7-W09)

```text
Name and stability: on_vcpu_blocked(entity, hint, now) and
                    on_wake_immediate(entity, sources, now); internal hook
                    points reserved for P7-W09; signatures stable within P7.
Purpose and caller: single authoritative observation points so accounting
                    does not double-count; called exactly once per outcome by
                    B-1.
Inputs / outputs: see [P7-W09 design](../p7-w09-accounting-diagnostics/README.md)
                  §2 for the consumed record fields.
Preconditions / postconditions: hooks must not fail the block path and must
               perform bounded work (P0-W12 release-trimming rules apply).
State and ownership change: mutate accounting records owned by W09.
Concurrency/allocation context: IRQ-safe, allocation-free, no locks beyond
               W09's own record ownership.
Errors / failure guarantee: hook failure is an accounting bug, not a
               scheduling failure; it must not alter BlockOutcome.
Security/authorization checks: none.
Logic: call-through; no scheduling decisions.
Validation: P7-V19 coherence — one block event produces exactly one
            accounting increment.
```
