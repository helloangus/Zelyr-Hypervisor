# P0-W22 Stage Dependency Map — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The checkable P0-to-P1+ handoff map, the P0 deliverable register,
the consumption mapping with reuse conditions and blocking implications, the
completion-report content contract, and the P1 reading-order drill required
by [P0-W22](../../plans/p0-w22-stage-dependency-map.md).  
**Owner/change context:** P0-W22 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W22. It converts the bounded
work-package plan into one normative stage-level handoff-map document that
registers every P0 work package's deliverable contract with its authoritative
location, owner, validation IDs, and truthful delivery status; maps the
consumption of those deliverables by P1, P2, P3, and later stages with reuse
conditions and blocking implications; fixes what the P0 completion report
must contain; and is validated by a P1-planner reading-order drill. It
deliberately does **not** plan P1+ internal modules or implementation order
(plan out of scope), restate any deliverable's contract content (the
register carries pointers, never content), write the completion report
itself (that is the stage completion review's output, produced at completion),
or claim that any P0 package is complete.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P0 task
book](../../task-book-v0.1.md), and the P0-W22 plan. This document is the
proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W22 plan → this
design → Coding Guidelines. In particular:

- The task-book outcome for W22 is: P0 outputs and later-stage consumers are
  **mapped so P1 need not recreate P0** (P0-V15). Task-book §7 names the P0
  dependency map as a member of the P1 handoff package; the acceptance
  wording requires a P1 Plan Agent to find complete inputs and distinguish
  delivered, unverified, and later work.
- The [P1 task book](../../../p1/task-book-v0.1.md) §1 already states what P1
  expects ("P0 must supply the documented workspace/toolchain, AArch64
  target, build and QEMU entry points, logging/panic baseline,
  version/build metadata, unsafe-governance, address/error conventions and
  CI gates") and the rule for gaps: "Missing P0 inputs are upstream defects,
  not permission to redesign P0 inside P1." The map consumes that statement
  as the blocking-implication rule; it does not re-derive it.
- The [plan index](../../plans/README.md) is the authoritative
  prerequisite/consumer map at package granularity; the handoff map lifts
  that view to deliverable/stage granularity and must not contradict it.
- [W05](../p0-w05-documentation-baseline/README.md) owns document classes and
  stage separation; the map's stage-root placement is a resolved decision of
  this design with an explicit coordination rule and failure boundary
  (decision 1), because W05's separation table does not yet name a
  stage-root governance location.
- [W21](../p0-w21-stage-plan-implementation-workflow/README.md) (a
  prerequisite) owns the completion-layer contract the completion-report
  content must fit; the map applies it to P0 specifically.

Classification: the deliverable register, the consumption mapping, the
completion-report content contract, the update rules, the discovery wiring,
and the drill (all in the supporting files) are **Required** for W22 closure.
Automation of status refresh, map extensions to P2→P3 handoffs at the same
granularity, and cross-stage tooling are **Reserved** with triggers. P1+
internal module or sequencing plans, contract content restatement, completion
claims, and all code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Summarize W01–W21 deliverable contracts, validation evidence, and owners (work sequence 1) | [Handoff-map contract](01-handoff-map-contract.md) §3 | P0-V15 (W22-DV01) |
| Map consumption by P1 EL2 boot, P2 platform/memory, P3 SMP, and later stages (work sequence 2) | [Handoff-map contract](01-handoff-map-contract.md) §4 | P0-V15 (W22-DV02) |
| Completion-report requirements: links, verified/unverified scope, open issues (work sequence 3) | [Handoff-map contract](01-handoff-map-contract.md) §5 | P0-V15 (W22-DV03) |
| P1-planner reading-order drill; P0 infrastructure need not be redesigned (work sequence 4) | [workflow](02-implementation-and-review.md) step 4 | P0-V15 (W22-DV05, DV06) |
| Document discoverability and coherence | [workflow](02-implementation-and-review.md) step 3 | P0-V09 (W22-DV04) |
| Downstream handoff to P0 completion review and P1 planning | [workflow](02-implementation-and-review.md) handoff checklist | W22 closure (W22-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed state (2026-09-18, worktree branch `docs/p0-implementation-designs`):
no handoff map or completion-report contract exists anywhere in the tracked
tree; the only cross-package consumer view is the [plan
index](../../plans/README.md) prerequisite/consumer table, which maps plans to
plans but not deliverables to stages, and the P1 task book's supply list,
which states P1's expectations without pointing at P0 locations. Of the
packages the map will register, W01 is delivered with recorded evidence;
W02–W10 and W16–W19 have proposed designs in this tree (their contract
documents are not yet delivered); W11–W15 sibling designs are being prepared
in parallel and are referenced by slug and P0-Wxx ID only; W20 and W21 have
proposed designs in this tree. Each ledger row below states the missing
foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P1 can find complete P0 inputs | Inputs scattered across up to twenty-two packages; no single entry point names them all | Deliverable register: one row per package with contract pointer, location, owner, validation IDs, status | "Find without reconstructing" requires one checkable table | W22 (this design); rows consume plan-index and sibling-design data | W22-DV01, DV05 |
| Delivered / unverified / later work distinguishable | Only W01 has evidence; statuses are implicit in scattered records | Status vocabulary with evidence pointers, updated by rule | The acceptance wording names exactly this distinction | W22; statuses sourced from stage records | W22-DV01, DV06 |
| P1/P2/P3 consumption with reuse conditions and blocking implications | P1 task book states its supply list; plan index maps plan-to-plan; no deliverable-to-stage view exists | Consumption mapping: deliverable → consumer stage → reuse condition → blocking implication | Without reuse conditions a consumer cannot tell whether it may rely on an input; without blocking implications a gap has no owner | W22; blocking rule cited from the P1 task book | W22-DV02 |
| Completion-report requirements defined | No contract for the report's content exists; report does not exist | Completion-report content contract (links, per-ID status, open issues), pointed to a future verification location | Task book §7 makes the report a handoff-package member; its required content must be fixed before it is written | W22 content; report authored by the stage completion review | W22-DV03 |
| Drill: P1 planner's reading order works | Never performed; P1's reading order crosses P0 records that do not exist yet | Drill procedure with reachable/blocked/failed outcomes over the P1 supply list | Work sequence 4 makes the drill the acceptance exercise | W22 | W22-DV05/DV06 |

No ledger row requires planning P1+ internals or restating a contract, so no
architecture decision blocker is outstanding. The dependency on sibling
designs being delivered (or at least their contracts being findable per the
stage implementation index) is an implementation-time condition: the map's
register rows are filled from whatever is findable at implementation time and
mark the rest with their owning package, never with invented locations.

## Resolved design decisions and their authority

1. **Normative home and placement:** `docs/stages/p0/p0-handoff-map.md` — a
   stage-level normative governance document beside the task book, versioned
   `v0.1` with the full status header. Rationale: the map is P0 deliverable
   governance consumed across stages, not a work-package design or record;
   placing it beside the task book keeps the handoff package (task book +
   completion report + map) discoverable from the stage root. **Coordination
   rule:** the [W05 stage-separation
   table](../p0-w05-documentation-baseline/01-taxonomy-and-baseline-contract.md)
   §7 enumerates stage sublocations but not a stage-root governance document;
   the map declares itself a member of W05's "Stage documents" class and, if
   the delivered W05 rules do not admit a stage-root location, the placement
   question is raised under W05's policy-decision threshold and recorded —
   the map is not silently forced into a non-conforming location, and the
   interim location stays until the owner decides.
2. **Reference-don't-restate:** every register row and consumption row
   carries pointers (contract document path, record path, validation IDs);
   no row restates a deliverable's contract content. The map is a view, not
   a second authority; a row that duplicates contract text is a review
   failure.
3. **Status vocabulary:** exactly four row states — `delivered` (with a
   pointer to recorded verification evidence), `design-proposed` (with a
   pointer to the design), `plan-only` (pointer to the plan), and `blocked`
   (with the blocking cause and owner). These are the checkable form of the
   acceptance wording's delivered / unverified / later-work distinction; no
   other status word may appear.
4. **Consumption mapping shape:** consumer-stage rows (P1 with per-package
   pointers, P2, P3, and a standing later-stage rule), each carrying the
   consumed P0 deliverable, the reuse condition (which status the consumer
   may rely on), and the blocking implication (what the consumer must do —
   record an upstream defect per the P1 task book's cited rule — and must
   not do — redesign P0 inside the consuming stage). No row states or
   implies P1+ internal sequencing.
5. **Completion-report contract:** the map fixes the report's required
   content (per-package record and evidence links, per-validation-ID status
   across P0-V01–V15, open issues including pending ADR-register items,
   reserved-scope and not-verified boundaries, handoff-package assembly), its
   future location `docs/stages/p0/verification/p0-completion-report.md`,
   and its author — the stage completion review, not W22. W22 writes the
   contract; the report appears only at completion.
6. **Drill design:** the drill walks the P1 planner's actual reading order —
   the P1 task book §1 supply list and the P1 plan index prerequisites —
   through the map, confirming each input is findable, its status is
   distinguishable, and no step requires redesigning P0 infrastructure. The
   drill is discoverability evidence only.
7. **Self-registration:** the register covers W01–W21 per the plan; W22's own
   record and this map enter the handoff package through the completion
   report's assembly rule rather than through a self-referential register
   row, keeping the register exactly the plan's scope.

## Work breakdown and loading order

1. Read [the handoff-map contract](01-handoff-map-contract.md) for the
   artifact groups, the register and consumption-table shapes, the status
   vocabulary, the update rules, and the completion-report contract.
2. Apply the changes in the order stated in [the implementation
   workflow](02-implementation-and-review.md): audit delivered status of all
   packages, author the map document, wire discovery, run the drill, then
   close.
3. Store drill observations and outcomes in
   `../../verification/p0-w22-stage-dependency-map-verification.md`, and
   record decisions taken and changed artifacts in
   `../p0-w22-stage-dependency-map-record.md` only when implementation
   begins. Neither this design nor a written record may claim W22 or P0
   complete.

## Design-level state and lifecycle

W22 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked map document whose rows are the mutable
state, updated by rule:

```text
no handoff map; P1 inputs findable only by reading every package
  -> p0-handoff-map.md committed (register W01–W21, consumption map,
     completion-report contract)
  -> rows carry truthful statuses sourced from the stage records
  -> drill over the P1 reading order evidenced
  -> as packages deliver, their rows move to delivered with evidence
     pointers in the same change as the delivering package's records
  -> at stage completion, the completion review authors the report per the
     map's contract, assembling the handoff package
  -> P1 planning consumes the map; later stage-to-stage maps extend, never
     rewrite, this pattern
```

The map owns the row data and statuses; each deliverable's owning document
owns its content; the verification records own the evidence. A row whose
status contradicts the pointed-to evidence is a review failure fixed in the
same change; the map never becomes a second statement of any contract or a
substitute for evidence.

## Explicitly excluded interfaces

No Rust type, function, crate, API, ABI, build surface, CI workflow, or
check is designed or authorized by W22. Additionally excluded: P1+ internal
module, sequencing, or implementation-order planning; any contract content
in a register or consumption row; the completion report itself (authored at
completion by the stage review); any P1 plan or task-book edit; and any
completion claim. Writing any of these under W22 is a scope conflict to be
raised at review.

## Downstream handoff

- **P0 completion review** receives the map as the checklist for assembling
  the completion report and handoff package, with per-item status and
  evidence pointers already resolved.
- **P1 planning** receives the single entry point the task book §1 supply
  list resolves against: every input findable, every status distinguishable,
  every gap carrying an owner and the upstream-defect rule instead of a
  redesign temptation.
- **P2/P3 and later stages** receive the standing consumption rows and the
  extension pattern for their own stage-to-stage maps.
- **W01–W21** receive the update rule: delivering a package means moving its
  map row to `delivered` with an evidence pointer in the same change —
  a bounded, mechanical obligation, never a rewrite of their contracts.
- **[W21](../p0-w21-stage-plan-implementation-workflow/README.md)** receives
  confirmation that the map fits the completion-layer contract its workflow
  document fixes; disagreements route to that document's thresholds.
- **[W20](../p0-w20-ci-baseline/README.md)** receives the consumption row
  that carries its enforcement evidence into the handoff package as the
  P0-V08 member.
