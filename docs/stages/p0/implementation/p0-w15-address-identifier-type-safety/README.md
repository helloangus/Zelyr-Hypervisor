# P0-W15 Address & Identifier Type-Safety Requirement — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The address-space and resource-identity semantics future designs
must keep distinct, the explicit-conversion principle at boundaries, and the
design/code review requirements required by
[P0-W15](../../plans/p0-w15-address-identifier-type-safety.md).  
**Owner/change context:** P0-W15 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W15. It converts the bounded
work-package plan into one normative type-safety requirements document, the
discovery wiring that makes it findable at design entry, and a
non-prescription review. It deliberately does **not** define any Rust
newtype, field, trait, constructor, conversion API, width, or module
placement (plan out-of-scope), does **not** create a crate or any code (the
tree has no workspace), and does **not** decide where the future address/ID
types live — that belongs to the P1+ designs that introduce the first
address- or identifier-bearing interfaces.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) — whose
newtype and checked-arithmetic rules this package complements at design-entry
level, without restating them. Before editing it must also follow the Coding
Guidelines preflight, including the repository `AGENTS.md`, documentation
index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
P0 task book, and P0-W15 plan. This document is the proposed detailed design
for those changes; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W15 plan → this
design → Coding Guidelines. In particular:

- ADR §13 suggests newtype encapsulation of HPA/IPA/GVA/VMID/vCPU IDs to
  avoid bare-integer conflation; the Coding Guidelines make the
  implementation rule binding (semantic newtypes for HPA/HVA/GPA/GVA, IDs,
  lengths, pages, handles; no naked `usize`/`u64`; checked arithmetic and
  conversions for externally influenced calculations). What neither provides
  — and what this package delivers — is the authoritative *semantic
  inventory* (which address spaces and identities exist and must never
  interchange) and the *design-entry review requirements* applied before any
  type exists.
- The task-book outcome for W15 is: future addresses and identifiers
  **retain semantic type distinctions** (P0-V09), with the constraints
  discoverable at design entry and without falling into concrete API design.
- Interpretation of the task-book P0 task-list bullet naming "新类型 ID、
  地址类型": the P0 deliverable is the requirement baseline this design
  defines, not the types. Task book §1 places crate boundaries, module
  trees, data layouts, and API signatures out of P0 scope, and exit
  criterion 6 forbids P0 deliverables from encoding future implementation
  decisions; the W15 plan confirms the package defines no Rust newtype or
  conversion API. The concrete types arrive with the P1+ designs that first
  need them. This reading is recorded as an open question for owner
  confirmation (workflow §5); resolving it the other way would be a scope
  change through [W06](../p0-w06-adr-governance/README.md)'s mechanism, not
  a local edit.
- Prerequisite status: [W05](../p0-w05-documentation-baseline/README.md)
  (document conventions) is a proposed design, not a delivery; the current
  tree's `docs/README.md` header mandate is the fallback convention.
- Consumers: P1+ architecture, platform, memory, VM, and IRQ designs (per
  their stage plan indexes: host address space, Stage-2, page allocation,
  handle lifecycle, interrupt routing). [W11](../p0-w11-platform-portability-guardrails/README.md)
  and [W14](../p0-w14-panic-failure-classification/README.md) reference this
  package's subject in their checklists by slug; this design returns the
  citation without depending on their content.

Classification: the type-safety requirements document (address-semantics
inventory, identity-semantics inventory, interchange and conversion rules,
review checklists, reserved semantics with triggers) and its discovery wiring
are **Required** for W15 closure. Concrete type definitions, widths,
conversion functions, trait designs, crate placement, and any future
address/ID-bearing API are **Reserved** to the P1+ designs that introduce
them. All code, crates, module trees, and CI artifacts are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Address semantics that must be separated (work item 1) | [Semantics contract](01-semantics-and-review-contract.md) §3 | P0-V09 (W15-DV01) |
| Identity semantics listed with non-interchangeability (work item 2) | [Semantics contract](01-semantics-and-review-contract.md) §4 | P0-V09 (W15-DV02) |
| Design/coding review items requiring explicit semantics at conversions and boundaries (work item 3) | [Semantics contract](01-semantics-and-review-contract.md) §5–§6 | P0-V09 (W15-DV03) |
| The requirement does not reverse-specify module or object implementations (work item 4) | [workflow](02-implementation-and-review.md) step 4 | P0-V09 (W15-DV04) |
| Constraints discoverable at design entry (P0-V09) | [workflow](02-implementation-and-review.md) step 3 | P0-V09 (W15-DV05) |
| Downstream consumability by P1+ designs | [workflow](02-implementation-and-review.md) handoff checklist | W15 closure review (W15-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at `4e631ee`): no tracked document inventories
the address or identity semantics, states interchange prohibitions, or
provides review items for conversion explicitness. ADR §13's one-line
suggestion and the Coding Guidelines' implementation rule are the only
type-safety statements; both address code, not design entry. No Rust source
exists, so no violation can exist yet. W01 is completed; W05 (prerequisite)
is a proposed design. Each ledger row below states the missing foundation
the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Address semantics are enumerated and separated (work item 1) | ADR names HPA/IPA/GVA examples in one line; no inventory | Address-semantics inventory: host virtual, host physical, guest physical/IPA, guest virtual, with meaning and validity notes | P1+ memory/address designs need one authoritative list so no design re-derives it differently | W15 (this design) within ADR §13/ADR-018 subjects | W15-DV01 |
| Identity semantics are enumerated and non-interchangeable (work item 2) | ADR names VMID/vCPU-ID examples; no inventory | Identity-semantics inventory: VM, vCPU, physical CPU, hardware VMID, IRQ (physical/virtual), plus reserved identities | Conflations like software VM identity vs AArch64 Stage-2 VMID are classic defects the inventory forecloses | W15 | W15-DV02 |
| Conversion explicitness is a review requirement (work item 3) | Absent | Interchange/conversion rules plus design-review and code-review checklist items | "转换必须显式化" must be checkable when designs and code are reviewed | W15; Coding Guidelines restated by pointer only | W15-DV03 |
| No reverse-specification of implementations (work item 4) | Not verified | Non-prescription statement plus walkthrough confirming no type/trait/width is implied | Acceptance demands the constraint stay semantic | W15 | W15-DV04 |
| Discoverable at design entry (P0-V09) | No routing row; nothing for a P1 planner to find | `docs/README.md` routing row and stage index row | A P1 design that cannot find the red lines will not follow them | W15 | W15-DV05 |
| P1+ designs can consume | Nothing to consume | Handoff statements to P1+ design authors | Handoff is the plan's stated purpose (语义红线) | W15 delivers; consumers cite | W15-DV06 |

No row requires a type, trait, width, or crate, so no decision blocker is
outstanding for this design.

## Resolved design decisions and their authority

1. **Normative home:** `docs/development/address-identifier-type-safety.md`
   is the sole normative home of the inventories, interchange rules, and
   review requirements, following the development-policy pattern
   (W02/W04/W05/W07/W11/W12/W13/W14).
   [W05](../p0-w05-documentation-baseline/README.md) may re-home it;
   semantic ownership stays with the document.
2. **Inventories are semantics, never types.** Each inventory entry states:
   semantic name and acronyms, meaning, typical producers/consumers,
   qualitative validity concerns (range/alignment/uniqueness), and
   interchange prohibitions. No Rust representation, width, field, trait, or
   constructor is stated; the first P1+ design that introduces a type
   decides representation within the Coding Guidelines' rules.
3. **Address inventory: four required semantics.** Host virtual address
   (HVA), host physical address (HPA), guest physical address (GPA/IPA —
   one semantic, both acronyms recorded, with the note that the project's
   Stage-2 subject uses IPA), guest virtual address (GVA). Reserved with
   trigger: device-visible/DMA-translated addresses (a distinct semantic
   only where translation exists — the IOMMU/SMMU designs introduce it;
   P0 does not name it as a type or field).
4. **Identity inventory: five required semantics plus reservations.**
   Required: VM identifier (the software object identity per ADR-013/ADR-051
   — carries no privilege), vCPU identifier (VM-scoped), physical CPU
   identifier, AArch64 Stage-2 VMID (the hardware TLB tag — explicitly a
   *different* semantic from the VM identifier, never interchangeable, never
   user-visible as authority), and interrupt identifiers (physical INTID
   space and virtual interrupt space stated as distinct semantics; the GIC
   designs refine their sub-kinds). Reserved with triggers: capability
   handle (generation-bearing — P5's subject), memory-object/region
   identifiers (P2), device identifiers (device-framework stage), domain
   identifiers (bootstrap/management stage).
5. **Interchange and conversion rules.** Distinct semantics never pass as
   bare integers across an interface boundary. A value changes semantic only
   through an explicit, named translation at a boundary the design declares
   (for example a Stage-2 translation producing a new HPA value — never a
   cast or reinterpretation of the same value). A translated value is a new
   value in the target semantic with its own validity. Identifiers carry no
   arithmetic or ordering unless their design declares one; addresses follow
   the Coding Guidelines' checked-arithmetic rule with explicit units
   (bytes/pages) stated at the operation. Externally influenced values
   (guest, firmware, discovery data) enter only as validated values in their
   destination semantic.
6. **Review checklists inside the document.** Design-review items: bare-
   integer interfaces for inventoried semantics; conversion naming;
   validity/width statements left unstated; untrusted-entry validation;
   translation-presented-as-cast. Code-review items: newtype use per the
   Coding Guidelines (restated by pointer, not by copy); no transmute or raw
   reinterpretation between semantics; no arithmetic on identifiers;
   suggested search techniques noted as techniques, not gates.
7. **Non-prescription is structural.** The document contains no type names,
   no trait shapes, no widths, no module or crate placement, and no
   conversion function signatures; its walkthrough (workflow step 4) must be
   completable without implying any of them. Adding such content under W15
   is the scope violation the review is required to catch.

## Work breakdown and loading order

1. Read [the semantics contract](01-semantics-and-review-contract.md) for
   the artifact groups, both inventories, the interchange rules, and the
   checklists.
2. Apply the changes in the order stated in
   [the implementation workflow](02-implementation-and-review.md): verify
   prerequisite surfaces, author the requirements document, wire discovery,
   run the non-prescription walkthrough, close.
3. Store actual review commands, output, environment, and result in
   `../../verification/p0-w15-address-identifier-type-safety-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w15-address-identifier-type-safety-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W15 complete.

## Design-level state and lifecycle

W15 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked requirements document plus its discovery
links. Their documentary lifecycle:

```text
type-safety stated only as an ADR suggestion and coding rules
  -> address-identifier-type-safety.md committed (inventories, interchange
     rules, review checklists, reserved semantics)
  -> docs/README.md routing row + stage index row committed
  -> non-prescription walkthrough evidenced
  -> P1+ designs cite the red lines at design entry; the first address/ID-
     bearing design defines its types within the Coding Guidelines' rules
  -> reserved semantics (DMA-visible addresses, handles, object/device/
     domain IDs) activate only through their owning designs
  -> inventory changes only through the document's thresholds
```

The requirements document owns every semantic statement. A future design
that passes inventoried semantics as bare integers, or implies an implicit
conversion, is a review failure, not a local choice.

## Explicitly excluded interfaces

No Rust newtype, struct, field, trait, constructor, conversion function or
trait impl, width, module, crate, public API, ABI, or CI workflow is designed
or authorized by W15. The inventories are semantic; the checklists are review
surfaces. Defining any excluded item under W15 is a scope conflict against
the P1+ designs that own their subjects and must be stopped at review.

## Downstream handoff

- **P1+ architecture, platform, memory, VM, and IRQ designs** (per their
  stage plan indexes) receive the semantic red lines: every interface over
  an inventoried semantic is typed or explicitly justified, every conversion
  is a named translation, and every untrusted entry point validates before
  use; a design that cannot show this is defective at design review.
- **The Coding Guidelines** are complemented, not restated: W15 supplies the
  design-entry inventory the implementation rules presuppose; the document
  links to the guidelines for the coding-side rules.
- **W11** is honored in reverse: its R6 checklist item cites this document;
  W15 adds nothing to W11's layering rules.
- **W14** receives the note that classification and containment reviews
  assume address/identity distinctions are type-level, not convention-level;
  no taxonomy change is implied.
- **Future owning designs** (Stage-2/VMID in the P4 subjects, handles in P5,
  interrupt identifiers in P6, IOMMU-visible addresses in the SMMU stage)
  receive reserved slots with activation triggers; activating one is their
  design decision, never an edit here.
- **W05** may re-home the document; **W22** maps this deliverable into the
  P1 handoff package by its own plan scope.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
