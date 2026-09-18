# P0-W21 Stage / Plan / Implementation Workflow — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The layer responsibility flow from ADR to acceptance, the plan-to-
code admission criteria, the traceability and evidence-placement rules, and
the discoverability drill required by
[P0-W21](../../plans/p0-w21-stage-plan-implementation-workflow.md).  
**Owner/change context:** P0-W21 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W21. It converts the bounded
work-package plan into one normative engineering-workflow document that fixes
what each delivery layer (ADR, task book, work-package plan, detailed design,
implementation record, verification record, completion report) answers,
produces, consumes, and may decide; the admission rules that gate the
transition from plan to code; and the cross-reference rules that bind
implementation traceability to verification evidence. It deliberately does
**not** define the ADR lifecycle or its escalation labels
([P0-W06](../p0-w06-adr-governance/README.md) owns both), redefine document
classes, locations, or metadata rules
([P0-W05](../p0-w05-documentation-baseline/README.md) owns the taxonomy and
stage separation), restate the [branch and pull-request integration
workflow](../../../../development/integration-workflow.md) merge policy, restate
the contributor path ([P0-W19](../p0-w19-reproducible-development-workflow/README.md)
assembles it), or substitute process text for actual detailed designs — the
plan's out-of-scope clause forbids exactly that, and no task book or process
document may carry function or module implementation.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P0 task
book](../../task-book-v0.1.md), and the P0-W21 plan. This document is the
proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W21 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W21 is: the ADR → task book → plan/design →
  implementation → verification workflow is **explicit** (P0-V09, P0-V15) —
  stage layers are not mixed, and a later agent can determine which documents
  to read and produce.
- The task book §3 delivery hierarchy and `docs/README.md` reading rules
  already state the hierarchy in short form. Where this design's document
  elaborates those statements it must not contradict them; where elaboration
  would change a rule, the source document is edited in the same change
  (mirroring the W05 approach to the `docs/README.md` mandate).
- [W05](../p0-w05-documentation-baseline/README.md) §7 explicitly defers "the
  responsibility flow and admission criteria between layers" to W21. This
  design accepts that delegation: W05 fixes where documents live and what
  they may contain; W21 fixes who decides what between the layers.
- [W06](../p0-w06-adr-governance/README.md) owns the escalation mechanism —
  the `ADR Required` and `Architecture Change Request` label semantics, the
  ADR state machine, and the supersession path. The workflow document cites
  that mechanism; it never redefines it.
- The plan's work sequence 4 requires a discoverability drill with one P0
  package and one future P1 package; the drill proves discoverability only
  and must never be recorded as execution evidence.

Classification: the layer responsibility matrix, the admission criteria, the
traceability and evidence-placement rules, the discovery wiring, and the
drill (all in the supporting files) are **Required** for W21 closure.
Per-stage customizations beyond the general rules (for example a future
stage-specific workflow document), automation of the admission checks, and
process templates for future document types are **Reserved** with recorded
triggers. ADR lifecycle content (W06), taxonomy and metadata rules (W05),
stage task books/plans content, the integration policy text, contributor-path
assembly (W19), and all code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Per-layer questions answered, artifacts produced, inputs required, decision boundaries (work sequence 1) | [Layer contract](01-layer-contract.md) §3 | P0-V09 (W21-DV01) |
| Admission criteria from plan to code: when an approved detailed design is required, when an ADR/architecture change is required (work sequence 2) | [Admission and traceability](02-admission-and-traceability.md) §2 | P0-V09/P0-V15 (W21-DV02) |
| Implementation-traceability and verification-evidence placement with mutual cross-references (work sequence 3) | [Admission and traceability](02-admission-and-traceability.md) §3 | P0-V09 (W21-DV03) |
| Discoverability drill with one P0 and one future P1 package (work sequence 4) | [workflow](03-implementation-and-review.md) step 4 | P0-V15 (W21-DV05, DV06) |
| Document discoverability and coherence | [workflow](03-implementation-and-review.md) step 3 | P0-V09 (W21-DV04) |
| Downstream handoff to every later stage and to W22 | [workflow](03-implementation-and-review.md) handoff checklist | W21 closure (W21-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed state (2026-09-18, worktree branch `docs/p0-implementation-designs`):
the layer hierarchy exists only as short statements scattered across
documents — the [task book §3](../../task-book-v0.1.md) delivery hierarchy,
`docs/README.md` routing and precedence rules, the Plan Agent and Coding
Guide authority orderings, and `AGENTS.md` scope rules. No tracked document
consolidates what each layer answers, produces, consumes, and may not decide;
no document states a single admission rule for "when does a change need an
approved detailed design"; cross-reference rules between implementation
records and verification records are nowhere written. W01 is delivered; W05
and W06, this package's plan-index prerequisites, have proposed designs whose
documents do not yet exist. Each ledger row below states the missing
foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Stage layers are not mixed | Hierarchy stated in fragments; no per-layer contract with non-overstepping boundaries | Layer responsibility matrix: question answered, artifact and location, required inputs, decision authority, escalation boundary, outputs, consumers per layer | "Not mixed" is reviewable only when each layer's boundary is written down | W21 (this design); locations per W05 | W21-DV01 |
| Later agents know what to read and produce | An agent assembles the answer from five-plus documents, with no entry point | One normative workflow document routing each activity to its layers and documents | P0-V15 requires a P1 planner to find inputs without reconstructing policy | W21 | W21-DV05/DV06 drill |
| Plan-to-code admission criteria | Rules exist as instances (task book §3, Coding Guide preflight, plans/README reading order) but no single admission rule | Admission section consolidating the instances by reference and fixing the design-required boundary | A Coding Agent must be able to answer "may I code?" without reading the whole tree | W21; instances stay authoritative in their homes | W21-DV02 |
| Traceability placement and cross-references | Locations fixed by `docs/README.md` layout (elaborated by the W05 design); mutual cross-reference rules absent | Traceability rules binding records, designs, plans, and validation IDs by reference keys | Traceability that is not mechanical cannot be reviewed coherently | W21; locations stay W05's | W21-DV03 |
| Discoverability drill (P0 + P1 package) | Never performed | Drill procedure with reachable/blocked/failed outcomes over a P0 and a P1 package chain | Work sequence 4 makes the drill the acceptance exercise | W21 | W21-DV05/DV06 |

Prerequisite boundary: W05 and W06 are proposed designs, not deliveries. The
fallback is the same as W06's: the header, location, and status mandates this
design consumes are already present in `docs/README.md` and the task book in
short form, so the workflow document can be written against the existing
mandates; if delivered W05/W06 rules differ, they are followed; a substantive
conflict is raised, not absorbed.

## Resolved design decisions and their authority

1. **Normative home:** `docs/development/stage-workflow.md` is the sole
   normative home of the layer responsibility flow, admission criteria, and
   traceability rules. Rationale: it is a development-governance document
   beside the agent guides it routes between, discoverable in one link from
   the documentation index, matching the approved W02/W03/W07 pattern.
   [W05](../p0-w05-documentation-baseline/README.md) may re-home it later;
   semantic ownership stays with the document.
2. **Reference-don't-restate:** the workflow document is an authority map,
   not a second authority. W05 owns locations and classes; W06 owns
   escalation and ADR lifecycle; the integration policy owns merge rules;
   W19 owns the contributor path; each stage's plan index owns per-package
   prerequisites. Where the workflow document must state a rule another
   document owns, it links; two statements of one rule is a review failure.
3. **Layer set:** seven layers — ADR, stage task book, work-package plan,
   detailed design, implementation record, verification record, and stage
   completion review/report — exactly the plan's scope list. The P1 handoff
   package is treated as an output of the completion layer, not an eighth
   layer (its P0-specific content is
   [W22](../p0-w22-stage-dependency-map/README.md)'s to define; W21 fixes the
   general completion-layer contract under which such a package is produced).
4. **Admission rule shape:** a decision table, not prose. For each activity
   kind (code-bearing change; policy/documentation change in a governance
   package; task-book/plan work; architecture proposal; conflict handling)
   the table names the required entry conditions — which documents must
   exist and be read, which approvals must be held — before the activity
   starts. The table consolidates existing instances by reference; it
   creates no new authority.
5. **Traceability keys:** the P\<stage\>-Wxx work-package IDs and
   P\<stage\>-Vxx validation IDs are the cross-reference keys; every
   implementation record links the design and plan it implements and names
   the verification record; every verification record maps its evidence to
   validation IDs and points back to the record. Completion claims live only
   in verification records, matching the W05 records classes.
6. **Drill packages:** the P0 drill package is
   [P0-W02](../../plans/p0-w02-rust-toolchain-baseline.md) — the only package
   with a complete proposed chain (plan, design, and record locations) — and
   the P1 drill package is
   [P1-W10](../../../p1/plans/p1-w10-qemu-boot-regression.md), whose
   prerequisites reach across P0's delivered baseline per the P1 plan index.
   The drill follows each layer's chain from the new workflow document and
   records reachable / blocked / failed outcomes; it proves discoverability
   and nothing else.
7. **No process-for-design substitution:** the workflow document states, as
   normative content, that process documents never substitute for an approved
   detailed design and that a task book never carries function or module
   implementation — turning the plan's out-of-scope clause into a checkable
   rule of the layer matrix rather than a preamble sentence.

## Work breakdown and loading order

1. Read [the layer contract](01-layer-contract.md) for the artifact groups
   and the per-layer matrix the workflow document must contain.
2. Read [the admission and traceability contract](02-admission-and-traceability.md)
   for the admission decision table and the cross-reference rules.
3. Apply the changes in the order stated in [the implementation
   workflow](03-implementation-and-review.md): verify prerequisite surfaces,
   author the workflow document, wire discovery, run the drill, then close.
4. Store actual drill observations, commands, and outcomes in
   `../../verification/p0-w21-stage-plan-implementation-workflow-verification.md`,
   and record decisions taken and changed artifacts in
   `../p0-w21-stage-plan-implementation-workflow-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W21 complete.

## Design-level state and lifecycle

W21 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked workflow document plus its discovery
links. Its documentary lifecycle:

```text
fragmented layer statements across entry documents
  -> stage-workflow document committed (layer matrix, admission table,
     traceability rules)
  -> discovery links (docs/README routing row, stage index row) committed
  -> drill over one P0 and one P1 package chain evidenced
  -> every later stage routes its plan/design/record/evidence decisions
     through this document
     (W06 escalation is cited by it; W22's P0 handoff map applies it;
      P1+ planners start their reading order from it)
  -> later changes mutate only through its thresholds
```

The workflow document owns the between-layer statements; every layer's
internal content rules stay with their owning documents (W05 taxonomy, W06
lifecycle, the plan and coding guides). A workflow statement that contradicts
an owning document is a review failure resolved by fixing the workflow
document's citation — or, if the owning rule itself must change, by the
owning document's change path in the same change.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, build surface,
CI workflow, or check is designed or authorized by W21. Additionally
excluded: ADR lifecycle states, thresholds, and label semantics (W06);
document classes, metadata fields, and directory separation (W05); merge
policy text (the integration workflow's owning change); contributor-path
assembly (W19); task-book or plan content; and any future design or contract
text. Writing any of these under W21 is a scope conflict to be raised at
review.

## Downstream handoff

- **Every later plan and stage** receives the single entry point: a planner
  or coding agent reading the workflow document can determine which documents
  to load, which approvals to hold, where records and evidence land, and
  which escalations apply, for any stage.
- **[W22](../p0-w22-stage-dependency-map/README.md)** receives the layer
  matrix and completion-layer contract as the shape its P0 handoff map and
  completion-report contract must fit; W22 applies the general rules to P0's
  specific deliverables.
- **W06** receives the citation rule: its escalation mechanism is referenced,
  never restated, and the workflow document's admission table points to it
  for every conflict row.
- **W05** receives the boundary confirmation: its §7 deferral is discharged
  by this design; the taxonomy keeps location authority while the workflow
  owns responsibility flow.
- **W19 and the integration policy** receive the non-restatement rule: the
  workflow document links both and edits neither.
- **Stage completion reviews (P0 and later)** receive the completion-layer
  contract the review must satisfy: aggregation from verification records
  only, no claim without evidence, handoff package assembled from layer
  outputs.
