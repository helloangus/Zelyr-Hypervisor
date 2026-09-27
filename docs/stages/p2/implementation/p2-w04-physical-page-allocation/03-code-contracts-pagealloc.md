# P2-W04 Code Contracts — Physical Page Allocator

**Status:** Proposed contracts; implementation blocked by W04-LAYOUT, target
integration additionally blocked by W04-MAP.\
**Parent:** [P2-W04 detailed design](README.md).  
**Version:** v0.2; supersedes contradictory September 18 pseudocode.

## 1. `FrameStateTable`

Own exclusive storage for each managed region, indexed by checked local offset
only. Keep Free/Allocated states, original allocation head/order, continuation
relations and allocator identity. Physical holes and protected metadata have no
mutable table entries. `get`, state/tag updates and independent recount are
bounded; construction rejects insufficient or aliased component storage.

The two-bit state component is not the entire metadata record. Concrete widths,
layout, alignment and stable owner identity are the W04-LAYOUT gate in
[01 §6](01-scope-and-foundations.md). No raw physical address creates storage.
Validation: W04-DV01/DV07, including sparse banks and wrong allocation boundaries.

## 2. `PageAllocator::plan_metadata`

`plan_metadata(&UnsealedMemoryMap) -> Result<MetadataPlan, PlanError>` is pure,
allocation-free planning. Use the complete checked sizing terms and deterministic
host-tail placement in [01 §6](01-scope-and-foundations.md). Bound candidates,
managed regions and physical metadata extents by W03's 46 normalized spans,
not its 8 input RAM banks. Each binding records candidate, resulting managed
span, hosting metadata extent and disjoint storage offset/length.

Reject insufficient hosts, overflow or capacity exhaustion with no partial
plan. Size from pre-carve candidate frames to avoid circular undersizing;
combine successive host-tail carvings and update available host capacity after
each placement. W03 validates physical metadata extents during seal; W04 owns
storage layout and binding validation. Validation: W04-DV01 with W03-DV09.

## 3. `PageAllocator::init`

Logical signature: `init(sealed, plan, exclusive_metadata_storage) ->
Result<PageAllocator, AllocatorInitError>`. Storage ownership is supplied by the
W04-MAP adapter or host buffers; concrete borrow types are frozen at its gate.
A shared `&dyn FrameStorage` is not authority to manufacture mutable aliases.

```text
require plan physical extents == sealed.metadata_ledger
require plan managed spans == sealed.allocatable_spans()
require storage corresponds to metadata extents, is exclusive and sufficient
for binding in plan.region_bindings:
    managed = exact corresponding sealed allocatable span
    storage = exclusive disjoint slice for binding
    initialize local states/tags and seed maximal aligned free buddies
    audit region without accessing managed frame contents
require managed == free and used == 0
require reserved_meta == sealed metadata union frame count
publish Ready
```

Do not subtract metadata from sealed allocatable again or search for metadata
inside that domain. Errors: `SealMismatch`, `DomainMismatch`, `WindowFailure`,
`MetadataInsufficient`, `ConservationBroken`. Failure publishes no allocator;
no boot retry is implied. Validation: W04-DV01, including nonempty metadata.

## 4. Buddy mechanics (internal)

`find_block`, `split_block`, `insert_block`, `remove_block`, `coalesce` operate
inside one managed region. Store all list nodes in metadata, with capacity for
one node per candidate frame. Validate resources before editing; valid split
and free paths cannot encounter ordinary `MetadataFull` after Ready.

During splitting, retain one half as the result candidate and insert only the
other half. At release, insert the final coalesced block exactly once. Check
buddy order and region membership before removal. No cross-hole coalescing and
no writes to managed pages. W04-LAYOUT must freeze list representation and
transaction order; metadata corruption is a fatal invariant error, not OOM.
Validation: W04-DV02–DV05/DV08, including worst-case fragmentation.

## 5. `PageAllocator::allocate` and `allocate_contiguous`

`allocate(order: Order) -> Result<AllocatedFrames, PageAllocError>`;
`allocate_contiguous(count: PageCount) -> Result<AllocatedFrames, PageAllocError>`.
Both require Ready, exclusive `&mut self`, no allocation or blocking/IRQ context.

`AllocatedFrames` privately contains full range/order, requested usable prefix
and allocator identity. It is neither Copy nor Clone, has no public constructor
and no Drop side effects. Read-only accessors expose the allocation dimensions.

Check `order <= 18`, count > 0, checked ceil-log2 and bounds before mutation.
An order above 18 is `OrderTooLarge`; zero count is `InvalidCount`; no suitable
block is `OutOfFrames` with region stats. A successful order-18 block is
2^18 * 4096 = 1 GiB. Mark the head and continuations of the full block before
returning its handle. Count-based allocation holds surplus until full release;
its accounting uses the full block. Failed calls preserve full logical state.
Validation: W04-DV02–DV06.

## 6. `PageAllocator::free_contiguous`

`free_contiguous(alloc: AllocatedFrames) -> Result<(), FreeFailure>`, where
`FreeFailure` returns both a typed reason and the unchanged handle. No allocation
or blocking; `&mut self` is required. The caller retains ownership on failure.

```text
check allocator identity or ForeignAllocator
check managed containment or UnmanagedFree
check range/count/alignment/order or BadFreeRange
check original head/order and all continuations:
    Free -> DoubleFree; different head/order -> BadFreeRange
preflight all remaining invariants before mutation
clear identity; mark Free; coalesce; insert final block once
```

Module-private negative fixtures may present duplicate/corrupted descriptors;
safe public callers cannot construct them. Metadata cannot be freed and mixed
allocations cannot be combined into a larger free. Failure preserves tags,
lists, counters and state. Validation: W04-DV03/DV07, including cross-allocator
handles and error-return ownership.

## 7. `AllocationStats`

`stats()` returns managed/free/used totals, `reserved_meta` and per-region
managed/free/used values. Capacity is `MAX_ALLOCATABLE_SPANS`. Metadata totals
come from the sealed physical ledger, counted once even when multiple regions
share a hosting extent. Component bindings account for storage bytes, not a
second count of physical metadata frames.

```text
managed == sum(sealed.allocatable_spans().counts) == free + used
reserved_meta == count(sealed.metadata_ledger union)
ram == sealed.allocatable_frames + sealed.protected_frames
```

`reserved_meta` is included in protected, excluded from managed. Audits recount
states/tags/lists independently and compare cached counts. W06 and W09 use these
same equations. Validation: W04-DV09; exact conservation after every operation.

## 8. Explicitly unauthorized interfaces

No allocation by requested physical address, Guest transfer, reclaim/unprotect,
SMP locking, zeroing/poisoning or implicit GlobalAlloc. W04's architecture adapter
maps only metadata; ordinary allocated-page mappings belong to W05 or later
consumers. No layout, aperture or unsafe implementation is authorized while its
corresponding detailed-design gate remains open.
