# Zelyr Stage Workflow

**Status:** Normative stage-process governance.  
**Scope:** The seven-layer responsibility flow from architecture decision to
stage completion, the per-layer fields (question, authority, escalation
boundary, admission, consumers, failure behavior), the plan-to-code admission
decision table, and the traceability/evidence-placement rules. It does not
own any layer's internal content rules, document locations (the
[documentation baseline](documentation-baseline.md) owns them), ADR lifecycle
states (the [ADR process](../adr/README.md) owns them), or merge policy (the
[integration workflow](integration-workflow.md) owns it).  
**Version:** v0.1  
**Owner/change context:** P0-W21 stage/plan/implementation workflow;
consolidates existing authorities into one decision surface and creates no
new authority — where a consolidated rule and its owning instance ever
diverge, the instance governs and this document is corrected.  
**Supersedes:** The absence of an explicit layer-responsibility and admission
map.

## 1. Layer model

Seven layers carry every stage from architecture to completion. Locations
are cited from the documentation baseline's class inventory and stage
separation; this document never re-enumerates directory rules.

| Layer | Artifact(s) and location (cited) | Inputs (required before the layer acts) | Outputs |
|---|---|---|---|
| L1 — Architecture decision (ADR) | `docs/adr/` documents and index per the ADR class | a recorded conflict, a deliberate architecture proposal, or a pending decision; the ADR lifecycle | accepted/rejected/deferred/superseded decisions; the escalation record |
| L2 — Stage task book | `docs/stages/<id>/task-book-v*.md` | the ADR baseline; the stage's mandate | required outcomes, constraints, validation matrix, exit criteria; a bounded work-package map |
| L3 — Work-package plan | `docs/stages/<id>/plans/` plus the plan index | the task book; the ADR; the plan-index prerequisite contracts | goal, scope classification, bounded work sequence, acceptance, handoff; prerequisite/consumer links |
| L4 — Detailed design | `docs/stages/<id>/implementation/<slug>/` entry plus supporting files | the plan; the Coding Guide (for code-bearing work); routed detailed references | implementation-level contracts, steps, validation matrix, handoff — or, for governance packages, the artifact contracts the plan deferred |
| L5 — Implementation record | `docs/stages/<id>/implementation/<slug>-record.md` | the design being implemented | decisions taken, changed artifacts, deviations; links to design, plan, and verification record |
| L6 — Verification record | `docs/stages/<id>/verification/<slug>-verification.md` | the design's validation matrix; actual executions/reviews | per-ID run/failed/blocked/not-run evidence with proves/does-not-prove statements; the only location of completion claims |
| L7 — Stage completion review and report | the stage verification area; the handoff package contents | every package's L5/L6 records; the task book exit criteria | the completion claim for the stage; the aggregated handoff package |

## 2. Per-layer fields

Every layer row carries these six fields.

### L1 — Architecture decision

- **Question answered:** what architecture applies and how it changes.
- **Decision authority:** architecture and its change process (per the ADR
  process).
- **Escalation boundary:** not a design or implementation substitute; an
  accepted decision is never silently changed (labels apply per the ADR
  process).
- **Admission:** the admission table (§3), row "architecture proposal".
- **Consumers:** every layer; especially L2 (task books) and L4 (designs).
- **Failure behavior:** a missing or conflicting decision blocks downstream
  layers until the ADR path completes; never improvised around.

### L2 — Stage task book

- **Question answered:** what must this stage achieve.
- **Decision authority:** stage outcomes and validation IDs within the ADR.
- **Escalation boundary:** must not carry function, module, or file-level
  implementation detail.
- **Admission:** the admission table (§3), row "task-book or plan
  authoring".
- **Consumers:** L3 (plans), L7 (completion review).
- **Failure behavior:** a task-book defect is corrected through the plan
  guide's change path in the same change; agents do not reinterpret outcomes.

### L3 — Work-package plan

- **Question answered:** how is one outcome bounded into work, order, and
  acceptance.
- **Decision authority:** package boundaries, sequencing, and acceptance
  wording within the task book.
- **Escalation boundary:** must not prescribe commands, crate structure, or
  signatures.
- **Admission:** the admission table (§3), row "task-book or plan
  authoring".
- **Consumers:** L4 (designs), and every agent selecting a package.
- **Failure behavior:** a plan that defers a decision to a design that does
  not exist blocks implementation (record it), never licenses designing
  while coding.

### L4 — Detailed design

- **Question answered:** how is this package implemented and validated.
- **Decision authority:** everything the plan classifies as stage-local
  design freedom.
- **Escalation boundary:** must not resolve conflicts with accepted
  decisions (labels apply per the ADR process) and must not claim
  completion.
- **Admission:** the admission table (§3), rows "code-bearing change" and
  "governance/policy change".
- **Consumers:** L5/L6 and the implementing agent.
- **Failure behavior:** a design gap or contradiction is a blocker recorded
  against the owning package; a design states "proposed" until delivered.

### L5 — Implementation record

- **Question answered:** what was actually done and decided.
- **Decision authority:** none — it records, it does not decide new rules.
- **Escalation boundary:** must not invent policy to justify a deviation;
  deviations are stated and owned, never absorbed.
- **Admission:** the admission table (§3) rows for the change being made.
- **Consumers:** L6, L7, reviewers.
- **Failure behavior:** a change without its record is a review failure;
  command logs and completion claims do not belong here.

### L6 — Verification record

- **Question answered:** what evidence exists and what does it prove.
- **Decision authority:** none — it reports evidence, it does not
  reinterpret acceptance.
- **Escalation boundary:** must not upgrade "blocked" or "not run" into
  "passed"; completion claims exist only here.
- **Admission:** the design's validation matrix, executed or reviewed.
- **Consumers:** L7, the stage's consumers, future auditors.
- **Failure behavior:** an evidence gap is stated as such; it is never
  waived by process text.

### L7 — Stage completion review and report

- **Question answered:** is the stage done, and what does the next stage
  receive.
- **Decision authority:** the completion claim, only from L6 evidence and
  L2 exit criteria.
- **Escalation boundary:** must not accept a stage whose exit criteria lack
  evidence.
- **Admission:** every package's L5/L6 records present and truthful.
- **Consumers:** the next stage's planning; the P1-style handoff package.
- **Failure behavior:** a missing evidence item fails completion; the stage
  stays open until evidence exists.

## 3. Admission decision table

"Holds" means the named artifact exists in its authoritative location with a
status that permits the activity, and the actor has read it per the
documentation index's routing rules.

| Activity | Must hold before starting | Escalation when blocked |
|---|---|---|
| Code-bearing change in a work package | the approved plan; an approved detailed design for the change; the Coding Guide preflight completed; the plan index's prerequisite contracts delivered or their absence recorded as a blocker | a plan that defers a needed decision to "the owning design" with no design in that location is a blocker to record, never a license to design while coding |
| Governance/policy change inside a P0-style package (documents, contracts, configuration) | the approved plan; the design-level artifact contract the plan defers decisions to (the established P0 pattern), or an explicit statement in the plan that the plan itself is the contract | a normative rule change with no owning document is a policy decision per the documentation baseline's thresholds, not a local edit |
| Task-book or plan authoring/change | the Plan Agent guide; the ADR baseline; the applicable task book; the stage-work-package-planning skill | task books and plans are never written from implementation notes; the hierarchy is one-way downward for content and upward for escalation |
| Architecture proposal or accepted-decision change | a draft ADR per the ADR lifecycle and template | never executed as a side effect of another package; a superseding ADR is its own reviewed change |
| Conflict with an accepted decision or declared threshold discovered during work | label the issue `ADR Required` (work is blocked) or `Architecture Change Request` (deliberate proposal) with the ADR process's definitions and record it | the affected work stops at the boundary; no local redesign, no silent reinterpretation |
| Merge and integration | the integration workflow policy; the configured required checks (CI baseline) | merge is never granted by a layer below L7's stage completion; policy-bound bypasses are recorded, never routine |
| Stage completion claim | every package's verification evidence per the task-book validation matrix; the exit criteria reviewed | a missing evidence item fails completion; it is never waived by process text (the exact failure this package exists to prevent) |

Corollaries:

- **A plan alone never authorizes code.** The plans-index reading order
  ("Coding Guide plus an approved detailed design for code changes") and the
  task book's delivery-hierarchy rule are restated here by reference as the
  first row's basis.
- **Process text never substitutes for design.** Adding admission prose to a
  process document in order to start coding without a design is a review
  failure, not a shortcut.

## 4. Traceability and evidence-placement rules

The P\<stage\>-Wxx work-package ID and the P\<stage\>-Vxx validation ID are
the traceability keys.

1. **Design ↔ plan:** every detailed design's header names its parent plan;
   every requirement the design maps carries the plan wording it implements
   (the requirement → design-location → acceptance table is the established
   P0 pattern and is required for code-bearing and governance designs alike).
2. **Implementation record ↔ design ↔ plan:** the implementation record names
   the design directory it implements, the plan it satisfies, the changed
   artifacts, decisions taken, and deviations; it links to its verification
   record. It carries no command logs and no completion claims.
3. **Verification record ↔ validation matrix:** the verification record maps
   each of its entries to the design's validation-matrix IDs and the
   task-book P\<stage\>-Vxx IDs, states run/failed/blocked/not-run with
   command, environment, timestamp, and reason, and states proves /
   does-not-prove per entry. Completion claims exist only here.
4. **Same-change rule:** documentation that a change invalidates is updated
   in the same change (documentation baseline mandate, cited); a contract
   lagging its implementation is a defect, not a TODO.
5. **Placement rule:** decisions live in implementation records; evidence
   lives in verification records; neither migrates into designs, task books,
   plans, or process documents (the documentation baseline's stage
   separation, cited not restated).
6. **Handoff rule:** a stage's handoff package (L7 output) is assembled by
   linking layer outputs — records, evidence, and governing contracts — never
   by copying their content; the stage dependency map (P0-W22) applies this
   rule to P0's package specifically.
7. **Failure attribution rule:** every blocked or failed activity names its
   owning layer and package, so a failure never strands an agent without an
   owner (the contributor workflow's per-stage pattern, generalized).

## 5. Non-mixing rules

- Process documents (this workflow document, guides, policies) never
  substitute for an approved detailed design, and a task book never carries
  function or module implementation — each stated as a checkable rule with
  §2's layer fields as its enforcement surface.
- Each artifact's content stays inside its documentation-baseline class; a
  rule needed by two layers is written once in its owning layer and linked by
  the other.
- A document may not create a new layer, location, or implicit class by
  inventing one; new classes follow the documentation baseline's thresholds.
- Statuses are truthful per layer: a design says "proposed"; only L6 says
  "passed"; only L7 says the stage is complete.
