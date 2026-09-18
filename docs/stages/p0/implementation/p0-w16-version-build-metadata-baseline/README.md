# P0-W16 Version & Build Metadata Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The build identity and compatibility metadata baseline and its
extension points required by [P0-W16](../../plans/p0-w16-version-build-metadata-baseline.md).  
**Owner/change context:** P0-W16 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W16. It converts the bounded
work-package plan into small, reviewable documentation and policy changes: one
normative version/build metadata contract document, documentation-discovery
wiring, and two cross-reviews with the diagnostics and artifact-naming rules.
It deliberately does **not** create a build script, manifest, or embedding
mechanism, choose a target triple, define ABI or machine-model version values
or formats, implement any diagnostic or telemetry path, or configure CI. Those
belong to W03, W04, W12, future ABI/machine-model designs, and W20
respectively.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
[the metadata contract](01-metadata-contract.md) for the required artifact
content and [the implementation workflow](02-implementation-and-review.md) for
ordered steps and the validation matrix. Before editing it must also follow the
Coding Guidelines preflight, including the repository `AGENTS.md`, documentation
index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P0 task book](../../task-book-v0.1.md), and the P0-W16 plan. This document is
the proposed detailed design for those changes; it is not a completion record
and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W16 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W16 is: build identity and compatibility metadata
  have a **defined baseline and extension point** (P0-V14: a target artifact
  can be associated with source and declared compatibility metadata).
- The plan scope names the governed subjects: project version, revision,
  profile, target, timestamp policy, dirty indicator, capability summary, and
  the reserved ABI/machine-version positions. The plan's out-of-scope clause
  forbids freezing the final wire format and defining the future ABI or machine
  model; this design therefore fixes the *field schema and policies*, never a
  binary or wire representation.
- ADR-040 requires `schema_version`, `machine_version`, and
  `management_abi_version` to be versioned independently of the project. W16
  reserves their positions in the metadata model and must not give them values,
  formats, or compatibility rules.
- ADR-047 separates build profiles from features and runtime policy. Profile
  semantics are W04's deliverable; W16 defines the metadata *slot* that
  references them.
- W03 owns the AArch64 target boundary and the first baseline artifact's
  production. W16 defines the identity questions and fields any produced
  artifact must be associable with; it must not name a triple or an output
  path.
- W12 (diagnostics baseline) requires every diagnostic record to be associable
  with build/version identity and explicitly aligns that requirement with W16;
  W16 supplies the minimum identity set that alignment consumes.
- No target artifact exists in the repository today, so W16 closure can prove
  the metadata contract and its cross-reviews; the artifact-level association
  demanded by P0-V14 is proven by the producing package's evidence and the
  stage-level artifact/build identity review, not by W16 alone.

Classification: everything in [the metadata contract](01-metadata-contract.md)
§§2–4 and the cross-review obligations are **Required** for W16 closure. The
`capability_summary` population semantics (W04), concrete association mechanics
such as embedding or sidecar production (owned by the producing package's
design, first W03), compatibility version values and formats (future ABI and
machine-model designs), and automated metadata checking (a candidate future
W07/W20 gate) are **Reserved** with recorded triggers. Wire-format freezes,
ABI/machine-model definitions, target triples, build scripts, embedding code,
diagnostic implementations, and CI configuration are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Artifact identity questions and the source of each answer | [Metadata contract](01-metadata-contract.md) §2 | P0-V14 (W16-DV01, DV03) |
| Identity field schema and required subset | [Metadata contract](01-metadata-contract.md) §3 | P0-V14 (W16-DV01) |
| Timestamp and dirty-tree reproducibility/traceability policy | [Metadata contract](01-metadata-contract.md) §4 | P0-V14 (W16-DV02) |
| Project/build identity distinguished from ABI and machine versions; positions reserved | [Metadata contract](01-metadata-contract.md) §5 | P0-V14 (W16-DV01, DV02) |
| Consistent reference from diagnostics (W12) and naming (W17) | [Metadata contract](01-metadata-contract.md) §6, [workflow](02-implementation-and-review.md) step 4 | P0-V09/P0-V14 (W16-DV04) |
| Document discoverability and coherence | [workflow](02-implementation-and-review.md) step 3 | P0-V09 (W16-DV05) |
| Downstream consumability by W17/W19/W12 and P1+ | [workflow](02-implementation-and-review.md) handoff checklist | W16 closure review (W16-DV07); artifact-level P0-V14 proof arrives with the producing package |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, tracked tree at `4e631ee`): no tracked
document declares a project version, a source-revision convention, a build
metadata field set, or a timestamp/dirty policy; no Cargo workspace, manifest,
target definition, or build artifact exists; `.github/workflows/` contains only
a `.gitkeep` marker. [P0-W01 is completed](../p0-w01-repository-baseline/README.md) (root
navigation, license, tracked placeholders) and [P0-W02 has a proposed
design](../p0-w02-rust-toolchain-baseline/README.md); W03, W04, and W12 have
approved plans but no designs or implementation. Designs for several sibling packages
are being prepared in parallel on the same branch; this design references them
by slug and by P0-Wxx ID without assuming their content. ADR-040 already
requires independent versioning of schema, machine, and management-ABI
versions; no repository statement operationalizes that for build metadata. Each
ledger row states the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A target artifact can be associated with its source version | No version declaration or identity schema anywhere | Contract document declaring the identity field schema, each field's source, and a required subset for the first artifact | Without a declared schema there is nothing to associate an artifact with | W16 (this design) | W16-DV01 schema review; artifact-level proof deferred to the producing package (W03) |
| Association with declared build/compatibility information | No artifact exists; no profile or target definitions | Required-subset rule referencing W04 profile semantics and W03 target boundaries by contract, not by value | The first artifact must be born against a fixed requirement, not retrofit one | W16 schema; W04/W03 first population | W16-DV01/DV02; P0-V14 stage review once an artifact exists |
| Timestamp policy (work sequence 2) | Absent | Determinism-first timestamp policy: identity time derives from the source revision; wall-clock time is auxiliary only | An unreproducible identity field would silently break P0-V14's "declared" association | W16 policy | W16-DV02 policy review |
| Dirty-tree policy (work sequence 2) | Absent | Clean/dirty definitions, mandatory dirty marking, and a clean-only rule for evidence artifacts | A dirty build claimed as a clean revision falsifies source association | W16 policy | W16-DV02 |
| Project/build identity separated from ABI and machine versions; positions reserved (work sequence 3) | ADR-040 states the principle; no repository statement applies it to build metadata | Reserved field positions with an explicit non-definition rule | Conflating project version with ABI/machine versions would pre-empt independently versioned future contracts | ADR-040; W16 reserves positions only | W16-DV01/DV02 |
| Metadata consistently referenceable by diagnostics (work sequence 4, W12 alignment) | W12 plan requires identity association; no shared statement exists | Minimum diagnostic identity set plus a two-way cross-review obligation | Diagnostics that cannot cite identity cannot be traced to source (P0-V14's purpose) | W16 defines the set; W12 consumes | W16-DV04 cross-review (blocked-marked if the W12 contract is not yet in-tree) |
| Metadata consistently referenceable by artifact naming (work sequence 4, W17) | No naming rules exist; W17 is a parallel design | Name-encodable subset and a mapping owned jointly with W17 | Names and metadata that disagree give two answers to one identity question | W16 fields; W17 grammar | W16-DV04; W17-DV05 on its side |

No row above requires selecting a dependency, a target triple, a wire format,
or an ABI value, so no decision blocker is outstanding for this design. The
known deferred items (association mechanics, capability-summary semantics,
compatibility version values) are assigned to their owning designs above, not
left implicit.

## Resolved design decisions and their authority

1. **Policy home:** `docs/development/version-build-metadata.md` is the sole
   normative home of the identity questions, field schema, and timestamp/dirty
   policies. Rationale: development policy documents live under
   `docs/development/` (toolchain and integration-workflow precedents); W05 may
   later re-home it under its taxonomy without changing semantic ownership.
2. **Initial project version:** `0.1.0`, declared once, inside the contract
   document, with a single-declaration rule and a migration rule to the future
   workspace manifest (first created by the owning build-baseline design,
   expected W03 per its plan scope). Rationale: the repository has no version
   declaration today and its normative documents are the v0.1 generation; a
   project version is explicitly inside the plan scope (project version).
3. **Identity fields and required subset:** the schema in
   [the contract](01-metadata-contract.md) §3. The minimum set the first target
   artifact must be associable with is `project_version`, `source_revision`,
   `dirty`, `target_architecture`, and `build_profile`; `capability_summary`
   activates when W04's semantics land; `platform` activates with W03's
   platform boundary naming. Rationale: exactly the fields the task-book
   P0-V14 wording demands, nothing speculative.
4. **Timestamp policy:** determinism-first. The only identity timestamp is one
   derived from the source revision (its commit timestamp); wall-clock build
   time is auxiliary provenance, never part of identity and never required to
   reproduce an artifact. Rationale: the plan demands a reproducibility /
   traceability policy, and a wall-clock identity field would make identical
   sources produce non-identical identities.
5. **Dirty-tree policy:** clean and dirty are defined against tracked state;
   dirty builds are legitimate local development but their artifacts must carry
   the dirty indication, and release/gate/verification evidence artifacts must
   come from clean trees. Rationale: a dirty artifact presented as a clean
   revision would falsify source association — the exact failure P0-V14 exists
   to prevent.
6. **Compatibility reservations:** `schema_version`, `machine_version`, and
   `management_abi_version` appear in the schema as reserved positions with no
   values, formats, or compatibility rules. Rationale: ADR-040 makes them
   independently versioned; defining them here would pre-empt future ABI and
   machine-model designs and violate the plan's out-of-scope clause.
7. **Association mechanics deferred:** how fields reach or ride with an
   artifact (embedding, sidecar, or name encoding) is owned by the producing
   package's design — first W03 for the baseline artifact, W12 for runtime
   retrieval, W17 for the name-encoded subset. W16 owns the questions, fields,
   and policies only. Rationale: W16 precedes any build capability; fixing
   mechanics now would prescribe another package's design.
8. **Cross-review as a closure condition:** W16 is not closed until its
   contract has been cross-reviewed against the W12 identity-association
   requirement and the W17 naming grammar, or the review is recorded as
   blocked-by-prerequisite with the conflict surface named. Rationale: plan
   work sequence 4 makes the consistency check part of the package, not a
   courtesy.

## Work breakdown and loading order

1. Read [the metadata contract](01-metadata-contract.md): the artifact-group
   ownership table, the mandatory identity questions, the field schema, the
   timestamp/dirty policies, the compatibility reservations, and the W12/W17
   linkage rules.
2. Apply the changes in the order stated in
   [the implementation workflow](02-implementation-and-review.md): record the
   prerequisite status assumptions, write the contract document, wire
   discovery, run the cross-reviews, then close with the validation matrix.
3. Store actual commands, review output, and run/blocked status in
   `../../verification/p0-w16-version-build-metadata-baseline-verification.md`,
   and record changed artifacts, the declared version, and any deviation in
   `../p0-w16-version-build-metadata-baseline-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W16 complete; the artifact-level half of P0-V14 is delivered by the
   producing package and the stage review.

## Design-level state and lifecycle

W16 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked contract document plus its discovery links.
Their documentary lifecycle:

```text
no identity/metadata declaration
  -> version-build metadata contract committed (questions, schema, policies,
     reservations, linkage rules)
  -> single project-version declaration live (0.1.0)
  -> discovery links (docs/README routing row, stage index row) committed
  -> cross-reviews with W12 and W17 recorded (or blocked-with-surface)
  -> first target artifact produced against the required subset (producing
     package, expected W03)
  -> later changes mutate only through the contract's mutation rules
     (W04 activates capability_summary; W17 encodes the name subset;
      W19 references identity in the workflow; future ABI/machine designs
      fill the reserved compatibility positions)
```

The contract document is the owner of every policy statement; no other tracked
file may declare an identity field, a version, or a timestamp/dirty rule. A
conflict between this contract and a producing package's design is a review
failure to reconcile in the same change, not a local choice; if reconciliation
is impossible it is raised as a cross-package design conflict and, where an
ADR-level constraint is touched, labelled `ADR Required`.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, Cargo manifest, target triple,
build script, environment-variable contract, wire format, binary layout, or
public API is designed or authorized by W16. The three reserved compatibility
positions are named but undefined by explicit ADR-040 authority; giving any of
them a value or representation requires the applicable future design. Adding
any excluded item is a scope conflict and must be stopped at review.

## Downstream handoff

- **W17** receives the identity field set and the rule that artifact names may
  encode only a declared subset of it; the field-to-name mapping is W17's
  grammar with W16's fields as the authority. W16 must not have written the
  grammar.
- **W19** receives the identity questions as the "which artifact am I looking
  at" step of the clone-to-verified path, and the clean-tree requirement as
  the precondition for evidence-producing runs.
- **W12** receives the minimum diagnostic identity set for its per-record
  association requirement; the alignment cross-review is W16's closure
  condition and W12's consumption point.
- **P1+** (EL2 bring-up, crash diagnostics, later artifact production) receive
  the reserved compatibility positions and the extension rules; filling them
  requires the consumer's own design, never an edit of this contract's
  reservations.
- **W03** receives the required subset as a production requirement for the
  first target artifact; the association mechanics are W03's design freedom
  within the contract's field semantics.
- **W20** may later consume automated metadata checks as candidate CI checks;
  W16 defines no check implementation.
