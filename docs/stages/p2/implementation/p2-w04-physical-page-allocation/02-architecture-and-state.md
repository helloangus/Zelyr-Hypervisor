# P2-W04 Architecture, State, and Lifecycle Design

**Status:** Proposed detailed design; W04-LAYOUT and W04-MAP remain admission gates.\
**Parent:** [P2-W04 detailed design](README.md).\
**Version:** v0.2; 2026-09-27 contract reconciliation; no implementation claimed.

## 1. Logical modules

`pagealloc::plan` sizes and places metadata against the draft; `table` owns
per-region frame states and allocation identity; `buddy` indexes free blocks;
`api` validates operations; `stats` derives counts. The pure modules consume
injected storage. The P2 architecture adapter owns writable metadata access,
not generic allocation logic or a permanent physical direct map.

## 2. Core state

```text
PageAllocator {
  regions: BoundedList<BuddyRegion, MAX_ALLOCATABLE_SPANS>,
  metadata_ledger, allocator_identity, exclusive_metadata_storage,
}
BuddyRegion {
  span: sealed allocatable span,
  local_frame_states, allocation_heads_and_continuations,
  free_lists[MAX_ORDER + 1], free_frames,
}
```

Local table indices cover only the corresponding managed span. Free/Allocated
state and allocation-head/order information are authoritative; lists are an
index. Metadata and physical holes are not table entries. Allocated blocks have
one head with order and allocator identity, and validated continuation markers.
The exact representation and storage formula are W04-LAYOUT, not two-bit-only
metadata. All nodes live in the reserved metadata storage. Worst-case capacity
is preallocated so a valid release never needs fallible node allocation.

## 3. Metadata bootstrap and lifecycle

```text
W03 draft -> W04 plan -> W03 seal -> P2 metadata mapping -> W04 init -> Ready
```

`MetadataPlan` binds each original candidate to its resulting managed span and
to a storage extent plus offset/length. Init checks plan/seal equality, obtains
exclusive metadata storage through the adapter, and initializes each sealed
allocatable span directly. Metadata cannot be found *inside* such a span.

Before publishing Ready, audit: domain equals sealed allocatable; metadata
extents equal the sealed metadata ledger; bindings and storage slices are
disjoint and adequate; every managed frame starts Free; free-list/table counts
agree. At init `managed = free`, `used = 0`. `reserved_meta` is separate,
protected memory, never part of managed. Failure publishes no allocator.

## 4. Allocation and free mechanics

- `allocate(order)` selects a free block in deterministic region/order order,
  splits to the requested order, then records exactly one allocated head and
  its continuations. Check all fallible conditions before mutation.
- `allocate_contiguous(count)` requires count > 0, checks ceil-log2 and maximum
  order, and holds the full power-of-two block. `usable` records the requested
  prefix; accounting and release always use the full block.
- `free_contiguous` verifies owner identity, original head, exact range/order,
  continuation state and region containment before changing anything. Then
  clear allocation identity, mark Free and coalesce within that region only.
- Metadata capacity is proven before Ready; no expected mid-operation failure
  is permitted. Internal corruption is an invariant stop, not ordinary OOM.
  Failure tests compare full logical state as well as stats; equal counts
  alone cannot establish rollback of list topology or allocation identity.

## 5. Debug detection model (P2-E06)

| Defect | Required detection |
|---|---|
| Foreign allocator handle | `ForeignAllocator`, before domain lookup/mutation |
| Outside managed domain, metadata or physical hole | `UnmanagedFree` |
| Free block presented again in internal negative fixtures | `DoubleFree` |
| Wrong order, interior/partial range, two allocations combined | `BadFreeRange` via original head and continuation validation |
| Misalignment or count/order mismatch | `BadFreeRange` |

`AllocatedFrames` has private fields, no public constructor, no Copy/Clone and
no implicit Drop free. A failed consuming release returns the unchanged handle
alongside the typed error. Production safe callers cannot fabricate duplicate
handles; module-private test fixtures exercise corrupted descriptors. There is
no residual exemption for order mismatch. Ownership identity must remain stable
if the allocator value moves; freeze its representation under W04-LAYOUT.

## 6. Concurrency, allocation, and interrupt context

Single boot CPU, one owner, `&mut self`, no IRQ use or interior locking. P3 owns
SMP synchronization. Storage is reserved before Ready; operations never acquire
heap memory. Buddy split/coalesce steps are order-bounded, but state/tag updates
and audits may traverse frames; do not claim whole-call O(log n) before the
representation and list lookup costs are frozen. No performance target is added.

## 7. Failure model

Allocation returns typed `OrderTooLarge`, `InvalidCount` or `OutOfFrames` with
per-region statistics. Release returns typed errors from §5 plus the unchanged
handle. Planning/init errors include insufficient metadata, seal/domain/storage
mismatch and conservation failure; all stop boot without a published allocator.
Failed expected operations leave table, tags, lists and counters unchanged.

## 8. Security model summary

Sealed domain authority, complete source protection, exclusive metadata storage,
checked indexing and original-allocation validation enforce the boundary. No
zeroing, poisoning, or writes to managed frame contents are performed by W04.
W05 owns its separate backing-page mapping and initialization. Under-declared
firmware protection remains an inherited limitation; W04-MAP and W04-LAYOUT
remain design work, not runtime evidence.
