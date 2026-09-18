# P6-W13 Telemetry and Evidence Map

**Status:** Proposed detailed design; implementation and validation not
claimed.  
**Parent:** [P6-W13 detailed design](README.md).  
**Audience:** load for all evidence work. Telemetry mechanisms are owned by
[P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md) and
[P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md);
QEMU evidence conventions by
[P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md); CI
by [P0-W20](../../../p0/plans/p0-w20-ci-baseline.md). This file composes
boundaries over those contracts and cites them; it never restates them.

## 1. Telemetry coverage map (P6-V24)

### 1.1 Required data classes

| Data class (P6-V24 wording) | Source events (logical; owning package) | Correlation keys |
|---|---|---|
| Host IRQ class and per-pCPU counts | IRQ classified/completed (W03); SGI sent/received (W04) | pCPU id; INTID class; run id |
| Timer behavior | EL2 deadline set/fire/cancel (W05); Guest timer program/expiry/deferred (W06) | pCPU id; vCPU id; run id |
| vIRQ lifecycle | injected/pending/presented (W07/W08); completed (W08/W09) | VM id; vCPU id; vIRQ id; run id |
| Maintenance data | the six W09 events (entry, conditions, reconciled, refill, unexpected, deferred) | pCPU id; vCPU id; run id |

### 1.2 Correlation rules

- Every event carries, at minimum, the pCPU identity and the run identifier;
  events on the vCPU side additionally carry VM id, vCPU id, and vIRQ id as
  applicable to their class.
- A "run" is the declared environment execution instance recorded by the
  harnesses (W11/W12) and the P0-W09 conventions; correlation is by run id
  plus the keys above — never by timestamp proximity alone.
- Guest records (W11 channel) correlate to Host events through the run id
  and the per-scenario host-correlation field of the W11 record schema.

### 1.3 Availability requirement

P6-V24 asks that the classes be "correlated and available": the W13 review
must demonstrate one continuous chain per class — Host event, vCPU-side
event, Guest record (where the class has one) — for at least the exercise
runs of W09–W12, using cited evidence only.

### 1.4 Coverage-gap rule

If a required emission point has no owning W01–W12 design (or its owning
design deferred it), the gap is recorded in the W13 record with the owning
package named; W13 does not implement the emission. The gap either resolves
through the owning package's change or lands in the unresolved-items list of
the consumer review.

## 2. Latency measurement method (P6-V25)

### 2.1 Timestamp chain (physical IRQ → Guest handler)

| Mark | Where captured | Owning evidence |
|---|---|---|
| T0 — physical event receipt | EL2 IRQ entry path (W03 contract), per-pCPU monotonic source (W05) | W03/W05 telemetry |
| T1 — injection decision (pending enqueue) | W07 injection contract | W07 telemetry |
| T2 — presentation at Guest entry | W08 LR load | W08 telemetry |
| T3 — Guest handler mark | W11 Guest result record timestamp | W11 records |

### 2.2 Method rules

- Report per-scenario-class deltas T1−T0, T2−T1, T3−T2, and T3−T0 over the
  declared repetitions of the W11 scenarios; timer latency uses the W06
  chain (programmed deadline → Guest expiry mark).
- Every reported number carries: environment declaration (QEMU
  configuration class, pCPU count, build ids), repetition count, and
  collection date. No comparison to other hypervisors, no extrapolation to
  hardware, no KPI threshold.
- Guest-side timestamps use the Guest's architectural counter view;
  cross-source deltas are stated with their clock-domain caveat in the
  record (T0–T2 Host domain, T3 Guest domain) — the method must say which
  deltas are same-domain and which are not.

## 3. QEMU integration and repeat evidence (P6-V26)

- **Composition:** the P6-V26 evidence set is the union of: W09-DV01–DV05
  (maintenance progression), W10-DV01–DV05 (semantics rows), W11-DV02–DV05
  (Guest scenarios), W12-DV01–DV06 (robustness cases) — cited from their
  verification records, plus one declared repeat of the positive/negative/
  recovery selection.
- **Positive / negative / recovery coverage:** at minimum one positive path
  (VG-IRQ-01 class), one negative (an FI-A rejection), and one recovery
  (post-storm invariant spot-check from FI-D) must appear in the repeat set.
- **Environment statement:** every cited run carries the environment class
  and configuration reference per the P0-W09 conventions; the W13 record
  names the declared QEMU environment and its limits.
- **Repeat semantics:** a repeat re-executes a declared selection, not the
  full matrices; drift between original and repeat is recorded as an
  investigation item, never averaged away.
- **Proof boundary:** P6-V26 success means determinate outcomes in the
  declared QEMU environment. It does not prove real-hardware behavior, an
  untested machine ABI, or scheduler correctness (P7).

## 4. P0–P5 regression composition (P6-V23)

- **Authoritative set:** the upstream regression inventory published by the
  P5-W10 closeout
  ([plan](../../../p5/plans/p5-w10-closeout-p6-handoff.md)), which itself
  composes the P2–P5 integration/regression packages
  ([P2-W09](../../../p2/plans/p2-w09-qemu-integration-regression.md),
  [P3-W13](../../../p3/plans/p3-w13-qemu-smp-regression.md),
  [P4-W08](../../../p4/plans/p4-w08-qemu-integration-regression.md)) and
  the P0 gate suites.
- **Execution:** under the P0-W20 CI wiring and P0-W09 QEMU conventions,
  after P6 changes land; results recorded as determinate / indeterminate
  per inventory item.
- **Failure boundary:** if the inventory is absent, unevidenced, or
  internally contradictory, P6-V23 is **blocked** and the block is recorded
  for the consumer review. W13 must not reconstruct an inventory from plan
  text — that would manufacture the very evidence the stage forbids.
- **Proof boundary:** determinacy of the upstream set after P6 changes; not
  upstream correctness, not hardware behavior.

## 5. Unsafe and dependency delta boundaries

- The P6 unsafe-inventory delta is reconciled from the implementation
  records of W01–W12 under the P0-W10 governance; W13 verifies the delta is
  stated, audited, and linked — it does not re-audit code.
- Dependency changes reported by P6 packages are listed with their owning
  records; W13 adds none.

## 6. Record schema for P6-DOC-04 and P6-DOC-05

Both W13-owned records carry: status header (no completion claim), scope,
source-of-truth links, and per-item tables with evidence link, run status,
environment, and proof boundary. P6-DOC-05 additionally carries the §2
method statement and the collected baseline tables; P6-DOC-04 the P6-V01–
P6-V28 rows composed from the owning records. Neither record may contain a
fact without an evidence link or an explicit limitation statement.
