# P7-W05 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, verify the Coding Guidelines preflight and the W01 register
rows W05 consumes (P7-IN-01/03/04/05). The sibling designs W02–W04 are the
mechanism authorities; their contracts are assumed stable at the named
seams. A `blocked` W01 row whose failure boundary names W05 (notably
P7-IN-04 cross-CPU transport for multi-pCPU scenarios) stops the affected
scenario scope, not necessarily the whole package (single-pCPU rotation
can proceed; the matrix cannot).

Stop and obtain direction instead of guessing when:

- a W02/W03/W04 seam differs at integration — renegotiate at design level
  via the W01 §5 procedure; never adapt W05 by bypassing a gate, engine,
  or ownership rule;
- the cross-CPU transport cannot carry a reschedule request (multi-pCPU
  scenarios impossible) — record the blocked scope; do not invent a
  transport in W05;
- making a scenario pass appears to require weights, priorities, or
  requeue-head tricks — those are Reserved; the fix belongs to the
  scenario definition or a design change, not to v0 policy speculation;
- the fairness window constants would have to be tuned per scenario to
  pass — that indicates a mechanism defect (lost enqueue, broken
  eligibility) to be diagnosed, not a constant to per-scenario-tune; or
- any hot-path allocation, IRQ-context queue mutation, or cross-queue
  nesting seems needed — that contradicts the discipline; stop and
  redesign.

## 2. Ordered implementation steps

### Step 1 — prerequisite inspection

Target: the implementation record (created in this step).

Work: read the W01 register rows and the cited P3-W14 plan plus the
W02–W04 contracts; record the assumed seams (gate, switch, triggers,
eligibility, idle hook, transport) in
`../p7-w05-shared-mn-multivm-record.md` with failure boundaries.

**Acceptance:** each seam named with plan path and mismatch handling;
scenario feasibility per current register status noted.  
**Failure/blocker:** a blocked row naming W05 limits scenario scope; the
limitation is recorded, not worked around.

### Step 2 — entity and run queue

Target: `entity` + `runqueue` modules.

Work: implement `SchedulingEntity` and `RunQueue` per
[Contracts 1.1/2.1](03-code-contracts-shared-scheduler.md); capacity wired
to the stage configuration; host unit tests (FIFO order, duplicate
rejection, capacity, remove) and the presence-mark property test.

**Acceptance:** all discipline tests pass; no allocation on hot-path
operations; per-queue lock never nested with lifecycle/ledger locks.  
**Failure/blocker:** a needed feature outside the discipline (e.g.,
priority ordering) is Reserved — stop and escalate.

### Step 3 — policy seam and RoundRobin v0

Target: `policy` module.

Work: implement the seam and `RoundRobin` per Contracts 3.1/3.2; rotation
property test; fairness-window simulation on fake queues.

**Acceptance:** k-entity rotation shows no starvation; requeue reasons
map exactly to the Contract 3.2 match; no weights/RT code exists.  
**Failure/blocker:** a fairness simulation failure indicates a discipline
bug — fix the mechanism; do not tune the policy.

### Step 4 — enqueue rule

Target: `enqueue` module.

Work: implement `choose_enqueue_pcpu` (Contract 3.3) over the W03
eligibility seam and the online set; wire it as the single enqueue entry
for wakeup and requeue paths.

**Acceptance:** rule tests (pinned, affinity subset, all-offline);
exactly one enqueue path exists (review).  
**Failure/blocker:** a second enqueue path appearing in integration is a
design violation — consolidate.

### Step 5 — control loop

Target: `loopctl` module.

Work: implement `run_scheduler_loop` (Contract 4.1) against fakes for
gate/switch/triggers/idle; branch-coverage tests including bounded
rejection retry and the deadline-arm asymmetry passthrough.

**Acceptance:** every loop branch tested; no unbounded retry; idle hook
invoked with the W08 contract shape (blocking semantics asserted by W08
later).  
**Failure/blocker:** needing loop-side policy (e.g., "try harder before
idle") is W08/W05-policy scope — stop.

### Step 6 — target integration and scenario bring-up

Target: real seams; scenario instrumentation.

Work: bind to the real W02 gate/switch, W04 triggers/deadline, W03
eligibility, and the P3 per-CPU attachment; implement the assertion hooks
(Contract 5.1) behind the test/diagnostic feature surface; bring up S1
(single-pCPU rotation) first, then the multi-pCPU scenarios as W08's
transport/idle land.

**Acceptance:** S1 passes on target with clean invariant samples; the
scenario table is executable as defined; per-scenario blockers recorded.  
**Failure/blocker:** a scenario failing for a mechanism reason is a joint
diagnosis with the owning design; scenario constants are never per-run
tuned.

### Step 7 — records, validation, closure

Work: complete the implementation record; run the
[validation matrix](05-validation-and-handoff.md); write evidence to
`../../verification/p7-w05-shared-mn-multivm-verification.md`; confirm the
handoff checklist. Completion is claimed only in the verification record,
only for what ran.

## 3. Evidence rules

- Planned, run, blocked, failed remain distinct; a scenario blocked by a
  W08 dependency is recorded as blocked-with-reason, not skipped silently.
- Host/simulated evidence never substitutes for the P7-V10–V12 target
  rows.
- Expected implementation profile: no new `unsafe`, no new dependencies,
  no public API, no policy framework speculation; deviations reported per
  the repository rules.
