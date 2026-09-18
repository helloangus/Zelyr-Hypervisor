# P3-W10 SMP Safety Audit — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The audit's artifact groups, classification taxonomy, per-item
record schema, checklist structure, and acceptance rules required by
[P3-W10](../../plans/p3-w10-smp-safety-audit.md) — the design of the
audit, **not** audit results.  
**Owner/change context:** P3-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W10. It defines *how* the
P0–P2 and P3-foundation SMP-safety audit is performed and what its
artifact must contain, so that executing the audit is a bounded,
reviewable activity: the seven authoritative artifact groups it inventories
(logging/console, physical and heap/object allocation, platform
information, global registries, diagnostic state, and the catch-all), the
five-way classification taxonomy the plan and task book fix
(immutable-after-boot, CPU-local, atomic, lock-protected, boot-only), the
per-item record schema with its evidence and remediation fields, the
decision rules that make each classification checkable rather than
asserted, and the acceptance rule that an unclassified item is a visible
closure blocker. This document deliberately contains **no audit findings,
no classifications of actual state, and no completion claim**: the audit
record is produced when the audit runs, under
`../p3-w10-smp-safety-audit-record.md`, after its prerequisites
(W01–W09 deliverables and available P0–P2 implementation/verification
records) exist.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then
loads:

| Supporting file | Load it for |
|---|---|
| [01-audit-framework-and-checklist.md](01-audit-framework-and-checklist.md) | artifact groups, taxonomy decision rules, record schema, acceptance/blocker rules |
| [02-workflow-validation-and-handoff.md](02-workflow-validation-and-handoff.md) | ordered audit workflow, validation matrix, failure model, handoff checklist |

Before auditing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W10 plan → this
design → Coding Guidelines. Binding constraints:

- The plan fixes the taxonomy and the acceptance: classify every reviewed
  mutable state as immutable-after-boot, CPU-local, atomic,
  lock-protected, or boot-only, and "confirm each unclassified state
  blocks closure" (P3-V10). The taxonomy is therefore not this design's
  to extend — a state that fits none of the five classes is a design
  conflict to raise, not a sixth class invented locally.
- The plan's out-of-scope lines are binding: no claim that all code is
  race-free without evidence; no redesigning predecessor contracts
  without authority (a remediation that would change a P0–P2 contract is
  an Architecture Change Request to its owner, not a W10 edit); no
  expansion into VM/Stage-2/scheduler audits (P4 audits its own new
  state independently — recorded in the handoff).
- The audit consumes [P3-W06](../p3-w06-concurrency-synchronization/README.md)
  as the classification standard for the `atomic` and `lock-protected`
  classes (AP pattern ids, ladder class/rank citations) and
  [P3-W05](../p3-w05-smp-boot-synchronization/README.md)'s phase gate as
  the standard for `boot-only`; W01's anti-assumption invariants are
  review criteria for platform/topology state.
- The task book names the review objects: "logging/console, physical and
  heap/object allocation, platform information, global registries,
  diagnostic state, and other mutable infrastructure" — the artifact
  groups in [01 §2](01-audit-framework-and-checklist.md) follow that
  list exactly, plus the P3 foundations.

Classification:

- **Required** for W10 closure: the artifact-group definitions, the
  taxonomy decision rules, the per-item record schema, the
  evidence/blocker rules, the ordered audit workflow, and the
  audit-record acceptance structure (produced when the audit runs).
- **Reserved** with recorded triggers: a machine-checkable inventory
  format (trigger: an approved tooling design — the P3 record is a
  reviewable document, not a database); re-running the audit as a
  stage-gate automation (trigger: CI governance, P0-W20 owner); audit
  extensions to P3-W11 telemetry state beyond the group coverage
  (trigger: W11's design, if it lands after the audit window — the
  catch-all rule in [01 §2.7](01-audit-framework-and-checklist.md)
  handles the timing).
- **Out of Scope:** audit results or classifications (produced in the
  record when the audit runs); remediation implementation (owned by each
  finding's owner package); redesigning P0–P2 contracts (Architecture
  Change Request to the owner); VM/Stage-2/scheduler/guest state audits
  (P4+); performance or race-freedom proofs; unsafe-code audit *content*
  (P0-W10 governance owns the inventory; W10 consumes it as evidence).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inventory reviewed shared and local state with owner, access contexts, classification | [groups](01-audit-framework-and-checklist.md) §2, [schema](01-audit-framework-and-checklist.md) §4 | P3-V10 (W10-DV01, DV02) |
| Classify each item into the five classes | [taxonomy rules](01-audit-framework-and-checklist.md) §3 | P3-V10 (W10-DV03) |
| Identify remediation and unresolved blockers | [remediation/blocker rules](01-audit-framework-and-checklist.md) §5–§6 | P3-V10 (W10-DV04) |
| Integrate findings with synchronization, exceptions, telemetry, test, closure work | [integration map](01-audit-framework-and-checklist.md) §7; [workflow](02-workflow-validation-and-handoff.md) step 5 | W10 closure review (W10-DV05) |
| Review unresolved items against P0 unsafe governance and ADR boundaries | [escalation rules](01-audit-framework-and-checklist.md) §6 | W10 closure review (W10-DV05) |
| Confirm each unclassified state blocks closure | [acceptance rule](01-audit-framework-and-checklist.md) §6 | P3-V10 (W10-DV04) |
| Audit evidence without treating the plan as completion evidence | [workflow](02-workflow-validation-and-handoff.md) steps 3–4; matrix | P3-V10 (W10-DV01–DV04) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`):
P0 documentation scaffold only — no workspace, no sources, therefore no
mutable state exists to audit yet. Only P0-W01/P0-W02 have implementation
and verification records; P1/P2 have planning sets only. Sibling P3
designs W01–W09 exist as proposed designs on this branch; W11–W15 are
being prepared in parallel. This design therefore defines the audit's
structure now, and gates its execution on the prerequisites below —
auditing a repository with no implementation would produce an empty
record that proves nothing.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every reviewed mutable state has a classification | No implementation exists; no state to classify | The audit framework (groups, schema, decision rules) now; the executed record when prerequisites land | An audit without a fixed framework produces per-auditor classifications that cannot be reviewed | W10 (this design) | W10-DV01 (framework review); record later |
| P0–P2 infrastructure is within audit scope | P1/P2 have no implementation evidence; only P0-W01/W02 records exist | The prerequisite gate + the "blocked classification" rule for missing predecessor evidence | Auditing absent code cannot yield classifications; the gap must be a visible blocker, not a silent skip | W10 gate; P0–P2 owners (evidence) | W10-DV04 |
| Every classification is checkable, not asserted | No standard exists to check against | Decision rules citing W06 AP/LOL ids, W05 phase gate, W04 privacy guarantees per class | "It's fine" is not a classification; each class must name its proof obligation | W10 rules; W06/W05/W04 standards | W10-DV03 |
| Unclassified state blocks closure | No blocker mechanism exists | The blocker ledger section of the record schema with named consumers (W15 closure) | P3-V10's wording makes the ledger the acceptance surface | W10 schema; W15 consumes | W10-DV04 |
| P3 foundations are within scope | W01–W09 designs exist; implementations will follow | The P3-foundations group (G6) with its per-design item seeds | Excluding P3's own new state would audit predecessors but not the stage's own additions | W10 group; W01–W09 designs | W10-DV02 |
| Findings integrate with stage consumers | No integration path exists | The integration map (W06/W09/W11/W12/W13/W15) | Findings that no consumer receives are write-only documentation | W10 map | W10-DV05 |

No ledger row requires this design to classify actual state, fix tooling,
or redesign predecessor contracts; no new decision blocker is outstanding
here. The prerequisite gate itself is the recorded boundary that keeps the
audit honest.

## Resolved design decisions and their authority

1. **The audit is record-driven and per-item, with a fixed schema.** Every
   reviewed item gets one record: id, group, owner, source references,
   access contexts, classification, standard citations, evidence links,
   remediation owner, and blocker status. Rationale: P3-V10's acceptance
   ("every reviewed mutable state has a classification and unresolved
   items are visible closure blockers") is only checkable if items are
   individually enumerable — a prose summary cannot be audited.
2. **The five classes are fixed by the plan; each class has entry rules
   naming its proof standard.** `atomic` must cite a W06 AP pattern;
   `lock-protected` must cite a W06 ladder class/rank and the lock
   instance; `CPU-local` must show single-writer ownership or W04's
   privacy guarantees; `boot-only` must cite the W05 phase gate (or a
   single-threaded pre-release region) protecting every write;
   `immutable-after-boot` must show one publication point and no later
   writer. Rationale: checkability — a classification that names no
   standard is an assertion.
3. **The audit record is produced only after the prerequisite gate
   opens; missing predecessor evidence is a recorded blocker, not an
   empty pass.** P1/P2 currently have no implementation evidence; when
   the audit runs, each group's P0–P2 items are either auditable against
   real records or entered as `Blocked` with the missing-evidence reason
   — and `Blocked` items are closure blockers exactly like unclassified
   items. Rationale: the plan's "claiming all code is race-free without
   evidence" prohibition, applied to the audit itself.
4. **Remediation is routed, never performed, by the audit.** Each finding
   names its owner package and the design change required; W10's artifact
   is the routing document. A remediation that would alter a P0–P2
   contract or an ADR constraint is labeled `Architecture Change
   Request` (or `ADR Required`) with the owner named. Rationale: plan
   out-of-scope ("redesigning predecessor contracts without authority")
   and the guardrail that every mutable state has one owner.
5. **The catch-all group (G7) has a discovery protocol, not a vibe.** A
   documented sweep procedure (per-crate/module review of statics,
   atomics, locks, and global buffers, against the workspace the
   approved build design defines) bounds what "other mutable
   infrastructure" means, so coverage is reviewable. Rationale: the task
   book's "other mutable infrastructure" must be enumerable or the audit
   claim is unfalsifiable.
6. **P2-ACR-01 stays visible.** The unresolved P2 architecture item
   recorded by the P2 task book remains on the blocker ledger until
   authorized resolution, as an explicit carry-over item. Rationale:
   W01's design already routed it here; P3 closure must not lose it.
7. **The audit is compact by design (three files).** The plan's
   audit-shaped scope needs a framework file and a workflow/validation
   file beside the entry README — no per-module contract files, because
   W10 designs no code interface. Rationale: the skill's rule against
   splitting short designs for folder-making.

## Work breakdown and loading order

1. Read [01-audit-framework-and-checklist.md](01-audit-framework-and-checklist.md)
   for the artifact groups, taxonomy decision rules, record schema, and
   blocker rules — this is the audit's constitution.
2. Read [02-workflow-validation-and-handoff.md](02-workflow-validation-and-handoff.md)
   for the ordered workflow (including the prerequisite gate), the
   validation matrix, and the handoff checklist.
3. When the prerequisites are met, execute the workflow and produce
   `../p3-w10-smp-safety-audit-record.md`; record evidence in
   `../../verification/p3-w10-smp-safety-audit-verification.md` only for
   what was actually performed. Neither file is created by this design,
   and no document may claim W10 complete.

## Explicitly excluded interfaces

No code interface, type, function, crate, or tool is designed or
authorized by W10 — the plan's out-of-scope line on "detailed algorithms
or Rust APIs" belongs to its parent stage rule set, and this package's
deliverable is a documented audit framework plus (when executed) records.
No remediation is implemented by W10; no P0–P2 contract is edited; no
P4+ state is audited; no race-freedom claim is available from any W10
artifact; the P0-W10 unsafe inventory is consumed as evidence, not
restated. An audit record that contains a sixth classification, a
silent skip, or a remediation implemented in-repo by the auditor is a
scope violation to stop at review.

## Downstream handoff

- **W11** receives the classified telemetry/diagnostic items and the
  unclassified-item ledger as input to the observability design (what
  W11 may instrument, and what contention/event state exists).
- **W12** receives the audit's per-item access contexts as the stress
  design's shared-state target list (what is worth stressing).
- **W13** receives the blocker ledger and the audit status as
  regression-scope context (a blocked group narrows what the matrix may
  claim).
- **W15** receives the blocker ledger as a mandatory closure input: P3
  cannot close with open W10 blockers, per P3-V10.
- **W14/P4** receive the completed audit record as part of the P4 handoff
  and the explicit statement that P4 audits its new VM/Stage-2 state
  independently (plan handoff sentence).
- **P0–P2 owners** receive routed remediation findings and any
  Architecture Change Request labels through their own closure
  processes — W10 routes, does not fix.
