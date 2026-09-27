# P2-W04 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Load the documents named in the [entry README](README.md); inspect the
tracked tree. P0/P1 and W01/W02 exist; W03/W04 do not. Close W04-LAYOUT
before allocator coding and W04-MAP before target integration. These are
P2-owned design deliverables, not missing P1 completion evidence. Host storage
is an exclusive buffer; fixture maps must use the W03 draft/plan/seal path.



Stop and obtain direction instead of guessing when:

- W03's draft/seal contracts differ from what the planner needs — W03
  contract revision (design conflict), not a local workaround;
- an allocation consumer wants address-directed allocation, caching, or
  reclaim — Reserved/Out of Scope ([01 §3](01-scope-and-foundations.md));
  design change required;
- SMP locking or per-CPU behavior is requested — P3 scope (p3-w06); stop;
- host tests would need real physical memory or arch code — the design
  forbids that dependency ([01 §7](01-scope-and-foundations.md));
  restructure the test;
- a fix would make an error path stateful ("just mark it freed anyway") —
  violates the stateless-failure invariant ([02 §7](02-architecture-and-state.md));
  stop.

## 2. Ordered implementation steps

### Step 1 — frame-state table

Target: `pagealloc::table`, contract
[03 §1](03-code-contracts-pagealloc.md).

Work: locally indexed frame states and allocation identity over exclusive
storage; head/order/continuation validation; audit counts.

**Acceptance:** boundary fixtures (managed-domain edges, holes, head/tail states,
storage-length checks) pass; no raw index can bypass the table (review
check).  
**Failure/blocker:** a needed P0 primitive missing → upstream defect path
([01 §2](01-scope-and-foundations.md) A3); do not improvise.

### Step 2 — planner

Target: `pagealloc::plan`, contract [03 §2](03-code-contracts-pagealloc.md).

Work: metadata sizing ([01 §6](01-scope-and-foundations.md)) and
smallest-host placement over draft spans.

**Acceptance:** planner output on multi-span fixtures is exactly the §6
rule; plan ranges are inside allocatable spans; joint test with W03's
seal accepts the plan (with W03 landed) or the documented fixture path
(before).  
**Failure/blocker:** planner wanting information the draft map does not
expose → W03 contract revision; stop.

### Step 3 — buddy mechanics

Target: `pagealloc::buddy`, contract [03 §4](03-code-contracts-pagealloc.md).

Work: free-list nodes in metadata storage; find/split/insert/coalesce with
the fixed edit order.

**Acceptance:** randomized allocate/free sequences (host, injected spans)
maintain I2/I3 after every operation; coalescing stops at region
boundaries; worst-case fragmentation cannot exhaust nodes after Ready.\
**Failure/blocker:** any case where an interrupted operation leaves table
and lists inconsistent — restructure the edit order; do not add recovery
ad hoc.

### Step 4 — API operations

Target: `pagealloc::api`, contracts
[03 §3, §5, §6](03-code-contracts-pagealloc.md).

Work: init with audit; allocate; allocate_contiguous; free_contiguous.

**Acceptance:** init audits catch each seeded mismatch class
(`SealMismatch`, `DomainMismatch`, `ConservationBroken`); property tests:
random valid sequences keep the hard gate and conservation; each error
input yields its typed error with state unchanged (before/after stats
equal, plus full tag/list-state equality).\
**Failure/blocker:** any input that can steer allocation toward a
protected frame is a critical breach — stop and fix the domain
construction, never the test.

### Step 5 — accounting

Target: `pagealloc::stats`, contract [03 §7](03-code-contracts-pagealloc.md).

Work: `AllocationStats` from the table with fast-copy cross-check.

**Acceptance:** conservation exact after every operation in the property
loop; divergence injection triggers the invariant stop in a test harness.  
**Failure/blocker:** n/a beyond §1 rules.

### Step 6 — boundary and determinism review

Target: whole module set.

Work: confirm no locks/atomics, no board names, no arch code, no writes
to managed frames, fixed region iteration order; confirm the
original-allocation identity checks ([02 §5](02-architecture-and-state.md))
reject wrong order, partial/combined blocks and foreign allocator handles.

**Acceptance:** review note with zero violations.  
**Failure/blocker:** hits removed or escalated per §1.

### Step 7 — host validation pass and evidence

Target: verification record
`../../verification/p2-w04-physical-page-allocation-verification.md`;
implementation record
`../p2-w04-physical-page-allocation-record.md`.

Work: run the matrix of [05 §3](05-validation-and-handoff.md)
(W04-DV01–DV10), statuses incl. not-run (QEMU, real-memory init, SMP).

**Acceptance:** every row statused with evidence; no completion claim
beyond what ran.  
**Failure/blocker:** failures recorded as failures with diagnosis.

### Step 8 — closure review

Work: read this design as W05 (can the heap back slabs, handle
exhaustion, and release?), W06 (are stats sufficient and live?), W09 (are
totals comparable across repeated boots?), P3-via-W10 (is the
concurrency boundary stated enough to design locks against?), P4-via-W10
(can Guest RAM needs be met or are the limits documented?); confirm the
handoff checklist of [05 §4](05-validation-and-handoff.md).
