# P2-W06 Architecture and State Model

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W06 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `inspect::sections` | Define the six section records and their fill rules from source types | None (value types) | `&PlatformInfo`, `&BootMemoryMap`, `AllocationStats`, `HeapStats`, `&BootContextRecord` (A5) | Section values | No source mutation; no consistency decisions |
| `inspect::compose` | Orchestrate section fill, run the consistency registry, publish report or one diagnostic | The `InspectionReport` value it returns | The four source references + boot-context record | `Result<InspectionReport, InspectionError>` | No rendering; no retry; no partial publication |
| `inspect::consistency` | Named checks C1–C4 over composed sections/sources | None (pure functions) | Sections + source references | `Result<(), ConsistencyDiagnostic>` per check | No repair, no warning-only modes — a broken check is fatal (README Decision 4) |
| `inspect::render` | Write the report to a `fmt::Write` sink in stable order | One line buffer (bounded) | `&InspectionReport`, sink | `Result<(), RenderError>` | No value recomputation — render reads stored fields only; this is what makes DV07 reviewable |

Module names are stage-local design freedom owned by this design; physical
placement awaits P0-W03 ([01 §5](01-scope-and-foundations.md)).

## 2. Report model

```text
InspectionReport {
  boot_context: BootContextSection,     # A5 record or NotRecorded marker
  platform:    PlatformSection,         # from PlatformInfo (A1)
  memory_map:  MemoryMapSection,        # from BootMemoryMap (A2)
  pages:       PageAllocatorSection,    # from AllocationStats (A3)
  heap:        HeapSection,             # from HeapStats (A4)
  generated_from: SourceFingerprints,   # cheap identity marks (§4), no content
}
```

Section content (authoritative enumeration; P2-H01–H03):

- `BootContextSection` — exception level, boot-CPU hardware identity as
  recorded by P1, or `NotRecorded`.
- `PlatformSection` — CPU topology (count usable/disabled, boot-CPU match),
  per-fact states for GIC/timer/PSCI/console/bootargs (state + reason
  class, not payload content), RAM bank count and totals, reserved-range
  count by source, boot-artifact count, discovery counters (skipped nodes,
  skipped properties, anomalies), capability summary rows.
- `MemoryMapSection` — per-class protected totals (by `ProtectedSourceId`
  class), allocatable span list (first frame, count), `MapSummary` totals,
  outside-RAM warning count, anomaly counts (clips, merges, dedupes).
- `PageAllocatorSection` — managed/free/used/reserved frames total and per
  region, region count.
- `HeapSection` — budget, pages used, headroom, per-class
  slabs/used/free-slot counts, large-allocation count.
- Consistency outcomes are not stored as data: composition fails closed on
  the first broken check, so a published report implies C1–C4 held.

## 3. Composition lifecycle

```text
no report
  -> compose(platform, map, pages, heap, bootctx)
       -> fill sections in fixed order (boot_context, platform, memory_map,
          pages, heap)
       -> run checks C1..C4 in fixed order
            pass  -> Ok(InspectionReport)          # immutable value
            fail  -> Err(ConsistencyBroken{check}) # nothing published
  -> render(report, sink)                          # any number of times
  -> report dropped                                # no Drop side effects
```

There is no update or refresh operation: state changes (allocations,
frees) invalidate the report by definition, and the owner simply composes a
new one. This keeps the report an honest snapshot instead of a stale cache
that pretends to be live. Composition points in P2: the boot sequence's
"allocator ready" diagnostic point and host tests. W09 compares renders
across boots; W08 asserts check failures under injected divergence.

## 4. Consistency check registry

| ID | Assertion | Inputs | Failure meaning |
|---|---|---|---|
| C1 | `platform.ram_total_frames == memory_map.summary.ram_frames` | A1, A2 | The map was not built from the active platform facts — a construction-order or divergence defect |
| C2 | `summary.ram_frames == summary.allocatable_frames + summary.protected_total` (recomputed from section fields) | A2 | Map accounting equation broken (W03's audit should have caught this; C2 is the independent cross-render verification demanded by P2-H04) |
| C3 | `pages.managed == memory_map allocatable frames − pages.reserved_meta` and `managed == free + used + reserved_meta` | A2, A3 | Allocator domain does not match the sealed map, or allocator conservation broke |
| C4 | `heap.pages_used <= pages.used` and heap page count consistent with large + slab page counts from `HeapStats` | A3, A4 | Heap holds pages the page allocator does not account as used — ownership-chain break |

Checks are O(small): sums over bounded lists, no full-table recounts (W04's
`audit()` remains the deep checker, invoked by its own callers). All
arithmetic checked; a checked-arithmetic failure inside a check is itself
`ConsistencyBroken{check}` with an overflow detail.

## 5. Ownership, concurrency, and failure model

- **One owner, no sharing.** The report is a plain value returned to the
  caller; sources are borrowed only for the duration of `compose`. No
  global registry, no singleton (Coding Guidelines; plan-agent guardrail
  against convenience globals).
- **Concurrency context.** P2 is single-core boot phase; `compose` takes
  shared references and the allocators' `stats()` are `&self`, so
  composition is race-free by the existing ownership discipline. W06 adds
  no lock, atomic, or per-CPU assumption; the P3 concurrency boundary
  recorded by W04/W05 applies unchanged.
- **Failure model.** Exactly two failure shapes: `ConsistencyBroken{check,
  detail}` from composition (fatal at boot per P0-W14 invariant-violation
  class; host tests assert the value) and `RenderError` from the sink
  (write failure; composition state unaffected, report stays valid for
  another sink). No third shape exists; partial reports and partial renders
  are impossible (render of a section is all-or-line-buffer, and a failed
  render is simply truncated at the sink).
- **Security posture.** Inspection renders no untrusted content (DT strings,
  property values, blob bytes are excluded by [01 §3](01-scope-and-foundations.md)
  rule 4), so the report cannot become a log-injection channel. Ranges and
  counts render as typed values formatted by checked paths.
