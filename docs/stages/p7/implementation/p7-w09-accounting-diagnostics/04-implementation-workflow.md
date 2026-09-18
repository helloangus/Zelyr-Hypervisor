# P7-W09 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
preflight set, and the supporting file for its step; and it re-inspects the
current tracked tree (`git ls-files`). All inputs are assumed contracts
([01-accounting-model.md](01-accounting-model.md) §2): confirm the W01
reconciliation record (`../p7-w01-entry-contract-reconciliation-record.md`,
when it exists) has not flagged them as blocked, and that no owner published
a contradicting frozen contract since.

Stop and obtain direction instead of guessing when:

- the P0-W12/W13 governance surfaces are not implementable yet — declare the
  events, block emission, record the prerequisite ([C-5](03-code-contracts-trace-diagnostics.md));
  never emit ad-hoc strings;
- a producer design changed a hook signature or added a transition without a
  hook — reconcile with that design, never count by re-implementing its
  logic;
- implementing observability appears to require a transport, buffer, backend,
  or management surface — scope violation (parent README exclusions); or
- the monotonic time basis (T-3) cannot provide same-critical-section
  reading pairs — coherence rule 4 is unimplementable; stop and escalate.

## 2. Ordered implementation steps

### Step 1 — reconcile assumed contracts and the hook inventory

Target: implementation record
(`../p7-w09-accounting-diagnostics-record.md`, created in this step).

Work: read the W02–W08 sibling designs and the P0/P3/P4/P5/P6 records
actually present; record per-contract status for T-1..T-7 and H-1..H-5 as
satisfied, changed, or absent/blocked. Confirm every hook in the §6 inventory
exists in the delivered producer designs or mark the gap as blocked.

Acceptance: per-contract and per-hook status table with deltas and blockers.

Failure/blocker: a missing producer hook is a coverage gap (P7-V19 untestable
for that event class) — record and stop that row's implementation.

Evidence: implementation record.

### Step 2 — implement the record types and update contracts

Target: A-1..A-5 per
[02-code-contracts-accounting.md](02-code-contracts-accounting.md), embedded
in the object owners' structures.

Work: implement records with typed IDs, checked arithmetic, and the
single-writer update paths; wire the hook bodies exactly once per producer
outcome; verify no update site exists outside the contracts.

Suggested observation: host-side unit tests replaying randomized hook
histories against record invariants (when the P0-W08 host testing baseline
exists).

Acceptance: property tests show monotonic accumulators consistent with hook
histories; a reviewer can locate zero out-of-contract update sites.

Failure/blocker: T-3 reading-pair gap — stop per §1.

Evidence: implementation record; test inventory.

### Step 3 — implement trace events and the reason vocabulary

Target: C-1, C-2 per
[03-code-contracts-trace-diagnostics.md](03-code-contracts-trace-diagnostics.md).

Work: fix the `SwitchReason` mapping against the delivered F-4 disposition
set; declare the event set and register it through the P0-W13 governance
review (C-5); wire emissions to the P0-W12 channels with the declared
trimmability classes. If the governance mechanism is not yet implementable,
declare-only and record the block.

Acceptance: every disposition maps to one reason; every emission carries the
required identity/reason/time fields; no unregistered string events.

Failure/blocker: governance unavailable — declare-only per §1.

Evidence: implementation record; registration outcome.

### Step 4 — implement the diagnostic snapshot and failure report

Target: C-3, C-4.

Work: implement the fixed-capacity snapshot writes inside the existing hooks
and the deterministic report renderer; verify the report contains the full
P7-V21 element list; verify no Guest-controlled bytes can reach it (enumerate
every field's source).

Acceptance: fault-injection tests produce complete, ordered, deterministic
reports; the write set of the snapshot stays within the producer hooks.

Failure/blocker: a required P7-V21 element without a source record means the
producer coverage is incomplete — back to step 1, record and stop.

Evidence: implementation record.

### Step 5 — governance-compatibility review and evidence

Target: verification record
(`../../verification/p7-w09-accounting-diagnostics-verification.md`).

Work: run the validation matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md) as far as
prerequisites exist; record passed/failed/blocked/not-run per row with
commands, environment, timestamps, and reasons. Counter/trace rows requiring
a running scheduler need the producer packages' baselines; without them,
record not run with the owning package named.

Acceptance: every matrix row has a status; failures carry diagnosis; the
review confirms P0-W12/P0-W13 compatibility and the no-crash-dump boundary.

Failure/blocker: a failed row is recorded with diagnosis; no weakening.

## 3. Evidence destinations

- Implementation decisions, deltas, changed files:
  `../p7-w09-accounting-diagnostics-record.md`.
- Command output, run/not-run status:
  `../../verification/p7-w09-accounting-diagnostics-verification.md`.
- Neither file may claim W09 complete; completion evidence lives only in the
  verification record, only for what actually ran.

## 4. Ordering and review constraints

- Hot-path cost: record updates are bounded, allocation-free, and single-
  writer; trace emissions follow the channel contract and may defer, but
  never block the producer path.
- Layering: no transport, encoding, or backend code; no management surface;
  channel use strictly through the P0-W12-governed API.
- Safety: no Guest-controlled bytes in records, events, or reports; no Host
  pointer disclosure (P5-W07 review rule reused as policy here).
- No new `unsafe` is authorized by W09; touched `unsafe` stays within
  existing audited boundaries with `SAFETY` commentary intact.
- New `unsafe`, ABI/public API, dependency, or telemetry-channel changes:
  none authorized; their appearance is a review failure.
