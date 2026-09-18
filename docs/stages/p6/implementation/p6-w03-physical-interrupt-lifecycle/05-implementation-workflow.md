# P6-W03 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Implementation starts only when the entry review finds: the W02 readiness
evidence (P6-V02/V03 rows recorded), the P1 IRQ entry contract present
(single dispatch target, stable entry context), the P3 per-CPU storage and
synchronization contracts present, and the pinned GIC specification
revisions from the W01 record.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 posture differs from what this design consumes (EOImode,
  readiness ledger shape, access surface) → stop; record against the W02
  design; do not re-configure around it;
- the P1 entry path cannot call a registered dispatch target without new
  W03-owned assembly → the entry path belongs to P1; raise, do not add a
  second vector path;
- observed hardware behavior contradicts the pinned revision's band or
  EOI rules → Specification Investigation stop; record; do not add
  hardware-conditional workarounds;
- a consumer subsystem (W05/W08) needs more than the minimal callback →
  that is a design change to this package's contract, not a local
  extension; raise it;
- the implementation appears to need Guest-visible behavior, SGI policy,
  or scheduler hooks → scope violation; stop.

## 2. Ordered implementation steps

### Step 1 — entry review and record skeleton

Target: `../p6-w03-physical-interrupt-lifecycle-record.md` (created in
this step).

Work: record the entry-review result, the W02 facts consumed (supported
range, posture), the pinned revisions, and the fixed constants chosen
under this design's authority (DISPATCH_BOUND initial value, rate-limit
window/threshold) with their rationale. State explicitly that nothing is
implemented.

**Acceptance:** record names every consumed fact and constant with its
authority.  
**Failure/blocker:** missing entry evidence is a recorded stage block.

### Step 2 — typed identification layer

Target: the `irq-identify` module
([03](03-code-contracts-identification.md)).

Work: implement the ID family, `SupportedRange`, band transcription from
the pinned revision, and `classify_acknowledge()`; exhaustive unit tests
over the encoded value space.

**Acceptance:** every possible IAR value classifies to exactly one
outcome; boundary-value tests pass; no naked u32 in any public surface.  
**Failure/blocker:** band edges ambiguous in the pinned revision →
Specification Investigation stop (§1).

### Step 3 — registry and enable/disable mechanics

Target: the `irq-registry` module
([04](04-code-contracts-dispatch-completion.md) §3).

Work: implement slots, registration (with ledger checks), and the
per-class enable/disable mechanics through the W02 access surface,
including the distributor-lock path for SPIs.

**Acceptance:** unit tests show double-registration rejection,
registration-before-enable publication ordering, per-class mechanics and
ownership errors.  
**Failure/blocker:** ledger query shape mismatch with W02 → stop and
reconcile with the W02 contract owner.

### Step 4 — dispatch loop and stats

Target: the `irq-dispatch` and `irq-stats` modules
([04](04-code-contracts-dispatch-completion.md) §1, §6).

Work: implement `handle_irq_entry` with the five named outcomes, single
EOI site, bound, and counters; synthetic-sequence tests on the fake
backend (ordinary, repeated, simultaneous, spurious, unknown,
consumerless, bound-exit).

**Acceptance:** loop invariants INV-A..INV-E hold in all synthetic
sequences; counter accounting exact.  
**Failure/blocker:** an invariant cannot hold without new state → stop;
the design does not authorize new lifecycle state.

### Step 5 — P1/P3 integration

Target: the dispatch-target wiring in the P1 IRQ vector path and per-CPU
stats records in P3 storage.

Work: install `handle_irq_entry` as the P1 IRQ dispatch target; place
stats in per-CPU storage. Integration changes to P1/P3 files stay within
their declared extension points.

**Acceptance:** single entry path; no new vector assembly; per-CPU
placement verified by the P3 isolation tests extended for the stats
records.  
**Failure/blocker:** integration needs more than the declared extension
point → stop; the change belongs to the P1/P3 contract owner.

### Step 6 — telemetry wiring

Target: events of [02](02-architecture-and-state.md) §7.

Work: register the four event kinds; implement rate limiting.

**Acceptance:** events compile under telemetry gates; high-rate sequences
produce aggregated, bounded output in tests.  
**Failure/blocker:** P0 telemetry gap → established diagnostic-log
fallback recorded for W13.

### Step 7 — QEMU execution and evidence

Target: `../../verification/p6-w03-physical-interrupt-lifecycle-verification.md`.

Work: run the P6-V20/V22-oriented scenarios of
[06](06-validation-and-handoff.md) §2: ordinary SGI/PPI dispatch through a
registered test consumer; consumerless and unknown outcomes observed
safely; repeated/simultaneous delivery; bound exit under a synthetic
high-rate SGI storm (declared limits); counters and rate limiting
observed. Record run/not-run per row with the QEMU proof boundary.

**Acceptance:** all run rows show the documented safe outcomes; no row
claims production DoS resistance.  
**Failure/blocker:** a failure is recorded as failed with diagnosis; no
loosening of bounds, masks, or outcome rules to pass.

### Step 8 — closure review

Work: run the review matrix, confirm the handoff checklist, verify
against the plan work sequence and task-book rows P6-V20/V22. Completion
is claimed only in the verification record, only for what was run.

## 3. Implementer error-handling rules

- One EOI site, one acknowledge site, one classification path — no
  fast-path duplicates.
- No `unwrap` on ID construction in dispatch paths; classification
  guarantees validity ([03](03-code-contracts-identification.md) §3).
- Consumer invocation is the only indirect call; the callback pointer
  comes from a published immutable slot.
- Every loop iteration is bounded and every outcome is counted.
