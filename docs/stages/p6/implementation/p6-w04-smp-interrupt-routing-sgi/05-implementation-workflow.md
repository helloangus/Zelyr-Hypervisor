# P6-W04 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Implementation starts only when the entry review finds: W02 readiness
evidence (ledger, initial routing, lock class), W03 dispatch evidence
(classification, registration, counters), the P3 online-set and
notification contracts evidenced, and the pinned GIC specification
revisions from the W01 record.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the P3 notification handoff names an SGI range different from this
  design's partition → recorded conflict; the P3 contract governs
  ([01 §1.3](01-scope-and-foundations.md)); do not renumber locally;
- W02/W03 surfaces differ from the assumed shapes (ledger queries,
  registration, counters, lock class) → stop; reconcile with those
  designs' owners;
- observed SGI1R or IROUTER behavior contradicts the pinned revision →
  Specification Investigation stop; no workarounds, no
  platform-conditional encodings;
- the implementation appears to need scheduler semantics (reschedule,
  wakeup), TLB semantics, message payloads, or Guest-targeted sends →
  scope violation; stop;
- a new crate boundary, dependency, or target change appears necessary →
  owned by their P0 packages; raise, do not improvise.

## 2. Ordered implementation steps

### Step 1 — entry review and record skeleton

Target: `../p6-w04-smp-interrupt-routing-sgi-record.md` (created in this
step).

Work: record the entry-review result; the P3 partition-fact check
(explicitly resolving [README decision 4](README.md) against the actual P3
handoff); the consumed W02/W03 facts; the pinned revisions. State that
nothing is implemented.

**Acceptance:** record names every consumed contract and resolves the
partition question with a citation.  
**Failure/blocker:** partition conflict → recorded per §1; the affected
row stops until resolved.

### Step 2 — target and decomposition layer

Target: the `sgi-target` module
([03](03-code-contracts-sgi-send.md) §1).

Work: implement `SgiTarget` constructors over the eligibility predicate,
and the deterministic decomposition; unit tests with synthetic ledger
views and affinity-boundary cases.

**Acceptance:** every form validates or rejects with the named error;
decomposition covers exactly the validated set, deterministically.  
**Failure/blocker:** eligibility needs a ledger query W02 does not
provide → stop; reconcile with the W02 contract owner.

### Step 3 — send path and accounting

Target: the `sgi-send` and `sgi-accounting` modules
([03](03-code-contracts-sgi-send.md) §2, §4).

Work: implement `send_sgi` with partition checking and per-sender
counters; implement the accounting view and drift report; register the
generic-event and validation rows' consumers per the partition.

**Acceptance:** unit tests show: validation-before-emission (zero writes
on rejection), write ordering per decomposition, partition rejection,
drift computation on synthetic sequences.  
**Failure/blocker:** receipt counters unavailable from W03 stats → stop;
reconcile with the W03 contract owner.

### Step 4 — SPI route-change protocol

Target: the `spi-routing` module
([04](04-code-contracts-spi-routing.md)).

Work: implement the route table, `change_spi_route` with the
locked check-check-write sequence and 64-bit accessor, `query_spi_route`,
and the rejection paths; sequence tests on the fake backend for enabled,
pending, active, and racing-change cases.

**Acceptance:** every rejection leaves state unchanged and observed;
accepted changes produce the ordered write sequence; determinacy
statement of [04](04-code-contracts-spi-routing.md) §2 holds in tests.  
**Failure/blocker:** the W02 lock object or accessor variant is missing →
stop; the change belongs to the W02 surface owner.

### Step 5 — telemetry wiring

Target: events of [02](02-architecture-and-state.md) §5.

Work: register the four event kinds; sampled send events; route-change
events.

**Acceptance:** events compile under telemetry gates; payloads carry IDs
and affinities, never platform names.  
**Failure/blocker:** P0 telemetry gap → established diagnostic-log
fallback recorded for W13.

### Step 6 — QEMU execution and evidence

Target: `../../verification/p6-w04-smp-interrupt-routing-sgi-verification.md`.

Work: run the P6-V04/V05/V06 scenarios of
[06](06-validation-and-handoff.md) §2 in the declared QEMU environment:
CPU0→CPU1 SGI with receipt attribution; CPU1→CPU0; multi-target and
declared IRM form; accounting convergence; SPI re-route to a second pCPU
and back with enable/disable interplay; rejection paths; drift
observation on the failed-target case if a local failure can be staged
within declared limits. Record run/not-run per row with the QEMU proof
boundary.

**Acceptance:** target-attributed delivery and determinate completion
observed for every run row; rejections observed as named errors.  
**Failure/blocker:** a failure is recorded as failed with diagnosis; no
loosening of eligibility, checks, or accounting to pass.

### Step 7 — non-policy review

Work: run the W04-DV08 review ([06](06-validation-and-handoff.md) §2):
confirm the non-policy commitments of
[01 §5](01-scope-and-foundations.md) hold in the implementation — no
scheduler-named symbols, no payload, no Guest path, no balancing.

**Acceptance:** review recorded with search results and code inspection
notes.  
**Failure/blocker:** a commitment violation is a design conflict to fix
or escalate, not a review note to carry forward.

### Step 8 — closure review

Work: run the review matrix, confirm the handoff checklist, verify
against the plan work sequence and task-book rows P6-V04–V06. Completion
is claimed only in the verification record, only for what was run.

## 3. Implementer error-handling rules

- Validation strictly precedes emission; a rejected send writes nothing.
- No lock is held across an SGI1R write (send is lock-free); the
  distributor lock is held only across the route-change sequence.
- Partition rows are constants; unassigned IDs are unconstructible send
  arguments (type-level where feasible, checked at minimum).
- All failure outcomes are named; no silent partial delivery exists.
