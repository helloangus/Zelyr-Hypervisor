# P7-W13 Scheduler Performance Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reproducible scheduler measurement baseline and
static-versus-scheduled comparison required by
[P7-W13](../../plans/p7-w13-performance-baseline.md).  
**Owner/change context:** P7-W13 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W13. The plan requires a
reproducible measurement baseline — preemption, context-switch,
scheduling-decision, idle-to-wakeup latency, switches/second, environment
declaration, and a static-pinned-versus-scheduled comparison — whose record
(P7-V29) makes the measurements comparable and states what they do not prove.
This design fixes the **method before any number**: metric definitions bound
to observable event pairs, clock-domain rules, environment declaration,
warm-up/repetition and raw-data-retention policy, scenario set, comparison
conditions, and the no-KPI / no-algorithm-selection constraints. It contains
no measurements and authorizes none until the method is satisfied; a number
produced outside this method is not baseline evidence.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- the normative measurement method (metrics, clock domains, environment,
  repetition/retention policy) →
  [measurement method](01-measurement-method.md);
- the scenario set and the static-versus-scheduled comparison →
  [scenarios and comparison](02-scenarios-and-comparison.md);
- the ordered workflow, validation matrix, and handoff →
  [workflow and acceptance](03-workflow-and-acceptance.md).

Before executing anything, the agent must also follow the Coding Guidelines
preflight, including the repository `AGENTS.md`, documentation index, ADR
baseline, P7 task book, P7-W13 plan, and the P7-W01 recorded input boundary.
This document authorizes no hypervisor source change.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W13 plan → this design
→ Coding Guidelines. In particular:

- ADR-057 leaves the default shared-scheduler algorithm undecided. The plan's
  work sequence requires an explicit review that measurements do not select
  that algorithm; this design therefore forbids any use of the baseline as a
  policy argument and defines no comparative ranking between scheduling
  strategies.
- ADR-048 makes structured telemetry a first-class capability; every metric
  here is an aggregation over the P7-W09 trace/accounting contract, never a
  stopwatch reading over serial text.
- ADR-003 makes QEMU `virt` the reference environment and ADR-045 keeps
  support tiers distinct: baseline numbers describe the declared QEMU
  environment only. They are not hardware predictions and never become
  performance contracts, CI thresholds, or KPIs (plan out-of-scope).
- The plan scopes this package to a baseline and comparison; optimization,
  tuning, and scheduling-policy selection are out of scope, and the task book
  reserves weighted fairness and RT/latency instrumentation to later stages
  (ADR-017).

Classification:

- **Required:** metric definitions M-01–M-05 and the derived comparison M-06;
  clock-domain rules; mandatory environment declaration; warm-up/repetition
  and interleaving floors; raw-data retention; scenario set SC-01–SC-05 and
  the static-basis configurations; comparison conditions and limits; the
  no-KPI and no-algorithm-selection rules; evidence layout; closeout inputs
  to P7-W14.
- **Reserved:** hardware (RK3566/Orange Pi 3B) baselines; statistical rigor
  beyond the declared floors; tail-latency percentile depth beyond the
  declared summary; long-term trending across commits; comparison against
  other hypervisors. Each requires a new design decision.
- **Out of Scope:** hypervisor source changes, instrumentation added to
  production paths (the metrics consume the W09 contract as it exists),
  benchmark framework or crate introduction, scheduler-policy selection,
  optimization work, performance-KPI or regression-gate definitions,
  real-hardware claims.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W09 accounting and P6 baseline terminology | [Method](01-measurement-method.md) §1–§2 | W13-DV01 |
| Define measurable scenarios, units, environment and noise limitations | [Method](01-measurement-method.md) §3–§6; [scenarios](02-scenarios-and-comparison.md) §1–§2 | P7-V29 (W13-DV02) |
| Define static pinned and scheduled comparison conditions | [Scenarios](02-scenarios-and-comparison.md) §3–§4 | P7-V29 (W13-DV03) |
| Review that measurements do not select an ADR-057 algorithm | [Workflow](03-workflow-and-acceptance.md) step 4 | P7-V29 (W13-DV04) |
| Record planned baseline evidence and hand off closeout inputs | [Workflow](03-workflow-and-acceptance.md) step 5 | W14-DV prerequisite (W13-DV05) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs` at
4e631ee): the repository is a P0 documentation scaffold — no workspace, no
sources, no telemetry implementation, no measurement artifacts, and
`docs/stages/p7/verification/` holds only a `.gitkeep`. P0–P6 are planned, not
implemented; sibling P7 designs are being prepared in parallel and are
consumed by path and package ID only. No measured number exists anywhere, so
every value this design speaks of is a planned measurement.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reproducible baseline exists (P7-V29) | No metrics, no method, no numbers | The method of [01](01-measurement-method.md) (metrics, clock domains, environment, repetition/retention) fixed before execution | Reproducibility is a property of a declared method, not of collected numbers | W13 (this design); W09 contract for observables | W13-DV02; verification record |
| Required quantities measurable (preemption, switch, decision, idle-to-wakeup, switches/second) | W09 trace/accounting contract not yet implemented | Evidenced W09 trace events and counters covering the metric event pairs | Each metric is an event-pair aggregation; a missing event blocks its metric | P7-W09 (P7-V19–V21) | W13-DV01 binding table |
| Environment and noise limitations declared | No environment exists to declare | Mandatory environment-declaration field list (§5 of method) applied per run | Numbers without environment are not comparable and not evidence | W13 | W13-DV02 |
| Static-versus-scheduled comparison | No static-basis configuration evidence | W03 pinned/static-baseline configuration contract plus comparison conditions (§3–§4 of scenarios) | A comparison needs a declared basis and equal conditions on both sides | P7-W03; W13 conditions | W13-DV03 |
| No algorithm selection (ADR-057) | n/a — constraint | Explicit review step and the no-KPI rules | The plan makes the review a required work item | W13 | W13-DV04 |
| Closeout inputs to W14 | None | Baseline record location, limitation statements handed to W14 | P7-V30 requires the baseline to be reviewable at closure | W13 hands off; W14 consumes | W13-DV05 |

No row requires inventing a crate, benchmark framework, or instrumentation
mechanism. The runtime prerequisites (P6 timer contract, W09 observability,
W03 static-basis configuration, and a schedulable P7 implementation) are
blocking-if-absent per the P7-W01 boundary.

## Resolved design decisions and their authority

1. **Method-first ordering is normative.** No measurement is baseline evidence
   unless it satisfies [the method](01-measurement-method.md): declared metric
   event pairs, one clock domain per metric, complete environment declaration,
   and the repetition/retention floors. Rationale: the plan's goal is a
   *reproducible* baseline; the method is the reproducibility.
2. **Metric definitions M-01–M-06 with IDs.** Stage-local design freedom owned
   by this design (rationale: the plan names the required quantities but no
   authority fixes their observable definitions; stable IDs let W14, and later
   P8-W17, reference individual metrics unambiguously).
3. **Single clock domain per metric, taken from the P6/W09 timestamp
   contract.** Host wall-clock is bookkeeping-only. Cross-domain derivation is
   prohibited unless a conversion is declared in the record. Rationale:
   mixing QEMU-virtual time with host time is the primary way a QEMU baseline
   becomes meaningless; the task book's Platform Investigation row forbids
   promoting QEMU timing into architecture claims.
4. **TCG/KVM virtualization mode is a mandatory environment field, and
   cross-mode comparison is prohibited within one baseline.** Rationale:
   translation-mode timing differences exceed scheduler effects; combining
   them would make every number irreproducible in spirit.
5. **Warm-up/repetition/interleaving floors are policy minimums.** Values in
   [the method](01-measurement-method.md) §6 are owned by this design
   (implementation may raise them with recorded rationale; lowering requires a
   new design decision). Rationale: floors make "reproducible" reviewable
   before any run exists.
6. **Raw per-repetition data is retained; summaries report median, min, max,
   and sample count — no single-number headline.** Rationale: the record must
   let a reviewer recompute any summary; a lone mean invites KPI misuse.
7. **No-KPI and no-gate rule.** Baseline numbers are never embedded as code
   thresholds, CI gates, or targets, and never cited as scheduler quality.
   Rationale: plan out-of-scope items ("commercial KPI, optimization target")
   and the ADR-057 review obligation.
8. **Static-versus-scheduled comparison is same-workload, same-environment,
   same-window only.** Rationale: the comparison's only permitted reading is
   the measured cost of scheduler control under declared conditions; any
   broader reading would be an ADR-057 policy argument.
9. **Evidence layout.** The baseline record is written to
   `docs/stages/p7/verification/p7-w13-performance-baseline-verification.md`
   with raw data under
   `docs/stages/p7/verification/p7-w13-performance-baseline-artifacts/`;
   method bindings and parameter selections go to
   `docs/stages/p7/implementation/p7-w13-performance-baseline-record.md`.
   Files are created only when work starts.

## Work breakdown and loading order

1. Read [the measurement method](01-measurement-method.md) to bind metrics to
   W09 observables and to learn the clock, environment, and repetition rules.
2. Read [the scenarios and comparison](02-scenarios-and-comparison.md) to
   configure the measurement runs and the static basis.
3. Execute [the workflow](03-workflow-and-acceptance.md) steps in order:
   bind observables, verify prerequisites, run scenarios per the method,
   compute summaries, run the ADR-057 review, hand off closeout inputs.
4. Record actual runs, raw data, summaries, and environment in the
   verification record; record bindings and deviations in the implementation
   record. Neither this design nor a record may claim W13 complete;
   completion is claimed only in verification material with method-compliant
   evidence for P7-V29.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
Guest-visible interface, scheduler mechanism, instrumentation hook, benchmark
framework, script, or CI workflow is designed or authorized by W13. Metrics
consume the W09 observability contract as it exists; if a metric's event pair
is not derivable from that contract, the metric is blocked, and adding trace
events is a change owned by the W09/scheduler designs, not by W13. Any
host-side extraction of numbers from captured evidence is analysis tooling
recorded in the implementation record and subject to P0 dependency
governance; it is not a project interface.

## Downstream handoff

- **P7-W14** receives the baseline record location, the per-metric limitation
  statements, the environment declaration of the runs, and the no-KPI rules,
  as closure inputs (P7-V30).
- **P8** never consumes W13 directly. Through P7-W14, P8-W17 (Linux telemetry
  and non-KPI performance baseline) may reference the P7 method and its
  declared limitations as prior art; P7 numbers are never P8 targets,
  thresholds, or regression gates, and no P7 number becomes a machine-ABI or
  hardware commitment.
