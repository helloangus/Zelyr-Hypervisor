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

## 2. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W04 relies on | Failure boundary |
|---|---|---|---|---|
| A1 | W03 `UnsealedMemoryMap` planning queries + `MetadataPlan` seal + sealed `BootMemoryMap` queries | [W03 design](../p2-w03-boot-memory-map-ownership/README.md) | `allocatable_spans()` for planning and init; seal records the metadata plan; sealed map is immutable | A W03 contract mismatch is a design conflict to record; W04 must not recompute protection or filter unsealed maps |
| A2 | Host physical-access window with read/write coverage of allocatable RAM | P1-W08 (assumed; W01's A2 plus write access) | Metadata table lives in physical frames; a window access function turns frame ranges into `&mut [FrameState]` storage | Absent window → blocked upstream defect (same rule as W01); no identity map improvisation |
| A3 | P0 diagnostics/failure-class channel and frame/address primitives | P0 plans (assumed) | Fatal-stop path for boot-phase allocator errors; typed spans | Missing → blocked upstream defect |
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
  minus the metadata plan (established at init, audited);
- I2: a frame is in at most one buddy free list, or marked `Allocated`, or
  marked `ReservedMeta` — never two states;
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
while buddy gives O(log n) allocate/free with explicit coalescing and
compact free lists (per-order `PhysFrameRange` chains). Bitmap-style
per-frame state is retained anyway as the ownership/debug table (Decision 3
of the [README](README.md)), so the debug strength of the bitmap approach
is kept while allocation mechanics stay buddy.

ADR §18 names the buddy-versus-bitmap question as pending ADR-level
freeze. Reconciliation: the P2 task book (current stage authority) delegates
the algorithm choice to the approved detailed design — this document —
under its "must not freeze an allocator algorithm [in plans]" rule; the
choice here is therefore recorded as stage-local design freedom under that
delegation, not as an architecture decision. If the ADR register later
freezes a different algorithm, the superseding decision reworks internals;
the external contract (typed order/count allocation, never-protected,
stateless errors, accounting) is intentionally algorithm-independent so the
swap is contained. This paragraph is the reviewable reconciliation record.

## 6. Metadata sizing rule (planner input)

Per allocatable span: ownership table cost = ceil(frames / 4) bytes; buddy
free-list heads = (`MAX_ORDER`+1) × 8 bytes per region; per-span bookkeeping
constant < 64 bytes. The planner rounds each span's requirement up to whole
frames and prefers the *smallest* span that can host its own metadata
(fragmentation containment), then the next, so the largest span loses the
least capacity. All metadata for a span is contiguous at that span's tail.
These rules belong to W04 (planner); W03 only validates plan ranges —
keeping the split of authority from
[W03 §6](../p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md).

## 7. Layering

The allocator is platform-layer logic with one hardware edge: the A2 window
for metadata access. It must not contain arch registers, cache/TLB
maintenance (it allocates *frames*; mapping them is a later-stage stage-1
concern), board names, or QEMU constants. Host testability is structural:
every algorithm path operates on injected span tables and a caller-provided
metadata storage slice, so no physical memory is needed to test logic.
