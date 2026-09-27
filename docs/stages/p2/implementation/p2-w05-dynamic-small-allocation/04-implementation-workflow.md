# P2-W05 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

Load the documents named in the [entry README](README.md); inspect the
tracked tree. P0/P1 and W01/W02 exist; W03/W04 are not implemented. W05
uses buffer-backed pages on host and the revised W04 non-Copy handles. Close
W05-MAP before target backing and W05-GLOBAL before the global adapter.



Stop and obtain direction instead of guessing when:

- W04's frame API differs from the assumed contract — W04 contract
  revision (design conflict); no page-path workaround;
- W05-GLOBAL lacks a safe publication/reentrancy/lifetime or pinned-target
  failure-path contract — keep the checked API evidence separate; do not
  fabricate a shared-to-mutable adapter;
- a consumer asks for realloc, zeroing, caches, or >page alignment —
  Reserved ([README](README.md); [01 §3](01-scope-and-foundations.md));
  design change required;
- SMP locking or CPU-local pools are requested — P3 scope (p3-w06); stop;
- a stress test seems to need performance measurement — out of scope by
  plan; the test is asserting invariants, not speed.

## 2. Ordered implementation steps

### Step 1 — layout routing and constants

Target: `heap::layout`, contract [03 §2](03-code-contracts-heap.md).

Work: route table per the constants of
[02 §2](02-architecture-and-state.md); zero-size rule; typed rejections.

**Acceptance:** route fixtures cover every class boundary, the page path,
zero size, and both rejections; total function; no allocation.  
**Failure/blocker:** a needed route beyond the constants is a design
edit — update the design rationale first, not silently.

### Step 2 — slab mechanics

Target: `heap::slab`, contract [03 §3](03-code-contracts-heap.md) (slab
portion).

Work: in-band header init, bitmap set/clear with uniqueness, free-count
accounting, magic validation.

**Acceptance:** slot boundary fixtures (first/last slot, header slots
permanently allocated, class-size interplay with header); bitmap truth
invariant after every op in a mini property loop.  
**Failure/blocker:** header/geometry conflicts (a class that cannot host
the header plus one slot) are constants work — revisit
[02 §2](02-architecture-and-state.md), document, then implement.

### Step 3 — directory and acquisition

Target: `heap::directory`, contracts [03 §1, §3](03-code-contracts-heap.md).

Work: fixed in-Heap directory insert/remove/lookup for W04-acquired backing;
full-block budget accounting; W05-MAP rollback and unmap-before-free lifecycle.

**Acceptance:** directory fixtures (fill to `MAX_HEAP_ENTRIES`, typed
exhaustion, lookup misses); budget enforcement exact; every page in the
directory traces to an `AllocatedFrames` value (joint check with W04 when
it lands).  
**Failure/blocker:** a lookup that cannot decide ownership exactly — stop
and strengthen the directory, never guess.

### Step 4 — checked core API

Target: `heap::core`, contracts
[03 §3–§5](03-code-contracts-heap.md).

Work: alloc/dealloc routes, typed errors, stats, audit.

**Acceptance:** full property loop: random interleavings of alloc/dealloc
across classes and the page path, including invalid handles, double
frees, exhaustion probes — invariants 1–6 of
[02 §5](02-architecture-and-state.md) hold after every operation; all
error variants observed with state-unchanged proof (stats equality).  
**Failure/blocker:** any invariant reachable in a broken state — fix the
operation, not the test.

### Step 5 — `GlobalAlloc` adapter (conditional)

Target: `heap::global`, contract [03 §6](03-code-contracts-heap.md).

Work: first close W05-GLOBAL, then implement its concrete state ownership,
publication, reentrancy guards and non-allocating failure path. A thin wrapper
over shared `&Heap` is not an approved substitute.

**Acceptance:** checked corpus plus pinned-target adapter linking and
publication/reentry tests; null return and separate fatal caller path tested.
**Failure/blocker:** missing design or target integration evidence is recorded
as blocked; checked-API evidence does not imply global adapter readiness.



### Step 6 — boundary review (static arrays, performance, scope)

Target: whole module set.

Work: confirm the [01 §6](01-scope-and-foundations.md) review position in
the implementation record (pre-heap arrays temporary; heap permanent;
directory capacities are containment); confirm zero benchmarking, zero
locks, zero board names; confirm W04's API untouched.

**Acceptance:** review note with zero violations.  
**Failure/blocker:** hits removed or escalated per §1.

### Step 7 — host validation pass and evidence

Target: verification record
`../../verification/p2-w05-dynamic-small-allocation-verification.md`;
implementation record
`../p2-w05-dynamic-small-allocation-record.md`.

Work: run the matrix of [05 §3](05-validation-and-handoff.md)
(W05-DV01–DV09), statuses incl. not-run (adapter if deferred; QEMU; SMP).

**Acceptance:** every row statused with evidence; correctness language
only — no performance claims anywhere.  
**Failure/blocker:** failures recorded as failures with diagnosis.

### Step 8 — closure review

Work: read this design as W06 (are stats sufficient?), W08 (are error and
stress fixtures exportable?), W09 (are determinism invariants checkable
across boots?), P3/P4-via-W10 (is the boundary and backing contract
actionable?); confirm the handoff checklist of
[05 §4](05-validation-and-handoff.md).
