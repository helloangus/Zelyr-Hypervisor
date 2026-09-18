# P6-W13 Telemetry, Regression, and Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Collection of P6's factual observability, latency-baseline method,
QEMU integration/repeat evidence, P0–P5 regression, required documentation
records, and the P7/P8 consumer handoff, per
[P6-W13](../../plans/p6-w13-telemetry-regression-handoff.md).  
**Owner/change context:** P6-W13 composition and handoff; this design owns
the coverage map, evidence boundaries, the P6-DOC-04/P6-DOC-05 records, and
the consumer-review structure — it owns no new mechanism, telemetry event
implementation, or machine ABI.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W13. It is a composition
package: it defines what evidence P6 must assemble, from where, under which
upstream contracts, with which correlation and proof boundaries — and how
the P7/P8 handoff distinguishes proven mechanisms from reserved or excluded
work. It produces **no results and no completion claim**: stage closure
stays unclaimed until every exit criterion has real evidence.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-telemetry-and-evidence-map.md](01-telemetry-and-evidence-map.md) — the
  telemetry coverage map under the P0 contracts, correlation keys, the
  latency-measurement method, the QEMU integration/repeat evidence
  boundaries, and the P0–P5 regression composition. Load for evidence work.
- [02-workflow-and-handoff.md](02-workflow-and-handoff.md) — ordered steps,
  the P6-DOC ownership map, the validation matrix (P6-V23–P6-V28), the
  stage-exit review, and the P7/P8 consumer-handoff structure by package ID.

Before editing, follow the Coding Guidelines preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P6 task
book](../../task-book-v0.1.md), and the [P6-W13
plan](../../plans/p6-w13-telemetry-regression-handoff.md). This document is
proposed design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W13 plan → this
design → Coding Guidelines. In particular:

- ADR-048 makes structured telemetry a first-class capability; ADR-049
  requires layered validation; the task book §1 Required clause demands
  telemetry, QEMU integration/regression, implementation records, validation
  evidence, performance baseline, and P7/P8 handoff "only after actual work
  has occurred."
- Telemetry mechanisms and event namespaces are owned by the P0 contracts —
  [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)
  (logging/diagnostics) and
  [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md)
  (trace event namespace). This design cites and composes them; it never
  restates or redefines them, and W13 implements no telemetry mechanism.
- The plan excludes: implementing unplanned mechanisms, asserting stage
  closure without evidence, defining a machine ABI or scheduler policy, raw
  log dumps as a substitute for records, and real-hardware support claims.

Classification:

| Class | Items |
|---|---|
| **Required** | P6 telemetry coverage map (P6-V24 data classes) with correlation keys; the latency-measurement method (P6-V25) without a KPI claim; QEMU integration/repeat evidence boundaries (P6-V26); P0–P5 regression composition (P6-V23); P6-DOC-04 validation-matrix record and P6-DOC-05 performance-baseline record; the P6-V27 documentation reconciliation; the P6-V28 P7/P8 consumer review with proven/reserved/excluded classification. |
| **Reserved** | Exact metric retention or aggregation formats (owned by P0 telemetry contracts as they evolve); additional baseline environments beyond the declared QEMU reference. |
| **Out of Scope** | Implementing unplanned mechanisms or telemetry event code (route gaps back to owning packages); machine ABI or vGIC contracts (P8); scheduler policy or run-state semantics (P7); wall-clock/RTC and advanced time scaling (task book §1); real-hardware validation; stage-closure assertion. |

## Requirement → design-location → acceptance mapping

| Plan requirement (P6-W13) | Detailed-design location | Acceptance |
|---|---|---|
| Inspect all P6 package records, P0 telemetry/QEMU governance, available evidence | [workflow](02-workflow-and-handoff.md) step 1 | completeness verdict per package in the implementation record |
| Structured telemetry, correlation, latency-baseline, integration, repeat, upstream-regression evidence boundaries | [map](01-telemetry-and-evidence-map.md) §1–§5 | boundaries defined with owners and failure cases |
| Reconcile implemented facts, limitations, unsafe delta, capability status, validation results into prescribed layers | [workflow](02-workflow-and-handoff.md) step 4; [§3 record map](01-telemetry-and-evidence-map.md) absent → [§2 of workflow] | every P6-DOC deliverable located, owned, and factual |
| Review QEMU/environment limits, evidence completeness, unresolved investigations, every P6 exit criterion | [workflow](02-workflow-and-handoff.md) step 5; §3 exit review | exit-criteria table complete with evidence links or named gaps |
| P7/P8 consumer review distinguishing proven mechanisms from reserved/excluded work | [workflow](02-workflow-and-handoff.md) §5 consumer matrix | per-consumer mapping with proof boundaries |
| Factual handoff; closure unclaimed until exit evidence exists | [workflow](02-workflow-and-handoff.md) step 6; §6 handoff checklist | no completion claim anywhere |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p6-implementation-designs`):
P0 documentation scaffold — no telemetry code, no QEMU automation, no P6
implementation or verification records; `docs/stages/p6/verification/`
contains only a `.gitkeep`. P0–P5 are planned but unimplemented, so the
telemetry contracts, the QEMU evidence conventions, the CI wiring, and the
P0–P5 regression inventory are **assumed contracts** from their plans. The
P6-W01–W12 designs are being written in parallel and are referenced by slug
and ID only. Each ledger row states the missing foundation the plan outcome
necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Structured IRQ/timer/vIRQ/maintenance telemetry, correlated and available (P6-V24) | No telemetry mechanism exists | P0-W12/P0-W13 contracts implemented; W01–W12 events emitted per their designs; the coverage map of [01](01-telemetry-and-evidence-map.md) §1 | Correlation needs both the mechanism (P0) and the emitters (W01–W12); W13 composes and verifies coverage only | P0 contracts; W01–W12 designs; W13 map | P6-V24 review |
| Latency baseline recorded with environment and method, no unearned KPI (P6-V25) | No measurement exists | The measurement method of [01](01-telemetry-and-evidence-map.md) §2 over implemented W03/W06–W09/W11 paths | A baseline is a method plus recorded data; without the method definition the numbers are uninterpretable | W13 (method); W11 (Guest timestamps); W03/W08 (Host timestamps) | P6-V25 evidence |
| QEMU integration and repeat evidence, positive/negative/recovery (P6-V26) | No QEMU evidence exists | P0-W09 QEMU evidence conventions; W09–W12 scenario/case runs | Integration evidence composes the packages' runs with declared environments and repeats | P0-W09; W09–W12 verification records | P6-V26 evidence |
| P0–P5 regression determinate after P6 changes (P6-V23) | No upstream regression inventory exists | The authoritative upstream regression inventory from the P5-W10 handoff; P0-W20 CI wiring | A regression run needs a fixed upstream set; P5-W10 is its named owner | [P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md); [P0-W20](../../../p0/plans/p0-w20-ci-baseline.md) | P6-V23 evidence |
| P6 documentation in prescribed layers (P6-V27) | No P6 records exist | The P6-DOC ownership map ([workflow](02-workflow-and-handoff.md) §2) and the two W13-owned records | Factual documents must exist before a documentation review can pass | W01–W13 per the map | P6-V27 review |
| P7/P8 consumer review distinguishes proven/reserved/excluded (P6-V28) | No handoff exists | The consumer matrix ([workflow](02-workflow-and-handoff.md) §5) built from W09–W12 handoff statements | Consumers need per-deliverable proof boundaries, not summaries | W13 (matrix); W09–W12 (inputs) | P6-V28 review |

No row requires implementing an unplanned mechanism: telemetry gaps found
during coverage verification are routed back to the owning package per
[01](01-telemetry-and-evidence-map.md) §1.4, and a missing upstream
inventory blocks P6-V23 honestly rather than being reconstructed here.

## Resolved design decisions and their authority

1. **Telemetry under the P0 contracts, composition only.** All P6 telemetry
   flows through the P0-W12 logging/diagnostic and P0-W13 trace-namespace
   contracts; W13 defines the required P6 coverage classes and correlation
   keys ([01](01-telemetry-and-evidence-map.md) §1) and verifies coverage,
   but owns no event code. A required emission point that no W01–W12 design
   owns is a recorded gap routed to the owning package — never implemented
   by W13. Authority: plan scope ("inspect the P0 telemetry/QEMU
   governance"); ADR-048; task book §1 Out of scope (implementation in task
   book/plans).
2. **Latency method before numbers.** The P6-V25 baseline is defined as a
   method — timestamp chain, correlation keys, environment declaration, and
   repetition semantics ([01](01-telemetry-and-evidence-map.md) §2) — and
   only then populated from real runs. No KPI, no comparison to other
   hypervisors, no real-hardware claim. Authority: P6-V25 wording
   ("recorded with environment and method, without an unearned KPI claim").
3. **Regression composition is upstream-authoritative.** The P0–P5
   regression set is exactly the inventory published by the P5-W10 closeout
   (composed from the P2–P5 integration/regression packages, e.g.
   [P2-W09](../../../p2/plans/p2-w09-qemu-integration-regression.md),
   [P3-W13](../../../p3/plans/p3-w13-qemu-smp-regression.md),
   [P4-W08](../../../p4/plans/p4-w08-qemu-integration-regression.md));
   W13 runs it under the P0-W20/P0-W09 wiring and records determinacy. If
   the inventory is absent or unevidenced, P6-V23 is **blocked** and
   recorded — never reconstructed from memory or plan text. Authority:
   P5-W10 handoff; P6-V23 wording.
4. **P6-DOC ownership split.** P6-DOC-01 is owned by the W01/W02 records;
   P6-DOC-02 by the W10 record
   (`p6-interrupt-semantics-v0.md`); P6-DOC-03 by the W05/W06 records;
   P6-DOC-04 and P6-DOC-05 are owned by W13 at
   `docs/stages/p6/implementation/p6-validation-matrix.md` and
   `docs/stages/p6/implementation/p6-performance-baseline.md` (created only
   when their factual content exists). W13 composes links and never
   duplicates another record's content. Rationale: AGENTS.md layer
   separation; each deliverable has one owner. Stage-local design freedom
   owned here for the two W13-owned locations only. Authority: task book §7.
5. **Evidence classification and non-duplication.** W13's records cite
   W09–W12 verification records and link raw artifacts; they never copy
   results into new prose (one fact, one location). Every cited item carries
   its run status (passed/failed/blocked/not-run) and proof boundary.
   Rationale: plan exclusion of raw log dumps; checklist §4 evidence
   discipline. Authority: plan scope.
6. **Consumer review by package ID.** The P6-V28 review is a per-consumer
   matrix ([workflow](02-workflow-and-handoff.md) §5) naming exactly which
   P7 and P8 packages consume which P6 deliverable, each with its proof
   boundary and exclusions — not a prose summary. Authority: plan step 5;
   task book §7 consumer boundaries.
7. **Closure stays unclaimed.** Every W13 artifact states planned vs actual;
   stage closure is asserted nowhere, and the exit-criteria review
   ([workflow](02-workflow-and-handoff.md) §4) exists precisely to keep
   missing evidence visible. Authority: plan goal; task book §7.

## Work breakdown and loading order

1. Read this README and the Coding Guidelines in full.
2. Load [01-telemetry-and-evidence-map.md](01-telemetry-and-evidence-map.md)
   for all evidence work (coverage, latency, regression, QEMU boundaries).
3. Execute [02-workflow-and-handoff.md](02-workflow-and-handoff.md) in
   order: completeness inspection, coverage verification, latency collection,
   regression run, documentation reconciliation, exit review, consumer
   review.
4. W13's own evidence lands in
   `../../verification/p6-w13-telemetry-regression-handoff-verification.md`
   (created when evidence exists); its decisions and deviations go to
   `../p6-w13-telemetry-regression-handoff-record.md` when implementation
   begins; its records P6-DOC-04/P6-DOC-05 live at the §1 decision-4 paths.
   None of these may claim W13 or stage completion.

## Explicitly excluded interfaces

No public/stable Rust API, ABI, machine ABI, wire format, telemetry event
name, or persistent layout is authorized by W13. W13 creates records and
composition documents, not code: no telemetry event implementation, no
measurement instrumentation in Host or Guest code, no CI workflow content
(P0-W20), no QEMU script (P0-W09 conventions), and no scheduler or vGIC
artifact. If composition appears to require any of these, the need is routed
to the owning package and recorded as a gap.

## Downstream handoff

- **P7** receives, per the consumer matrix
  ([workflow](02-workflow-and-handoff.md) §5), evidenced P6 mechanisms for
  preemption, block/wakeup, lifecycle, observability, stress, regression,
  and performance designs — with run-state, switch, wakeup, and policy
  semantics explicitly P7-owned and no P6 API frozen.
- **P8** receives the same for the virtualization-interface and timer
  designs it owns — with the vGIC MMIO/DTB/PSCI/machine-ABI contracts
  explicitly P8-owned and every QEMU-sourced boundary statement carried
  (QEMU success does not prove real-hardware behavior).
- **Stage review** receives the exit-criteria table and the unresolved-items
  list; P6 closure remains unclaimed by this design regardless of table
  contents.
