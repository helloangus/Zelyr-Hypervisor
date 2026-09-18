# P3-W10 Workflow, Validation, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W10 detailed design](README.md).

## 1. Ordered audit workflow (executed when prerequisites open)

### Step 1 — prerequisite gate

Target: the audit record's gate section.

Work: verify the prerequisites before any item is classified: W01–W09
deliverables exist as reviewed designs and, where the audit needs them,
implementation/verification records; P0–P2 records exist for the groups
they cover (currently only P0-W01/P0-W02 have records — the gate
anticipates the P1/P2 evidence arriving before the audit window).
Record the gate outcome per group.

Acceptance: every group is marked auditable or blocked-with-reason;
no group is silently skipped.

Failure/blocker: a group blocked by missing predecessor evidence is
entered per BL-2 — a visible closure blocker, not an empty pass
([01 §6](01-audit-framework-and-checklist.md)).

Evidence: audit record (created in this step).

### Step 2 — seed the item list

Target: audit record item list.

Work: enumerate candidate items per group from the group seeds
([01 §2](01-audit-framework-and-checklist.md)) plus the consuming
designs' ownership tables; every candidate gets a provisional Item ID.

Acceptance: each G1–G6 seed is represented or explicitly dismissed with
a reason; nothing is dropped silently.

Failure/blocker: a seed whose state cannot be located in the
implementation is a finding against the owning design (record exists but
state does not) — routed per RM-1.

Evidence: audit record.

### Step 3 — classify and evidence (per item)

Target: audit record per-item entries.

Work: for each item, determine owner, access contexts, and classification
under the §3 decision rules; attach citations and evidence; cross-read
the P0-W10 unsafe inventory for G7 candidates. Workflow states
(Pending-audit → Classified/Unclassified/Blocked) never appear in the
finalized record.

Acceptance: every finalized item carries a classification with the
required citations, or a blocker per §6.

Failure/blocker: a state fitting no class is a design conflict — raise
it (README authority rules); do not force-fit.

Evidence: audit record.

### Step 4 — G7 sweep

Target: audit record sweep section.

Work: run the discovery protocol ([01 §2.7](01-audit-framework-and-checklist.md))
over the tracked source tree; classify or merge every hit; record the
reviewed module list as completeness evidence.

Acceptance: module list covers the workspace; every hit accounted for.

Failure/blocker: an unaccountable hit (no owner, no design) is a finding
per RM-1.

Evidence: audit record.

### Step 5 — findings routing and ledger finalization

Target: audit record remediation section + blocker ledger.

Work: route every finding per RM-1..RM-5; apply BL-1..BL-5; verify
P2-ACR-01's ledger entry; mark the record final.

Acceptance: no finding without a named owner; ledger complete;
`Unclassified`/`Blocked` items all appear in the ledger.

Failure/blocker: pressure to "close clean" by reclassifying without
citations is the failure mode this step exists to prevent — the record
states what is true.

Evidence: audit record.

### Step 6 — closure review and handoff

Work: run the matrix below; confirm the handoff checklist; deliver the
ledger to W15 and the record reference into the P4 handoff content.
Record verification evidence in
`../../verification/p3-w10-smp-safety-audit-verification.md` only for
what was actually performed. The audit record and verification evidence —
never this design, and never the plan — are what P3-V10 reviews.

## 2. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W10-DV01 | Framework completeness | framework review | check groups against the task book's review-object list and the P3 designs' ownership tables | every named review object is in a group; G7 protocol defined | the audit's scope is total over P0–P3 mutable state; not that any item is classified |
| W10-DV02 | Item list seeded completely | seed review | per-group reconciliation of seeds vs record items | every seed represented or dismissed with reason | enumeration honesty; not classification correctness |
| W10-DV03 | Classifications are standard-cited | classification review | sample + full review of citations (AP/LOL ids, phase gate, privacy guarantees) per §3 | every classified item carries the required citations; no citation-free classification | classification checkability; not race-freedom of the code |
| W10-DV04 | Blocker rules effective | ledger review | apply BL-1..BL-5 to the finalized ledger; verify P2-ACR-01 entry | every unclassified/blocked item is on the ledger with named consumers | the acceptance mechanism of P3-V10; not that blockers are resolved |
| W10-DV05 | Findings routed; consumers integrated | routing review | every finding has one owner + RM label; integration map outcomes recorded | no orphan findings; map outcomes present | handoff readiness; not that remediations are done |
| W10-DV06 | Gate honesty | prerequisite-gate review | per-group auditable/blocked outcome against actual P0–P2/P3 evidence availability | no group audited without evidence; blocked groups named | audit validity; not predecessor completion |

Task-book trace: P3-V10 passes only when every reviewed mutable state has
a classification and unresolved items are visible closure blockers. W10
delivers the framework now and (when the gate opens) the executed record;
the record + verification evidence are the only acceptance surfaces.
Evidence states (passed / failed / blocked / not run) are recorded with
command, environment, and timestamp in
`../../verification/p3-w10-smp-safety-audit-verification.md`; the audit
record itself lives at `../p3-w10-smp-safety-audit-record.md`. Neither
file is created by this design.

## 3. Error, security, and observability model

- **Errors:** the audit's failure modes are process failures — a missed
  group, an uncited classification, a silent skip. The matrix and the
  blocker ledger are the controls; a mis-executed audit fails DV01–DV06
  review and is re-run, not patched.
- **Security:** none of this surface is guest-reachable or executable.
  The security-relevant contribution is indirect: the audit is where
  P0–P2 shared-state assumptions (allocator safety under SMP, console
  atomicity, diagnostic integrity) are forced to become explicit, which
  is a precondition for the isolation claims later stages make.
- **Observability:** the audit record is the observability artifact —
  item list, ledger, gate outcomes. It is consumed by W11 (instrumented
  surfaces), W13 (matrix scope), W15 (closure), and the P4 handoff.

## 4. Handoff checklist

Before handing W10 to a reviewer, provide:

- this design's review status (framework-only, until the gate opens) or,
  when executed, the audit record path and its gate outcome per group;
- DV01–DV06 evidence paths with run status, including explicit not-run
  entries while the prerequisite gate is closed;
- when executed: the blocker ledger contents (or its evidenced emptiness),
  the P2-ACR-01 ledger entry, the remediation routing list, and the
  `Architecture Change Request` / `ADR Required` labels with named
  owners;
- confirmation that the audit implemented no remediation, edited no
  P0–P2 contract, classified no P4+ state, and claimed no race-freedom;
- open items for W11/W12/W13/W15 and the P4 handoff — without resolving
  their contracts here.
