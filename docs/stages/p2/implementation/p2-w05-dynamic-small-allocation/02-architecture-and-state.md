# P2-W05 Architecture, State, and Lifecycle Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W05 detailed design](README.md).

## 1. Logical modules

| Module (logical) | Responsibility | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|
| `heap::layout` | Layout classification: class selection, page-path routing, alignment rules | `Layout` | Route decision | Any allocation |
| `heap::slab` | One slab's state: header, bitmap, alloc/free slot operations | Page buffer + class | Slot addresses, free counts | Page acquisition |
| `heap::directory` | Ownership map: every heap page → slab or large record | Init + acquisitions | Lookup results for dealloc/audit | Byte-level allocation |
| `heap::core` | Checked API: alloc/dealloc/stat routes, budget enforcement, invariants | `PageAllocator`, layouts | Typed results, `HeapStats` | `GlobalAlloc` mapping |
| `heap::global` | `GlobalAlloc` adapter + failure policy (requires W05-GLOBAL closure — [01 §4](01-scope-and-foundations.md)) | `heap::core` | `alloc`-contract behavior | Heap logic |

Module names are stage-local design freedom owned by this design; physical
placement follows the P0-W03 workspace (platform layer; ADR §13 working
name `hv-platform`; crate naming pending ADR-054).

## 2. Core structure and constants

```text
Heap {
  page_alloc: PageAllocator ownership boundary (injected &mut),
  classes: [ClassState; NUM_CLASSES],
  directory: SlabDirectory,           // fixed capacity MAX_HEAP_ENTRIES
  budget: HeapBudget { max_pages },   // default 256 pages = 1 MiB
  pages_used: PageCount,              // budget accounting
  stats_mirror: audit counters,
}
ClassState { partial_slabs: SlabList, full_count: usize }
Slab (one 4 KiB page, in-band header at offset 0):
  SlabHeader { magic: u32, class: u8, free_count: u16, bitmap: [u8; 32] }
  // bitmap covers up to 256 slots; first ceil(64/class_size) slots are
  // permanently reserved, including padding to the next class boundary
Directory entry { first_frame: PhysFrameNum, pages: PageCount,
                  kind: Slab(ClassIdx) | Large, used: bool }
  // `pages`/range always record the FULL W04 block (allocate_contiguous
  // holds any surplus above the requested count until free — see W04),
  // so dealloc releases exactly what was allocated.
```

Constants (stage-local, reviewable):

| Constant | Value | Rationale |
|---|---|---|
| `NUM_CLASSES` / class sizes | 8: {16, 32, 64, 128, 256, 512, 1024, 2048} | Covers pointer-sized records to medium buffers; page path beyond |
| `MAX_SMALL` | 2048 | Largest class; larger → page path |
| `MAX_SMALL_ALIGN` | 2048 | Class size ≥ requested align (powers of two); larger → page path |
| `MAX_PAGE_ALIGN` | 4096 | Page-aligned allocations satisfy align ≤ 4096; beyond → `UnsupportedAlignment` |
| `MAX_HEAP_ENTRIES` | 256 | Directory capacity: ≤ 256 concurrent slab/large records; exhaustion typed |
| `HEAP_BUDGET_DEFAULT` | 256 pages (1 MiB) | Containment default; settable via `HeapBudget` at init |
| Slab header size | 64 B | 64-byte reserved envelope; every class loses ceil(64/class_size) slots |

Ownership: one `Heap` value owned by the boot sequence. Class heads and the
fixed 256-entry directory live inside this bounded value; init allocates no
directory pages. The complete value needs a boot storage/stack fit review.
Slab headers live in W04-allocated backing pages. Directory entries retain the
original non-Copy W04 allocation handle and W05 mapping owner until release.
`pages_used` counts full slab/large blocks only; initial value is zero.

For class size C, total slots = 4096/C, reserved header slots = ceil(64/C),
usable slots = total - reserved. Reserved bits never become allocatable and
`free_count` counts only usable slots. Returned offsets are multiples of C;
unused padding after the 64-byte envelope stays protected from heap clients.

## 3. Address/pointer discipline

All slot addresses are computed from the page base (from W04's
`AllocatedFrames`) plus bounded offsets, with checked arithmetic; the
`Layout` route guarantees returned pointers satisfy size and alignment
before they are handed out. The heap stores physical frames and produces
virtual addresses through the W05-MAP backing adapter (on host,
exclusive buffers). Physical frame ownership alone is not a writable pointer. No pointer is ever fabricated from an integer not derived from
an owned page record — the directory is the only ownership map
(README Decision 6).

## 4. Acquisition path (backing safety)

New slab or large request → compute the full rounded buddy count → budget
and directory-capacity preflight → W04 allocation → W05-MAP acquisition →
commit directory and initialized slab/large record. Mapping failure rolls back
the allocation before publication; rollback failures are terminal invariants. Every heap byte therefore has a
W04 allocation behind it, which has W03's sealed map behind it: the
never-protected guarantee is inherited, and W05-DV01 asserts the chain by
construction review plus tests ([05 §3](05-validation-and-handoff.md)).
Release of a fully-free slab (one-page classes) may return the page to W04
— a designed behavior that keeps long-running stress tests from pinning
the budget; return-on-empty is enabled for classes with slab size = 1 page
(all classes) and is itself stress-tested.

## 5. Invariants (the stress-validation basis, P2-F04/O4)

1. **Ownership uniqueness:** every byte of every heap page belongs to
   exactly one directory entry and, for slabs, to at most one live slot.
2. **Bitmap truth:** slot allocated ⟺ bitmap bit set; `free_count`
   equals popcount complement; header magic intact (detects stray writes
   into header space).
3. **Budget truth:** `pages_used` equals the sum of directory entry page
   counts.
4. **Class truth:** a slot's size class equals its slab's class; dealloc
   uses the recorded slab class, not the caller's `Layout` size (the
   `alloc` contract passes the original layout, but the directory check
   does not trust it alone).
5. **Conservation:** after N alloc/dealloc pairs from any interleaving,
   heap free capacity returns to its baseline (leak-freedom).
6. **Determinism:** identical operation sequences over identical init
   state produce identical stats traces (supports W09 repeated-boot
   comparisons).

Host stress tests drive arbitrary sequences (including invalid frees and
exhaustion probes) and check 1–6 after every operation; QEMU boot runs
assert 5–6 at startup diagnostics. None of this measures performance —
explicitly out of scope ([README](README.md); plan step 5).

## 6. Lifecycle and the `GlobalAlloc` adapter

```text
PageAllocator ready (W04)
   |
   v  Heap::init(page_alloc, budget) -> Result<Heap, HeapError>
   |    (fixed directory and class state initialized in the owned Heap value)
   v
[Ready] <== alloc / dealloc via core (checked) or global adapter
   |         exhaustion/invalid -> typed Err (core) / fatal policy (adapter)
   v  (no P2 shutdown path; P3/P4 evolve lifetime via their designs)
```

Adapter admission is governed by W05-GLOBAL in
[01 §4](01-scope-and-foundations.md). Required behavior: allocation returns a
pointer or null at the trait boundary; infallible callers have a separate fatal
allocation-error path; invalid deallocation is terminal. Diagnostics must not
re-enter allocation. No global registration, shared-to-mutable conversion or
unsafe implementation is ready for coding until lifetime, interior mutability,
exclusive access and reentrancy rules have a reviewed concrete design.

The checked API remains a single-owner `&mut self` interface. That alone does
not implement the global adapter's `&self` interface safely.

## 7. Concurrency, allocation, and interrupt context

Single-core boot phase, one owner, `&mut self` operations; no locks, no
atomics, no IRQ-context use (P3 boundary, same as W04: p3-w06 owns future
synchronization; the value-shaped state with typed operations is the
designed hand-off point). The heap performs no allocation except through
W04 (it is the allocator of last resort above it); no call does long work
(slot selection is bounded; directory lookup, backing mapping and W04 costs
are accounted separately rather than claimed O(1)).

## 8. Failure model summary

Typed at core: `OutOfMemory{which, stats}` (class, slab-node, directory,
budget, backing), `InvalidFree{reason}` (unknown, double, misaligned,
cross-kind), `UnsupportedAlignment`, `BudgetExhausted`. Fatal at adapter:
the two policies of [01 §7](01-scope-and-foundations.md). All core errors
leave state unchanged (verified by stats equality in tests). Init failure
is fatal (no heap, no dynamic consumers).
