# P7-W07 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
preflight set, and the supporting file for its step; and it re-inspects the
current tracked tree (`git ls-files`). All inputs are assumed contracts
([01-pause-stop-fault-architecture.md](01-pause-stop-fault-architecture.md)
§2): confirm the W01 reconciliation record (`../p7-w01-entry-contract-reconciliation-record.md`,
when it exists) has not flagged them as blocked, and that no owner has
published a contradicting frozen contract since.

Stop and obtain direction instead of guessing when:

- the W02 transition set lacks any edge listed for L-1 — a W02 authority gap;
  record and stop;
- the P5 capability surface cannot express the required operation-level
  rights or denial classes — an A-1 gap; do not substitute role, VM-ID, or
  first-VM logic (ADR-013/051);
- implementing VM-pause completion appears to require waiting in a VM-exit or
  IRQ path — the model is wrong for the platform, stop and escalate; or
- implementation reaches for snapshot/migration, VM fault policy, management
  API, or restart semantics — scope violation (parent README exclusions).

## 2. Ordered implementation steps

### Step 1 — reconcile assumed contracts against the delivered P7 baseline

Target: implementation record (`../p7-w07-pause-stop-fault-record.md`,
created in this step).

Work: read the W02/W03/W04 sibling designs and the P4/P5 records actually
present; record per-contract status for L-1..W6-1 as satisfied, changed, or
absent/blocked. A changed contract that invalidates a decision in the parent
README stops this design (Architecture Change Request), not a local
adaptation.

Acceptance: per-contract status table exists with deltas and blockers named.

Failure/blocker: contradicting authority — stop per §1.

Evidence: implementation record.

### Step 2 — implement the pause/resume control paths

Target: scheduler control module — P-1, P-2, P-5, P-6 per
[02-code-contracts-pause-resume.md](02-code-contracts-pause-resume.md).

Work: implement the §3 decision table and the resume re-evaluation exactly.
Authorization is check-then-act: A-1 before any state inspection effect.
Wire the pause-pending generation check into the L-3 admission point (P-5)
and marker consumption into the exit path (P-2).

Suggested observation: host-side unit tests for the full §3 table and the
§4 resume re-evaluation (when the P0-W08 host testing baseline exists).

Acceptance: every PauseOutcome/ResumeOutcome value is produced by tests;
denials leave zero state (asserted by inspecting state before/after).

Failure/blocker: a missing L-1 edge or A-1 class stops per §1.

Evidence: implementation record; unit-test inventory.

### Step 3 — implement VM pause completion

Target: P-3, P-4 and the completion-detector update in P-2.

Work: implement the §5 model: generation set → member dispatch → exit-path
completion detection. Verify no code path spins, sleeps, or waits for a
remote pCPU; verify the bounded member scan.

Acceptance: after request_vm_pause acceptance, tests show no new admission of
any member and each running member reaching Paused; detect_vm_pause_completion
flips to Complete exactly when no member is Running.

Failure/blocker: any need to block awaiting completion is a §1 stop.

Evidence: implementation record.

### Step 4 — implement stop/fault containment

Target: F-1..F-5 per
[03-code-contracts-stop-fault.md](03-code-contracts-stop-fault.md), including
the exit-path disposition priority (F-4) replacing any ad-hoc ordering.

Work: implement the disposition priority as the single evaluation order at
the exit path; make the fault commit's write set reviewable (only the
faulting vCPU's scheduling state, accounting, diagnostics). Keep
commit_guest_fault free of capability checks and of panic paths.

Acceptance: interleaving tests (pause+stop+fault+block on one vCPU within one
exit) always produce exactly one consistent disposition; containment review
confirms the write set.

Failure/blocker: if F-0 classification cannot distinguish Guest faults from
Hypervisor invariant failures, stop — that is the P4-W06 boundary, not W07's.

Evidence: implementation record.

### Step 5 — integration pass and evidence

Target: verification record
(`../../verification/p7-w07-pause-stop-fault-verification.md`).

Work: run the validation matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md) as far as
prerequisites exist; record passed/failed/blocked/not-run per row with
commands, environment, timestamps, and reasons. QEMU rows without the P1/P4
baselines are recorded not run with the owning stage named.

Acceptance: every matrix row has a status; failures carry diagnosis; no
passing condition was weakened to pass.

Failure/blocker: a failed row is evidence of failure — record it; do not
adjust the design to make it pass.

## 3. Evidence destinations

- Implementation decisions, deltas, changed files:
  `../p7-w07-pause-stop-fault-record.md`.
- Command output, run/not-run status:
  `../../verification/p7-w07-pause-stop-fault-verification.md`.
- Neither file may claim W07 complete; completion evidence lives only in the
  verification record, only for what actually ran.

## 4. Ordering and review constraints

- Authorization ordering: A-1 check precedes every state effect in P-1, P-3,
  P-6, F-1; the fault path (F-2) is the single documented exception.
- Locks: lifecycle lock only inside L-1; no new lock order beyond P3-W06's
  baseline; markers are single-word atomics with the release/acquire pairings
  stated in the contracts.
- Bounded work: no waiting, spinning, or unbounded scans in VM-exit, IRQ, or
  hypercall contexts; member iteration bounded by fixed VM size.
- No new `unsafe` is authorized; touched `unsafe` stays within the existing
  audited P1/P3 boundaries with `SAFETY` commentary intact.
- New `unsafe`, ABI/public API, or dependency changes: none authorized; their
  appearance is a review failure.
