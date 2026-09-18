# P0-W21 Layer Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W21 detailed design](README.md).

## 1. Logical artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Stage workflow document | `docs/development/stage-workflow.md` | this design, the task book §3 hierarchy, `docs/README.md` mandates, the W05 taxonomy and W06 escalation designs (as delivered), the plan/coding guides | the sole normative home of the layer responsibility matrix, admission decision table, and traceability rules; it does not own any layer's internal content rules, define document locations, or restate any linked authority |
| Documentation routing | one row in `docs/README.md` routing table; stage index row in `docs/stages/p0/implementation/README.md` | workflow document location / design status | discoverability and truthful status; no restatement |
| Implementation record | `../p0-w21-stage-plan-implementation-workflow-record.md` (created when work starts) | decisions taken, drill observations summary, deviations | changed artifacts and decisions; no command logs |
| Verification record | `../../verification/p0-w21-stage-plan-implementation-workflow-verification.md` (created when evidence exists) | drill outcomes, review output | run/blocked/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for
the statement in its row. The record and verification paths are future
locations; this design does not create them.

## 2. Layer model (normative content of the workflow document)

The workflow document must fix these seven layers. For each layer the
document carries exactly the six fields of §3 below. Locations are cited from
the [W05 taxonomy](../p0-w05-documentation-baseline/01-taxonomy-and-baseline-contract.md)
§3/§7 as delivered — the workflow document links them and never re-enumerates
directory rules.

| Layer | Artifact(s) and location (cited, not restated) | Inputs (required before the layer acts) | Outputs |
|---|---|---|---|
| L1 — Architecture decision (ADR) | `docs/adr/` documents and index per the ADR class | a recorded conflict, a deliberate architecture proposal, or a pending decision; the W06 lifecycle | accepted/rejected/deferred/superseded decisions; the escalation record |
| L2 — Stage task book | `docs/stages/<id>/task-book-v*.md` | the ADR baseline; the stage's mandate | required outcomes, constraints, validation matrix, exit criteria; a bounded work-package map |
| L3 — Work-package plan | `docs/stages/<id>/plans/p<stage>-w<nn>-<slug>.md` plus the plan index | the task book; the ADR; the plan-index prerequisite contracts | goal, scope classification, bounded work sequence, acceptance, handoff; prerequisite/consumer links |
| L4 — Detailed design | `docs/stages/<id>/implementation/<slug>/` entry plus supporting files | the plan; the Coding Guide (for code-bearing work); routed detailed references | implementation-level contracts, steps, validation matrix, handoff — or, for governance packages, the artifact contracts the plan deferred |
| L5 — Implementation record | `docs/stages/<id>/implementation/<slug>-record.md` | the design being implemented | decisions taken, changed artifacts, deviations; links to design, plan, and verification record |
| L6 — Verification record | `docs/stages/<id>/verification/<slug>-verification.md` | the design's validation matrix; actual executions/reviews | per-ID run/failed/blocked/not-run evidence with proves/does-not-prove statements; the only location of completion claims |
| L7 — Stage completion review and report | the stage verification area; the handoff package contents | every package's L5/L6 records; the task book exit criteria | the completion claim for the stage; the aggregated handoff package |

## 3. Per-layer fields (normative content of the workflow document)

Every layer row in the workflow document must carry these six fields. The
content below is what the fields must state; the workflow document writes it
as normative text and cites owning documents where marked.

1. **Question answered** — the one question the layer exists to settle:
   L1 "what architecture applies and how it changes"; L2 "what must this
   stage achieve"; L3 "how is one outcome bounded into work, order, and
   acceptance"; L4 "how is this package implemented and validated"; L5 "what
   was actually done and decided"; L6 "what evidence exists and what does it
   prove"; L7 "is the stage done, and what does the next stage receive".
2. **Decision authority** — what the layer may decide alone: L1 architecture
   and its change process (per W06); L2 stage outcomes and validation IDs
   within the ADR; L3 package boundaries, sequencing, and acceptance wording
   within the task book; L4 everything the plan classifies as stage-local
   design freedom; L5 nothing — it records, it does not decide new rules;
   L6 nothing — it reports evidence, it does not reinterpret acceptance;
   L7 the completion claim, only from L6 evidence and L2 exit criteria.
3. **Escalation boundary (must not decide)** — L2 must not carry function,
   module, or file-level implementation detail; L3 must not prescribe
   commands, crate structure, or signatures; L4 must not resolve conflicts
   with accepted decisions (labels apply per W06) and must not claim
   completion; L5/L6 must not invent policy to justify a deviation; L7 must
   not accept a stage whose exit criteria lack evidence. A layer that needs a
   decision above its authority records the blocker or the escalation label;
   it never silently chooses.
4. **Admission conditions** — what must exist before the layer acts: the
   admission decision table of
   [the admission contract](02-admission-and-traceability.md) §2 is the
   single source; the layer rows cite it.
5. **Consumers** — who downstream reads the layer's outputs: each layer names
   its primary consumers (for example L4 outputs are consumed by L5/L6 and
   the implementing agent; L7 outputs by the next stage's planning).
6. **Failure behavior** — what a defect at the layer means: a missing input
   blocks the layer (recorded, never improvised); a contradiction with an
   owning document is resolved by the owning document's change path in the
   same change; a completion-shaped statement outside L6 is a review failure.

## 4. Non-mixing rules (normative content of the workflow document)

- Process documents (this workflow document, guides, policies) never
  substitute for an approved detailed design, and a task book never carries
  function or module implementation — each stated as a checkable rule with
  the layer fields above as its enforcement surface.
- Each artifact's content stays inside its W05 class; a rule needed by two
  layers is written once in its owning layer and linked by the other.
- A document may not create a new layer, location, or implicit class by
  inventing one; new classes follow W05's thresholds.
- Statuses are truthful per layer: a design says "proposed"; only L6 says
  "passed"; only L7 says the stage is complete.
