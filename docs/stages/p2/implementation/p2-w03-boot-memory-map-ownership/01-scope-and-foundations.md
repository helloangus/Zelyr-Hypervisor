# P2-W03 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W03 detailed design](README.md).

## 1. Package outcome

One sealed `BootMemoryMap` with this property: every physical frame of every
discovered RAM bank is classified exactly once — `Allocatable` or one of the
protected classes — the classification is checked (no overlap, no overflow,
no ambiguity), and no operation available to any P2 consumer can return or
reclassify a protected frame. The map is built in two phases (draft, seal)
so allocator metadata is protected before allocation exists.

## 2. P2-ACR-01 and the object-model boundary

The ADR roadmap bullet for P2 says to define minimal `MemoryObject`/
`MemoryRegion` structures; the source task book says P2 must not design P4's
object system. The P2 task book records this conflict as **P2-ACR-01 — ADR
Required** and resolves it for planning by having P2 define neither object.
This design inherits that resolution: the map defines ranges, classes, and a
source ledger — data needed by *any* future object model — and stops there.
If an ADR clarification later authorizes P2 object work, that is a
superseding design; nothing here may be silently reinterpreted as object
 groundwork. This restatement is reviewable under P2-V13.

## 3. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W03 relies on | Failure boundary |
|---|---|---|---|---|
| A1 | W02 `PlatformInfo`: RAM banks, `/reserved-memory` ranges (with sources/flags), boot artifacts, all as decoded records | [W02 design](../p2-w02-platform-discovery-normalization/README.md) | Records are decoded, checked-composed, DT-order, capacity-bounded | A shape mismatch is a W02 contract revision — design conflict, recorded; W03 never re-walks the DTB |
| A2 | W01 validated DTB range and reservation-block entries | [W01 design](../p2-w01-boot-platform-description-intake/README.md) | One DTB span; terminated, checked rsvmap list incl. zero-address flags | As A1 |
| A3 | P1 hypervisor image physical range (authoritative, single) | P1-W01/P1-W08 (assumed, unimplemented) | One immutable span; absence is a blocked upstream defect (same rule as W01 A3) | No image-range fact → map cannot protect the image → blocked; skipping protection is forbidden |
| A4 | P0 address newtypes and diagnostics channel | P0 plans (assumed) | `PhysAddr`, frame/page newtypes or the basis to define P2-local ones | Missing primitives → blocked upstream defect; no naked integers |

Host-side development: the whole map builder is pure logic over injected
records; W03 fixtures construct synthetic fact sets. QEMU accounting checks
are W09 evidence.

## 4. Page size, representation, and arithmetic policy

- **Granule:** 4 KiB (README Decision 1). All internal spans are
  frame-based: `PhysFrameNum` (P2-owned newtype over u64, or the P0
  equivalent when it exists — A4), `PageCount`, `PhysFrameRange { first,
  count }`. `PhysAddr`/`ByteLen` appear only at the input boundary.
- **Conversion:** `to_frames(base, len)` requires `base % 4096 == 0` and
  `len % 4096 == 0`; violations are `MapFatal::UnalignedRange` (fatal) —
  firmware describing unaligned RAM is a platform failure, and clipping
  protected ranges could silently shrink protection, so no clipping at this
  boundary.
- **Arithmetic:** every end-address or span composition is
  `checked_add`-based; overflow is `MapFatal::RangeOverflow` (fatal). The
  map builder re-checks even though W02 composed values carefully — belt
  and braces at the protection boundary, where the cost of a missed check
  is the hard gate.

## 5. Untrusted-input stance

W03's inputs are downstream of untrusted firmware data (via W01/W02). The
specific W03 obligation is *containment*: a hostile or buggy platform
description must not shrink protection. The policy implements this
structurally — protection sources win over RAM, ambiguity is fatal — so the
safe outcome never depends on W03 correctly judging which firmware statement
is "right".

## 6. Layering

The map is platform-layer logic: no arch registers, no board names, no QEMU
constants. Nothing in the module set may depend on allocator internals (it
is the other way around: W04 depends on W03 queries). The ADR-018 evolution
expectation is honored by the extension points of
[02 §6](02-architecture-and-state.md), not by speculative features.
