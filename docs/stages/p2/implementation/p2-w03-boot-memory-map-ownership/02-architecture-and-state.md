# P2-W03 Architecture, State, and Lifecycle Design

**Status:** Detailed design selected for the user-requested W03 implementation;
see the [implementation record](../p2-w03-boot-memory-map-ownership-record.md).
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
              | SharedProtection     -- multiple source classes cover one rounded page
```

`protected(class) = (class != Allocatable)`. Everything protected is
excluded from the allocation domain by construction, and the classification
is the only route through which a frame can be allocatable.

## 3. Lifecycle — draft and seal

```text
Inputs: W02 PlatformInfo records, W01 active DTB range only, P1 image range
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

The builder first validates facts and checked byte extents, then applies
RAM duplicate/adjacency handling and original-byte protection conflict checks,
then rounds protection and partitions the RAM union. Rule IDs are stable;
R4 and R7 exceptions are tested before the corresponding overlap rejection.

| # | Situation | Outcome | Class |
|---|---|---|---|
| R1 | Nonzero RAM base or length is not page-aligned | Fatal `UnalignedRange` | P2-D05 |
| R2 | Byte end, rounded end, or frame composition overflows | Fatal `RangeOverflow` | P2-D05 |
| R3 | RAM overlaps after exact duplicates are removed | Fatal `RamOverlap` | P2-D05 |
| R4 | RAM exact duplicate or adjacency | Deduplicate first, then merge adjacency; count separately | P2-D06 |
| R5 | Rounded protection intersects RAM | Exclude its union from allocation; retain all sources | P2-D07 |
| R6 | Original nonempty protection byte ranges overlap without R7 equivalence | Fatal `ProtectionConflict` with source identities | P2-D05 |
| R7 | Exact byte-range duplicate of the same class and protection flags | One protection extent, retain every source ID and count duplicate declarations | P2-D05 |
| R8 | Zero-size usable RAM record | Drop and count; no conversion | P2-D05 |
| R9 | Zero-size usable protection record | Drop and count; no rounding | P2-D05 |
| R10 | Protection partly or entirely outside RAM | Retain full source extent and count warning; classify only its intersection with RAM | P2-D05 |
| R11 | Metadata plan outside draft allocation, overlapping itself, or over capacity | Fatal `SealRejected` | P2-D08 |
| R12 | Present unusable/unsupported/absent/undiscovered input fact | Fatal `UnusableFact`; no silent filtering | P2-D05 |

The active DTB, image and artifacts have distinct ownership classes; duplicate
firmware reservations qualify for R7 only when byte extents and flags agree.
A header reservation and node reservation may share an extent without losing
either provenance. Repeated ingestion of one source identity is a builder
error, not a second declaration. Page-only overlap after rounding retains all
contributors: same-class pages keep that class, different-class pages use
`SharedProtection`. These pages count once in RAM totals. No precedence rule
chooses one owner or shrinks protection.

## 5. Normalized map invariants

The sealed `BootMemoryMap` is a sorted, disjoint array of
`(PhysFrameRange, RegionClass, SourceSet)` covering exactly the union of discovered
RAM frames (clipped per R5); "holes" (physical gaps between banks) are
simply absent entries — they are neither allocatable nor protected but
unknown, and queries report them as `OutsideRam`. Invariants checked at
seal: sortedness, disjointness, full coverage of the RAM union, protected
union equality with the rounded `ProtectedSet` intersected with RAM, and accounting
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
  `AllocatorMetadata(index)`) — the key P4 ownership accounting will attach to.
- Accounting totals (`MapSummary`: ram_frames, protected per class,
  allocatable, holes) are computed at seal and queryable; P4 extends the
  ledger, it does not recompute protection.

## 8. Concurrency, allocation, and failure context

Single-core boot phase; no locks or atomics (P3-W06 owns any future shared
access; the sealed map is immutable, so concurrent readers are safe later).
No allocation: bounded input limits are W02's 8 banks, 32 combined reservations
and 4 artifacts, plus image and DTB. Let B=8 and P=38 source slots. At most
B+P=46 draft allocatable spans exist. Define `MAX_ALLOCATABLE_SPANS=46` and
`MAX_METADATA_RANGES=46`; these are distinct from `MAX_MEMORY_BANKS=8`.
A conservative endpoint-partition bound is `2*(B+P+MAX_METADATA_RANGES)=184`
map entries, each with a bounded source set. Clip records are bounded by
B*(P+MAX_METADATA_RANGES); overflow is fatal, never truncation. Revisit these
bounds together if upstream capacities change. Fixed arrays also require a
boot-stack/storage review before target integration; a host allocation does
not prove fit on the P1 boot stack. No IRQ interaction. Sorting plus bounded
source-set construction is permitted; no allocator hot-path cost is claimed.

## 9. Failure model

Fatal classes: `MissingRequiredRange`, `UnalignedRange`, `RangeOverflow`, `RamOverlap`,
`ProtectionConflict`, `UnusableFact`, `SealRejected`, `CapacityExhausted` — each with
structured detail (the offending sources/identities, at class level, not a
memory dump). All are boot stops (P0-W14 platform-failure class): a map
that cannot honestly classify RAM cannot underlie allocation. There is no
retry and no partial map. Non-fatal anomalies (R4, R7, R8, R9, R10, clips)
are counted and visible to W06/W09.
