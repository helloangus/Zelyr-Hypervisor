# P3-W15 Stage Documentation and Acceptance — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The coherent, separated P3 planning/traceability/evidence/
closure/handoff navigation and review package required by
[P3-W15](../../plans/p3-w15-stage-documentation-acceptance.md).  
**Owner/change context:** P3-W15 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W15. It defines the
authoritative artifact groups P3 must keep coherent (task book, plans,
implementation designs and records, verification records, the P4 handoff
contract), the checks that make P3-V15 decidable (scope coherence, link
resolution, plan coverage, evidence-location truthfulness, exit
traceability, unperformed-work wording), and the closure-review package
structure that assembles the stage's evidence *only when that evidence
actually exists*. It deliberately does **not** fabricate or anticipatorily
write verification evidence, does not declare P3 complete (the task book
§7 rule: planned documents alone are not evidence of completion), does
not substitute this package for any implementation-level design, and does
not design the closure *outcome* — it designs the acceptance *apparatus*.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) (this is
documentation work; the guide's completion-reporting rules are the ones
this package operationalizes for the stage). It then loads only the
linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, prerequisite boundaries, resolved decisions |
| [02-acceptance-artifacts-and-checks.md](02-acceptance-artifacts-and-checks.md) | artifact groups, ownership table, the C1–C8 checks, traceability matrix construction |
| [03-workflow-and-review.md](03-workflow-and-review.md) | ordered workflow, validation matrix, error model, handoff checklist |

This is documentation-governance work, so the logical modules are
authoritative artifact groups and checks, not code; no code interface is
authorized (explicit statement in the entry README's excluded-interfaces
section).

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W15 plan → this
design → Coding Guidelines. Binding constraints:

- The task book §5 (P3-V01–V15), §6 (exit criteria and P4 handoff
  contents), and §7 (completion review questions) are the normative
  targets this package reconciles; P3-V15 requires coherent scope, links,
  plan coverage, evidence locations, and exit traceability without a
  premature completion assertion, and P3 closes only when P3-V01–V15
  have real evidence.
- The documentation layout rules (`docs/README.md`) fix the stage layer
  separation: task book (what), plans (approved design at planning
  level), implementation/ (designs, records, traceability),
  verification/ (evidence). W15 enforces the separation; it never merges
  layers to make navigation easier.
- The plan's out-of-scope list ("fabricating verification evidence,
  declaring P3 complete before all validation evidence exists,
  substituting this documentation package for implementation-level
  design") is operationalized as the honesty rules in
  [02](02-acceptance-artifacts-and-checks.md) §4.
- W15 consumes the other fourteen packages' outputs; it owns no other
  package's content. Defects found in upstream documents are reported to
  their owners (recorded as findings), not fixed here.

Classification:

- **Required** for W15 closure: the artifact-group ownership table; the
  checks C1–C8 with their evidence; the outcome-to-plan-to-validation
  traceability matrix; the evidence-location map; the navigation
  assembly (stage README/implementation index truthfulness as far as
  W15-owned); the open-issue register assembly; and the closure-review
  package *template* and its assembly procedure (executed only when the
  underlying evidence exists).
- **Reserved** with recorded triggers: closure-review execution and any
  `DONE`-adjacent status (trigger: P3-V01–V15 all evidenced); a
  stage-closure meeting/review-body process (trigger: a governance
  decision outside this package); post-closure documentation maintenance
  (trigger: P4 handoff feedback via the W14 contract lifecycle).
- **Out of Scope:** writing or editing implementation records or
  verification evidence (their packages own them); designing or
  re-designing any mechanism; the P4 handoff contract's content (W14's);
  editing plans or the task book to resolve coverage gaps (findings are
  reported); any completion claim; guest/P4 documentation.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Assemble required stage documents, plan links, implementation/verification locations, P4 handoff references | [artifact groups](02-acceptance-artifacts-and-checks.md) §2, evidence-location map §5 | P3-V15 (W15-DV02) |
| Reconcile every required task-book outcome with exactly one plan and validation path | traceability matrix construction §3 | P3-V15 (W15-DV03) |
| Review statuses, scope boundaries, dependency acyclicity, unperformed-work wording | checks C1–C4, C6 §4 | P3-V15 (W15-DV04) |
| Collect documentation-review evidence; record open issues | evidence rules §6, register §7 | P3-V15 (W15-DV05) |
| Produce the closure-review package only when evidence exists | package template and assembly gates §8 | P3-V15 (W15-DV06); execution deferred until P3 evidence exists |
| Keep P3 planning/traceability/evidence/closure/handoff coherent and separate | separation rules §4 | W15 closure review (W15-DV01/DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P3 has a task book, a plans index with fifteen plans, and
proposed implementation designs for W01–W05 in this worktree (W06–W10
parallel); there are no P3 implementation records, no P3 verification
records, and no P4 handoff contract artifact yet. The stage README and
implementation index are minimal or `.gitkeep` placeholders (the stage
implementation index is coordinator-owned and is not modified by this
design). Each ledger row below states the missing foundation the plan
outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Planning, traceability, evidence, closure, handoff are coherent and separate | Designs/plans exist; no records, no closure package; separation is currently implicit | The artifact-group ownership table fixing which document owns which statement per layer | "Coherent and separate" must be checkable: an ownership table makes every statement's home decidable | W15 (this design) | W15-DV01 |
| Links resolve; statuses truthful | Sibling designs are untracked work-in-progress; cross-links are dense | The C-checks over links and status headers, runnable against the actual tree | P3-V15 names links and statuses; a link check over the real tree is the only honest evidence | W15 | W15-DV02/DV04 |
| Every task-book outcome maps to exactly one plan and validation path | Task book §3 has 15 outcomes and §4/§5 validation rows; no reconciliation exists | The traceability matrix (outcome → plan → design → record → validation) as a concrete artifact | "Exactly one" is a set property; only a constructed matrix can show it | W15 | W15-DV03 |
| Evidence locations recorded without fabricating evidence | No P3 verification records exist | The evidence-location map with per-package expected paths and an explicit exists/absent column | Recording where evidence *will* live, marked absent until it exists, is the honest form; creating empty records would fabricate structure-as-evidence | W15 (map); producing packages (evidence) | W15-DV02/DV05 |
| Unresolved issues recorded | P2-ACR-01 and design seams (W06–W10 parallel) are open | The open-issue register assembled from upstream records and the W14 contract's register | Task book §6 requires unresolved issues in the handoff; W15 assembles the stage-wide view | W15 (assembly); owners (resolution) | W15-DV05 |
| Closure-review package produced only when evidence exists | Cannot be produced today | The package template with assembly gates (all P3-V01–V15 evidenced) and the honest deferral status until then | The plan's step 6 wording is a gate, not a formality | W15 | W15-DV06 |

No ledger row requires fabricating evidence or editing upstream
documents; the checks are designed to run against the tree as it really
is.

## Resolved design decisions and their authority

1. **Checks are enumerated and evidence-backed, not vibes.** Eight
   checks C1–C8 ([02](02-acceptance-artifacts-and-checks.md) §4) each
   state object, method, pass condition, and failure meaning; each run is
   recorded with date and result. Rationale: P3-V15's "coherent" must be
   decidable by a reviewer who was not present at drafting.
2. **Exactly-one plan coverage is a constructed matrix, not an
   assertion.** The traceability matrix has one row per task-book
   required outcome and per validation ID, with the plan, design,
   implementation-record, and verification-record columns; duplicate or
   missing mappings are findings. Rationale: the plan's step 3 wording is
   set-theoretic; only construction can show it.
3. **The closure-review package is gated.** Its assembly has a hard
   precondition (every P3-V01–V15 has real evidence in its verification
   record) and until then only the template and deferral status exist.
   The package presents evidence; it never substitutes for it. Rationale:
   plan step 6 and the task book §7 rule that planned documents alone are
   not evidence of completion.
4. **W15 reports findings; it does not repair upstream documents.** A
   coverage gap, broken link, or status lie in a plan/design/record is a
   finding with an owner; W15's own artifacts must be clean, and W15
   edits only its own artifacts. Rationale: one owner per artifact; the
   plan index and stage README routing are coordinator/plan-owned, and
   the task book is authority — none are editable here.
5. **Separation is enforced as a check, not a style preference.** C6
   verifies that no document hosts another layer's content (no evidence
   in designs, no design decisions in verification records, no closure
   claims anywhere before their evidence). Rationale: `docs/README.md`
   makes the separation normative; the plan's scope names it.
6. **The stage's exit traceability ends at P3-V15, and P3-V15 is not
   self-certifying.** The matrix includes P3-V15's own row; its evidence
   is the documentation-review record this package produces — and the
   closure review still requires P3-V01–V14 independently. Rationale: a
   documentation package cannot mark the stage closed; it can only make
   closure decidable (task book §7's questions are answered by the
   whole evidence set).
7. **Navigation truthfulness has boundaries.** The stage README and
   implementation index rows naming W15's own artifacts are W15's to keep
   truthful; rows owned by the coordinator or other packages are only
   *checked* (findings reported). Rationale: the hard constraint that
   this design not modify coordinator-owned index files, combined with
   the plan's navigation responsibility for its own package.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for
   the ledger, boundaries, and decisions.
2. Read [02-acceptance-artifacts-and-checks.md](02-acceptance-artifacts-and-checks.md)
   for the artifact groups, C1–C8, matrix construction, evidence rules,
   register, and the closure package template.
3. Execute per [03-workflow-and-review.md](03-workflow-and-review.md):
   checks and matrix first (they are honest against today's tree), the
   closure package only through its gate.
4. Record implementation decisions in
   `../p3-w15-stage-documentation-acceptance-record.md` and evidence in
   `../../verification/p3-w15-stage-documentation-acceptance-verification.md`
   only when produced. Until the stage's evidence exists, W15's own
   status is *planned/partial* exactly as its checks report.

## Explicitly excluded interfaces

No code interface, runtime state, ABI, or machine-readable governance
format is designed or authorized by W15 (a tooling/format decision is
Reserved). No upstream document — task book, plan, stage README,
implementation index rows owned by others, sibling designs, records — is
edited by this package; defects found are findings with owners. No
verification evidence is created, and no closure, completion, or `DONE`
status is produced by this design or its artifacts before the gate opens.
A closure package that summarizes absent evidence, or a check "passed"
without its recorded run, is a fabrication to stop at review.

## Downstream handoff

- **P3 closure review** consumes the checks, matrix, evidence-location
  map, register, and (when gated open) the closure-review package as the
  navigation and traceability basis for answering the task book §7
  questions.
- **P4** consumes the same navigation via the
  [P4-W01](../../../p4/plans/p4-w01-entry-contract-reconciliation.md)
  reconciliation and the [W14 contract](../p3-w14-p4-smp-handoff/README.md);
  W15 adds the stage-level pointers, not new guarantees.
- **Later stage packages (P5+)** inherit the pattern: this design is the
  P3 instance of a reusable closure discipline, not a P3-only document.
