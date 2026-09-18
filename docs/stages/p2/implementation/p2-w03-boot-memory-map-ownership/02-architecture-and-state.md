# P2-W03 Architecture, State, and Lifecycle Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W03 detailed design](README.md).

## 1. Logical modules

| Module (logical) | Responsibility | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|
| `bootmap::ranges` | Frame-span newtypes, conversion, overlap/containment predicates | W02/W01/P1 records | `PhysFrameRange` values, predicate results | Classification policy |
| `bootmap::classify` | Source records → protected-range set with `ProtectedSourceId` identities | Fact records | `ProtectedSet` | RAM handling |
| `bootmap::build` | Draft normalization: clipping, merging, sorting, conflict enforcement | `ProtectedSet` + RAM banks | `UnsealedMemoryMap` or `MapFatal` | Metadata planning (W04) |
| `bootmap::seal` | Metadata reservation recording, immutability freeze, final invariants | Draft + `MetadataPlan` (W04) | `BootMemoryMap` | Choosing the plan |
| `bootmap::query` | Class queries, allocatable spans, accounting totals | Sealed map | Query results for W04/W06/W09 | Any mutation |

Module names are stage-local design freedom owned by this design; physical
placement follows the P0-W03 workspace (platform/discovery layer; ADR §13
working name `hv-platform`; crate naming pending ADR-054).

## 2. Range classification

`RegionClass` (non-exhaustive for extension, P2-G01):

```text
RegionClass ::= Allocatable          -- frame-granular allocatable domain
              | HypervisorImage      -- P1-declared EL2 image (code+data+stack)
              | HypervisorMetadata   -- allocator/ownership metadata (seal phase)
              | ActiveDtb            -- the validated boot DTB
              | FirmwareReserved     -- DTB rsvmap entries + /reserved-memory
              | BootArtifact         -- e.g. initrd from /chosen
```

`protected(class) = (class != Allocatable)`. Everything protected is
excluded from the allocation domain by construction, and the classification
is the only route through which a frame can be allocatable.

## 3. Lifecycle — draft and seal

```text
Inputs: W02 PlatformInfo records, W01 DTB range + rsvmap, P1 image range
   |
   v  BootMapBuilder::draft(...)
[protected set assembly] --capacity/contradiction--> MapFatal
   |
   v
[RAM integration: clip, merge, sort, conflicts] --> MapFatal or draft
   |
   v
UnsealedMemoryMap  (immutable records; queries allowed; NOT an allocation
                    authority; W04 may read allocatable spans for planning)
   |
   v  W04 computes MetadataPlan (its design owns how)
   |
   v  UnsealedMemoryMap::seal(plan)
[metadata range re-classification + final invariant audit]
   |                                     \
   v                                       +--> MapFatal::SealRejected
BootMemoryMap (sealed, immutable; the sole allocation authority)
```

Rules:

- The draft exists to be *queried by W04's planner*; it is never an
  allocation source. This makes the P2-D08 property ("metadata protected
  before allocation begins") structural: there is no state in which pages
  are allocatable while metadata is unrecorded, because the allocation
  authority does not exist until seal.
- Sealing validates the plan against the draft (plan ranges must be inside
  draft-allocatable spans, frame-aligned, non-overlapping) and then
  reclassifies. A rejected seal is fatal to boot (W04 cannot operate
  without a map) — one diagnostic, no partial map.
- After seal there is no mutation, no unprotect, no re-seal
  (README Decision 7). The type design makes this enforced: sealed
  operations take `&self` only.

## 4. Conflict and anomaly policy (normative table)

Given the input records, the builder applies these rules in order; each
outcome is explicit and testable:

| # | Situation | Outcome | Class |
|---|---|---|---|
| R1 | RAM bank base or len not 4 KiB-aligned | Fatal `UnalignedRange` | P2-D05 |
| R2 | Any span composition overflows (`checked_add` fails) | Fatal `RangeOverflow` | P2-D05 |
| R3 | Two RAM banks overlap (not merely adjacent) | Fatal `RamOverlap` | P2-D05 |
| R4 | RAM banks adjacent or identical | Merged into one span, counted | P2-D06 |
| R5 | Protected range intersects RAM | RAM clipped around it; recorded in `ClipLog` | P2-D07 |
| R6 | Two protected ranges overlap, different sources or different identity | Fatal `ProtectionConflict` | P2-D05 |
| R7 | Two protected ranges identical, same source class (duplicate declaration) | Deduplicated, counted | P2-D05 |
| R8 | Zero-size RAM bank | Dropped, counted | P2-D05 |
| R9 | Zero-size protected entry | Dropped, counted (protects nothing) | P2-D05 |
| R10 | Protected range entirely outside any RAM | Recorded with warning (unprotectable but unallocatable anyway) | P2-D05 |
| R11 | Protected `seal` plan range not inside draft-allocatable domain | Fatal `SealRejected` | P2-D08 |

Rationale for R6 vs R7: duplicates are consistent statements; distinct
overlapping claims mean the platform contradicts itself about ownership,
and fabricating a precedence would invent facts (README Decision 3).
Rationale for R5: the hard gate outranks RAM completeness — a clipped bank
yields less allocatable memory, never more protection loss.

## 5. Normalized map invariants

The sealed `BootMemoryMap` is a sorted, disjoint array of
`(PhysFrameRange, RegionClass)` covering exactly the union of discovered
RAM frames (clipped per R5); "holes" (physical gaps between banks) are
simply absent entries — they are neither allocatable nor protected but
unknown, and queries report them as `OutsideRam`. Invariants checked at
seal: sortedness, disjointness, full coverage of the RAM union, protected
superset equality with the assembled `ProtectedSet`, and accounting
equation (below).

## 6. Ownership and state authority

| Object | Owner | Mutable state | Lifetime |
|---|---|---|---|
| Input records (W01/W02/P1) | Their producers | none | boot phase |
| `ProtectedSet`, draft | `BootMapBuilder` during construction | construction only | draft phase |
| `MetadataPlan` | W04 (W03 only validates it) | W04's | draft phase |
| `BootMemoryMap` | Boot sequence after seal | none (immutable) | boot phase |
| `ClipLog` / counters | Owned by the map | none after seal | boot phase |

One owner per state; no globals; the sealed map is handed to W04/W06 as a
borrow. There is deliberately no registry of "current map": the boot
sequence passes it, keeping single-owner discipline (Coding Guidelines).

## 7. Extension foundation (P2-G01–G03)

- `RegionClass` is `#[non_exhaustive]`-shaped by convention (documented
  extension protocol): later designs add classes; existing matches must
  treat unknown classes as protected (fail-closed) — this default is the
  load-bearing safety property of the extension design.
- `ProtectedSourceId` gives every protected range a stable identity
  (`HypervisorImage`, `ActiveDtb`, `DtbReservation(index)`,
  `ReservedMemoryNode(index)`, `BootArtifact(index)`,
  `AllocatorMetadata`) — the key P4 ownership accounting will attach to.
- Accounting totals (`MapSummary`: ram_frames, protected per class,
  allocatable, holes) are computed at seal and queryable; P4 extends the
  ledger, it does not recompute protection.

## 8. Concurrency, allocation, and failure context

Single-core boot phase; no locks or atomics (P3-W06 owns any future shared
access; the sealed map is immutable, so concurrent readers are safe later).
No allocation: the map uses fixed-capacity arrays whose limits are the same
constants as W02's capacities plus the clip-log bound (worst case 2×
`MAX_RESERVED_RANGES` + `MAX_MEMORY_BANKS` + small constants; exact bound
fixed in [03 §3](03-code-contracts-bootmap.md)); exceeding a map capacity is
a fatal `CapacityExhausted{which}` — silently truncating protection records
is forbidden. No IRQ interaction; total build cost O((banks + protected) log
(banks + protected)) for the sort.

## 9. Failure model

Fatal classes: `UnalignedRange`, `RangeOverflow`, `RamOverlap`,
`ProtectionConflict`, `SealRejected`, `CapacityExhausted` — each with
structured detail (the offending sources/identities, at class level, not a
memory dump). All are boot stops (P0-W14 platform-failure class): a map
that cannot honestly classify RAM cannot underlie allocation. There is no
retry and no partial map. Non-fatal anomalies (R4, R7, R8, R9, R10, clips)
are counted and visible to W06/W09.
