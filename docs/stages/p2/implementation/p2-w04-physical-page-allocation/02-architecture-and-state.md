# P2-W04 Architecture, State, and Lifecycle Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W04 detailed design](README.md).

## 1. Logical modules

| Module (logical) | Responsibility | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|
| `pagealloc::plan` | Compute `MetadataPlan` from draft-map allocatable spans ([01 §6](01-scope-and-foundations.md)) | Draft-map queries | `MetadataPlan` for W03 seal | Sealing (W03) |
| `pagealloc::table` | Frame-state table: construction over metadata storage, get/set, audit | Plan ranges, window storage | Per-frame state transitions | Free-list logic |
| `pagealloc::buddy` | Per-region free lists, split/merge mechanics | Sealed allocatable spans, table | Allocate/free block operations | Policy (orders are fixed) |
| `pagealloc::api` | Public operations: init from sealed map, allocate, allocate_contiguous, free, stats | Sealed map + plan | Typed results, `AllocationStats` | Metadata placement choice (plan module) |
| `pagealloc::stats` | Accounting derivation and conservation checks | Table + lists | `AllocationStats` | Rendering (W06) |

Module names are stage-local design freedom owned by this design; physical
placement follows the P0-W03 workspace (platform layer next to `bootmap`;
ADR §13 working name `hv-platform`; crate naming pending ADR-054).

## 2. Core state

```text
PageAllocator {
  regions: BoundedList<BuddyRegion, MAX_MEMORY_BANKS>,
  table: FrameStateTable,          // over metadata storage (via A2 window on target)
  base_frame: PhysFrameNum,        // table index origin (min managed frame)
  total_managed: PageCount,
}
BuddyRegion {
  span: PhysFrameRange,            // one sealed allocatable span (post-metadata)
  free_lists: [FreeList; MAX_ORDER + 1],   // chains of free block ranges
  free_frames: PageCount,          // fast accounting copy, audited vs table
}
FrameState ::= Unmanaged | Free | Allocated | ReservedMeta
```

Ownership: one `PageAllocator` value owned by the boot sequence. The table
is the per-frame authority (I2/I3 of
[01 §4](01-scope-and-foundations.md)); free lists are the allocation index.
`free_frames` copies exist only for O(1) stats and OOM detail and are
re-derived in audits — they may never disagree with the table.

Free-list representation: singly linked chains of block descriptors stored
*in the metadata area* (block range: first frame + order), not in the free
frames themselves — writing into free frames would violate Decision 7 (the
allocator never writes managed frames) and would corrupt any future
debugging that inspects frame contents. List nodes are bounded: worst case
one node per free block; node capacity is part of the metadata sizing rule
([01 §6](01-scope-and-foundations.md)); node-space exhaustion is a typed
`OutOfFrames` (with `MetadataFull` detail) rather than a silent corruption.

## 3. Metadata bootstrap and lifecycle

```text
Sealed map exists?  NO ──> boot order: W03 draft ──> plan ──> seal ──> v
                          (W03 owns this; W04 supplies the plan)
Sealed map exists?  YES
   |
   v  PageAllocator::init(sealed_map, window)
[derive per-span metadata needs] -> verify seal recorded exactly the plan
[build FrameStateTable: allocatable spans minus metadata = managed]
[seed free lists: all managed frames Free, split into buddies]
   |
   v
[Ready] <== allocate / allocate_contiguous / free_contiguous / stats
   |                    errors are typed; state unchanged on error
   v  (P2 has no shutdown path; P3/P4 evolve lifetime via their designs)
```

Init-time audit (all fatal `AllocatorInitError` on failure): the sealed
map's metadata ranges equal the plan; managed domain equals allocatable
spans minus metadata; table initialization matches; the conservation
equation holds (managed = free + reserved_meta). This is the structural
realization of the hard gate: the allocator never sees a protected frame
because its domain was subtracted before the first free list existed.

## 4. Allocation and free mechanics (overview; contracts in [03](03-code-contracts-pagealloc.md))

- `allocate(order)`: find the smallest order ≥ requested with a non-empty
  list in any region (region order fixed = sealed-map order for
  determinism); split down to `order`, marking split halves `Free` and
  list-inserting them; mark result `Allocated`. Exhaustion →
  `Err(OutOfFrames{order, region_stats})`, state unchanged.
- `allocate_contiguous(count)`: order = ceil_log2(count) (≤ `MAX_ORDER`
  else `OutOfFrames`); allocate the block; the surplus frames above
  `count` **remain part of the allocation** (held, not split back) and are
  returned to the pool only when the whole block is freed; the returned
  `AllocatedFrames` records the true block range/order plus the usable
  prefix. Rationale: freeing a truncated range with its original order
  would break buddy invariants (I2), and per-allocation size tags are out
  of P2 scope — holding the surplus keeps `free` exact and stateless. The
  fragmentation cost of the surplus is explicit, bounded by
  `2^order - count`, and covered by stress tests.
- `free_contiguous(range, order)`: validate (I4); mark `Free`; re-insert
  with buddy coalescing upward while the buddy is free and same order;
  stop at region boundary — coalescing never crosses regions (multi-region
  rule, P2-E04).
- Table transitions are the single points of state change; list edits and
  table edits happen in a fixed order inside each operation so an
  interrupted (host-test-injected) failure leaves the table consistent and
  auditable.

## 5. Debug detection model (P2-E06)

| Defect | Detection | Outcome |
|---|---|---|
| Free of unmanaged/protected frame | Table says `Unmanaged`/`ReservedMeta` or range outside domain | `Err(UnmanagedFree{range})` |
| Duplicate free | Table says `Free` for any frame in range | `Err(DoubleFree{first_offending})` |
| Free of never-allocated or partially-allocated span | Table says `Allocated` but boundary/order checks fail (I4) | `Err(BadFreeRange{reason})` |
| Order mismatch (range allocated as different order) | Per-allocation order recorded in table? No — detection via I4 plus audit sampling | `Err(BadFreeRange{order_mismatch})` when detectable; documented limit below |
| Accounting divergence | `stats()` recomputes from table and compares fast copies | `Err`-free; divergence is a fatal invariant stop (boot) |

Documented limit: P2 stores no per-allocation order tag, so a caller who
frees the right frames with the wrong order (both I4-valid) is not
detectable at free time; it becomes visible at the next audit or via
corruption in the field. The W05 heap and P4 consumers always pass the
order they received, and host stress tests cover the misuse class. Adding
an order tag is a Reserved extension if a later design requires it.

## 6. Concurrency, allocation, and interrupt context

- **Single-core boot phase, one owner** (README Decision 6): methods take
  `&mut self`; no locks, no atomics, no IRQ context use. This is the
  explicit stage boundary required by the task book: allocator locking,
  per-CPU pooling, and SMP safety are P3 scope (p3-w06 owns the design;
  p3-w04 consumes). The design constraint left for P3 is only that state
  is one value with typed operations — wrapping it in a lock or moving it
  per-CPU requires no P2 changes.
- **No dynamic allocation** during allocator operation (it *is* the
  allocator); all structures live in the metadata area.
- **No long work in any call:** allocate/free are O(log n) in region size;
  audits are O(managed) but run only at init and when explicitly invoked
  by tests/diagnostics.

## 7. Failure model

Typed errors: `OutOfFrames{order, region_stats}`, `UnmanagedFree`,
`DoubleFree`, `BadFreeRange`, `OrderTooLarge`, `MetadataFull`. All leave
state unchanged (verified by before/after stats equality in tests). Boot-phase
caller policy: any allocator `Err` during boot is a fatal stop with the
typed diagnostic (P0-W14 resource-exhaustion/invariant classes); host tests
consume the typed values directly. Init failures (`AllocatorInitError`) are
always fatal — the system cannot run without a trustworthy allocator.

## 8. Security model summary

Trust anchor: the sealed W03 map. Containment: domain subtraction before
first use; audits at init; table-authority invariant; no writes to managed
frames; checked arithmetic everywhere; fixed region iteration order for
deterministic behavior under identical inputs (supports W09 repeated-boot
accounting comparisons). Residual risk: the documented order-mismatch
limit ([§5](02-architecture-and-state.md)) and under-declared firmware
reservations (W03's limit, inherited) — both recorded for W10.
