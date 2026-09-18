# P2-W03 Boot Memory Map and Ownership Foundation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The normalized, checked host physical-memory map that identifies
every P2-protected range before any dynamic page allocation, required by
[P2-W03](../../plans/p2-w03-boot-memory-map-ownership.md).  
**Owner/change context:** P2-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W03. It converts the bounded
work-package plan into a code-bearing design for exactly one object system:
a two-phase boot memory map — a draft built from W02 facts plus P1's image
range, and a sealed immutable map after W04's allocator-metadata reservation
is recorded — with explicit conflict, overflow, and zero-size treatment and
a reserved extension path toward later ownership accounting. It deliberately
does **not** select or place allocator metadata itself (W04 plans it; W03
only records it), does not define `MemoryObject`/`MemoryRegion` (blocked by
[P2-ACR-01](../../task-book-v0.1.md), see
[01 §2](01-scope-and-foundations.md)), does not design Guest-memory transfer
or Stage-2 (P4), and does not decide any policy for releasing the original
DTB (Reserved).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the linked supporting file for its assigned step, after the Coding
Guidelines preflight (repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md),
[P2-W03 plan](../../plans/p2-w03-boot-memory-map-ownership.md)).

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, P2-ACR-01 handling, assumed contracts, page-size and representation decisions. |
| [02-architecture-and-state.md](02-architecture-and-state.md) | Range classification, conflict-policy rules, the draft→seal lifecycle, and ownership of every state. |
| [03-code-contracts-bootmap.md](03-code-contracts-bootmap.md) | Exact function/type contracts with pseudocode. |
| [04-implementation-workflow.md](04-implementation-workflow.md) | Ordered implementation steps with acceptance and failure handling. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | Validation matrix (P2-V05), error/security/observability model, handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W03 plan](../../plans/p2-w03-boot-memory-map-ownership.md) → this design
→ Coding Guidelines. Binding constraints:

- The task book's hard gate: **every protected physical range is excluded
  from allocation for every valid allocation/free sequence** (exit
  criterion 2). W03 is the sole authority that defines "protected"; W04 may
  only derive its allocation domain from this map.
- ADR-018 reserves ownership-oriented memory evolution; ADR §4 defines the
  later object model. P2-ACR-01 (task book §3) records that the ADR roadmap
  bullet and the source task book conflict about P2 defining minimal memory
  objects; this design follows the task book (no objects) and keeps
  P2-ACR-01 visible as unresolved.
- ADR-043/052: no board names; the map is built from discovered facts and
  declared P1 ranges only.
- Checked arithmetic on all externally influenced ranges (Coding
  Guidelines); typed frame/page/address newtypes (plan-agent guardrail;
  P0-W15 semantics).

Classification. **Required:** RAM collection from W02, protection of the
hypervisor image, active DTB, DTB reservation block, `/reserved-memory`
ranges, boot artifacts, and allocator metadata; the conflict/overflow/
zero-size policy; map normalization; the draft→seal lifecycle; the
protected-source ledger; accounting totals. **Reserved** (recorded
triggers): extension of `RegionClass`/`ProtectedSourceId` for later
ownership states (VM-owned, shared, DMA-pinned, COW, balloon), a DTB
copy/release policy, NUMA/zone attributes, unprotect/reprotect operations.
**Out of Scope:** allocator algorithm/metadata placement (W04), P4
Guest-memory transfer and Stage-2, `MemoryObject`/`MemoryRegion` (P2-ACR-01),
DTB release policy, inspection output (W06), and Orange Pi runtime support.

## Requirement-to-design mapping

The tracked sources define P2-D01–D08 and P2-G01–G03 at group granularity
only; rows below are this design's reviewable enumeration from the plan's
scope wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-D01 | Collect RAM banks from W02 without assuming one bank or a QEMU layout | [03 §4](03-code-contracts-bootmap.md) | P2-V05 (W03-DV01) |
| P2-D02 | Protect the P1-authoritative hypervisor image range | [02 §2](02-architecture-and-state.md), [03 §4](03-code-contracts-bootmap.md) | P2-V05 (W03-DV02) |
| P2-D03 | Protect the active DTB and its validated reservation-block entries | [02 §2](02-architecture-and-state.md), [03 §4](03-code-contracts-bootmap.md) | P2-V05 (W03-DV03) |
| P2-D04 | Protect `/reserved-memory` ranges and boot artifacts (initrd) | [02 §2](02-architecture-and-state.md), [03 §4](03-code-contracts-bootmap.md) | P2-V05 (W03-DV04) |
| P2-D05 | Checked overlap, overflow, zero-size, and unaligned-range treatment with explicit outcomes | [02 §4](02-architecture-and-state.md), [03 §5](03-code-contracts-bootmap.md) | P2-V05 (W03-DV05, DV06) |
| P2-D06 | Normalized map: sorted, disjoint, classified partition of discovered RAM | [02 §5](02-architecture-and-state.md), [03 §6](03-code-contracts-bootmap.md) | P2-V05 (W03-DV07) |
| P2-D07 | Allocatable domain = RAM minus every protected range; no protected page allocatable | [02 §4](02-architecture-and-state.md), [03 §6](03-code-contracts-bootmap.md) | P2-V05 (W03-DV08); the hard gate |
| P2-D08 | Allocator metadata becomes protected before any allocation begins (two-phase seal) | [02 §3](02-architecture-and-state.md), [03 §7](03-code-contracts-bootmap.md) | P2-V05 (W03-DV09) |
| P2-G01 | `RegionClass` extensible without redesign | [03 §3](03-code-contracts-bootmap.md) | W03 closure review (W03-DV10) |
| P2-G02 | `ProtectedSourceId` ledger with stable identities for P4 ownership extension | [02 §6](02-architecture-and-state.md), [03 §3](03-code-contracts-bootmap.md) | W03 closure review (W03-DV10) |
| P2-G03 | Accounting totals with extension room; no memory objects claimed | [03 §8](03-code-contracts-bootmap.md), [01 §2](01-scope-and-foundations.md) | P2-V05 review; P2-ACR-01 visible |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, no W01/W02
implementation, and no P1 image-range fact. W03 is designed against W02's
fact records and W01's validated DTB range as assumed prerequisites; the P1
image range is an assumed P1 contract exactly as in
[W01 §2](../p2-w01-boot-platform-description-intake/01-intake-boundary.md)
(A3).

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Normalized, checked map identifying every P2-protected range (P2-V05) | No map code or facts exist | Map builder over W02 records + P1 range with the §4 conflict policy | Protection must exist before allocation is even possible | W03 (this design), consuming W01/W02 outputs | W03-DV01–DV08 host tests |
| Explicit fatal outcomes for incompatible ownership (plan step 3) | n/a | Conflict-policy table with fatal classes | Silent overlap resolution would break the hard gate | W03 | W03-DV05/DV06 |
| Sole permitted allocation domain identified (plan step 1; W04 step 1) | n/a | Sealed map with `allocatable_spans()` query | W04 must have exactly one authority | W03; consumed by W04 | W03-DV08 + W04 consumption review |
| Reserved/ownership extension foundation (P2-G01–G03) | n/a | Extensible classification + source ledger + totals | ADR-018 evolution requires stable extension points | W03 | W03-DV10 review |
| W02 facts and P1 image range available (plan step 1) | Not implemented upstream | Assumed contracts with failure boundaries | Task book §2 upstream-defect rule | W01/W02/P1 owners | Upstream verification when available; host fixtures meanwhile |
| P2-ACR-01 remains visible (task book §3) | Recorded in task book | Restated here; no object design | Conflict is unresolved and must block object work | ADR owner | P2-V13 stage review |

No ledger row selects an allocator or memory-object design; the absent
upstream implementations are ordered prerequisites handled with host
fixtures.

## Resolved design decisions and their authority

1. **Page size fixed at 4 KiB for the P2 map and allocator domain.**
   Rationale: the reference platform and fixture use 4 KiB granules, and
   P0/P1 target decisions (assumed contracts) are designed around the AArch64
   4 KiB granule; frame-based bookkeeping needs one fixed granule. A different
   granule is a design change, not a constant edit.
2. **Frame-based internal representation.** The map stores
   `PhysFrameNum`/`PageCount` spans; byte ranges from W02 are converted once
   at the boundary with checked arithmetic, and unaligned RAM bank bounds are
   fatal ([02 §4](02-architecture-and-state.md)). Rationale: page-granular
   allocation is the package's purpose; byte-vs-frame confusion is the
   classic bug class the typed newtypes exist to prevent.
3. **Protected-wins conflict policy.** A protected range overlapping RAM
   clips the RAM (recorded); protected-protected overlap is fatal except
   exact duplicates from the same source class; RAM-RAM overlap is fatal.
   Rationale: firmware protection statements are authoritative over the RAM
   description (the hard gate is about never allocating them), but two
   disagreeing protection statements are unresolvable — choosing one would
   invent facts.
4. **Two-phase draft→seal lifecycle.** W04 plans metadata against the draft;
   sealing records the metadata range as protected and freezes the map. The
   allocator initializes only from the sealed map. Rationale: makes "metadata
   is protected before allocation begins" (P2-D08) a structural property
   instead of a sequencing hope, and keeps the sealed map immutable.
5. **Zero-size and out-of-RAM protected entries are recorded, not fatal;**
   zero-size RAM banks are dropped with a counter; out-of-RAM protected
   ranges are recorded with a warning. Rationale: harmless anomalies must be
   observable without stopping boot, while every genuinely ambiguous
   *ownership* situation is fatal (Decision 3).
6. **The map claims nothing outside discovered RAM.** MMIO and other
   non-RAM physical areas are not represented; P2's map is the RAM map.
   Rationale: task book scope is boot RAM/protected ranges; device maps are
   later-stage discovery scope.
7. **No DTB release, no unprotect path.** The sealed map has no mutation
   API in P2. Rationale: Reserved per the plan; a release policy needs a
   superseding design (it would touch W01's handle lifetime and W04's free
   lists).

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md), then
   [02-architecture-and-state.md](02-architecture-and-state.md) for the
   classification and conflict policy before any code.
2. Implement per [04-implementation-workflow.md](04-implementation-workflow.md);
   contracts in [03-code-contracts-bootmap.md](03-code-contracts-bootmap.md).
3. Validate per [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Record decisions/deviations in
   `../p2-w03-boot-memory-map-ownership-record.md` and evidence in
   `../../verification/p2-w03-boot-memory-map-ownership-verification.md` when
   that work starts; nothing here claims W03 complete.

## Explicitly excluded interfaces

No allocator types or calls (W04); no `MemoryObject`/`MemoryRegion` or any
ownership-transfer operation (P2-ACR-01); no Stage-2/Guest-memory concepts
(P4); no mutation of the sealed map; no serialization of the map (a versioned
export format, if ever needed, is a later design); no board names. W06
consumes query results only; W04 consumes `allocatable_spans()` and the
sealing handshake only.

## Downstream handoff

- **W04** ([../p2-w04-physical-page-allocation/README.md](../p2-w04-physical-page-allocation/README.md))
  receives the draft map for metadata planning, the seal handshake, and the
  sealed map as the sole allocation-domain authority. W04 may not bypass
  queries or recompute protection.
- **W06** ([../p2-w06-platform-memory-inspection/README.md](../p2-w06-platform-memory-inspection/README.md))
  renders map/classification/accounting queries against the sealed map.
- **W08/W09** receive the conflict-policy table and accounting totals as
  regression and QEMU-integration expectations.
- **P4** (via W10; see p4-w02/p4-w03 plans) receives the protected-range
  ledger and the extension foundation for ownership accounting — explicitly
  not a Guest-memory mechanism and not memory objects.
