# P2-W04 Code Contracts — Physical Page Allocator

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W04 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3. Pseudocode is an
outline, not production code. Names are stage-local design freedom owned by
this design. References: sealed map / draft queries = W03
([bootmap contracts](../p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md));
`MetadataPlan` shape is defined there and constructed here.

## 1. `FrameStateTable`

```text
Name and stability: pagealloc::table::FrameStateTable, internal.
Purpose and caller: per-frame authority (states Unmanaged/Free/Allocated/
  ReservedMeta; 2 bits per frame); used by buddy mechanics, debug checks,
  and accounting.
Inputs / outputs: constructed over a `&mut [u8]` storage slice (the
  metadata area via the A2 window on target; a plain buffer on host) plus
  base_frame and the managed-span list; operations: get(frame) ->
  FrameState, set(frame, state) -> Result<_, TableError>, count(state) ->
  PageCount (O(n), audit only).
Preconditions / postconditions: storage length == ceil(managed_frames /
  4); all transitions are checked (set of a frame outside the managed
  domain is TableError::NotManaged — unreachable by construction, kept as
  a debug assertion).
State and ownership: owned by PageAllocator; the only writer of frame
  state.
Concurrency/allocation context: &mut self discipline; no allocation; no
  locks (P3 boundary).
Errors: TableError only signals misuse (programming error); logic
  operations are infallible.
Security checks: the table is the I2/I3 authority
  ([01 §4](01-scope-and-foundations.md)); no aliasing mutable views.
Logic: byte-indexed bitfields; get/set with checked index arithmetic.
Validation: W04-DV01 (table mechanics), DV07 (detection).
```

## 2. `PageAllocator::plan_metadata`

```text
Name and stability: pagealloc::plan::plan_metadata(draft:
  &UnsealedMemoryMap) -> Result<MetadataPlan, PlanError>. Internal;
  called once, before W03 seal.
Purpose and caller: compute W03's MetadataPlan per the sizing and
  placement rules of [01 §6](01-scope-and-foundations.md).
Inputs / outputs: draft-map allocatable spans; output: plan with one
  contiguous tail range per hosting span (may be fewer ranges than spans)
  plus the computed per-span metadata sizes for init cross-check.
Preconditions / postconditions: pre — draft from W03; post — every plan
  range is frame-aligned and inside an allocatable span (W03 re-validates;
  R11 is the backstop).
Concurrency/allocation context: no allocation; pure planning.
Errors: PlanError::NoHostSpan (a span too small to host its own metadata
  with zero frames left for allocation is recorded, not fatal, if another
  span can host it; a platform where no valid plan exists is fatal at
  init, not here — the plan simply fails and boot stops via W03/seal
  diagnostics).
Security checks: planner never proposes ranges outside allocatable spans;
  the arithmetic is checked.
Logic (pseudocode):
    plan = []
    for span in draft.allocatable_spans():        # fixed order
        need = ceil((span.count + LIST_NODE_RESERVE(span)) / 4)  # bytes -> see §6 rules
        need_frames = align_up(need, PAGE_SIZE) / PAGE_SIZE + bookkeeping
        host = smallest span with span.count - need_frames > 0
        plan.add(tail_range(host, need_frames))
    return plan
Validation: W04-DV01 (with W03-DV09 joint seal test).
```

## 3. `PageAllocator::init`

```text
Name and stability: PageAllocator::init(sealed: &BootMemoryMap, window:
  &dyn FrameStorage) -> Result<PageAllocator, AllocatorInitError>.
  Internal; called once per boot after seal.
Purpose and caller: build the allocator from the sealed map; the only
  construction path.
Inputs / outputs: sealed map (allocation-domain authority); window (A2:
  maps the metadata plan ranges to writable storage). Output: ready
  allocator.
Preconditions / postconditions: pre — seal recorded exactly
  plan_metadata's output (the plan is passed alongside for
  cross-checking); post — I1–I5 of [01 §4](01-scope-and-foundations.md)
  hold; conservation equation holds; free lists seeded.
Concurrency/allocation context: single-core boot; no allocation (metadata
  storage is provided by the window).
Errors and failure guarantee: AllocatorInitError{SealMismatch,
  DomainMismatch, WindowFailure, ConservationBroken} — fatal; no
  allocator is published.
Security/authorization checks: the init audit is the structural hard
  gate ([02 §3](02-architecture-and-state.md)): managed domain is
  recomputed from the sealed map's allocatable spans minus the sealed
  metadata ranges, not taken from the plan's word.
Logic (pseudocode):
    for span in sealed.allocatable_spans():        # fixed, deterministic order
        meta = sealed.protected_ranges().find(AllocatorMetadata, inside span)
        managed = span minus meta                  # checked interval subtraction
        if managed.is_empty(): region with zero frames (recorded)
        storage = window.storage_for(meta)?        # A2 assumed contract
        table.init(managed, storage, base = min managed frame)
        mark meta frames ReservedMeta in table
        seed lists: for each order desc, carve managed span into max blocks,
                    insert as Free, table.set(Free)
    audit() or Err(ConservationBroken)
    return Ok(allocator)
Validation: W04-DV01; joint W03-DV09.
```

## 4. Buddy mechanics (internal)

```text
Name and stability: pagealloc::buddy::{find_block, split_block,
insert_block, remove_block, coalesce}, internal helpers over one
BuddyRegion + the table.
Purpose and caller: the O(log n) free-block index; called by api
operations only.
Preconditions / postconditions: list edits and table transitions follow
the fixed order of [02 §4](02-architecture-and-state.md); coalescing
never crosses region boundaries; every insert/remove keeps I2/I3.
Errors: find_block -> Option; others infallible given I4-validated input.
Security checks: node-space exhaustion returns MetadataFull upward (no
silent truncation).
Logic (pseudocode):
    find_block(region, order): for o in order..=MAX_ORDER:
        if free_lists[o].nonempty(): remove block; return (block, o)
    split_block(block, order, target): while order > target:
        order -= 1; half = lower_half(block, order)
        insert_block(half, order); table.set(half, Free)
    coalesce(range, order): while order < MAX_ORDER:
        buddy = buddy_of(range, order)              # first XOR 2^order
        if !in_region(buddy) or table.get(buddy) != Free
           or !is_exactly_buddy(buddy, order): break
        remove_block(buddy, order); range = merge(range, buddy); order += 1
    insert_block(range, order)
Validation: W04-DV02–DV05 (via API), DV07 (defect paths).
```

## 5. `PageAllocator::allocate` and `allocate_contiguous`

```text
Name and stability: allocate(order: Order) -> Result<AllocatedFrames,
PageAllocError>; allocate_contiguous(count: PageCount) ->
Result<AllocatedFrames, PageAllocError>. The W04 output contract; W05's
exclusive backing surface.
Purpose and caller: exclusive frame-range allocation; AllocatedFrames
{ range: PhysFrameRange, order: Order, usable: PageCount } carries back
exactly what free requires: `range`/`order` describe the full buddy block
(for `allocate`, usable == range.count); `usable` is the caller's usable
prefix (== count for allocate_contiguous; the surplus above `usable`
stays allocated until free — [02 §4](02-architecture-and-state.md)).
Inputs / outputs: order <= MAX_ORDER; count >= 1; output block
frame-aligned, order-aligned, fully inside one region's managed span.
Preconditions / postconditions: pre — allocator initialized; post —
table[range] == Allocated for the whole block; conservation holds; on
Err state unchanged ([02 §7](02-architecture-and-state.md)).
Concurrency/allocation context: &mut self; single-core boot; no locks; no
allocation; O(log n).
Errors and failure guarantee: OrderTooLarge, OutOfFrames{order,
region_stats}; no partial mutation.
Security/authorization checks: returned ranges derive only from free
lists seeded from the managed domain — protection cannot be expressed in
any input; caller cannot request a specific address (no such parameter
exists).
Logic (pseudocode):
    allocate(order):
        if order > MAX_ORDER: Err(OrderTooLarge)
        for region in regions (fixed order):
            if let Some((block, o)) = find_block(region, order):
                split_block(block, o, order)
                table.set(block, Allocated)
                return Ok(AllocatedFrames{ block, order, usable: block.count })
        Err(OutOfFrames{order, region_stats: snapshot()})
    allocate_contiguous(count):
        order = ceil_log2(count)                     # checked
        block = allocate(order)?                     # state changes only here
        # NO surplus split-back: the surplus above `count` stays allocated
        # with the block so `free` remains exact ([02 §4] rationale)
        return Ok(AllocatedFrames{ range: block.range, order,
                                   usable: count })
Validation: W04-DV02–DV06.
```

## 6. `PageAllocator::free_contiguous`

```text
Name and stability: free_contiguous(alloc: AllocatedFrames) -> Result<(),
PageAllocError>. Takes the value returned by allocate (struct = capability:
a caller cannot free a range it was not handed, and cannot invent an
order). `range`/`order` always describe the FULL block, including any
held surplus from allocate_contiguous; `usable` is ignored by free.
Purpose and caller: return frames to the pool with full validation.
Preconditions / postconditions: pre — alloc was returned by this
allocator and not yet freed; post — frames Free and coalesced per
[02 §4](02-architecture-and-state.md); conservation holds; on Err state
unchanged.
Concurrency/allocation context: &mut self; no allocation.
Errors and failure guarantee: UnmanagedFree, DoubleFree, BadFreeRange
([02 §5](02-architecture-and-state.md) table) — typed, stateless.
Security/authorization checks: I4 validation before any state change;
metadata ranges cannot be freed (they were never allocated — table check).
Logic (pseudocode):
    range = alloc.range; order = alloc.order
    if order > MAX_ORDER or range.first % 2^order != 0: Err(BadFreeRange)
    if !managed_contains(range): Err(UnmanagedFree)
    for frame in range: if table.get(frame) != Allocated:
        -> Free seen: Err(DoubleFree); Unmanaged/ReservedMeta: Err(UnmanagedFree)
    table.set(range, Free); coalesce(range, order); insert_block
    return Ok(())
Validation: W04-DV03/DV07.
```

## 7. `AllocationStats`

```text
Name and stability: stats() -> AllocationStats { managed, free, used,
reserved_meta, per_region: BoundedList<RegionStats, MAX_MEMORY_BANKS> }.
Consumed by W06; asserted by tests; reported by W09.
Purpose and caller: managed/free/used/reserved accounting (P2-E08).
Preconditions / postconditions: conservation: managed == free + used +
reserved_meta, computed from the table (authority), with fast copies
cross-checked (divergence is a fatal invariant stop, [02 §5](02-architecture-and-state.md)).
Validation: W04-DV09.
```

## 8. Explicitly unauthorized interfaces

No allocation by address; no zeroing/poisoning hooks in the baseline; no
mapping of allocated frames (stage-1 concerns are not W04); no lock or
per-CPU parameter; no reclaim/unprotect; no GlobalAlloc impl (W05's
design); no serialization. `AllocatedFrames` is deliberately a plain value
with no Drop side effects — release is always explicit.
