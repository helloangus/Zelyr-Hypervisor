# P2-W04 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W04 detailed design](README.md).

## 1. Package outcome

A live allocator with this property: for every sequence of `allocate`/
`free_contiguous` calls accepted by the API, every returned frame range lies
entirely inside W03's sealed-map allocatable domain, every range ever
returned and not yet freed is marked `Allocated` in the ownership table,
and `AllocationStats` satisfies its conservation equation at every step.
Exhaustion and invalid operations are typed errors with unchanged state.

## 2. Current prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W04 relies on | Failure boundary |
|---|---|---|---|---|
| A1 | W03 `UnsealedMemoryMap` planning queries + `MetadataPlan` seal + sealed `BootMemoryMap` queries | [W03 design](../p2-w03-boot-memory-map-ownership/README.md) | `allocatable_spans()` for planning and init; seal records the metadata plan; sealed map is immutable | A W03 contract mismatch is a design conflict to record; W04 must not recompute protection or filter unsealed maps |
| A2 | Exclusive writable metadata storage | P2-W04-owned architecture adapter, not yet designed/implemented | RW/XN access to sealed metadata only; exclusive borrowed storage | W04-MAP gate below; no P1 writable-window assumption or identity-map improvisation |
| A3 | Existing address types and fatal diagnostics | P0/P1 implementation | Reuse `PhysAddr`/`ByteSize`, W03 frame types and boot diagnostic path | Record an actual missing interface rather than assuming the workspace is absent |
| A4 | W05 will be the only P2 consumer of the frame API | [W05 plan](../../plans/p2-w05-dynamic-small-allocation.md) | API sized for slab backing, not general export | If another P2 consumer appears, that is a design change |

## 3. Scope classification detail

**Required** (P2-E01–E08): the metadata bootstrap and planning handshake;
per-region buddy free lists; the frame-state table; order allocation,
exact-count allocation, and free; typed OOM with per-region detail; the
four debug detections (invalid, duplicate, unmanaged, order-mismatched
free); accounting (managed/free/used/reserved); full host testability.

**Reserved** with triggers: contiguous spans above `MAX_ORDER` (trigger:
an approved consumer design needs them — e.g., P4 Guest RAM policy);
allocator-strategy swap (trigger: ADR-level freeze per §5); poisoning of
freed frames (trigger: a diagnostics design asks); metadata relocation or
multiple metadata strategies (trigger: real-board needs); per-CPU pools and
all synchronization (trigger: P3 design).

**Out of Scope:** small-object allocation (W05); Guest memory transfer,
Stage-2, VMID (P4); inspection (W06); board-specific behavior; performance
benchmarks (the task book claims no allocator performance target for P2).

## 4. Untrusted-input stance and safety invariants

Allocator inputs are *not* guest data in P2, but the W03 map derives from
untrusted firmware descriptions, so the allocator inherits containment
duties: the allocator must assume nothing about span contents and must make
protection non-bypassable. Invariants every contract preserves:

- I1: the managed domain is exactly the sealed map's allocatable spans
  (metadata was removed by seal; do not subtract it again);
- I2: each managed frame is Free or Allocated exactly once. Metadata is
  outside the managed domain and belongs only to the sealed metadata ledger;
- I3: free-list membership implies `Free` in the table (the table is the
  authority; lists are the index);
- I4: `free` accepts only ranges that are frame-aligned, order-aligned
  (`first % 2^order == 0`), fully `Allocated`, and were returned as one
  allocation of that order;
- I5: every arithmetic on frame numbers/orders is checked; order ≤
  `MAX_ORDER` enforced at entry.

## 5. Algorithm selection and the ADR §18 reconciliation

Choice: **regioned buddy allocator**, orders 0..=18. Rationale versus the
bitmap alternative: the plan requires contiguous multi-page allocation,
merge-on-free behavior, and bounded work per operation; a frame bitmap
would need O(n) scans for contiguous runs and has no natural coalescing,
while buddy bounds split/coalesce levels with explicit coalescing and
compact free lists (per-order `PhysFrameRange` chains). Bitmap-style
per-frame state is retained anyway as the ownership/debug table (Decision 3
of the [README](README.md)), so the debug strength of the bitmap approach
is kept while allocation mechanics stay buddy.

ADR §18 names the buddy-versus-bitmap question as pending ADR-level
freeze. Reconciliation: the P2 task book (current stage authority) delegates
the algorithm choice to an approved detailed design; this document retains
the existing buddy proposal
under its "must not freeze an allocator algorithm [in plans]" rule; the
choice here is therefore recorded as stage-local design freedom under that
delegation, not as an architecture decision. If the ADR register later
freezes a different algorithm, the superseding decision reworks internals;
the external contract (typed order/count allocation, never-protected,
stateless errors, accounting) is intentionally algorithm-independent so the
swap is contained. This paragraph is the reviewable reconciliation record.

## 6. Metadata sizing and design admission

Each normalized draft span is a region candidate, independent of the input
RAM-bank count. Capacity follows W03 `MAX_ALLOCATABLE_SPANS=46`. Use local
`frame - region.first` indexing after containment checks; physical holes never
consume ownership-table storage. One bank may yield many regions.

For each candidate of N frames, reserve a checked upper bound computed from:

```text
state_bytes(N) + allocation_identity_bytes(N)
+ free_node_count(N) * free_node_stride
+ (MAX_ORDER + 1) * list_head_stride + region_bookkeeping
+ alignment_padding_for_every_component
```

`free_node_count(N)=N` is the conservative reservation for one node per possible
free block. No free-list node lives in a managed frame. Allocation identity
includes head/order, continuation-to-head relation and allocator identity;
two bits per frame alone are insufficient. Size using the original candidate
N, before carving metadata, so no self-sizing fixed point is needed. All size
arithmetic and storage offsets are checked; every component is disjoint.

Plan candidates in physical order; choose the smallest remaining span that
fits each rounded requirement while leaving at least one managed frame, with
physical-address tie-break. Accumulate carvings at host tails, never repeatedly
select the same unmodified tail. A small candidate may use another host's
storage. Merge adjacent metadata carvings per host; preserve candidate-to-storage
bindings and offsets. Reject a plan that cannot host every candidate's metadata;
never silently drop RAM or expose an incomplete domain. Tail carving preserves
at most 46 managed regions and 46 physical metadata extents.

**W04-LAYOUT — detailed design still required before allocator coding:** freeze
record widths, strides, exact tag/identity representation, byte alignment,
checked upper bounds and failure-transaction ordering. The former
`LIST_NODE_RESERVE` placeholder did not provide these. Prove that splitting and
freeing any valid sequence cannot exhaust the reserved nodes. Insufficient
storage is an init/planning error; a valid free must not fail `MetadataFull`.
This revision fixes the required accounting terms, not an unreviewed byte ABI.

**W04-MAP — detailed design still required before target integration:** the P2
architecture adapter must specify aperture VA/PA bounds, table ownership and
capacity, bootstrap table placement, RW/XN attributes, publication/barriers/TLB
ordering, read-only DTB and image alias exclusion, exclusive mutable-slice
lifetime, failure before publication and terminal handling after partial
publication. Only sealed metadata extents may be exposed. Any new tables must
be image-owned or already protected before allocation. The adapter's storage
owner outlives every borrow and cannot remap or issue overlapping mutable views.
The current P1 maps and W01 DTB aperture do not satisfy this gate.

## 7. Layering

The allocator is platform-layer logic with one hardware edge: the A2 window
for metadata access. It must not contain arch registers, cache/TLB
maintenance (the P2 metadata adapter owns those operations), board names, or QEMU constants. Host testability is structural:
every algorithm path operates on injected span tables and a caller-provided
metadata storage slice, so no physical memory is needed to test logic.
