# P2-W05 Dynamic Small-Allocation Foundation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The dynamic small-object allocation contract with explicit
failure and release behavior, backed by the safe page-allocation foundation,
required by [P2-W05](../../plans/p2-w05-dynamic-small-allocation.md).  
**Owner/change context:** P2-W05 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W05. It converts the bounded
work-package plan into a code-bearing design for one object system: a
fixed-size-class slab heap layered exclusively on W04's frame allocator —
eight size classes from 16 B to 2 KiB in one-page slabs with per-slab
bitmaps, a bounded page-backed path for large or over-aligned requests, a
slab directory for ownership checks, a heap budget for resource
containment, a fallible checked API, and a `GlobalAlloc` adapter so
`alloc`-based dynamic allocation becomes available to later P2 consumers
and to P3/P4 designs. It deliberately does **not** design VM/vCPU or
scheduler objects (P4/P7), does not benchmark performance (explicitly out
of scope), does not change W04's contract, and does not retro-fit the
boot-phase bounded arrays of W01–W03 onto the heap (their role is defined
and bounded by this design — see
[01 §6](01-scope-and-foundations.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then
loads only the linked supporting file for its assigned step, after the
Coding Guidelines preflight (repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md),
[P2-W05 plan](../../plans/p2-w05-dynamic-small-allocation.md)).

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed contracts (including the `alloc`/target interplay), the static-array boundary review, and failure policy. |
| [02-architecture-and-state.md](02-architecture-and-state.md) | Size-class/slab structure, directory, budget, lifecycle, `GlobalAlloc` adapter, and the single-core concurrency boundary. |
| [03-code-contracts-heap.md](03-code-contracts-heap.md) | Exact function/type contracts with pseudocode. |
| [04-implementation-workflow.md](04-implementation-workflow.md) | Ordered implementation steps with acceptance and failure handling. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | Validation matrix (P2-V07), error/security/observability model, handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W05 plan](../../plans/p2-w05-dynamic-small-allocation.md) → this design
→ Coding Guidelines. Binding constraints:

- ADR-014 authorizes EL2 heap/slab/object allocation; the task book requires
  it never rest on fixed static arrays as the *permanent* runtime model —
  the heap is that permanent model, and this design documents the temporary
  role of the pre-heap boot arrays
  ([01 §6](01-scope-and-foundations.md)).
- The task book's plan-level freeze prohibition is satisfied the same way as
  W04: heap/slab shape choices left open by the plan are fixed here as
  stage-local decisions with rationale ([01 §5](01-scope-and-foundations.md)).
- Explicit failure behavior: exhaustion and invalid frees are typed values;
  the design defines exactly where a failure is fatal (boot phase) versus an
  assertable `Err` (host tests) — no undefined behavior on exhaustion
  (P2-F02).
- Single-core boot phase; allocator locking and SMP rules are P3 scope — the
  boundary is stated in every contract (task book requirement).

Classification. **Required:** size-class slab machinery, the checked
allocation API with typed failures, release with invalid/double-free
detection, the budget, the stress-validation invariants, host testability.
**Reserved** (recorded triggers): additional/adjustable size classes,
grow-in-place and realloc semantics, alignment above page size, per-class
policies or caches, cross-CPU pools (P3), TLSF/buddy-style general
allocators, external allocator crates (P0-W18 governance). **Out of
Scope:** VM/vCPU/scheduler object designs, performance benchmarking, W04
internals, inspection output (W06), and board-specific behavior.

## Requirement-to-design mapping

The tracked sources define P2-F01–F04 at group granularity only; rows below
are this design's reviewable enumeration from the plan's scope wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-F01 | Dynamic storage for runtime objects and variable platform-information needs beyond the boot-phase bounded arrays | [02 §2](02-architecture-and-state.md), [03 §3](03-code-contracts-heap.md) | P2-V07 (W05-DV02) |
| P2-F02 | Explicit allocation failure: typed exhaustion, budget containment, defined `GlobalAlloc` failure policy | [03 §4](03-code-contracts-heap.md), [01 §7](01-scope-and-foundations.md) | P2-V07 (W05-DV03, DV04) |
| P2-F03 | Release capability with slab-directory ownership checks; invalid/double free detected | [03 §5](03-code-contracts-heap.md) | P2-V07 (W05-DV05) |
| P2-F04 | Repeated allocation/free stress preserves stated invariants; correctness separated from performance | [02 §5](02-architecture-and-state.md), [05 §3](05-validation-and-handoff.md) | P2-V07 (W05-DV06, DV07) |
| Static-array boundary (plan work sequence 4) | Pre-heap arrays documented as temporary; heap is the permanent model; heap itself is dynamic within budget | [01 §6](01-scope-and-foundations.md) | W05 closure review (W05-DV08) |
| Backing safety (plan step 1) | All heap memory derives from W04 allocations; no page bypass | [02 §4](02-architecture-and-state.md) | P2-V07 (W05-DV01) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, no W01–W04
implementation, and no target definition deciding `alloc` availability
(P0-W03 owns the target and `build-std` decisions). W05 is designed against
W04's frame API as an assumed prerequisite; the `GlobalAlloc` adapter is
designed conditionally on the P0 target supplying `alloc` (assumed
contract, explicit failure boundary in
[01 §4](01-scope-and-foundations.md)).

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Explicit failure/release behavior under exhaustion and stress (P2-V07) | No heap code exists | Slab heap with typed failures and invariant-preserving release | Without typed failure there is no explicit behavior to validate | W05 (this design) | W05-DV02–DV07 host tests |
| Backing capacity from W04's protected/OOM contract (plan step 1) | W04 designed, not implemented | Assumed W04 frame API with failure boundary | Heap pages must inherit the hard gate transitively | W04 owner; W05 consumer | W05-DV01 + joint tests when W04 lands |
| Permanent dynamic model replacing fixed static arrays (plan step 4) | W01–W03 use bounded boot arrays by design | Documented temporary role + heap as the post-boot model | The prohibition targets permanence, not existence | W05 records the boundary; W01–W03 keep their boot-phase role | W05-DV08 review |
| Stress validation basis distinct from performance claims (plan step 5) | Nothing | Invariant list + stress properties in the validation matrix | Correctness evidence must not leak into performance claims | W05 | W05-DV06/DV07 |
| `alloc` availability for the adapter | Target undefined (P0-W03 pending) | Conditional design: adapter requires `alloc`; inner checked API is unconditional | The dynamic-allocation contract must not hinge on an unmade toolchain decision | P0-W03 owns target; W05 owns both layers | W05-DV04 (adapter) only if `alloc` lands; inner API always |

No ledger row invents a crate, target, or performance target; upstream
absence is handled with host fixtures and the conditional adapter.

## Resolved design decisions and their authority

1. **Fixed size-class slab heap**: classes {16, 32, 64, 128, 256, 512,
   1024, 2048} bytes; one 4 KiB page per slab; in-band slab header
   (class, free count, allocation bitmap); per-class partial-slab lists.
   Rationale versus alternatives in
   [01 §5](01-scope-and-foundations.md): P2's dynamic needs are small
   records and strings; fixed classes give O(1) alloc/free, deterministic
   behavior for stress validation, tiny metadata, and full host
   testability — without the complexity of TLSF/buddy heaps or an
   external crate (P0-W18 has approved none).
2. **Bounded page-backed path for large/over-aligned requests**, tracked
   in a fixed-capacity directory. Rationale: keeps one ownership authority
   for every heap byte while bounding directory cost; >`MAX_SMALL` sizes
   and >page alignments are the documented boundary
   ([02 §2](02-architecture-and-state.md)).
3. **Heap budget** (`HeapBudget`, default 256 pages = 1 MiB) enforced at
   slab/page acquisition. Rationale: the heap must not be able to consume
   the allocatable domain silently — resource containment is a designed
   property (ADR-018 direction), and exhaustion must be an explicit typed
   event, not an emergent one.
4. **Two-layer API**: a total, checked inner API returning `Result` (the
   P2-V07 validation surface), and a thin `GlobalAlloc` adapter that maps
   failures to the `alloc` contract with a fatal allocation-error handler
   at P2 ([01 §7](01-scope-and-foundations.md)). Rationale: `GlobalAlloc`
   cannot return typed errors, and host tests need assertable failures;
   the split serves both.
5. **Fatal allocation-error policy for the boot phase**: heap exhaustion
   through the `alloc` surface halts with a diagnostic (P0-W14
   resource-exhaustion class). Rationale: P2 has no recovery consumer;
   retryable/soft-failure policy is Reserved for designs that need it.
   The inner checked API remains the fallible path for future consumers.
6. **Dealloc of a pointer the directory does not own is an invariant
   violation**: `Err(InvalidFree)` from the inner API; fatal halt via the
   adapter. Rationale: the only legal callers are hypervisor components —
   an unknown pointer is a bug, not untrusted input; the slab directory
   makes this check exact rather than heuristic.
7. **No zeroing, no drop semantics, no realloc in P2.** Rationale: callers
   own initialization; `alloc` types handle their own drop; grow-in-place
   is Reserved. The adapter implements exactly `alloc`/`dealloc`
   (`realloc` defaults through them).

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md)
   (boundaries, static-array review, failure policy), then
   [02-architecture-and-state.md](02-architecture-and-state.md)
   (structure, lifecycle, adapter).
2. Implement per [04-implementation-workflow.md](04-implementation-workflow.md);
   contracts in [03-code-contracts-heap.md](03-code-contracts-heap.md).
3. Validate per [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Record decisions/deviations in
   `../p2-w05-dynamic-small-allocation-record.md`; evidence in
   `../../verification/p2-w05-dynamic-small-allocation-verification.md`
   when that work starts; nothing here claims W05 complete.

## Explicitly excluded interfaces

No page-allocator changes (W04's API is consumed, never extended); no
lock types or per-CPU hooks (P3; p3-w06 owns synchronization); no VM/vCPU
or capability objects; no performance instrumentation or benchmarks; no
board names or QEMU constants; no crate/workspace manifests. W06 consumes
`HeapStats` only; P3/P4 consume the recorded contract via W10.

## Downstream handoff

- **W06** ([../p2-w06-platform-memory-inspection/README.md](../p2-w06-platform-memory-inspection/README.md))
  renders `HeapStats` (per-class use, budget headroom, page-backed
  allocations); W05 guarantees the numbers are live-derived.
- **W08/W09** receive exhaustion, invalid-free, and repeated-lifecycle
  expectations as regression and QEMU-integration criteria
  (P2-V10/P2-V11).
- **P3** (via W10; p3-w04/p3-w06 plans) receives the dynamic-allocation
  capability with the stated single-core boundary; P3 owns locking,
  per-CPU pools, and any blocking-allocation policy.
- **P4** (via W10; p4-w02–p4-w04 plans) receives the heap as the backing
  for VM/vCPU object allocation in their designs — the objects themselves
  are explicitly not designed here.
