# P0-W17 Artifact Naming Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The machine-processable artifact-naming grammar, dimension
vocabularies, category inventory, and evolution rules required by
[P0-W17](../../plans/p0-w17-artifact-naming-baseline.md).  
**Owner/change context:** P0-W17 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W17. It converts the bounded
work-package plan into small, reviewable documentation and policy changes: one
normative artifact-naming contract document (grammar, vocabularies, evolution
rules, category inventory), documentation-discovery wiring, and cross-reviews
with the build-metadata contract (W16) and the workflow/CI consumers
(W19/W20). It deliberately does **not** freeze any artifact file format or
generation process, create a build script or output directory layout, name a
target triple, choose a profile value, define an ABI or machine model, or
configure CI. Formats and generation belong to the owning designs (first W03);
profiles to W04; CI to W20.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
[the naming contract](01-naming-contract.md) for the grammar and rules,
[the category inventory](02-artifact-category-inventory.md) for the required
class table, and [the implementation workflow](03-implementation-and-review.md)
for ordered steps and the validation matrix. Before editing it must also follow
the Coding Guidelines preflight, including the repository `AGENTS.md`,
documentation index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P0 task book](../../task-book-v0.1.md), and the P0-W17 plan. This document is
the proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W17 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W17 is: artifact names are **machine-processable
  and distinguish relevant dimensions** (P0-V14: naming rules are associable
  with build identity).
- The plan scope names the governed subjects: naming dimensions, stability,
  machine processability, and coverage of future artifact categories. The
  plan's out-of-scope clause forbids freezing the final file format or
  generation process of each future artifact; the grammar therefore fixes
  *names*, never contents, and per-category extensions stay informative until
  the owning design declares the format.
- ADR-054 leaves the official project name pending (the root README's
  "Zelyr" is the repository working name). The grammar keeps a name field
  whose value is the working token and whose final value follows ADR-054;
  W17 must not freeze an official name.
- ADR-002 makes AArch64 the first architecture; ADR-003 names QEMU `virt` and
  Orange Pi 3B/RK3566 as the reference and first hardware platforms. These
  ground the initial `arch` and `platform` vocabulary entries as designators;
  the concrete target definitions remain W03's and platform governance stays
  with the platform-guardrails owner (P0-W11 for rules, later designs for
  values).
- ADR-047 and W04 own build-profile semantics; the `profile` vocabulary is
  populated from W04's delivered governance, not invented here.
- W16 owns the identity field schema; a name is a *view* of that identity, so
  every name field must trace to a W16 schema field. W16's contract is a
  prerequisite by subject even though its delivery may still be a parallel
  proposed design.

Classification: the grammar (including character set and parseability rules),
the vocabularies and their governance, the category inventory, the identity
linkage mapping, and the evolution rules are **Required** for W17 closure. The
first concrete artifact name instance (arrives with W03's baseline artifact),
official name resolution (ADR-054), concrete platform/profile value
populations (W03/W04), and any automated name-conformance checking (candidate
future W07/W20 gate) are **Reserved** with recorded triggers. File formats,
generation processes, output directory layouts, target triples, ABI or machine
models, and CI configuration are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inventory of future artifact categories | [Category inventory](02-artifact-category-inventory.md) | P0-V14 (W17-DV03) |
| Distinguishable naming dimensions | [Naming contract](01-naming-contract.md) §3, §5 | P0-V14 (W17-DV01, DV02) |
| Machine processability (grammar, character set, parseability) | [Naming contract](01-naming-contract.md) §4 | P0-V14 (W17-DV01) |
| Stability and no human-memory reliance | [Naming contract](01-naming-contract.md) §6 | P0-V14 (W17-DV02) |
| Association with build identity (W16) | [Naming contract](01-naming-contract.md) §7 | P0-V14 (W17-DV05) |
| Supports the P0 target artifact without limiting future formats | [Naming contract](01-naming-contract.md) §8, [inventory](02-artifact-category-inventory.md) | P0-V14 (W17-DV04, DV06) |
| Document discoverability and coherence | [workflow](03-implementation-and-review.md) step 4 | P0-V09 (W17-DV06) |
| Downstream consumability by W19/W20 and later build/validation work | [workflow](03-implementation-and-review.md) handoff checklist | W17 closure review (W17-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, tracked tree at `4e631ee`): no tracked
document defines an artifact naming rule, grammar, or category inventory; no
build artifact exists (no workspace, manifest, target, or build baseline);
`.gitignore` already ignores generated outputs — `/target/`, `/build/`,
`/dist/`, `/out/`, and the extensions `*.log`, `*.elf`, `*.bin`, `*.img`,
`*.qcow2` — so generated artifacts are untracked by existing policy; root
README uses the working name "Zelyr" while ADR-054 leaves the official name
pending. The [W16 metadata design](../p0-w16-version-build-metadata-baseline/README.md)
is proposed in parallel; W03/W04 have approved
plans without designs. Designs for sibling packages are being prepared in
parallel on the same branch and are referenced by slug and P0-Wxx ID without
assumed content. Each ledger row states the missing foundation the plan
outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Names are machine-processable | No naming rule anywhere | Positional grammar with fixed field order, declared separators, restricted character set, and parseability rules | Without a parseable grammar, names cannot be processed without human memory — the plan's stated failure to avoid | W17 (this design) | W17-DV01 grammar review |
| Names distinguish relevant dimensions | No dimension list | Dimension set (`name`, `class`, `arch`, `platform`, `profile`, `version`, `revision`, dirty modifier) with per-class applicability | Indistinguishable artifacts are the plan's named risk; dimensions are how distinction is achieved | W17 dimensions; values per owning packages | W17-DV01/DV02 |
| Future artifact categories covered (work sequence 1) | No inventory | Category inventory with reserved class tokens for the plan's listed categories and an extension rule | Categories registered now cannot be silently renamed or collided with later | W17 tokens; formats stay with owning designs | W17-DV03 inventory review |
| Stability; no reliance on human memory (work sequence 3) | Absent | Determinism rule (identity-derived names, no timestamps in names) and vocabulary tables as the token authority | A name only a human can interpret or reproduce is not machine-processable | W17 policy | W17-DV02 |
| Names associable with build identity (P0-V14) | W16 schema proposed in parallel | Field-to-identity mapping with W16's schema as the sole authority | A name and a metadata record that disagree give two answers to one identity question | W16 fields; W17 mapping | W17-DV05 cross-review (blocked-marked if W16 contract not yet in-tree) |
| Supports the P0 target artifact (work sequence 4) | No artifact exists; W03 not designed/implemented | Grammar-level support: `hypervisor` class applicability declared; concrete first name instance deferred to W03 | The grammar must be born against the first real consumer's shape without pre-empting W03's design | W17 grammar; W03 first instance | W17-DV04; first-instance check recorded `not run` until an artifact exists |
| Does not limit future formats (plan out-of-scope guard) | Risk of overfreezing | Per-category non-commitment rule: extensions informative; formats/generation owned by consuming designs | Freezing formats here would violate the plan's out-of-scope clause | W17 rule | W17-DV06 review |

No row above requires selecting a dependency, target triple, profile value, or
file format, so no decision blocker is outstanding for this design. One
pre-existing observation is recorded, not resolved here: the root README
working name ("Zelyr") and ADR-054's listed working names differ; ADR-054
already tracks the pending official-name decision (see [the naming
contract](01-naming-contract.md) §5.1).

## Resolved design decisions and their authority

1. **Policy home:** `docs/development/artifact-naming.md` is the sole
   normative home of the grammar, vocabularies, category inventory, identity
   mapping, and evolution rules. Rationale: development-policy precedent under
   `docs/development/`; W05 may re-home it later without changing semantic
   ownership.
2. **Grammar shape:** a positional, dash-separated grammar
   (`<name>-<class>-<arch>-<platform>-<profile>-<version>-<revision>[+dirty]`)
   with a single `-` field separator, `+dirty` as the only modifier, and
   per-class declared applicability (which dimensions a class carries or
   omits). Rationale: fixed arity plus a separator that cannot occur inside
   values makes names strictly parseable without context — the plan's core
   demand.
3. **Character set:** field values are lowercase `[a-z0-9]`, with `.` allowed
   only inside the `version` field; hyphens are forbidden inside values so the
   separator is unambiguous; multi-word designators are concatenated
   (e.g. `qemuvirt`, `orangepi3b`) and mapped to their repository names in the
   vocabulary tables. Extensions never carry identity information.
4. **Name field value:** the lowercase working token `zelyr`, taken from the
   root README's repository name, explicitly marked as subject to ADR-054;
   resolution of the official name is a reviewed migration, not a silent
   rename. Rationale: the grammar needs a deterministic value today without
   usurping ADR-054.
5. **Determinism:** governed names are pure functions of identity inputs
   (W16 fields); no wall-clock timestamps in names; repeated builds of the
   same identity may produce the same name and disambiguation belongs to the
   output layout of the owning build design, never to name mutation.
6. **Governed scope:** the grammar applies to artifacts that leave the build
   tree — published, archived as evidence, or consumed by other tooling. The
   build tool's internal layout (for example a future Cargo `target/` tree) is
   not governed by W17.
7. **Identity linkage:** every name field traces to a W16 identity field; the
   mapping table lives in the naming contract but W16's schema is the sole
   authority for field meaning. Names encode a bounded subset; they never
   replace the metadata record.
8. **Evolution thresholds:** adding a class token or vocabulary entry is an
   ordinary reviewed change; adding or reordering dimensions, or changing the
   character set, is a recorded policy decision that bumps the grammar version
   with migration guidance; a change that would freeze names into an external
   ABI contract is `ADR Required` (naming is not an ABI unless a future design
   makes it one).

## Work breakdown and loading order

1. Read [the naming contract](01-naming-contract.md) (grammar, character set,
   vocabularies, applicability, stability, identity mapping, evolution,
   scope boundary) and [the category inventory](02-artifact-category-inventory.md)
   (required class table and its columns).
2. Apply the changes in the order stated in
   [the implementation workflow](03-implementation-and-review.md): record
   prerequisite status assumptions, write the naming document, wire discovery,
   run the cross-reviews, then close with the validation matrix.
3. Store actual review commands, output, and run/blocked status in
   `../../verification/p0-w17-artifact-naming-baseline-verification.md`, and
   record changed artifacts and any deviation in
   `../p0-w17-artifact-naming-baseline-record.md` only when implementation
   begins. Neither this design nor a written record may claim W17 complete.

## Design-level state and lifecycle

W17 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked naming contract document (carrying the
category inventory) plus its discovery links. Their documentary lifecycle:

```text
no naming rules
  -> artifact-naming contract committed (grammar v0.1, vocabularies,
     inventory, mapping, evolution rules)
  -> discovery links (docs/README routing row, stage index row) committed
  -> cross-reviews with W16 (identity mapping) recorded (or blocked-with-
     surface); W19/W20 consumability noted
  -> first governed name instance appears with W03's baseline artifact
  -> later categories join only through the inventory's extension rules
     (validation-guest images, boot packages, snapshots, migration streams,
      reports ... as their owning designs arrive)
```

The naming document is the owner of every naming statement; the vocabulary
tables are the only token authorities. A produced artifact whose name deviates
from the grammar fails the producing package's review; fixing the artifact
name, not the grammar, is the default remedy. If reconciliation is impossible
the conflict is raised cross-package, and anything touching ADR-level
constraints is labelled `ADR Required`.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, Cargo manifest, target triple,
build script, output-directory layout, file format, binary layout, ABI, wire
format, or public API is designed or authorized by W17. Per-category expected
extensions in the inventory are informative placeholders, not format
decisions. Adding any excluded item is a scope conflict requiring the
applicable detailed design (at minimum W03 for the first artifact and its
output location, W04 for profile values, W16 for identity semantics, W20 for
CI) and must be stopped at review.

## Downstream handoff

- **W19** receives the grammar as the "identify the artifact you just built"
  step of the clone-to-verified path, including the host-vs-target distinction
  in governed scope.
- **W20** receives the grammar as the input for any future name-conformance or
  evidence-labeling check; W17 defines no check implementation.
- **W03** receives the `hypervisor` class applicability and the obligation to
  produce the first governed name instance with the baseline artifact; the
  output location and extension/format decision are W03's design freedom
  within the grammar.
- **W16** receives the mapping-table review: every name field traces to its
  schema, and the name stays a view, never a second identity authority.
- **W04** receives the `profile` vocabulary slot; its delivered profile
  semantics populate the tokens.
- **P1+** (validation-guest images P4+, boot packages P10, snapshots P16,
  migration P17, reports wherever evidence formats arrive) receive reserved
  class tokens and the extension rules; activating a category requires the
  consumer's own design for format and generation, never a silent grammar
  edit.
