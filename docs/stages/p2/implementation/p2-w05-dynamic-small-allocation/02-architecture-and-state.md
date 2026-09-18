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
| `heap::global` | `GlobalAlloc` adapter + failure policy (compiles only when `alloc` exists — [01 §4](01-scope-and-foundations.md)) | `heap::core` | `alloc`-contract behavior | Heap logic |

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
  // bitmap covers up to 256 slots; slot 0..header slots are permanently
  // marked allocated (header occupies the first 64 bytes)
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
| Slab header size | 64 B | magic + class + count + 32 B bitmap; costs slots only in the 16 B class |

Ownership: one `Heap` value owned by the boot sequence; the slab headers
and directory live in pages W04 allocated (in-band) — the heap has no
static metadata arrays beyond the fixed `Heap` struct itself, whose
`ClassState`/`directory` storage is part of the (bounded) struct. This is
the reviewed containment boundary ([01 §6](01-scope-and-foundations.md)).

## 3. Address/pointer discipline

All slot addresses are computed from the page base (from W04's
`AllocatedFrames`) plus bounded offsets, with checked arithmetic; the
`Layout` route guarantees returned pointers satisfy size and alignment
before they are handed out. The heap stores physical frames and produces
addresses through the same A2-style window convention as W04 (on host,
buffers). No pointer is ever fabricated from an integer not derived from
an owned page record — the directory is the only ownership map
(README Decision 6).

## 4. Acquisition path (backing safety)

New slab or large request → budget check (`pages_used + need ≤ max_pages`)
→ `page_alloc.allocate_contiguous(pages)` → directory entry insert →
slab header initialize or large record. Every heap byte therefore has a
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
   |    (directory storage acquired first via the page path)
   v
[Ready] <== alloc / dealloc via core (checked) or global adapter
   |         exhaustion/invalid -> typed Err (core) / fatal policy (adapter)
   v  (no P2 shutdown path; P3/P4 evolve lifetime via their designs)
```

Adapter rules ([01 §4](01-scope-and-foundations.md)):
- `alloc(layout)`: route via `heap::layout`; `Ok(ptr)` or null (per the
  `GlobalAlloc` contract) — with the fatal error handler firing on the
  null path for boot-phase policy (README Decision 5).
- `dealloc(ptr, layout)`: directory-validated release; invariant breach →
  fatal halt (README Decision 6).
- No `unsafe` beyond the adapter trait impls and the page-buffer view
  fabrication, each with a `SAFETY` argument; all logic sits in `heap::core`
  as safe code over raw page records.

## 7. Concurrency, allocation, and interrupt context

Single-core boot phase, one owner, `&mut self` operations; no locks, no
atomics, no IRQ-context use (P3 boundary, same as W04: p3-w06 owns future
synchronization; the value-shaped state with typed operations is the
designed hand-off point). The heap performs no allocation except through
W04 (it is the allocator of last resort above it); no call does long work
(O(1) small paths, O(directory) only in audits).

## 8. Failure model summary

Typed at core: `OutOfMemory{which, stats}` (class, slab-node, directory,
budget, backing), `InvalidFree{reason}` (unknown, double, misaligned,
cross-kind), `UnsupportedAlignment`, `BudgetExhausted`. Fatal at adapter:
the two policies of [01 §7](01-scope-and-foundations.md). All core errors
leave state unchanged (verified by stats equality in tests). Init failure
is fatal (no heap, no dynamic consumers).
