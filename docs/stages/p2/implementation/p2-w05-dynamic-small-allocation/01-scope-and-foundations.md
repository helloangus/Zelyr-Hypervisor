# P2-W05 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W05 detailed design](README.md).

## 1. Package outcome

A heap with this property: every byte it hands out belongs to exactly one
owned allocation, every allocation derives from W04 frame allocations
(which derive from W03's sealed map — so the hard gate is inherited
transitively), every failure is a typed value at the checked layer or a
defined fatal policy at the `alloc` layer, and repeated allocation/free
cycles preserve a short, machine-checkable invariant list.

## 2. Requirement-derived obligations

From the plan's scope wording ("runtime-object and variable
platform-information needs, explicit allocation failure, release
capability, and repeated allocation/free stress validation"):

- **O1 (needs):** small variable-size records and strings available
  *after* the boot phase's bounded-array phase, for later P2 consumers and
  as the P3/P4 foundation.
- **O2 (failure):** exhaustion — of classes, slabs, directory, budget, or
  backing pages — is explicit and typed; no UB, no silent shrinkage.
- **O3 (release):** every allocation can be released exactly once; the
  heap detects wrong-pointer, wrong-layout, and double release.
- **O4 (stress basis):** a named invariant list plus stress properties
  that host tests can drive arbitrarily long, with zero performance
  claims.

## 3. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W05 relies on | Failure boundary |
|---|---|---|---|---|
| A1 | W04 frame API: `allocate(order)`, `allocate_contiguous(count)`, `free_contiguous`, typed errors, `AllocationStats` | [W04 design](../p2-w04-physical-page-allocation/README.md) | Exclusive backing; `AllocatedFrames` values as capabilities | A mismatch is a W04 contract revision — design conflict, recorded; W05 never pages around W04 |
| A2 | P0 target supplies the `alloc` crate (core/alloc build path) when the adapter is compiled | P0-W03 (target/build-std decision; pending) | `core::alloc::GlobalAlloc`, `Layout` | If the target lands without `alloc`, the inner checked API still stands; the adapter is deferred with the gap recorded — the dynamic-allocation contract does not silently vanish |
| A3 | P0 diagnostics/failure-class channel; alloc-error handler site | P0-W12/W14 (assumed) | Fatal halt path for the adapter's failure policy | Missing → blocked upstream defect |
| A4 | Host test environment (P0-W08) | P0 plan | All slab/directory logic host-testable with buffer-backed "pages" | n/a — structural requirement of this design |

## 4. The `alloc` interplay, precisely

`no_std` dynamic allocation needs (a) a global allocator implementation,
(b) an allocation-error handler, both target-dependent. Because P0-W03 has
not fixed the target or `build-std`, this design *requires* the layering of
[02 §6](02-architecture-and-state.md): the entire heap logic is
`core`-only and target-independent; the adapter is an isolated module that
compiles only when A2 holds. This is a designed dependency, not an
accident: it keeps P2-V07 (the package's validation) satisfiable on host
regardless of the toolchain decision, and it isolates the eventual
`alloc_error_handler` policy decision (fatal per
[README Decision 5](README.md)) in one auditable place.

## 5. Shape selection rationale (stage-local decisions)

Chosen: fixed size-class slab heap (8 classes, one-page slabs, in-band
headers, bitmap allocation) plus a page-backed large path, with a budget.
Rejected alternatives, for the record:

- **Bump-only arena:** trivial but cannot free; violates O3.
- **TLSF / general-purpose heaps:** better worst-case bounds than needed;
  larger metadata, more complex invariants, harder stress validation —
  no P2 consumer requires general sizes beyond the page path.
- **External allocator crate:** no approved dependency (P0-W18); audit
  cost for boot-critical code; deterministic-stress goals favor in-house
  simplicity. Adoption later goes through governance.
- **Buddy-style small allocator:** duplicates W04's mechanics at byte
  granularity; worse metadata cost per small object.

Class set and slab geometry are constants reviewable in
[02 §2](02-architecture-and-state.md); changing them is a design edit with
re-measured metadata, not a silent tweak.

## 6. The fixed-static-array boundary review (plan work sequence 4)

The task book prohibits fixed static arrays becoming the permanent runtime
model. The reviewed position:

- W01–W03 deliberately use fixed-capacity boot storage **before any
  allocator exists**; those arrays are the documented *temporary pre-heap
  model*, bounded by their designs' capacity diagnostics
  ([W02 §3](../p2-w02-platform-discovery-normalization/01-scope-and-foundations.md),
  [W03 §8](../p2-w03-boot-memory-map-ownership/02-architecture-and-state.md)).
- W05's heap is the **permanent dynamic model** from its init onward:
  slab lists grow by acquiring pages, within the budget.
- Within W05, two capacities remain bounded by design — the large-request
  directory and the slab directory. These are *resource containment*
  (exhaustion is typed, not truncation), and their limits are recorded as
  Reserved extension points, not as a hidden static model.
- Any future consumer that outgrows a W01–W03 boot array migrates to the
  heap through its own design; W05 provides no migration machinery in P2.

This section is the reviewable answer to the plan's step-4 review and is
re-checked at W05 closure (W05-DV08).

## 7. Failure policy (normative)

| Layer | Event | Behavior |
|---|---|---|
| Inner checked API | Class/slab/page/budget/directory exhaustion | `Err(HeapError::OutOfMemory{which, stats})`; state unchanged |
| Inner checked API | Invalid/double/foreign free; null or misaligned pointer | `Err(HeapError::InvalidFree{reason})`; state unchanged |
| `GlobalAlloc` adapter | Underlying `alloc`-contract failure (null return path) | Fatal halt with diagnostic (P0-W14 resource exhaustion; README Decision 5) |
| `GlobalAlloc` adapter | Dealloc invariant breach detected | Fatal halt with diagnostic (invariant violation; README Decision 6) |
| Any layer | Internal consistency check fails (audit) | Fatal halt (invariant violation) |

Rationale: at P2 every allocation failure is a boot-phase condition with no
recovery consumer; guests do not exist, so no failure here is
guest-caused. Later stages may soften specific paths through their own
designs using the checked API.

## 8. Layering and testability

The heap is platform-layer logic next to `pagealloc`: no arch code, no
board names, no locking (P3 boundary), no allocation during its own
operation other than through W04. Host testability is structural: pages
are injected buffers; the whole invariant list is exercisable on host
(O4). Physical-memory specifics live only in the W04 window below it.
