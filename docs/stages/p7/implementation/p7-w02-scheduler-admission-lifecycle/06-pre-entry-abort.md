# P7-W02 post-admission, pre-entry abort contract

**Status:** Approved companion design (project owner) v0.1, 2026-10-02; implementation and evidence
pending. **Parent:** [W02](README.md). **Consumer:**
[W04 switch sequence](../p7-w04-preemption-context-switch/04-code-contracts-context-switch.md).
**Producer:** [P4-W10](../../../p4/implementation/p4-w10-multivm-stage2-handoff/README.md).
The project owner explicitly confirmed “我确认批准” on 2026-10-02; this approves
this companion and its W02/W04/W05 binding edits, not every unrelated P7 design.
See the [approval record](../../../../testing/documentation-audit/design-approval.md#owner-approval).
This completes the P7-DISPATCH-UNWIND design approval, not its runtime gate.

## 1. Authority and identity

W02 remains the sole lifecycle-engine/current_vcpu writer. W04 owns hardware
preparation order and cleanup orchestration; P4/P6 own their completion facts;
W05 owns queue insertion and current eligibility/control-request handling.
There is no new vCPU lifecycle value and no synthetic Guest exit.

A successful admit_for_entry atomically commits Dispatch→Running,
current_vcpu=Some(vcpu), and an Admitted attempt record under the owner's
short critical section. EntryPermit is non-Copy/non-Clone and binds
(CPU, vCPU identity/generation, non-wrapping admission epoch). The record lives
in bounded per-CPU storage; exhausted epochs reject admission. Other lifecycle
observers use the same owner synchronization, so cannot observe half a commit.
This attempt phase is transaction metadata, not a competing run-state boolean.

`Admitted -> Entering -> Entered -> Exited` is the successful path;
`Admitted -> Aborting -> Aborted` is the safe pre-entry failure path.
Dropping a permit does not clear Running/current_vcpu or recycle the attempt.
The mechanism marks Entering only after all fallible preparation/entry-lease
checks succeed and immediately before the architecture entry operation. From
Entering, no ordinary abort is allowed: an unknown entry outcome is fail-stop,
or a proved actual exit follows the normal exit path. No post-Guest-use state
can be rolled back by claiming the permit was unused.

The existing lifecycle cell lock remains a leaf. The owner critical section is
local IRQ masking plus that one cell lock, not a new outer lock. Implement a
private engine-owned commit helper that validates the existing transition table
and updates cell/attempt/current slot before releasing the cell guard; it must
not recursively call lock-taking try_transition. The public/internal standalone
try_transition path keeps its original lock contract. No arbitrary callback or
other lock is allowed inside this helper. Remote diagnostic snapshots read an
owner-published generation and revalidate cell/slot under the same cell protocol;
a raw unlocked slot read is not a coherent lifecycle assertion. Control paths
cannot transition an admitted Running cell without its owner attestation.

## 2. Cleanup bundle and operation

`abort_admission(permit, SafePreEntryCleanup) -> AbortedCandidate` is called on
the admitted CPU, with no queue/data lock held and Guest never entered.
The cleanup bundle binds the attempt epoch and contains private producer facts:

- P4-W10 Rejected or Restored, or explicit successful restoration/detachment after
  a previously successful installation; no unused Stage-2 EntryLease remains.
- P6 deadline cancelled/not armed, prepared virtual timer/LR state withdrawn or
  restored to its documented safe boundary, and no pending preparation retaining
  the incoming candidate. Each subsystem marks NotPrepared only if its step was
  never reached; a guessed boolean cannot replace completion.
- W04 context-preparation storage has no unretired consumer reference and the
  mechanism did not cross Entering. The outgoing Guest's prior exit stays final.

W04 obtains these facts through producer cleanup APIs in reverse preparation
order. Missing or indeterminate cleanup preserves the attempt/current slot and
resources, emits a terminal diagnostic and stops this dispatch path. It never
requeues, fabricates Guest Faulted or clears the slot to make the scheduler run.

Under one owner critical section, abort rechecks exact permit/epoch, Admitted,
current_vcpu and Running; reserves Aborting, invokes the existing lifecycle
engine's Deschedule event in a pre-entry-abort context, clears the slot,
and commits Aborted. Any identity/state mismatch is invariant failure with no
partial state update. Dropped/duplicate/foreign permits cannot abort another
attempt. Private transition engine failure after preflight is fatal, not a
recoverable partial-clear path. Release the critical section before queue work.

## 3. Control races and accounting

AbortedCandidate transfers one candidate to W05, not a queue membership promise.
Pending Stop/Pause/Fault requests use W07's normal owner transition path before
W05 considers enqueue; they cannot directly mutate an Admitted Running cell.
W05 rechecks lifecycle, placement eligibility, scheduler mode and queue membership
under its existing queue protocol, then inserts at most once or hands the
candidate to its control owner. Concurrent requests arriving after the check use
that same protocol. No unlocked predicate alone authorizes enqueue.

Counters distinguish admitted attempts, actual Guest entries, safe pre-entry
aborts and actual exits. An abort increments neither Guest execution time nor
exit count. Identity/epoch trace pairs tie GateAdmitted to exactly Entered or
Aborted/terminal failure; no lost candidate, duplicate Running or duplicate queue
member is accepted. Complete_exit retires only actual Entered execution.

## 4. Implementation and validation

Implement the bounded attempt/permit first, then atomic gate/abort transitions,
then W04 cleanup bundle binding and W05 queue handoff. Do not patch lifecycle
fields from the switch mechanism. Existing W02 admission/exit contracts consume
this attempt record; P4/P6 implementation evidence is needed for real cleanup.

Required Host tests: failure at every preparation step, duplicate/foreign/stale
permit, epoch exhaustion, stop/pause/placement race at each boundary, failure
before and after Entering, missing P6 cleanup, P4 Indeterminate, and a previously
completed outgoing exit. Assert engine/current-slot consistency, exact attempt
accounting and one queue membership. P7-V09/V11 plus W02 admission validation
must exercise real producer failures in the declared QEMU profile; model-only
results cannot close P7-DISPATCH-UNWIND. No production tests ran for this design.
