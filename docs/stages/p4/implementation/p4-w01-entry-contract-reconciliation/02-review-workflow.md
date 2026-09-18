# P4-W01 Review Workflow, Validation, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before reviewing, the implementer verifies it has loaded the documents named
in the parent README and the [plan index reading order](../../plans/README.md).
Useful read-only discovery: `git ls-files` (enumerate the real tree),
`git log --oneline` (fix the reviewed revision), and a check of which
`docs/stages/p*/implementation/` and `docs/stages/p*/verification/` paths
contain records versus `.gitkeep` only.

Stop and record instead of guessing when any of the following occurs:

- an upstream contract document contradicts the ADR or the P4 task book —
  classify `Architecture Change Request` (or `ADR Required` if the baseline is
  implicated); do not reinterpret either side locally;
- an upstream verification record exists but cannot be linked from the matrix
  row it should support — record the traceability gap; do not assume the
  evidence covers more than it states;
- a P4 sibling detailed design (`../p4-w02-stage2-address-space/README.md`
  through `../p4-w09-closeout-p5-handoff/README.md`) names a prerequisite this
  matrix lacks — add the row per [01 §2](01-reconciliation-contract.md); do
  not leave the dependency unreconciled; or
- executing the review appears to require implementing or repairing any P0–P3
  mechanism — that is out of scope by definition; record the row as the
  classification requires and stop.

The review's own failure mode is an incomplete or unverifiable claim: any
matrix row without a path or an explicit absence marker fails the P4-V01
review and must be completed, not waived.

## 2. Ordered implementation steps

### Step 1 — fix the review baseline and inspect the sources

Target: W01 implementation record (`p4-w01-entry-contract-reconciliation-record.md`,
created in this step).

Work: record the repository revision, date, and reviewer context. Execute the
inspection pass over the ADR, the P4 task book and plan index, the P0–P3 task
books and plans, and every existing implementation/verification record, and
write one inspection-list row per source with its path and what it states.

Suggested observation: `git ls-files docs/stages` to enumerate stage
documents; verify each claimed path exists in the tree.

**Acceptance:** every source named in [01 §2](01-reconciliation-contract.md)
R01–R21 has an inspection row with a real path and revision.  
**Failure/blocker:** a missing expected source is itself a finding to carry
into the matrix as `absent`; it does not stop the review.

### Step 2 — complete the reconciliation matrix

Target: reconciliation matrix section of the implementation record.

Work: for each row of [01 §2](01-reconciliation-contract.md), fill `contract
state`, `evidence state`, `assumed status`, and `gap label` from the actual
tree. Cite exact paths. Add consumer-specific rows where W02–W09 designs name
dependencies the minimum set missed.

**Acceptance:** no empty status cells; every citation resolves; every P4
package (W02–W09) appears in at least one row.  
**Failure/blocker:** a row that cannot be resolved either way is recorded as
`absent` with a gap classification, never left blank.

### Step 3 — bind the P4 entry-assumption ledger

Target: assumption-ledger section of the implementation record.

Work: record each [01 §3](01-reconciliation-contract.md) item A1–A9 as bound
for P4, re-checking each authority citation against the current ADR and task
book text. Where a sibling P4 design re-owns an item, note the re-ownership
and the design path.

**Acceptance:** every A-item is recorded with its authority and status; no
item is silently narrowed.  
**Failure/blocker:** an item whose authority text changed between design and
review is re-raised as a conflict per [01 §4](01-reconciliation-contract.md).

### Step 4 — classify gaps and conflicts

Target: gap and conflict list section of the implementation record.

Work: for every non-`delivered` row, apply the §4 classification rules. Verify
that P2-ACR-01 appears as `ADR Required` and remains unresolved, that the A9
scenario-provenance open question is recorded, and that no classification
resolves a labeled item locally.

**Acceptance:** a complete labeled gap list; zero unlabeled gaps; zero local
resolutions.  
**Failure/blocker:** disagreement about a label is recorded as an open
question with both citations; the reviewer does not force a label.

### Step 5 — record the accepted contract set for W02–W09

Target: handoff section of the implementation record.

Work: per consumer package, state the accepted rows (with assumed-contract
status), the gaps that block which acceptance evidence, and the inherited §4
failure boundary. This is the artifact the consumer designs cite.

**Acceptance:** each of W02–W09 can determine, from this section alone, which
inputs it may rely on and what stays blocked.  
**Failure/blocker:** a consumer whose needed rows are all blocked is recorded
as a fully blocked consumer — that is a truthful handoff, not a review
failure.

### Step 6 — execute the P4-V01 review and record evidence

Target: verification record
(`../../verification/p4-w01-entry-contract-reconciliation-verification.md`).

Work: run the validation matrix below. Record per-check status (passed /
failed / blocked / not run) with the observed condition and date. Then run the
[handoff self-review](#4-handoff-self-review) and close the record. Completion
is claimed only in the verification record, only for what was actually
reviewed.

**Acceptance:** P4-V01 evidence exists: the linked review with per-prerequisite
evidence/consumer/block status, all classifications recorded, no undocumented
assumption.  
**Failure/blocker:** a failed check is recorded as failed with the missing
item; the review is completed only when the check passes on the recorded
revision.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W01-DV01 → P4-V01 | matrix completeness review | inspect the executed matrix against [01 §2](01-reconciliation-contract.md) R01–R21 plus consumer-added rows | every row has contract/evidence/assumed status and ≥1 consumer; all citations resolve at the recorded revision | the entry boundary is enumerable; not that any upstream mechanism works |
| W01-DV02 → P4-V01 | assumption-ledger review | compare recorded A1–A9 items against ADR/task-book text | every item bound with authority; no undocumented P4 assumption remains in W02–W09 designs' inputs | the P4 interpretation baseline is explicit; not that consumers actually complied (checked at their reviews) |
| W01-DV03 → P4-V01 | gap classification review | re-derive labels per [01 §4](01-reconciliation-contract.md) from the recorded rows | every gap labeled; P2-ACR-01 visible and unresolved; no local resolutions | conflicts are surfaced honestly; not that they are resolved |
| W01-DV04 → P4-V01 | consumer handoff review | read the handoff section as each of W02–W09 | each consumer can name its accepted rows, blocked acceptances, and failure boundary without inventing inputs | handoff readiness; not that the consumers are implemented |
| W01-DV05 → P4-V01 | repeatability check | re-run two matrix rows end-to-end (for example R09 and R17) from a clean reading | the second execution reproduces the recorded status deterministically from the tree | the review procedure is stable; not upstream correctness |
| W01-DV06 → P4-V01 | scope purity review | search the record for implementation claims, upstream repairs, or mechanism decisions | none present; record contains review content only | W01 stayed a reconciliation package; not P4 runtime correctness |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with the observed condition, date, and reason. No validation here proves any
P4 runtime behavior, and none may be reported as doing so.

## 4. Handoff self-review

Before handing W01 to a reviewer, verify:

- the goal-to-baseline ledger rows in the entry README all have executed
  counterparts in the implementation record;
- every missing upstream artifact is either an assigned prerequisite row with
  assumed-contract status or a labeled gap — none is silently dropped;
- no inferred crate, file tree, API, target, or board behavior exceeds source
  authority; the record cites paths, not recollections;
- the record states the reviewed revision and date and can be re-run;
- W02–W09 consumer sections exist and name their blocked acceptances;
- P2-ACR-01 and the A9 provenance open question are present and unresolved;
- the record contains no completion claim, and evidence locations point to
  `../p4-w01-entry-contract-reconciliation-record.md` and
  `../../verification/p4-w01-entry-contract-reconciliation-verification.md`
  without creating or pre-filling either beyond actual content.

## 5. Error, security, and observability model

W01 adds no runtime error model, guest-input path, or telemetry stream. Its
"failure" surface is the truthfulness of the review: a wrong `delivered`
status, an absorbed gap, or a silently resolved conflict is the package's
security defect, because W02–W09 would then build on undocumented assumptions.
Observability is the record itself: revision, per-row status, labels, and
run/not-run evidence are the only accepted outputs. The security posture is
inherited, not created: W01 re-binds the ADR untrusted-Guest and layering
invariants (ledger items A2, A7) as entry constraints and confirms they are
carried into the consumer designs.

## 6. Handoff checklist

Provide to the P4 stage reviewer and the W02–W09 design owners:

- the exact reviewed revision, date, and changed/created files (implementation
  record; verification record when evidence exists);
- DV01–DV06 statuses including explicit not-run entries;
- the accepted contract set per consumer package, with assumed-contract
  statuses;
- the labeled gap list, including P2-ACR-01 (`ADR Required`) and the A9
  scenario-provenance open question;
- confirmation that no P0–P3 document was modified, no P4 mechanism was
  designed, and no runtime or validation claim was made.
