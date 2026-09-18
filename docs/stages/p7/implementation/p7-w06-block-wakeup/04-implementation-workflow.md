# P7-W06 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
preflight set, and the supporting file for its step; and it re-inspects the
current tracked tree (`git ls-files`). All P0–P6 inputs are assumed contracts
([01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §2): the
implementer does not verify upstream implementations (they do not exist yet),
but must confirm that the W01 reconciliation record
(`../p7-w01-entry-contract-reconciliation-record.md`, when it exists) has not
flagged the required inputs as blocked. If any assumed contract's owner has
since published a contradicting frozen contract, stop and raise the conflict —
do not adapt silently.

Stop and obtain direction instead of guessing when:

- the W02 lifecycle contract lacks a transition this path needs
  (`Blocked→Runnable`, `Blocked→Paused`, `Paused→Blocked`) — L-1 gap, record
  and stop;
- the P6 timer/vIRQ surfaces cannot express the deadline fold or the wake
  adapters — P-2/P-3/P-4 gap, blocked prerequisite;
- implementing the protocol appears to require new synchronization primitives
  outside P3-W06's handed-down rules — raise for design review; or
- implementation seems to require a wait-queue structure, a wakeup API, or
  P6 event-delivery changes — scope violation (see parent README exclusions).

## 2. Ordered implementation steps

### Step 1 — reconcile assumed contracts against the delivered P7 baseline

Target: no code; the implementation record
(`../p7-w06-block-wakeup-record.md`, created in this step).

Work: read the W02/W05/W08 sibling designs and the P4/P6 records actually
present, and confirm each assumed contract L-1..C-1 is either satisfied,
changed (record the delta and stop if it invalidates this design), or absent
(blocked prerequisite). Record per-contract status.

Acceptance: the record lists every assumed contract with status and, for
changed contracts, the reconciled reading.

Failure/blocker: a contradicting contract is an Architecture Change Request,
not a local adaptation.

Evidence: implementation record.

### Step 2 — implement the per-vCPU wake-event state and block path

Target: scheduler module — wake-event state, `BlockExitHint`/`BlockOutcome`
(B-3), `poll_blocking_eligibility` (B-2), `try_block_current_vcpu` (B-1), and
the accounting hook calls (B-5 stubs if W09 is not yet present).

Work: implement per [02-code-contracts-block-path.md](02-code-contracts-block-path.md).
Keep the path allocation-free and bounded; wire the exit path so blocking-class
exits (P-1) reach B-1 and nothing else does. Do not implement exit
classification — consume P4's.

Suggested observation: host-side unit tests for B-1/B-2 outcomes under the
P0-W08 host testing baseline (when present).

Acceptance: every B-1 outcome is reachable in host tests; no `unsafe` beyond
the audited P1/P3 boundaries this path already calls into; the two-phase
ordering in §5 of [01](01-block-wakeup-architecture.md) is preserved
(reviewer-checkable pairing comments).

Failure/blocker: an unreachable outcome means an assumed seam is missing —
stop per §1.

Evidence: implementation record; unit-test inventory.

### Step 3 — implement the deadline fold and timer adapter

Target: B-4 `fold_deadline_on_block`, W-3 `on_pcpu_deadline_irq`.

Work: implement per the contracts, strictly through the P6-W05 arm/rearm
surface. Enforce the idle-refusal rule when folding fails. Bound the home-set
scan and document the bound.

Acceptance: fold/idle-refusal behavior matches B-4; a simulated deadline IRQ
wakes exactly the due home vCPUs.

Failure/blocker: if P-3's surface cannot express "earliest of existing
obligations", stop — that is a P6 contract gap, not a local hack.

Evidence: implementation record.

### Step 4 — implement the wakeup path and adapters

Target: W-1, W-2, W-4, W-5, W-6, W-7 exclusion table.

Work: implement per [03-code-contracts-wakeup-path.md](03-code-contracts-wakeup-path.md).
Implement W-1's full outcome table; ensure enqueue (S-2) and reconsideration
(S-3) calls are made exactly once per successful wake. If W05/W08 seams are
absent at implementation time, code against their assumed signatures and mark
the call sites as pending their records.

Acceptance: the exclusion table passes a state-by-state test; concurrent
wake/block interleaving tests (bounded, host-side) show no lost wake and no
duplicate run under the test's interleavings.

Failure/blocker: a race the protocol cannot close under the S-1 rules is a
design defect — stop and escalate; do not add ad-hoc sleeps or retry loops.

Evidence: implementation record.

### Step 5 — integration pass and evidence

Target: verification record
(`../../verification/p7-w06-block-wakeup-verification.md`).

Work: run the validation matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md) to the extent its
prerequisites exist; record run/failed/blocked/not-run per row with commands,
environment, and timestamps. QEMU rows require the P1/P4 QEMU baselines;
without them they are recorded not run, not improvised.

Acceptance: every matrix row has a status; not-run rows name their owner
package and blocker.

Failure/blocker: a failed row is recorded as failed with diagnosis; do not
weaken a passing condition to pass.

## 3. Evidence destinations

- Implementation decisions, deltas, and changed files:
  `../p7-w06-block-wakeup-record.md`.
- Command output, run/not-run status: `../../verification/p7-w06-block-wakeup-verification.md`.
- Neither file may claim W06 complete; completion evidence lives only in the
  verification record and only for what actually ran.

## 4. Ordering and review constraints

- Locks: only P3-W06-sanctioned primitives; the lifecycle lock is taken only
  inside L-1; no nested lock ordering beyond P3-W06's baseline is introduced.
- IRQ context: W-1/W-3/W-4 paths must stay within the bounded-work rules of
  the Coding Guidelines; no allocation, no waiting, no unbounded scans.
- No `unsafe` is introduced by W06 logic itself; any `unsafe` touched is the
  existing audited arch/HAL boundary and must keep its `SAFETY` commentary
  intact.
- New `unsafe`, ABI changes, or dependency changes: none are authorized by
  this design; their appearance is a review failure.
