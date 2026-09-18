# P2-W04 Physical Page Allocation Foundation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The safe, observable host physical-page allocation and release
capability required by
[P2-W04](../../plans/p2-w04-physical-page-allocation.md).  
**Owner/change context:** P2-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W04. It converts the bounded
work-package plan into a code-bearing design for one object system: a
regioned buddy page allocator whose entire allocation domain is derived from
W03's sealed map, with a two-bit frame-ownership table serving as both the
debug/invalid-free detector and the accounting authority, a
metadata-planning handshake with W03 (draft → plan → seal), explicit typed
OOM, and per-region plus total accounting. It deliberately does **not**
design small-object allocation (W05 builds on this one), does not design SMP
locks or per-CPU pools (P3; p3-w06 owns synchronization), does not design
Guest-memory transfer or Stage-2 (P4), and does not define contiguous
allocation beyond the documented order-plus-split contract.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the linked supporting file for its assigned step, after the Coding
Guidelines preflight (repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md),
[P2-W04 plan](../../plans/p2-w04-physical-page-allocation.md)).

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed contracts, algorithm-selection rationale (including the ADR §18 pending item). |
| [02-architecture-and-state.md](02-architecture-and-state.md) | Buddy structure, ownership table, accounting model, metadata bootstrap, lifecycle, single-core concurrency boundary. |
| [03-code-contracts-pagealloc.md](03-code-contracts-pagealloc.md) | Exact function/type contracts with pseudocode. |
| [04-implementation-workflow.md](04-implementation-workflow.md) | Ordered implementation steps with acceptance and failure handling. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | Validation matrix (P2-V06), error/security/observability model, handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W04 plan](../../plans/p2-w04-physical-page-allocation.md) → this design
→ Coding Guidelines. Binding constraints:

- The task book exit criterion is the hard gate: no allocation/free sequence
  may ever return a protected frame; W03's sealed map is the sole source of
  the allocation domain.
- ADR-014 authorizes EL2 dynamic page allocation; ADR §18 lists "P2 物理
  页 allocator 首选 buddy 还是 bitmap+size-class 组合" as pending ADR-level
  freeze. The task book delegates the algorithm choice to the approved
  detailed design; this design therefore records the choice as stage-local
  design freedom under that delegation, with an explicit reconciliation
  note ([01 §5](01-scope-and-foundations.md)): if a later ADR freezes a
  different algorithm, an ADR-superseding change reworks internals while the
  external contract (typed frame allocation, never-protected guarantee,
  accounting) is designed to survive the swap.
- The task book requires allocation ownership debug metadata and accounting;
  the two-bit frame table is that metadata and is load-bearing, not
  optional.
- Single-core boot phase: the allocator has one owner and no interior
  synchronization; SMP rules are P3 scope and this design states the
  boundary explicitly (task book requirement; [02 §6](02-architecture-and-state.md)).

Classification. **Required:** metadata bootstrap and planning handshake,
per-region buddy free lists, ownership table, allocate/free with order and
exact-count paths, explicit OOM, invalid/duplicate/foreign-free detection,
accounting, host testability of the whole algorithm. **Reserved** (recorded
triggers): contiguous allocations above the maximum order, unprotect/reclaim
interfaces, allocator strategy swap (buddy → bitmap/size-class), per-CPU
pools and lock design (P3), large-page/huge-frame policy, metadata relocation
or compaction. **Out of Scope:** small-object allocation (W05), Guest memory
ownership transfer (P4), inspection output (W06), and any board-specific
behavior.

## Requirement-to-design mapping

The tracked sources define P2-E01–E08 at group granularity only; rows below
are this design's reviewable enumeration from the plan's scope wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-E01 | Allocation from the sealed map's allocatable spans, per region | [03 §5](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV02) |
| P2-E02 | Release with ownership validation; free restores exact prior state | [03 §6](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV03) |
| P2-E03 | Page alignment and order/size discipline (power-of-two blocks) | [03 §5](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV04) |
| P2-E04 | Multiple RAM regions supported with per-region free lists | [02 §2](02-architecture-and-state.md), [03 §4](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV05) |
| P2-E05 | Explicit typed OOM with region statistics; no partial state | [03 §5](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV06) |
| P2-E06 | Debug detection: invalid free, duplicate free, unmanaged-range free, misaligned/order-mismatched free | [03 §6](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV07) |
| P2-E07 | Protected-frame impossibility: domain derived only from sealed map; audit at seal | [02 §4](02-architecture-and-state.md), [03 §3](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV01, DV08); hard gate |
| P2-E08 | Managed/free/used/reserved accounting, queryable | [03 §7](03-code-contracts-pagealloc.md) | P2-V06 (W04-DV09) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, no W01–W03
implementation. W04 is designed against W03's sealed map and draft queries
as assumed prerequisites; the physical-memory access window is the same
assumed P1 contract as in
[W01 §2](../p2-w01-boot-platform-description-intake/01-intake-boundary.md)
(A2), extended to writes.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Safe allocation that never returns a protected range (P2-V06, hard gate) | No allocator code exists | Domain derived from sealed map only; structural seal-before-use ordering | Protection must precede capability | W04 consuming W03 | W04-DV01/DV08 property tests |
| Allocation/free with alignment, multi-region, OOM (P2-E01–E05) | Nothing | Regioned buddy with typed errors | The plan's outcome needs all five behaviors in one contract | W04 | W04-DV02–DV06 |
| Debug checks for invalid/duplicate/unmanaged free (P2-E06) | Nothing | Two-bit frame ownership table as allocator state | Detection requires per-frame state | W04 (task book mandates ownership debug metadata) | W04-DV07 |
| Managed/free/used/reserved accounting (P2-E08) | Nothing | Accounting derived from the same ownership table + free lists | Two bookkeeping systems would diverge | W04 | W04-DV09 |
| W03 map available (plan step 1) | W03 designed, not implemented | Assumed W03 contracts with failure boundary | Allocator without the map has no safe domain | W03 owner; W04 consumer | Joint seal test (W03-DV09/W04-DV01) when both land |
| Metadata needs a home before the allocator exists | Nothing | Bootstrap carve from region tails via W03's plan/seal | Metadata cannot be allocated from the not-yet-existing allocator | W04 plans; W03 seals | W04-DV01 |
| P3 will add concurrency | No SMP exists | Stated boundary: single owner, no interior locks; P3 wraps | Task book requires the boundary explicit | P3 (p3-w06) owns the lock design | P3 reviews; nothing to run in P2 |

No ledger row invents a crate, target, or runtime policy; upstream absence
is handled with host fixtures and the joint-seal path.

## Resolved design decisions and their authority

1. **Allocator algorithm: per-region buddy with bounded maximum order**
   (`MAX_ORDER = 18`, i.e. 2^18 frames = 256 MiB maximum single block;
   larger requests are typed `OutOfFrames` until the Reserved extension
   adds a spanning path). Rationale and ADR-§18 reconciliation in
   [01 §5](01-scope-and-foundations.md). Swap-tolerance: W04's external
   contract is algorithm-independent.
2. **Exact-count allocation = order allocation with a held surplus**
   (`allocate_contiguous(count)` allocates the ceil-log2-order block and
   keeps the surplus frames allocated until the block is freed); no
   separate first-fit machinery and no split-back in P2. Rationale: a
   split-back tail would make the truncated range unfreeable with its
   original order without per-allocation size tags; holding the surplus
   keeps `free` exact, stateless, and invariant-safe. The bounded
   fragmentation cost is explicit and stress-tested. P4's Guest RAM needs
   are expected to fit within `MAX_ORDER` blocks or use multiple
   allocations; its design decides ([01 §6](01-scope-and-foundations.md)).
3. **Two-bit frame-state table** (`Unmanaged / Free / Allocated /
   ReservedMeta`) as the single per-frame authority, doubling as debug
   detector and accounting source. Rationale: the task book requires
   ownership debug metadata; without per-frame state, duplicate-free and
   foreign-free detection would be heuristic. Cost: 2 bits/frame (64 KiB
   per GiB) — bounded and accounted as `HypervisorMetadata`.
4. **Metadata bootstrap by tail-carve with W03's plan/seal handshake.**
   W04 plans fixed-size metadata areas at the tail of allocatable spans
   against the draft map; W03 seals them as `HypervisorMetadata`; the
   allocator initializes only from the sealed map. Rationale: solves the
   bootstrap circularity structurally and keeps "metadata protected before
   allocation" a map property, not an allocator hope.
5. **All failure is typed and stateless:** failed allocate/free leaves
   every free list and table entry unchanged. Invalid frees are `Err` at
   the API; the boot-phase caller treats allocator errors as fatal stops
   with diagnostics (P0-W14 classes), while host tests assert `Err`
   variants directly.
6. **One owner, no interior locking.** The allocator is a value owned by
   the boot sequence; `&mut self` methods. SMP synchronization, per-CPU
   pools, and lock ordering are P3 scope (p3-w06); this boundary is
   explicit in every contract's concurrency field.
7. **The allocator never writes a frame it allocates** (no zeroing, no
   poisoning by default). Rationale: P2 has no consumer requiring
   initialization; zeroing cost at boot is unjustified. W05's slabs and
   P4's Guest RAM own their initialization policies. A debug build
   *may* poison freed frames as a Reserved diagnostic aid — not baseline.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md)
   (algorithm rationale and boundaries), then
   [02-architecture-and-state.md](02-architecture-and-state.md)
   (structure, bootstrap, lifecycle).
2. Implement per [04-implementation-workflow.md](04-implementation-workflow.md);
   contracts in [03-code-contracts-pagealloc.md](03-code-contracts-pagealloc.md).
3. Validate per [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Record decisions/deviations in
   `../p2-w04-physical-page-allocation-record.md`; evidence in
   `../../verification/p2-w04-physical-page-allocation-verification.md` when
   that work starts; nothing here claims W04 complete.

## Explicitly excluded interfaces

No `GlobalAlloc`/heap surface (W05); no lock types, atomics requirements,
or per-CPU hooks (P3); no Guest-memory donation/transfer; no device/DMA
pinning; no unprotect/reclaim; no huge-page policy; no crate/workspace
manifests; no board names or QEMU constants. W06 consumes `AllocationStats`
only; W05 consumes the frame-level API only.

## Downstream handoff

- **W05** ([../p2-w05-dynamic-small-allocation/README.md](../p2-w05-dynamic-small-allocation/README.md))
  receives the frame-level allocate/free contract as its exclusive backing;
  W05 adds size-class management above it and owns its own slab metadata.
- **W06** ([../p2-w06-platform-memory-inspection/README.md](../p2-w06-platform-memory-inspection/README.md))
  renders `AllocationStats` and map accounting; W04 supplies the types and
  guarantees the numbers are live-derived.
- **W08/W09** receive exhaustion, invalid-free, stress, and accounting
  expectations as regression and QEMU-integration criteria (P2-V10/P2-V11).
- **P3** (via W10; p3-w06, p3-w04 plans) receives the stated concurrency
  boundary: single-owner boot-phase allocator, interior-lock-free by
  design, ready to be wrapped; P3 owns the locking design and per-CPU
  policy. No allocator internals change for P3 unless its design says so.
- **P4** (via W10; p4-w02/p4-w03 plans) receives frame allocation, the
  protected-exclusion guarantee, and accounting as the Stage-2/Guest-memory
  foundation — explicitly not ownership-transfer mechanisms.
