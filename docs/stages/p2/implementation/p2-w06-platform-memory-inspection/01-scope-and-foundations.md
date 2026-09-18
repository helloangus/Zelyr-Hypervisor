# P2-W06 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W06 detailed design](README.md).

## 1. Package outcome

One call — `compose` ([03 §2](03-code-contracts-inspection.md)) — turns the
four live P2 sources into one `InspectionReport`; `render` turns the report
into deterministic text. The outcome in the strong sense demanded by P2-H04:
after composition, the report contains no value that did not come from a
source by reference, and no source fact is unreachable from the report's
sections. If a cross-source check fails, composition returns a named
`ConsistencyBroken{check}` diagnostic instead of a report — a divergent
report is never observable.

## 2. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W06 relies on | Failure boundary if delivered differently |
|---|---|---|---|---|
| A1 | `PlatformInfo` with `FactState` facts, capability summary, discovery counters; host-runnable, fixed-capacity | [W02 contracts](../p2-w02-platform-discovery-normalization/04-code-contracts-facts.md) | Read-only field access; states render without interpretation | If fact payloads change shape, W06's platform-section adapter changes — a W06 design revision, not a local edit; W06 never re-walks the DTB to compensate |
| A2 | Sealed `BootMemoryMap` with `allocatable_spans()`, `protected_ranges()`, `class_at()`, `summary()` (`MapSummary`, equation-audited) | [W03 contracts](../p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md) | Map-dump and C1/C2 inputs by reference | If the map query set changes, W06 sections follow the new queries; W06 must not recompute the map from W02 records to "check" W03 — that is W03's audit's job |
| A3 | `PageAllocator::stats() -> AllocationStats` (`&self`, conservation-checked) | [W04 contracts](../p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md) | Allocator-section values and C3 inputs | Stats are the only permitted surface; calling `audit()` from inspection is allowed only as an explicitly-labeled diagnostics step owned by the boot caller, not by W06 composition |
| A4 | `Heap::stats() -> HeapStats` (`&self`); `audit()` is `&mut self` | [W05 contracts](../p2-w05-dynamic-small-allocation/03-code-contracts-heap.md) | Heap-section values and C4 inputs | Same rule as A3; W06 never takes `&mut` of a source |
| A5 | Boot-context record: P1's boot-phase inventory (exception level, boot CPU hardware facts as recorded at entry) is available post-boot as a value W06 can render | P1-W03 (capability inventory), P1-W06 (diagnostics) — planned, not implemented | The boot-context section's data | If P1 delivers no renderable record, the section renders `not recorded` and W06 evidence marks that section blocked-by-upstream; reading arch registers inside W06 is forbidden (README Decision 5) |
| A6 | Diagnostic channel (P0-W12) and failure classification (P0-W14) usable at inspection points; `fmt::Write`-shaped sink | P0 plans, unimplemented | Renderer output path; fatal-stop policy for broken checks | If absent, rendering has no target on target hardware: blocked upstream defect; host tests still run against in-memory sinks |

The host-side unit-test environment is the P0-W08 baseline. Everything in
W06 — composition, checks, rendering — is pure logic over source values and
runs on host with fixture sources; nothing in W06 is target-only.

## 3. Single-source projection policy

Binding rules for every section and line:

1. **One value, one owner.** A rendered quantity names exactly one source
   (platform, map, page allocator, heap, boot context). Two sections never
   restate the same quantity from different derivations; where a quantity
   legitimately appears twice (e.g., allocatable frames in the map and
   allocator sections), it appears as the *same* stored field of the report,
   rendered twice — not as two computations.
2. **No interpretation upgrades.** W06 renders states as recorded: an
   `Unsupported(GicV2)` fact renders as unsupported-with-reason, never as
   "GIC available". Converting a state would falsify P2-V04's
   distinguishability guarantee downstream.
3. **Counters render verbatim.** W01 anomaly counters and W02 skip counters
   appear as numbers with labels; W06 adds no smoothing or thresholds.
4. **No content echoing.** Property values, bootargs strings, and blob bytes
   never render (W02's log-content hygiene); fact *states*, paths-as-
   recorded classes, counts, ranges, and totals do.
5. **Determinism.** Same source values ⇒ same report bits ⇒ same rendered
   text. No time, RNG, address-of, or environment input; fixed iteration
   order everywhere (map spans in map order, regions in W04's fixed order,
   classes in W05's class order).

## 4. Rendering and storage rules

- Report members are fixed-capacity (bounded by the same stage constants as
  the sources: `MAX_MEMORY_BANKS`, `MAX_RESERVED_RANGES`, W04's
  `MAX_MEMORY_BANKS` regions, W05's `NUM_CLASSES`); composition of a source
  that somehow exceeds a bound is impossible by construction (the sources
  are already capacity-bounded) — a capacity surprise is therefore a
  `CapacityExhausted` diagnostic inherited from the source, not a W06 state.
- Rendering writes through the sink incrementally; no intermediate string
  larger than one line buffer (bounded constant); no allocation
  (README Decision 6).
- Text format: one header line per section (`[section-name]`), one item per
  line, `key: value` fields. The format is stage-local and revisable;
  section names and order are the stable part W09 expectations reference.

## 5. Layering note

W06 is a diagnostic consumer that touches platform and memory-module types.
Physical module placement (crate, module path) is pending P0-W03's
workspace; this design fixes only logical adjacency: inspection depends on
the published types/queries of W02–W05 and on `core::fmt`, and on nothing
else. It adds no dependency in the reverse direction: no W02–W05 module may
reference W06. The `hv-platform-inspect` mode, if designed later, sits
above W06 and owns its own invocation policy.
