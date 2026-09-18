# P8-W11 Implementation Workflow and Review Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W11 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any step, the implementer verifies it has loaded the documents named in
the parent README and inspects the actual state of the prerequisites. Useful
read-only discovery: read the P7 stage implementation and verification material
referenced by the P7-W14 handoff
(`../../../p7/plans/p7-w14-documentation-p8-handoff.md`), the P8-W09/W10
records, and the W15 fixture manifest
(`../p8-w15-reproducible-linux-fixture/README.md` design and its manifest). P8
consumes only evidenced predecessor facts
([plans index](../../plans/README.md)); a planned document is not evidence.

Stop and obtain direction instead of improvising when any of the following
occurs:

- a P7 seam row in [01 §1](01-scenario-and-semantics-contract.md) is not
  evidenced, or the delivered contract differs from the assumed fact — record
  a `P7DependencyIssue` against the P7-W14 handoff and block the affected
  scenarios; do not adapt scenarios to a different seam shape;
- the machine facts needed to instantiate topology parameters (vCPU counts,
  placement expressibility) are not approved — block scenario parameter
  recording; do not invent values;
- the W15 fixture does not implement the [02 §5](02-observation-and-load-contracts.md)
  programs or marker formats — record an integration gap against W15 and block
  execution of affected scenarios;
- W16's harness design realizes the contracts of
  [02](02-observation-and-load-contracts.md) with different semantics — raise
  the conflict; this design owns the semantics, W16 owns mechanics;
- fixing an observed failure appears to require a scheduler, placement, or P7
  API change — that is an `Architecture Change Request` against P7, never a
  W11 edit;
- an anomaly fits no routing class — record it as an explicit block and carry
  it to closure review (step 6); do not force a class.

## 2. Ordered implementation steps

### Step 1 — verify prerequisites and record the seam baseline

Target: implementation record (`../p8-w11-scheduler-linux-integration-record.md`,
created in this step).

Work: for each row of [01 §1](01-scenario-and-semantics-contract.md), locate the
evidenced P7/P6/P8 fact and record its evidence pointer, or mark the row
blocked with the reason. Record the evidenced P7-W05 overcommit tuples and the
approved machine facts that S11-B–S11-E will instantiate.

Suggested observation: read the P7/P8 stage verification records; no runtime
action.

**Acceptance:** every seam row has an evidence pointer or an explicit blocked
entry; the parameter sources for topology values are named.
**Failure/blocker:** a missing or contradictory seam is a `P7DependencyIssue`
per §1; W11-DV04 is blocked, not skipped.

### Step 2 — review the scenario set and descriptors

Target: scenario descriptors per [02 §2](02-observation-and-load-contracts.md).

Work: instantiate S11-A…S11-E descriptors with recorded topology values,
workload parameters, observation windows, markers, and declared limits. Verify
validation rules: 1:1-only placement; no pinning in M:N; preservation sets
consistent with [01 §4](01-scenario-and-semantics-contract.md); bounds present
for every P-1..P-4 condition.

**Acceptance:** W11-DV02 review passes: five valid descriptors (S11-R remains
reserved), no descriptor selects scheduler policy or a machine ABI value.
**Failure/blocker:** an invalid descriptor fails review; fix the descriptor,
not the validation rules.

### Step 3 — confirm fixture and harness readiness

Target: W15 fixture manifest conformance; W16 harness realization of the
[02 contracts](02-observation-and-load-contracts.md).

Work: confirm the fixture provides the `spin-cpu`/`sleep-wake`/`sched-fanout`
programs and marker formats as required, and that W16's design realizes the
descriptor storage, evaluation, and routing semantics. Record pointers.

**Acceptance:** program and marker requirements traceable into the W15
manifest; contract semantics traceable into W16's approved design.
**Failure/blocker:** a deviation is recorded as an integration gap against the
owning package; execution of affected scenarios is blocked.

### Step 4 — execute scenarios through the W16 harness

Target: verification record
(`../../verification/p8-w11-scheduler-linux-integration-verification.md`).

Work: run S11-A…S11-E via W16's automation in dependency order (1:1 before
M:N). Record every command, run environment (QEMU version, host), timestamps,
console transcripts, and counter observations per window. Record what was not
run and why.

**Acceptance:** each executed scenario has a complete observation window and
marker transcript; each verdict is computable from recorded evidence.
**Failure/blocker:** a run failure is evidence — record failed/blocked with the
routed class; do not widen limits to force a pass.

### Step 5 — evaluate verdicts and accounting coherence

Target: verdicts and coherence findings in the verification record.

Work: apply `evaluate_scheduler_preservation` per scenario; require R1–R6 for
the scenarios that declare them and P-1..P-5 for all; perform the W11-DV05
accounting review.

**Acceptance:** S11-A…S11-E satisfy their declared preservation sets and
progress conditions — that is the P8-V16 substance; violations are routed per
[02 §4](02-observation-and-load-contracts.md) with evidence refs.
**Failure/blocker:** a violation stops the affected scenario and routes per §1
of the workflow; no limit is widened and no scheduler edit is made.

### Step 6 — closure review

Work: run the validation matrix (§3), confirm the handoff checklist (§5),
verify the package against the plan's acceptance wording and the task-book
P8-V16 row, and confirm that every limitation statement (no fairness claim, no
policy selection, scenario-bounded results) is carried into the record for
W16–W18 and W20. Completion is claimed only in the verification record, with
evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W11-DV01 → prerequisite review | seam evidence review | inspect P7/P6/P8 evidence against 01 §1 rows | every row evidenced or explicitly blocked; no assumed behavior consumed | the seam baseline is real; not that scenarios run |
| W11-DV02 → P8-V16 | scenario/descriptor review | review descriptors against 02 §2 rules and 01 §3 | five valid descriptors; no pinning in M:N; bounds declared; no policy or ABI value selected | scenarios are declared and bounded; not that Linux progresses |
| W11-DV03 → P8-V16 | preservation/progress condition review | review 01 §4–§5 against plan scope and P7 contract wording | every condition traces to a plan-listed seam and a declared limit; no unbounded claim | conditions are checkable and fair-claim-free |
| W11-DV04 → P8-V16 | scheduler integration matrix execution | run S11-A…S11-E via W16; collect transcripts and counters | all executed scenarios satisfy their preservation sets and progress conditions; each failure routed | Linux progresses 1:1 and M:N with preserved timer/interrupt/WFI/accounting semantics; does not prove fairness, performance, policy quality, or hardware behavior |
| W11-DV05 → P8-V16 | accounting coherence review | recompute counter coherence per 01 §5 P-3 | counters monotonic, coherent, complete for every window | accounting is trustworthy for the matrix; not the P7-W09 backend itself |
| W11-DV06 → P8-V16 | failure-routing review | inspect every recorded anomaly and its routed class | each routed per 02 §4; zero silent reclassifications; no scheduler edit attempted | the non-redesign rule held; not that upstream issues are resolved |
| W11-DV07 → closure | consumer consumability review | read the outputs as W16 (matrix content), W17 (accounting source), W18 (taxonomy), W15 (workload needs) | each consumer can act without inventing requirements | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Declaring scenarios
without W11-DV04 execution does not satisfy P8-V16, and P8-V16 must not be
reported as passed from a partial matrix. Nothing here contributes to
P8-V12–V15 (owned by W09/W10) or P8-V17–V26.

## 4. Error, security, and observability model

**Errors and failure guarantee.** W11 adds no runtime error path. Its failure
model is evidential: a blocked prerequisite, an invalid descriptor, a scenario
violation, or an incoherent counter is recorded as failed/blocked with its
routed class. The guarantee to preserve: a Guest- or scheduler-scenario failure
never becomes an EL2-level event except through the
`HypervisorInvariantViolation` route, which follows P0-W14
(`../../../p0/plans/p0-w14-panic-failure-classification.md`) and is never
absorbed as a VM-facing fault.

**Security.** Linux is an untrusted Guest (ADR-007). Workload programs are
fixture content but their outputs are still Guest-produced: marker text and
elapsed values are recorded observations, not trusted inputs to evaluation
control flow; counter sources are EL2-evidenced accounting paths only. Shared
scenarios must not rely on pinning, so placement declarations cannot be used to
concentrate trust in one mapping. No scenario exercises hypercalls, MMIO, or
DMA; probes of that kind belong to W12/W18.

**Observability.** The evidence surface is the verification record: per-window
counter observations from the evidenced P7-W09 fields, console transcripts with
stable markers, per-scenario verdicts with evidence refs, and the routing log.
Scenario-bounded limits and the explicit not-run list are part of the record;
without them the matrix is not reviewable by W16–W18 and W20.

## 5. Handoff checklist

Before handing W11 to a reviewer, provide:

- the exact changed-file list (expected: the implementation record; verification
  entries; no source or scheduler changes);
- the seam-evidence table with pointers or explicit blocked entries (W11-DV01);
- the five instantiated descriptors with their recorded topology values and
  evidence sources (W11-DV02);
- W11-DV04 per-scenario outcomes with transcripts, counters, verdicts, and
  routing log, including explicit not-run entries;
- the accounting coherence findings (W11-DV05) and failure-routing log
  (W11-DV06);
- confirmation that no scheduler, placement, time-slice, P7 API, EL2 trace,
  machine ABI, or Linux configuration change was made or requested from W11;
- open items: `P7DependencyIssue` records for the P7-W14 owners,
  `P8IntegrationGap` investigations with their bounded scope, and any S11-R
  reservation left for W19 — without resolving them here.
