# P6-W13 Workflow, Exit Review, and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W13 detailed design](README.md).  
**Companions:**
[01-telemetry-and-evidence-map.md](01-telemetry-and-evidence-map.md)
(evidence authority; §6 record schema).

## 1. Preconditions and failure boundary

Before any step, load the parent README's document set and inspect the tree
(`git ls-files`; no P6 implementation or verification records exist yet).
Every W01–W12 deliverable is an assumed contract until its implementation
and verification records exist; P0 governance
([P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md),
[P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md),
[P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md),
[P0-W20](../../../p0/plans/p0-w20-ci-baseline.md)) is likewise an assumed
contract.

Stop and record a blocker (never manufacture evidence, never implement a
missing mechanism) when:

- a P6 package lacks its implementation or verification record — the gap is
  named in the completeness table; W13 composes, it does not backfill;
- the upstream regression inventory
  ([P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md)) is absent —
  P6-V23 is blocked per
  [01](01-telemetry-and-evidence-map.md) §4;
- a telemetry emission point has no owner — coverage-gap rule of
  [01](01-telemetry-and-evidence-map.md) §1.4;
- a documentation deliverable would have to state a fact without evidence —
  it stays a limitation row instead;
- closure appears assertable — it is not, until every exit-criterion row has
  real evidence (§4).

## 2. P6-DOC ownership map

| Deliverable (task book §7) | Owner | Location | W13's role |
|---|---|---|---|
| P6-DOC-01 — GICv3 Bring-up Record | W01/W02 records | locations fixed by the W01/W02 designs | cite and link |
| P6-DOC-02 — Interrupt Semantics v0 | W10 record | `../p6-interrupt-semantics-v0.md` (W10 design's fixed path) | cite and link |
| P6-DOC-03 — Generic Timer Semantics v0 | W05/W06 records | locations fixed by the W05/W06 designs | cite and link |
| P6-DOC-04 — Validation Matrix (P6-V01–V28) | **W13** | `../p6-validation-matrix.md` (created when factual content exists) | compose per [01 §6](01-telemetry-and-evidence-map.md) |
| P6-DOC-05 — P6 Performance Baseline | **W13** | `../p6-performance-baseline.md` (created when factual content exists) | method + collected data per [01 §2](01-telemetry-and-evidence-map.md) |

Rule: one fact, one location. A deliverable owned elsewhere is never
restated in a W13 record — only cited with its run status and proof
boundary. The two W13-owned locations are stage-local design freedom owned
by this design (parent README decision 4); the deliverables themselves are
task-book-owned.

## 3. Ordered implementation steps

### Step 1 — completeness inspection

Target: implementation record
(`../p6-w13-telemetry-regression-handoff-record.md`, created in this step).

Work: locate every P6 package's implementation and verification records and
the P0 governance artifacts; verdict per package: complete / partial /
absent, with evidence locations. Record the unsafe-inventory delta and
dependency reports as stated by the owning records.

**Acceptance:** a complete P6-wide inventory exists with per-package
verdicts.  
**Failure/blocker:** absent packages are named gaps (§1); they route to the
consumer review's unresolved items, never to silent omission.

### Step 2 — telemetry coverage verification

Target: W13 record; gap list.

Work: apply [01 §1](01-telemetry-and-evidence-map.md) to the implemented
events; demonstrate the §1.3 correlation chains from cited runs; apply the
§1.4 gap rule to any missing emission point.

**Acceptance:** every P6-V24 data class either has a demonstrated chain or a
named, owned gap.  
**Failure/blocker:** an unowned gap blocks the P6-V24 row (blocked status),
with the gap routed to the owning package.

### Step 3 — latency baseline collection

Target: `../p6-performance-baseline.md` (created when data exists).

Work: execute the [01 §2](01-telemetry-and-evidence-map.md) method over the
declared W11 scenario repetitions; populate the baseline tables with
environment, repetitions, and clock-domain caveats; no KPI language.

**Acceptance:** every number carries environment, repetition count, method
reference, and caveat; nothing reports a threshold or comparison.  
**Failure/blocker:** missing Host or Guest timestamps block the affected
delta rows; they are recorded as not-collected with the reason.

### Step 4 — regression and QEMU integration/repeat evidence

Target: W13 verification record.

Work: run the upstream regression composition
([01 §4](01-telemetry-and-evidence-map.md)) and assemble the P6-V26 set
([01 §3](01-telemetry-and-evidence-map.md)) with one declared repeat;
record determinacy and drift investigations.

**Acceptance:** determinate/indeterminate status per inventory item; repeat
selection declared before running; drift items recorded as investigations.  
**Failure/blocker:** absent upstream inventory blocks P6-V23 per §1.

### Step 5 — documentation reconciliation

Target: `../p6-validation-matrix.md` and the linked records.

Work: reconcile implemented facts, limitations, capability status, unsafe
delta, and validation results into the P6-DOC map (§2); populate P6-DOC-04
rows from the owning records; verify every factual statement carries an
evidence link or limitation.

**Acceptance:** all five P6-DOC deliverables located, owned, and factual;
no unevidenced claim.  
**Failure/blocker:** a fact without evidence becomes a limitation row; a
deliverable whose owner has no record stays "not yet present" — named, not
improvised.

### Step 6 — exit review and consumer handoff

Target: W13 verification record; implementation record.

Work: run the §4 exit review and produce the §5 consumer matrix; run the
§6 handoff checklist. Stage completion is not claimed by W13 artifacts under
any circumstance; the exit table exists to keep missing evidence visible.

## 4. P6 exit-criteria review table (P6-V27 review basis)

The review fills this table from cited evidence; every row is
evidenced / partial / missing — never assumed.

| Exit criterion (task book §7) | Evidence source rows |
|---|---|
| 1. Host GIC initialization stable on QEMU virt; independent per-pCPU readiness; SGI + PPI/SPI routing evidenced | P6-V02–P6-V06 rows (W02/W04 records) |
| 2. Per-pCPU EL2 timer events and monotonic-time behavior evidenced | P6-V07, P6-V08 rows (W05 record) |
| 3. Guest virtual timers and authorized vIRQs complete documented paths across exits, masking, deferral, LR pressure, maintenance — no loss, no cross-vCPU delivery | P6-V09–P6-V19 rows (W06–W11 records) |
| 4. Invalid Guest requests, unknown/spurious IRQs, storm cases do not compromise Host or other-VM state | P6-V20–P6-V22 rows (W12 record) |
| 5. Telemetry, latency baseline, upstream regression, QEMU evidence, factual documents, unsafe delta, limitations recorded | P6-V23–P6-V27 rows (this package) |

Consumer review (P6-V28) then asks, per consumer: can you identify proven
P6 behavior, exclusions, evidence locations, and unresolved issues without
inferring a scheduler or machine ABI? A "no" on any consumer blocks the row.

## 5. P7/P8 consumer matrix (P6-V28)

| Consumer package | Consumes (P6 deliverable by owner) | Boundary the consumer must carry |
|---|---|---|
| [P7-W04](../../../p7/plans/README.md) (timer preemption) | W05 per-pCPU EL2 deadline timer evidence | no timer-redesign or time-slice value implied |
| P7-W06 (block/wakeup) | W06 deferred-expiry; W07/W08 event delivery; W09 maintenance behavior | run-state, block, wake semantics are P7-owned |
| P7-W02/W05 (lifecycle, M:N) | W10 semantics record; W08 save/restore across entry/exit | no scheduler policy or affinity claim from P6 |
| P7-W09 (accounting/trace) | W13 coverage map; P6-DOC-05 method and data | trace namespace stability is P0-W13's, not P6's |
| P7-W10/W12 (Guest suite, QEMU regression) | W11 maintained asset; W13 repeat composition | asset extends under P7's own designs; no layout freeze |
| P7-W11 (stress) | W12 storm limits and isolation boundary statements | smoke limits are not DoS guarantees |
| P7-W13 (performance baseline) | P6-DOC-05 with method and caveats | no KPI transfer; QEMU-only environment |
| P7-W14 (handoff) | this package's factual handoff and unresolved items | closure language stays factual |
| P8-W01 (entry reconciliation) | the P6-DOC set and unresolved items | machine ABI is P8-owned |
| P8-W07 (Linux vGICv3) | W08/W09 virtualization-interface and maintenance evidence, via their records | vGIC MMIO model is P8-owned; QEMU ≠ hardware |
| P8-W08 (Linux timer) | W06 Guest timer semantics record (P6-DOC-03) | no frozen timer ABI; P8 owns Guest DTB/PSCI |
| P8-W16/W18 (Linux regression, security isolation) | W11 asset reuse; W12 boundary statements | passthrough/DoS claims remain out of P6 scope |

(P7/P8 package names are cited as named consumers only; their content is
owned by their own task books and plans.)

## 6. Handoff checklist

Before handing W13 to a reviewer, provide:

- the exact changed-file list (W13-owned records only, plus citations);
- the §3-step-1 completeness inventory with per-package verdicts;
- W13-DV01–DV08 status (passed/failed/blocked/not-run) with evidence links;
- the P6-DOC map with each deliverable's location and ownership, including
  any "not yet present" row named honestly;
- the coverage-gap list, the blocked P6-V23 status if applicable, and the
  unresolved-investigations list for the consumer review;
- confirmation that no mechanism, telemetry event, CI workflow, QEMU
  script, machine-ABI statement, or completion claim was added;
- the §4 exit table and §5 consumer matrix, each row carrying its proof
  boundary — with the standing statement that QEMU success does not prove
  real-hardware behavior and that stage closure remains unclaimed until all
  exit evidence exists.
