# P0-W05 Documentation Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The document-class taxonomy, normative/informative status rules,
versioning and supersession metadata, stage-document separation, and
entry/link coherence required by
[P0-W05](../../plans/p0-w05-documentation-baseline.md).  
**Owner/change context:** P0-W05 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W05. It converts the bounded
work-package plan into one normative documentation-governance document, a
bounded coherence sweep over existing directory entries, discovery wiring, and
a referencability walkthrough. It deliberately does **not** define the ADR
lifecycle (W06), the cross-layer workflow and admission rules (W21), any
future detailed design or contract content, or document templates (templates
are delivered by the package that owns the recurring document type — W06
delivers the ADR template); it also must not rewrite existing documents for
style or freeze future implementation details.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, ADR baseline, P0 task book, and P0-W05 plan.
This document is the proposed detailed design for those changes; it is not a
completion record and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W05 plan → this design
→ Coding Guidelines. In particular:

- `docs/README.md` is already titled "documentation index and governance" and
  already mandates status/scope/version/owner/supersession headers for new
  normative documents, informative labeling, and same-change updates of
  affected contracts. W05 does not contradict or silently relocate those
  rules: the new baseline document elaborates them, and where elaboration
  would change a rule, the `docs/README.md` edit happens in the same change.
- The task book outcome for W05 is: normative/informative documentation,
  versioning, and stage-document locations are **defined** (P0-V09), with
  paths, responsibilities, and statuses independently reviewable and no
  mixing between layers.
- The plan's out-of-scope list bounds the work: no future detailed design is
  written into the task book or elsewhere, and no future implementation
  detail is frozen.

Classification: the taxonomy, metadata rules, stage-separation rules, and the
coherence sweep are **Required** for W05 closure. Directory re-homing of
existing contracts (for example re-homing the W02 toolchain contract), new
document classes beyond the inventory, and template authoring are **Reserved**
with recorded owners and triggers. ADR lifecycle content, workflow admission
rules, stage task books/plans/records content, and all code are **Out of
Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Document classes with normative/informative status and authoritative sources | [Taxonomy contract](01-taxonomy-and-baseline-contract.md) §3–§4 | P0-V09 (W05-DV01) |
| Version, status, owner/change-context, supersession, and code-sync rules | [Taxonomy contract](01-taxonomy-and-baseline-contract.md) §5–§6 | P0-V09 (W05-DV02) |
| Stage task book / plan-design / implementation / verification separation | [Taxonomy contract](01-taxonomy-and-baseline-contract.md) §7 | P0-V09 (W05-DV03) |
| Directory entries, cross-links, and later-package referencability | [Taxonomy contract](01-taxonomy-and-baseline-contract.md) §8, [workflow](02-implementation-and-review.md) steps 2–3 | P0-V09 (W05-DV04, DV05) |
| Downstream consumability by W06, W10–W22, P1+ | [workflow](02-implementation-and-review.md) handoff checklist | P0-V09/P0-V15 (W05-DV06, DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at merge 4e631ee): the documentation tree
already exists as a scaffold — `docs/README.md` (routing table, normative-
source list, layout sketch, per-stage separation sketch), directory READMEs
under `adr/`, `abi/`, `architecture/`, `machine-types/`, `platform/`,
`security/`, `testing/`, `templates/`, and `development/` (concise guides plus
detailed references), and per-stage trees with task books and plans. W01 is
completed; W02–W04 designs are proposed. No tracked document consolidates the
class taxonomy, the metadata rules, or the stage-layer responsibilities; the
existing statements are one-sentence summaries in `docs/README.md` plus the
task book's delivery-hierarchy section. Each ledger row below states the
missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Normative/informative status defined per class | `docs/README.md` states the principle; no per-class assignment exists anywhere | Class inventory with normative/informative status and authority role per class | "Independently reviewable" requires a single table a reviewer can check against | W05 (this design) | W05-DV01 |
| Versioning, status, owner, supersession, and sync rules defined | Header mandate is one sentence in `docs/README.md`; no field list, version format, bump rule, or supersession mechanics | Metadata-rules sections elaborating the existing mandate (no contradiction) | Rules that cannot be applied mechanically cannot be reviewed coherently | W05 elaboration; `docs/README.md` mandate preserved | W05-DV02 |
| Stage-document locations and separation defined | Layout sketch in `docs/README.md`; delivery hierarchy in the task book; no normative per-directory responsibility statement | Stage-separation section fixing what each directory may contain and what it must not | "层次之间不混写" is a reviewable rule only when written per directory | W05; task book §3 honored | W05-DV03 |
| Entries, cross-links, and referencability checked | Never checked as a set; docs were added incrementally | Coherence sweep procedure plus referencability walkthrough | Plan work item 4 requires the check, with fix-forward of findings | W05 | W05-DV04/DV05 |
| Later packages can locate homes for their outputs (P0-V15) | Not guaranteed for packages that have not started (W06, W10–W22) | Class inventory complete enough that every W06–W22 output has a home | Consumers must not re-create taxonomy per package | W05 | W05-DV06/DV07 |

No row requires inventing new document classes, moving existing files, or
writing content for future packages, so no decision blocker is outstanding
for this design. W01 is delivered; the ADR baseline exists; no other
prerequisite gates this package.

## Resolved design decisions and their authority

1. **Normative home.** `docs/development/documentation-baseline.md` is the
   sole normative home of the class taxonomy, metadata rules, and
   stage-separation rules; `docs/README.md` remains the mandatory entry and
   routing surface and keeps its existing true summary statements. This
   mirrors the approved W02/W03/W04 pattern (policy in `docs/development/`,
   one routing row, no duplication). Where the baseline elaborates a rule
   that `docs/README.md` states in short form, the elaboration must not
   contradict it; a needed rule change edits both files in the same change.
2. **Header fields are the existing mandate, made mechanical.** Status,
   Scope, Version, Owner/change context, and Supersedes (where applicable),
   exactly as `docs/README.md` already requires; the baseline adds the field
   semantics, the `vX.Y` version format, the bump rule, and the supersession
   mechanics (§5–§6 of the taxonomy contract). ADR-specific supersession
   details are left to W06 by explicit pointer.
3. **Class inventory is closed for P0.** The taxonomy contract §3 fixes the
   fourteen P0 classes (root navigation, documentation governance, ADRs,
   architecture descriptions, ABI, machine types, platform, testing,
   security, development guidance, templates, stage documents, and
   implementation/verification records as stage-internal classes). Adding a
   class follows the baseline's change thresholds; W05 does not pre-create
   classes for packages that have not designed their outputs.
4. **Stage separation is normative per directory.** The baseline fixes what
   each stage sublocation may contain and its prohibitions (task book: what;
   plans: bounded packages; implementation: designs and records;
   verification: evidence and completion reports), consistent with the task
   book's delivery hierarchy and `docs/README.md` layout. This is the
   *document-class* rule; the *responsibility flow* between layers remains
   W21's.
5. **Templates are a gating rule, not a W05 deliverable.** The existing
   `docs/templates/README.md` rule (approved template before creating
   recurring document types) is restated by pointer in the baseline; W05
   authors no template. W06 delivers the ADR template; other packages
   deliver theirs when their recurring type appears.
6. **Coherence sweep is review-driven and bounded.** Existing directory
   entries and links are corrected only where they contradict the taxonomy,
   lack a required statement, or misstate status. No style rewrites, no
   mass re-homing, no content authoring for future packages. Findings that
   would require moving an accepted contract (for example re-homing the W02
   toolchain contract under the new taxonomy) are recorded as Reserved
   follow-ups with the owning package's consent, not executed unilaterally.
7. **Records are stage-internal classes.** Implementation records
   (traceability) and verification records (evidence) are defined as
   stage-internal document classes with their own content rules (facts and
   decisions versus evidence and claims), so the "no completion claims in a
   design" rule has a written home.

## Work breakdown and loading order

1. Read [the taxonomy contract](01-taxonomy-and-baseline-contract.md) for the
   artifact groups, the class inventory to be written, the metadata rules,
   the stage-separation rules, and the bounded-edit policy.
2. Apply the changes in the order stated in the
   [implementation workflow](02-implementation-and-review.md): write the
   baseline document, run the coherence sweep, wire discovery, run the
   referencability walkthrough, then close.
3. Store actual commands, output, environment, and result in
   `../../verification/p0-w05-documentation-baseline-verification.md`, and
   record changed artifacts, sweep findings, and any deviation in
   `../p0-w05-documentation-baseline-record.md` only when implementation
   begins. Neither this design nor a written record may claim W05 complete.

## Design-level state and lifecycle

W05 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked governance document, the bounded edits it
justifies, and its discovery links. Their documentary lifecycle:

```text
implicit, scattered documentation governance
  -> documentation-baseline.md committed (taxonomy, metadata, separation)
  -> coherence sweep applied to existing entries and links
  -> docs/README.md routing row committed
  -> referencability walkthrough evidenced
  -> later packages mutate only through the baseline's rules
     (W06 fills the ADR lifecycle class and its template; W07 consumes
      coherence for its documentation gate; W10–W22 place their policy
      documents per the inventory; W21 defines the cross-layer workflow)
```

The baseline document owns every taxonomy and metadata statement; directory
READMEs own their directory's role statement; records own their facts. A
conflict between the baseline and any entry document is a review failure,
resolved in the same change, not by parallel prose.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, or build
surface is designed or authorized by W05. Additionally excluded: ADR
lifecycle content (W06), workflow/admission rules between layers (W21),
document templates (owning packages), task-book or plan content, and any
future design or contract text. Writing any of these under W05 is a scope
conflict to be raised at review.

## Downstream handoff

- **W06** receives the document-class conventions its ADR governance document
  and template must follow (status header fields, version format, index
  role), and the reserved slot for the ADR lifecycle class detail.
- **W07** receives the coherence rules its documentation-consistency gate can
  check mechanically (required headers per class, entry existence, link
  resolution); gate semantics remain W07's.
- **W10, W11, W14, W15, W18** receive the home and header conventions for
  their policy documents (unsafe, portability, failure classes, type-safety
  requirements, dependency governance) under `docs/development/` or their
  class directories per the inventory.
- **W12, W13** receive the same for diagnostics and trace-namespace
  governance documents.
- **W16, W17** receive the same for identity and naming governance documents.
- **W19** receives the entry-point map its clone-to-build workflow aggregates.
- **W20** receives the mechanical coherence rules its documentation check can
  enforce; classification of required checks remains W20's.
- **W21** receives the fixed stage-layer locations its workflow rules
  operate on; W21 defines responsibilities and admission criteria between
  layers, W05 fixed the locations and content classes.
- **W22** receives the inventory it maps into the P1 handoff package.
- **P1 and later stages** receive the standing rule set for any new document:
  find its class, apply its header, record its supersession, and place it in
  its stage tree.
- **W02 note honored:** the toolchain contract's handoff reserved to W05 the
  possible re-homing or re-versioning of `docs/development/toolchain-baseline.md`.
  This design defaults to *no* re-homing (the current location already fits
  the inventory); any re-homing is a Reserved follow-up executed with that
  consent rule, never silently.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
