# P2-W03 Code Contracts — Boot Memory Map

**Status:** Detailed design selected for the user-requested W03 implementation;
see the [implementation record](../p2-w03-boot-memory-map-ownership-record.md).
**Parent:** [P2-W03 detailed design](README.md).
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code. Names are stage-local design freedom owned by
this design.

## 1. Types — spans and conversion

### 1.1 Frame-span newtypes

```text
Name and stability: PhysFrameNum(u64), PageCount(u64), PhysFrameRange
  { first: PhysFrameNum, count: PageCount }, P2-local frame/page newtypes
  over the existing address types; internal.
Purpose and caller: frame-granular bookkeeping; used by all modules and by
  W04.
Inputs / outputs: construction from checked operations only; public
  arithmetic is total (returns Option) — there is no wrapping constructor.
Preconditions / postconditions: `PhysFrameRange::end()` is
  first + count (checked at construction, stored); ranges are never empty
  (count > 0); `contains`, `overlaps`, `adjacent`, `intersect` are the
  only spatial predicates.
State and ownership: value types.
Concurrency/allocation context: none; Copy-shaped.
Errors: constructors return Result; no panic paths.
Security checks: the newtype boundary is where naked integers stop — code
  review rejects any u64→frame conversion outside these constructors.
Logic: plain newtypes + predicate functions.
Validation: W03-DV01 predicate/conversion fixtures.
```

### 1.2 Byte-to-frame conversion

`ram_to_frames(base: PhysAddr, len: ByteSize)` requires nonempty aligned RAM;
`protect_to_frames(base: PhysAddr, len: ByteSize)` rounds nonempty protection
outward. Both return `Result<PhysFrameRange, MapFatal>`, allocate nothing and
change no state. Callers drop/count zero lengths first.

```text
end = base.checked_add(len) or RangeOverflow
ram: require base and len multiples of 4096 or UnalignedRange
protection: start = floor(base / 4096) * 4096
            end = checked_align_up(end, 4096) or RangeOverflow
first = start / 4096; count = (end - start) / 4096
return checked nonempty PhysFrameRange(first, count)
```

RAM uses `start = base`. Original byte ranges remain in the ledger for
R6/R7; conversion never authorizes bytes or frames outside discovered RAM.
Validation: W03-DV01, including byte-end overflow even when frame addition fits.

## 2. Types — classification and identity

```text
Name and stability: RegionClass, ProtectedSourceId, MapEntry
  { range: PhysFrameRange, class: RegionClass, sources: SourceSet }, MapFatal (enum:
  MissingRequiredRange{source}, UnusableFact{list, ordinal}, UnalignedRange, RangeOverflow,
  RamOverlap{detail}, ProtectionConflict{
  a, b}, SealRejected{detail}, CapacityExhausted{which}), MapAnomaly
  (merged_banks, deduped_banks, deduped_protections, dropped_zero_ram,
  dropped_zero_protection, outside_ram_warnings, clips: ClipRecord list).
  Internal; RegionClass/ProtectedSourceId are the P2-G01/G02 extension
  surface and are recorded for P4 via W10.
Purpose and caller: classification vocabulary and explicit outcomes.
Preconditions / postconditions: every MapFatal carries source identities,
  never raw memory contents.
Errors: n/a.
Security checks: unknown future classes must match as protected —
  `RegionClass::is_protected()` is the only classification predicate and
  defaults to protected for anything not explicitly Allocatable.
Logic: enums + small records; ClipRecord { protected: ProtectedSourceId,
  bank: index, kept: [PhysFrameRange] }.
Validation: W03-DV10 extension review.
```

`ProtectionSource` retains original bytes, rounded extent, class, flags and
stable ordinal identity. `SourceSet` is a bounded set of ledger indices (at most
38 input sources plus 46 metadata extents); equivalent geometry may share map
entries without deleting ledger identities. `protected_ranges()` exposes
normalized in-RAM entries and source sets; `source_ledger()` preserves complete
input extents. Capacity overflow never truncates either view.

## 3. `ProtectedSet` assembly

`bootmap::classify::assemble(platform, active_dtb_range, image)` returns a
bounded ledger or `MapFatal`, with no partial publication or allocation.
Inputs use the real W02 `Fact<T>` lists per [01 §3](01-scope-and-foundations.md).
Only W02 supplies reservations. Each ledger item retains original bytes,
rounded frames, class, flags and source IDs; ordinal identity includes whether
the W02 reservation came from the header or a node.

```text
require nonempty image and active_dtb_range or MissingRequiredRange{source}
add_checked(image, HypervisorImage)
add_checked(active_dtb_range, ActiveDtb)
for (i, fact) in platform.reserved().iter():
    r = require_usable(fact, Reserved, i)
    add_checked(r.span, source=(r.source, i), flags=(r.no_map, r.reusable))
for (i, fact) in platform.artifacts().iter():
    add_checked(require_usable(fact, Artifact, i), BootArtifact(i))
validate original-byte conflicts, applying R7 before R6
round protection outward; retain page-sharing source sets
```

Capacity is P=38 source slots, as derived in [02 §8](02-architecture-and-state.md).
Zero-based reservations are normal ranges, not null markers; warn only if
outside RAM. Exceeding any bound is `CapacityExhausted`. Validation:
W03-DV02–DV04, including one header reservation ingested exactly once.

## 4. Draft construction

```text
Name and stability: BootMapBuilder::draft(platform, dtb, image) -> Result<
  UnsealedMemoryMap, MapFatal>. Internal; called once per boot.
Purpose and caller: normalize RAM against the protected set; produce the
  query-only draft for W04 planning.
Inputs / outputs: as assemble(); output: sorted disjoint MapEntry array
  covering the RAM union, with clip log and counters.
Preconditions / postconditions: pre — fact records; post — invariants of
  [02 §5](02-architecture-and-state.md) except metadata (not yet known);
  R1–R5, R8, R10 enforced; capacity: endpoint bound in [02 §8](02-architecture-and-state.md).
Concurrency/allocation context: no allocation.
Errors: R1–R3/R6/R12 and capacity errors; nothing published on error.
Security checks: clipping is the only RAM shrink path; the builder cannot
  add RAM.
Logic (pseudocode):
    protected = assemble(...)?                       # §3
    ram = []
    for (i, fact) in platform.banks().iter():
        bank = require_usable(fact, Ram, i)          # R12
        if bank.len == 0: count and continue        # R8
        fr = ram_to_frames(bank.base, bank.len)     # R1/R2
        insert sorted; dedupe exact banks first, reject other overlap
    merge adjacent banks                           # R4
    entries = partition ram_union at rounded protection endpoints
    mark uncovered pieces Allocatable
    mark covered pieces with class and complete SourceSet; count each once
    record each bank/source intersection in the bounded clip log
    sort entries; assert disjoint + coverage(ram_union)
    return UnsealedMemoryMap { entries, protected, anomalies }
Validation: W03-DV05–DV08.
```

## 5. `UnsealedMemoryMap` — draft queries

```text
Name and stability: UnsealedMemoryMap { allocatable_spans() ->
  &[PhysFrameRange], protected_ranges() -> &[MapEntry],
  source_ledger() -> &[ProtectionSource], class_at(frame) -> ClassQuery }, internal;
  readable by W04's planner; NOT an allocation authority (type is distinct
  from BootMemoryMap precisely so the compiler can enforce that).
Purpose and caller: planning input for W04; nothing else.
Errors: ClassQuery ::= Allocatable | Protected(RegionClass) | OutsideRam.
Logic: binary searches over sorted entries; no mutation methods exist.
Validation: W03-DV08 (planner walkthrough).
```

## 6. Sealing

### 6.1 `MetadataPlan` acceptance shape

```text
Name and stability: MetadataPlan { ranges: BoundedList<(PhysFrameRange),
  MAX_METADATA_RANGES>, region_bindings } — the shape W03 validates;
  construction and exact storage layout belong to W04. Capacity is 46,
  derived from normalized spans, not RAM banks. Bindings identify the draft
  source span, resulting managed span(s), hosting metadata extent and
  disjoint storage offsets; W03 seals physical extents, W04 audits bindings.
Purpose: seal input; validated, then recorded.
Validation: W03-DV09.
```

### 6.2 `UnsealedMemoryMap::seal`

```text
Name and stability: UnsealedMemoryMap::seal(plan: &MetadataPlan) ->
  Result<BootMemoryMap, MapFatal>. Internal; called once; the only
  draft→sealed transition.
Purpose and caller: record allocator metadata as protected and freeze.
Preconditions / postconditions: pre — plan ranges frame-aligned,
  non-overlapping, each fully inside a draft Allocatable span, total count
  within capacity; post — sealed map with those ranges reclassified
  HypervisorMetadata, all [02 §5](02-architecture-and-state.md)
  invariants re-audited, accounting totals computed.
Concurrency/allocation context: no allocation; single-core boot.
Errors and failure guarantee: SealRejected{violation} — map remains in
  draft state (but boot will stop; there is no second seal); nothing
  partially mutated.
Security checks: R11 enforcement; the sealed map's protected superset is
  re-derived from the entry array and compared against
  union(rounded protected + plan) intersect RAM (equality required) — this
  is the audit that backs the
  hard gate.
Logic (pseudocode):
    for r in plan.ranges:
        if !r.is_frame_aligned(): return Err(SealRejected{alignment})
        if !draft.allocatable_spans().any(|s| s.contains_range(r)):
            return Err(SealRejected{outside_allocatable})
    new_entries = draft.entries with plan ranges split/reclassified
    audit(new_entries)                      # sorted, disjoint, coverage
    summary = compute_summary(new_entries)
    assert summary.allocatable == draft.allocatable - plan.frames
    return BootMemoryMap { entries, summary, anomalies }
Validation: W03-DV09.
```

## 7. `BootMemoryMap` — sealed queries

```text
Name and stability: BootMemoryMap { allocatable_spans, protected_ranges,
  metadata_ledger, source_ledger,
  class_at, summary() -> MapSummary }, the W03 output; consumed by W04
  (authority), W06 (rendering), W09 (accounting), and recorded for P4 via
  W10.
Purpose and caller: the sole allocation-domain authority.
Preconditions / postconditions: all invariants of
  [02 §5](02-architecture-and-state.md) hold for the lifetime; `&self`
  only.
MapSummary { ram_frames, allocatable_frames, protected_frames per class,
  outside_ram_warning_count } — the accounting equation
  ram = allocatable + protected holds exactly (audit-enforced).
Validation: W03-DV07/DV08/DV10.
```

Per-class totals are disjoint: pages with differing protection classes use
`SharedProtection`, not the sum of overlapping source lengths. Full source
extents remain queryable even outside RAM; only their RAM intersection enters
`protected_frames`. Metadata is already excluded from sealed allocatable.

## 8. Explicitly unauthorized interfaces

No mutation or reclassification after seal; no raw frame constructors outside
the checked range module; no iteration exposing
interior arrays in a way that lets a caller reconstruct and allocate from
protected spans (queries return borrow slices — W04's review must show it
filters by `Allocatable` only); no memory objects, no Guest-memory
concepts, no DTB release, no serialization.

## 9. Rust binding for the W03 delivery

The implementation uses `platform::bootmap` within the existing hypervisor
member; the source-sharing host harness compiles the same production code.
These concrete bindings preserve the contracts above:

- `SourceId` is the concrete `ProtectedSourceId` spelling, also including
  `Ram(ordinal)` for errors. Ordinals are from the original W02 lists.
- Query collections are bounded borrowed iterators, avoiding redundant cached
  arrays. `entries()` exposes the sorted partition and `ram_spans()` exposes
  the normalized RAM union. Clip `bank` indexes that union.
- `seal(self, metadata: &[PhysFrameRange])` accepts the physical-extents view
  of W04's plan and enforces the 46-range capacity. W04 owns plan construction,
  region bindings, sizing and writable access; W03 does not invent that type.
  Empty plans are valid for the map contract, not evidence of an allocator.
- Consuming the draft prevents retry or double seal. On failure there is no
  sealed output or partially published authority; the draft is retired.
- `SourceSet` stores at most 84 ledger indices in a private `u128` bitset.
  `DuplicateSource` additionally rejects internal identity re-ingestion;
  `SealRejected::Audit` covers invariant violations. `MapAnomaly` exposes
  counters; `clips()` exposes the separate bounded clip ledger.
- Sealed `MapSummary` includes hole frames between RAM banks and per-class
  protected totals. Adding a future class requires extending accounting slots
  and tests while retaining the fail-closed `is_protected()` predicate.

This binding does not authorize target boot integration. The
[record's storage/lifetime boundary](../p2-w03-boot-memory-map-ownership-record.md#downstream-handoff-and-execution-limits)
remains a prerequisite for W04/W09 target callers.
