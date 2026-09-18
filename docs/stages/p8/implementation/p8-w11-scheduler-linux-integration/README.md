# P8-W11 Scheduler and Linux Integration — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The P7 scheduler integration validation that lets a real Linux Guest
make progress in 1:1 and declared shared-CPU/M:N conditions, as required by
[P8-W11](../../plans/p8-w11-scheduler-linux-integration.md).  
**Owner-change context:** P8-W11 implementation handoff; integration is
validation-scoped work over the evidenced P7 scheduler seams. It owns no
scheduler mechanism.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W11. The plan makes W11 the
package that proves scheduler cooperation with a real Linux Guest: preemption,
timer restore, interrupt delivery, WFI, wakeup, runtime accounting, and Linux
progress without dependence on static pinning. This design converts that into
(a) a bounded, declared scenario set, (b) semantic-preservation and
progress conditions that stop short of a fairness claim, (c) observation and
failure-routing contracts, and (d) Guest-side workload requirements handed to
the fixture and regression packages. It deliberately does **not** design or
change the scheduler algorithm, runqueues, time-slice values, the placement
API, or any P7 mechanism; a scheduler change is never an authorized outcome of
this package.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- [Scenario and semantics contract](01-scenario-and-semantics-contract.md) —
  read before declaring or reviewing scenarios; upstream seam table, scenario
  set, preservation and progress conditions, failure model.
- [Observation and load contracts](02-observation-and-load-contracts.md) — read
  before building scenario storage, evaluation logic, or Guest workload
  definitions; normative interface obligations.
- [Implementation workflow and review](03-implementation-workflow-and-review.md)
  — read before executing; ordered steps, validation matrix, failure/security/
  observability model, and handoff checklist.

Before editing, the agent must also satisfy the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index, ADR baseline, P8 task book, and
the P8-W11 plan. Nothing in this document claims that any scenario has run or
that P7 delivers its planned behavior.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → evidenced P7 handoff
contracts → P8-W11 plan → this design → Coding Guidelines. In particular:

- ADR-016 fixes scheduler evolution (static pinned → preemptive M:N) and
  ADR-057 defers the default algorithm; W11 validates integration, it does not
  select or benchmark policy (that is P7's and later P8-W17's lane).
- The task book P8 boundary for P7 is "prove Linux integration; no scheduler
  algorithm or policy redesign". The plan repeats this: failures are reviewed
  as P7/P8 dependency issues, never as redesign authorization. This design
  encodes that rule as a hard routing decision in
  [the failure model](01-scenario-and-semantics-contract.md) §5.
- Consumers rely only on **evidenced** predecessor facts
  ([plans index](../../plans/README.md)); P7 is planned, not implemented. Every
  P7 fact this design consumes is therefore an assumed contract with an
  explicit failure boundary (ledger and 01 §1), not an observable.
- The machine ABI is not frozen (ADR section 18 via the task book). vCPU
  counts, placement values, and topology are parameters recorded at
  implementation time from the approved facts of [P8-W02](../../plans/p8-w02-machine-contract-governance.md)
  and [P8-W03](../../plans/p8-w03-linux-boot-contract.md); no value is fixed
  here.

Classification:

- **Required:** the declared 1:1 and M:N Linux scenario set; the
  semantic-preservation observation set (preemption, timer restore, vIRQ
  delivery, WFI/wakeup, lifecycle, switch isolation); the accounting and
  progress conditions without a final fairness claim; the failure taxonomy and
  non-redesign routing; Guest workload functional requirements with stable
  markers; evidence requirements delivered to W16–W18.
- **Reserved:** Linux-plus-Validation-Guest concurrent scheduling beyond the
  evidenced P7 two-VM facts (triggered only if [P8-W19](../p8-w19-validation-guest-dual-track/README.md)
  declares it); scheduler responsiveness or latency targets beyond declared
  scenario limits (P8-W17's lane); any overcommit tuning.
- **Out of Scope:** scheduler algorithm/queue/runqueue design; fairness policy
  and time-slice values; placement API; repairing or extending P7; W16 harness
  mechanics; Linux version/configuration selection (W15's manifest owns those
  facts); machine ABI values; new EL2 instrumentation surfaces.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W10 and the evidenced P7 scheduler contract | [ledger](#current-state-findings-and-goal-to-baseline-ledger); [scenarios contract](01-scenario-and-semantics-contract.md) §1 | W11-DV01 seam review |
| Define 1:1 and declared shared-CPU/M:N Linux scenarios | [scenarios contract](01-scenario-and-semantics-contract.md) §2–§3 | W11-DV02 scenario review; executed by W11-DV04 |
| Relate timer, vIRQ, WFI, lifecycle observations to P7 handoff limits | [scenarios contract](01-scenario-and-semantics-contract.md) §4 | W11-DV03 preservation review |
| Define accounting and progress conditions without a final fairness claim | [scenarios contract](01-scenario-and-semantics-contract.md) §5 | W11-DV03; W11-DV05 accounting review |
| Review failures as P7/P8 dependency issues, not redesign authorization | [scenarios contract](01-scenario-and-semantics-contract.md) §6; [load contracts](02-observation-and-load-contracts.md) §4 | W11-DV06 failure-routing review |
| P8-V16 scheduler integration matrix | [workflow](03-implementation-workflow-and-review.md) §2 step 5, §3 | W11-DV04 (executed via W16) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
the repository is a documentation scaffold — no Cargo workspace, no Rust
sources, no CI, and no implementation or verification records for P1–P7; only
P0-W01 has a verification record. P8 has its twenty plans and this
implementation area; no P8 design is approved yet and the machine contract v1
facts do not exist. Linux, the scheduler, and their integration are therefore
all planned, not observable. Each ledger row states the missing foundation the
plan outcome necessarily requires and who owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| 1:1 and declared M:N Linux progress (P8-V16) | No Linux Guest capability exists; single- and multi-vCPU Linux boot are W09/W10 work | Evidenced one-vCPU Linux shell path (P8-V12/V13) and evidenced 2/4-vCPU SMP path (P8-V14/V15) from W09/W10 | Scheduler integration can only be observed on Linux that already boots and runs | W09 ([design](../p8-w09-virtual-console-single-cpu-linux/README.md)), W10 ([design](../p8-w10-linux-smp-bringup/README.md)) | P8-V12–V15 evidence |
| Preemption, timer restore, vIRQ, WFI, wakeup preservation | P7 planned, unimplemented; P7-W14 handoff not delivered | Evidenced P7 contract for admission/lifecycle (W02), preemption/switch isolation (W04), M:N (W05), block/wakeup (W06), pause/stop (W07), SMP/idle (W08), accounting (W09) | These are the only seams W11 may observe; nothing else is authorized to exist | P7-W14 handoff (`../../../p7/plans/p7-w14-documentation-p8-handoff.md`); seams cited in 01 §1 | P7 stage verification record referenced by the P7-W14 handoff |
| Runtime accounting observations | No trace/counter subsystem exists; P0-W12 semantics and P7-W09 fields are planned | P7-W09-evidenced counters and switch-reason trace fields, aggregated per vCPU/pCPU/VM | Preservation verdicts must be computed from existing fields, not new ones | P7-W09; P0-W12 semantics | P7 accounting evidence; W11-DV05 |
| Declared scenario set exists | No scenario declarations anywhere | This design's scenario set (01 §3), parameterized by approved machine facts | The plan requires "declared" scenarios; no upstream authority fixed them | W11 (this design) | W11-DV02 |
| Guest workloads that generate scheduler load | No fixture exists | Workload functional requirements here (02 §5), realized in the W15 fixture | Linux-side load is fixture content; W15 defines the fixture by validation need | W15 ([design](../p8-w15-reproducible-linux-fixture/README.md)) | W11-DV02/DV04 via W15 manifest |
| Automated execution of scenarios | No harness exists | Scenario descriptors + verdict semantics as obligations on W16's design | W16 owns automated-regression mechanics; W11 owns what must be evaluated | W16 ([design](../p8-w16-automated-linux-regression/README.md)) | W11-DV04 evidence in W16's record |

No row above lets W11 invent a scheduler mechanism, an EL2 API, or a machine
value; where a prerequisite fails to deliver as assumed, the failure boundary
in [the workflow](03-implementation-workflow-and-review.md) §1 applies.

## Resolved design decisions and their authority

1. **Scenario set S11-A…S11-E with one Reserved entry.** The plan requires
   "declared 1:1 and shared-CPU/M:N scenarios" but fixes none. This design owns
   the declaration as stage-local design freedom (01 §3): 1:1 single-vCPU
   baseline, 1:1 four-vCPU SMP, M:N CPU-bound, M:N sleep/wakeup churn, and M:N
   mixed load. Rationale: the set maps one-to-one onto the P7 seams the plan
   lists (preemption, timer restore, WFI/wakeup, accounting) while staying
   bounded and reviewable.
2. **Scenario topology values are parameters, not facts.** The exact (vCPU,
   pCPU) tuples are taken at implementation time from the evidenced P7-W05
   matrix and the approved machine facts, recorded in the implementation
   record. Rationale: the plans index forbids relying on unimplemented
   predecessor facts, and the task book defers machine values to W02's route.
3. **No new EL2 instrumentation surface.** W11 evaluates preservation using
   only the trace fields and counters P7-W09 evidences (plus P6 timer/vIRQ
   facts). A missing field is a `P7DependencyIssue` (01 §6), not a new trace
   event ID or API. Rationale: the task book's P7 boundary and the plan's
   non-redesign rule.
4. **Guest workloads are fixture needs, not W11 code.** Linux-side load
   programs (`spin-cpu`, `sleep-wake`, `sched-fanout`, plus shell use) are
   specified as functional requirements with stable marker strings (02 §5) and
   realized by the W15 fixture; W11 writes no Linux or Guest code. Rationale:
   W15's plan defines initramfs content by validation need; W16 executes.
5. **Failure routing is a closed taxonomy.** All observed anomalies are routed
   to `P7DependencyIssue`, `P8IntegrationGap`, `LinuxBehaviorObservation`, or
   `HypervisorInvariantViolation` (01 §6, 02 §4); none authorizes a scheduler
   edit from W11. Rationale: plan work-sequence step 5.
6. **Progress is bounded, never a fairness theorem.** Every progress condition
   is a declared, per-scenario bound recorded at implementation time; W11 makes
   no starvation-freedom or fairness claim (mirroring P8-V16's "no
   final-fairness claim"). Rationale: plan acceptance wording.

## Work breakdown and loading order

1. Read [the scenario and semantics contract](01-scenario-and-semantics-contract.md)
   to understand which P7 seams are consumed, what the five scenarios are, and
   which preservation and progress conditions each scenario must satisfy.
2. Read [the observation and load contracts](02-observation-and-load-contracts.md)
   when building or reviewing scenario storage, verdict evaluation, failure
   routing, or Guest workload requirements; it fixes the interface obligations
   that W16's design must realize and W15's manifest must serve.
3. Execute in the order given in [the implementation workflow](03-implementation-workflow-and-review.md):
   verify prerequisites, review the seam table, record scenario parameters,
   then run scenarios through W16 and review verdicts and routing.
4. Store actual commands, observations, and results in
   `../../verification/p8-w11-scheduler-linux-integration-verification.md`, and
   record scenario parameters, changed artifacts, and deviations in
   `../p8-w11-scheduler-linux-integration-record.md` only when implementation
   begins. Neither this design nor a written record may claim W11 complete.

## Explicitly excluded interfaces

W11 designs no scheduler trait, runqueue, placement API, time-slice constant,
or scheduling policy; no new EL2 trace event, counter, hypercall, or
management-ABI surface; no machine ABI value, topology constant, or Linux
kernel configuration; no W16 harness internals; no Rust module or crate layout.
The only interfaces fixed here are the validation-side contracts in
[02-observation-and-load-contracts.md](02-observation-and-load-contracts.md),
which are obligations on consumer designs, not implementations. Adding any
excluded surface is a scope conflict to stop at review and, where it would
change the scheduler contract, an `Architecture Change Request` against P7.

## Downstream handoff

- **W16** ([design](../p8-w16-automated-linux-regression/README.md)) receives
  the scenario set, scenario-descriptor schema, stable workload markers, and
  preservation-verdict semantics as the normative content of its scheduler
  regression matrix (P8-V21/V22 rows referencing S11-*). W16 owns execution
  mechanics and evidence.
- **W17** ([design](../p8-w17-linux-performance-baseline/README.md)) receives
  the accounting observation set as the source of scheduler observations for
  its baseline; W11 itself records no performance or fairness figure.
- **W18** ([design](../p8-w18-security-isolation-regression/README.md)) receives
  the failure taxonomy and containment expectations (scenario failures remain
  VM-scoped) for its scheduler-related isolation rows, and the W11-routing rule
  that a HypervisorInvariantViolation is never absorbed as a Guest fault.
- **W15** ([design](../p8-w15-reproducible-linux-fixture/README.md)) receives
  the Guest workload functional requirements (02 §5) as declared validation
  needs its initramfs content must serve.
- **P7, via issue routing:** any `P7DependencyIssue` is recorded against the
  P7-W14 handoff; resolution belongs to P7's authorities, never to a W11 edit.
- **W20** closeout receives W11's limitation statements (no fairness claim, no
  policy selection) as inputs to the factual documentation route.
