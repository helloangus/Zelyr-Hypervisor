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
| A2 | Existing `aarch64-unknown-none-softfloat` target and core allocation types | P0 build-target baseline; no build-std path | `core::alloc::{GlobalAlloc, Layout}`; eventual alloc integration | W05-GLOBAL must freeze registration, safety and failure handling before adapter coding; target choice is not pending |
| A3 | Existing P0/P1 fatal diagnostic path | Current boot implementation | Non-allocating terminal diagnostics | W05 still owns allocation-error integration; do not assume it exists |
| A4 | Existing host-test member | `crates/host-test-baseline` | Buffer-backed pages and source-shared logic | Real target backing remains W05-MAP |

## 4. The `alloc` interplay, precisely

The target is already fixed; its selection is not a W05 prerequisite defect.
The checked heap remains `core`-only. An `alloc` consumer needs global allocator
registration and a toolchain-supported allocation-failure path in addition to
heap logic. Verify the pinned target integration without adding nightly features
or changing the build target. `GlobalAlloc`/`Layout` themselves are core types.

**W05-GLOBAL — open detailed-design gate:** freeze publication/unpublication,
interior mutability with exclusive access, stable storage lifetime, boot-CPU/IRQ
restriction, reentrancy detection and non-allocating error reporting. A shared
`&Heap` and single-core execution are not sufficient safety arguments. Define
null-return versus fatal caller behavior explicitly; null does not itself invoke
a handler. Global adapter implementation and claims of readiness remain blocked.

**W05-MAP — open detailed-design gate:** W05 owns a backing adapter that retains
the W04 allocation handle and establishes exclusive writable, non-executable
views of ordinary allocated pages. W04's metadata aperture cannot supply these
views. Specify aperture capacity, attributes, alias exclusion, publication/TLB
ordering, mapping failures and rollback. Release must end every byte borrow and
remove/invalidate the mapping before returning the original handle to W04;
unmap failure cannot return the pages to the pool. This is required for a live
heap, not merely an optional hardware test.

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
| `GlobalAlloc` adapter | Allocation failure | Null return at trait boundary; infallible allocation callers use a separately registered non-allocating fatal path; W05-GLOBAL must freeze integration |
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
(O4). Physical-memory specifics live in W05-MAP, separate from W04 metadata access.
