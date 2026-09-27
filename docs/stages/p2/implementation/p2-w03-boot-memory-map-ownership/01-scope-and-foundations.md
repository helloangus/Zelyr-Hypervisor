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

## 3. Current prerequisite contracts and failure boundaries

| Input | Current provider | W03 obligation |
|---|---|---|
| RAM, reservations, boot artifacts | W02 `PlatformInfo`: `banks()`, `reserved()`, `artifacts()` contain `Fact<T>` | Validate every present entry; never silently filter to usable values |
| Active DTB extent | W01 `ValidatedBootDtb::range()` | Protect the exact byte extent after outward rounding; do not ingest its reservation iterator again |
| Image extent | P1 linker-owned image and stack bounds | Require one checked nonempty extent; missing/empty input is `MissingRequiredRange` |
| Address types and diagnostics | Existing `boot::address::{PhysAddr, ByteSize}` and boot fatal path | Reuse these types; introduce only the missing frame/page newtypes |

W02 already combines header reservations and `/reserved-memory` records in
`reserved()`. Retain `ReservationSource` and flags, plus the list ordinal as a
stable source identity. W03 has no second DTB walk. An empty reservation or
artifact list is allowed; a present `Unusable`, `Unsupported`, `NotDiscovered`
or `Absent` list entry is `MapFatal::UnusableFact`, with list and ordinal.
The same rule applies to RAM; at least one nonzero usable RAM bank is required.
`Usable` zero-length entries are dropped and counted. W02's existing zero-range
counter remains a separate upstream counter; do not claim W03 observed entries
that W02 already omitted. Host tests use the real W01/W02 pipeline or a bounded
internal fixture adapter, not a public constructor bypassing `PlatformInfo`.

## 4. Page size, representation, and arithmetic policy

- Granule is 4 KiB. Reuse `PhysAddr`/`ByteSize` at the boundary and define
  checked `PhysFrameNum`, `PageCount`, `PhysFrameRange` internally.
- RAM conversion requires page-aligned base and length. Nonempty protection
  conversion checks `end = base + len`, rounds base down and end up, and checks
  the rounding for overflow. Never round a protected span inward.
- Keep original byte spans in the source ledger. Resolve ownership conflicts
  there before rounding; page overlap caused only by outward rounding is a
  protected union, not a new ownership conflict.
- Zero lengths are handled before conversion. Overflow is always fatal.
  Physical holes remain outside the map and are never an allocation domain.

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
