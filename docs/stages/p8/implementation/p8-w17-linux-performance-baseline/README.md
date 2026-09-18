# P8-W17 Linux Performance Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The first reproducible Linux observability baseline — measurement
method, environment contract, raw-data policy, and baseline-record structure —
required by [P8-W17](../../plans/p8-w17-linux-performance-baseline.md).  
**Owner/change context:** P8-W17 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W17. It defines *how* the P8
Linux observations are measured before any number exists: the metric catalog
with clock domains and observation points
([01](01-measurement-method.md)), and the environment declaration,
repetition/warm-up policy, raw-data retention, baseline-record structure, and
comparison rules ([02](02-baseline-record-and-workflow.md)). It is a
measurement design, not a result: it authorizes no optimization, sets no
target, encodes no final metric, and interprets nothing as hardware
performance. Every number that will ever be produced under it is a **baseline
observation with explicit limits**, never a KPI.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
[01](01-measurement-method.md) for any metric-definition or instrumentation
step and [02](02-baseline-record-and-workflow.md) for environment capture,
run procedure, record authoring, and comparison review. Before editing, the
agent must also follow the Coding Guidelines preflight (repository
`AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P8 task
book](../../task-book-v0.1.md), and the P8-W17 plan). This document claims no
measurement has been taken.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W17 plan → this
design → Coding Guidelines. In particular:

- ADR-048 and ADR section 12 make structured telemetry a first-class
  capability whose high-overhead events are compilable/removable — the
  baseline must therefore disclose instrumentation configuration on every
  record. ADR-003 scopes QEMU to deterministic reference testing; a QEMU/TCG
  observation is never a hardware performance statement. ADR-057 leaves the
  default scheduler algorithm undecided — no P8 measurement may be used to
  select it, and this design must not collect data structured as an
  algorithm bake-off.
- The task book binds this package to P8-V23: a baseline record carrying
  boot time, exits, Stage-2 faults, timer/vIRQ, switches, Host CPU use, and
  interrupt-latency observations *with environment and limits*, and no KPI
  claim. This design defines the conditions under which P8-V23 could pass.
- The plan's exclusions are binding: **no performance optimization, no
  targets, no competitive comparisons, no final metric encoding, and no
  interpreting QEMU measurements as hardware performance.** A baseline
  comparison may only describe; it must never gate.

Classification. **Required** for W17 closure: the metric catalog with clock
domains, the environment-declaration contract, the warm-up/repetition policy,
the raw-data retention rule, the baseline-record structure, and the
comparison-validity rules. **Reserved** with recorded triggers: statistical
treatment beyond the declared summary statistics (trigger: an approved
consumer design requiring it), instrumentation of P9 virtio paths (trigger:
P9), and any hardware-platform measurement reuse of this method (trigger:
P15). **Out of Scope:** implementing telemetry mechanisms (owned by the P0
telemetry governance and the owning stage designs), performance workloads
beyond the W10/W11 declared profiles, scheduler-policy conclusions (ADR-057
route), CI wiring, and real-hardware measurement.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W10–W12, W16, and telemetry governance (work seq 1) | README ledger; [workflow](02-baseline-record-and-workflow.md) §1 | prerequisite-contract review (W17-DV01) |
| Define metrics, environment, and limitation record required for comparison (work seq 2) | [method](01-measurement-method.md) §2–§4; [workflow](02-baseline-record-and-workflow.md) §2, §5 | P8-V23 (W17-DV02, DV03) |
| Relate measurements to automated fixture and regression scenarios (work seq 3) | [method](01-measurement-method.md) §5 | P8-V23 (W17-DV02) |
| Separate observation from a pass/fail performance objective (work seq 4) | [method](01-measurement-method.md) §6; [workflow](02-baseline-record-and-workflow.md) §6 | policy review (W17-DV05) |
| Review host/platform dependence and diagnostic-overhead disclosure (work seq 5) | [method](01-measurement-method.md) §4; [workflow](02-baseline-record-and-workflow.md) §2 | policy review (W17-DV03) |
| Hand comparison inputs (not optimization conclusions) to W20 and later stages | README handoff; [workflow](02-baseline-record-and-workflow.md) §10 | consumability review (W17-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation-only P0 scaffold — no hypervisor build, no telemetry
implementation, no QEMU runner, no prior performance record in any stage
(P7-W13's scheduler baseline is planned, not produced). P0-W12 (diagnostic
semantics) and P0-W13 (trace-event namespace) are plans only; P6-W13's latency
measurement method and P7-W13's baseline terminology are planned handoff
contracts. Every input W17 needs is therefore an assumed contract, and each
ledger row states the failure boundary if it delivers differently.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P8-V23: boot time, exits, Stage-2, timer/vIRQ, switches, Host CPU, interrupt latency observed | No metric definitions exist in any tracked file | Metric catalog with per-metric definition, clock domain, observation point, unit, and aggregation ([01](01-measurement-method.md) §2) | A number without a declared clock domain and observation point is not comparable or reviewable | W17 (this design); counter sources owned by P0-W13-governed telemetry and P7 accounting designs | W17-DV02 method review |
| P8-V23: environment and limitation record | No environment convention exists | Environment-declaration contract bound into every record ([02](02-baseline-record-and-workflow.md) §2) | Host/platform dependence is part of the observation, not metadata to omit | W17; fields reuse the W16 run-record environment block | W17-DV03 review |
| P8-V23: reproducible | No repetition policy exists | Warm-up/repetition/raw-data policy ([02](02-baseline-record-and-workflow.md) §3–§4) | Reproducibility requires declared repetition and retained raw data, not a single number | W17 (this design) | W17-DV04 policy review and raw-data retention audit |
| P8-V23: no KPI claim | n/a (governance absence) | Non-KPI rule and comparison-validity rules ([01](01-measurement-method.md) §6; [02](02-baseline-record-and-workflow.md) §6) | The plan forbids targets and comparisons-as-gates; the boundary must be written before data exists | W17 | W17-DV05 policy review |
| Measurements bound to regression scenarios | W16 matrix planned only | Scenario binding table ([01](01-measurement-method.md) §5) | Measurement without a fixed scenario is irreproducible; W16 rows are the declared fixed conditions | W16 rows; W15 fixture; W10/W11 workload profiles | scenario references present in every record |
| Interrupt-latency observations | P6-W13 method is a planned handoff contract | Method consumed from P6-W13 as assumed contract, re-expressed for the Linux context ([01](01-measurement-method.md) §2, metric M7) | Inventing a second, divergent latency method would make P6/P8 numbers incomparable | P6-W13 handoff (via `p6-w13-telemetry-regression-handoff`); W17 binds it | **failure boundary:** if P6-W13 delivers no method, M7 is a recorded blocked prerequisite, never locally reinvented |
| Telemetry counter sources | P0-W12/W13 governance planned only | Counter names consumed under the P0-W13 namespace as assumed contract | ad-hoc counter strings would become undeclared interfaces | P0-W13 namespace; owning-stage designs | **failure boundary:** missing namespace ⇒ counters recorded as blocked prerequisite |

No row above invents a crate, API, address, or hardware assumption. The
concrete v1 machine values remain an inherited `ADR Required` item (task book
§8, routed via [P8-W02](../p8-w02-machine-contract-governance/README.md));
measurements reference the approved configuration, never values fixed here.

## Resolved design decisions and their authority

1. **Method-first ordering.** The workflow authoring order is: metric
   definitions → environment contract → repetition policy → run procedure →
   records. No measurement run is authorized before the method artifacts
   exist and pass review. Rationale: the plan's own work sequence places the
   method before relation to scenarios and data; post-hoc metric definitions
   would make the baseline incomparable. Stage-local design freedom owned
   here.
2. **Seven-metric catalog.** Exactly the plan's observation set (boot time;
   exits and reasons; Stage-2 faults; timer/vIRQ observations; scheduler
   switches; Host CPU use; basic interrupt latency) is defined as M1–M7;
   adding a metric requires a reviewed design edit with the same rigor.
   Rationale: bounded scope; each addition otherwise reopens the method.
3. **Clock-domain explicitness.** Every metric declares exactly one clock
   domain (host monotonic harness clock; hypervisor accounting counters;
   Guest-observed counter/time sources; QEMU-process CPU accounting) and no
   record may mix domains in one summary statistic. Rationale: cross-domain
   averaging is the most likely way this baseline could mislead later stages.
4. **No pass/fail performance objective.** No threshold, budget, regression
   gate, or optimization conclusion is authorized anywhere in W17 artifacts.
   Rationale: explicit plan exclusion; also protects the ADR-057 decision
   route from being pre-empted by scheduler-shaped data.
5. **Baseline records are evidence, kept under `verification/`, immutable per
   review cycle.** Location:
   `docs/stages/p8/verification/p8-w17-linux-performance-baseline-verification.md`
   plus raw data under `docs/stages/p8/verification/assets/p8-w17/`. A
   superseding baseline appends a new record and never edits an old one.
   Rationale: `docs/README.md` assigns evidence to `verification/`; baseline
   values are observations about a past state, so rewriting them would
   destroy the only thing they prove. Stage-local convention owned here.
6. **Instrumentation-overhead disclosure is part of every record.** Each
   record states the telemetry/trace configuration used (per ADR-048's
   compilable-events rule) so that a low-overhead run and a fully
   instrumented run are never compared silently. Stage-local policy owned
   here.
7. **W16 envelope reuse.** Baseline runs execute the declared W16 scenario
   rows through the W16 harness contract and consume its environment and
   identity fields verbatim; W17 adds measurement collection, never a second
   execution path. Rationale: one execution authority per scenario prevents
   uncontrolled variance between "the regression run" and "the measured run".

## Work breakdown and loading order

1. Read [01](01-measurement-method.md) for metric definitions M1–M7, clock
   domains, overhead disclosure, scenario binding, and the non-KPI rule.
2. Read [02](02-baseline-record-and-workflow.md) for the environment
   contract, repetition/warm-up policy, run procedure, baseline-record
   schema, comparison rules, ordered workflow, validation matrix, and
   handoff checklist.
3. Execute the workflow of [02](02-baseline-record-and-workflow.md) §7 (steps 1–5)
   when implementation is authorized. Actual commands, outputs, environments,
   raw data, and run/not-run status go to
   `../../verification/p8-w17-linux-performance-baseline-verification.md` and
   `../../verification/assets/p8-w17/`; factual decisions (chosen summary
   statistic parameters, deviations) go to
   `../p8-w17-linux-performance-baseline-record.md` — all created only when
   that work begins. Nothing here claims a measurement exists.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
or persistent binary layout is designed or authorized by W17. Counter and
trace-event *names* consumed by the metric catalog remain owned by the
P0-W13 namespace governance and the owning stage designs; W17 defines which
existing sources are observed, not new event semantics. No benchmark script,
CI job, plotting tool, or hypervisor instrumentation code is designed here;
their concrete realization is an implementation choice recorded in the
implementation record. Adding any of these under W17 authority is a scope
conflict to stop at review.

## Downstream handoff

Per the [plan index](../../plans/README.md) consumer map:

- **W20** consumes the baseline record, its environment/limitation statement,
  and the raw-data index for the P8 evidence index and closure review; W20
  must present them as observations with limits, never as achievements.
- **P9+** receives comparison inputs only: the method, the environment
  contract, and the raw data. Later stages may rerun the same method under a
  new environment declaration and compare per
  [02](02-baseline-record-and-workflow.md) §6; they receive **no optimization
  conclusion, no performance target, and no license to quote QEMU/TCG numbers
  as hardware performance**. Virtio performance work (P9 onward) must define
  its own metrics; M1–M7 deliberately exclude device-I/O measurement.
- **Future maintenance stages** (including the P15 hardware port) may reuse
  the method's *structure* on other platforms only by issuing a new
  environment declaration and re-recording limits; the QEMU baseline itself
  never transfers across platforms.
