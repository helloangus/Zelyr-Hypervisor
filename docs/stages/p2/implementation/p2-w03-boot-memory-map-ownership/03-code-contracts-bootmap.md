# P2-W03 Code Contracts — Boot Memory Map

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W03 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code. Names are stage-local design freedom owned by
this design.

## 1. Types — spans and conversion

### 1.1 Frame-span newtypes

```text
Name and stability: PhysFrameNum(u64), PageCount(u64), PhysFrameRange
  { first: PhysFrameNum, count: PageCount }, P2-local newtypes unless the
  P0 base crate delivers equivalents (assumed contract A4,
  [01 §3](01-scope-and-foundations.md)); internal.
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

### 1.2 `to_frames`

```text
Name and stability: bootmap::ranges::to_frames(base: PhysAddr, len:
  ByteLen) -> Result<PhysFrameRange, MapFatal>. Internal.
Purpose and caller: the single byte→frame conversion at the input
  boundary; used by the builder for every record.
Preconditions / postconditions: base % 4096 == 0 and len % 4096 == 0 and
  len > 0 (zero handled by callers as R8/R9 before conversion); overflow
  checked; else `UnalignedRange`/`RangeOverflow`.
Logic (pseudocode):
    if base % PAGE_SIZE != 0 or len % PAGE_SIZE != 0: Err(UnalignedRange)
    first = base / PAGE_SIZE; count = len / PAGE_SIZE
    if first.checked_add(count).is_none(): Err(RangeOverflow)
    Ok(PhysFrameRange{ first, count })
Validation: W03-DV01.
```

## 2. Types — classification and identity

```text
Name and stability: RegionClass, ProtectedSourceId, MapEntry
  { range: PhysFrameRange, class: RegionClass }, MapFatal (enum:
  UnalignedRange, RangeOverflow, RamOverlap{detail}, ProtectionConflict{
  a, b}, SealRejected{detail}, CapacityExhausted{which}), MapAnomaly
  (merged_banks, deduped_protections, dropped_zero_ram,
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

## 3. `ProtectedSet` assembly

```text
Name and stability: bootmap::classify::assemble(inputs) -> Result<
  ProtectedSet, MapFatal>. Internal; called by the builder.
Purpose and caller: convert all protection statements into identified
  frame ranges, enforcing R6/R7/R9/R10 at set level.
Inputs / outputs: inputs = W02 PlatformInfo records (reserved ranges with
  sources/flags, boot artifacts), W01 DTB range + rsvmap entries, P1 image
  range (Option — None is a blocked prerequisite, see
  [W01 A3](../p2-w01-boot-platform-description-intake/01-intake-boundary.md));
  output: sorted, deduplicated set of (PhysFrameRange, ProtectedSourceId).
Preconditions / postconditions: pre — records decoded by W01/W02; post —
  sorted, R7-deduplicated, each entry identified; fixed capacity: entries
  <= 2*MAX_RESERVED_RANGES + MAX_MEMORY_BANKS + MAX_BOOT_ARTIFACTS + 2
  (image + DTB); overflow → CapacityExhausted (fatal).
Concurrency/allocation context: no allocation; single-core boot.
Errors: ProtectionConflict (R6), CapacityExhausted, conversion fatals.
Security checks: zero-address rsvmap entries flagged by W01 that decode to
  zero-base ranges are recorded as out-of-RAM warnings (R10), never
  silently dropped and never treated as the null range.
Logic (pseudocode):
    add(image_range, HypervisorImage)?            # A3; None -> blocked-defect stop
    add(dtb_range, ActiveDtb)?
    for (i, r) in rsvmap.entries(): add(to_frames(r)?, DtbReservation(i))?
    for (i, r) in platform.reserved(): add(to_frames(r)?, ReservedMemoryNode(i))?
    for (i, a) in platform.artifacts(): add(to_frames(a)?, BootArtifact(i))?
    sort + scan: overlap between distinct identities -> ProtectionConflict
                 exact duplicates same class      -> dedupe + counter
Validation: W03-DV02–DV04.
```

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
  R1–R5, R8, R10 enforced; capacity: entries <= banks + protected + clips
  (bounded constant above).
Concurrency/allocation context: no allocation.
Errors: fatal classes per R1–R3; nothing published on error.
Security checks: clipping is the only RAM shrink path; the builder cannot
  add RAM.
Logic (pseudocode):
    protected = assemble(...)?                       # §3
    ram = [] 
    for bank in platform.memory_banks:
        fr = to_frames(bank.base, bank.len)?         # R1/R2 fatal here
        insert fr into ram (sorted); overlap inside ram -> R3 fatal
        adjacency/identity -> merge (R4)
    entries = []
    for bank_span in ram:
        pieces = bank_span minus protected           # interval subtraction
        if pieces lost anything: record ClipRecord
        entries += (pieces, Allocatable)
    for p in protected ∩ ram_union: entries += (p, class_of(p.source))
    sort entries; assert disjoint + coverage(ram_union)
    return UnsealedMemoryMap { entries, protected, anomalies }
Validation: W03-DV05–DV08.
```

## 5. `UnsealedMemoryMap` — draft queries

```text
Name and stability: UnsealedMemoryMap { allocatable_spans() ->
  &[PhysFrameRange], protected_ranges() -> &[(PhysFrameRange,
  ProtectedSourceId)], class_at(frame) -> ClassQuery }, internal;
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
  MAX_METADATA_RANGES> } — the *shape* W03 validates; its construction is
  W04's design. MAX_METADATA_RANGES = MAX_MEMORY_BANKS + 4 (metadata may
  sit at each region tail plus bookkeeping spans).
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
  protected + plan (equality required) — this is the audit that backs the
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

## 8. Explicitly unauthorized interfaces

No mutation or reclassification after seal; no `From<u64>`-style raw
constructors for frame types outside `to_frames`; no iteration exposing
interior arrays in a way that lets a caller reconstruct and allocate from
protected spans (queries return borrow slices — W04's review must show it
filters by `Allocatable` only); no memory objects, no Guest-memory
concepts, no DTB release, no serialization.
