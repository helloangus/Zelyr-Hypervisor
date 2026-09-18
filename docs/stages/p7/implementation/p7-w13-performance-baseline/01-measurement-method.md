# P7-W13 Measurement Method

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W13 detailed design](README.md).

## 1. Terminology basis

The metric vocabulary builds on two assumed contracts, inspected first per
the plan's work sequence:

- **P7-W09 accounting and trace contract**
  ([design](../p7-w09-accounting-diagnostics/README.md)): the observable
  events, counters, and diagnostic fields of the scheduler. Every metric
  below is defined as an aggregation over this surface; exact event and field
  names are bound at implementation time from the evidenced W09 contract
  (P0-W13 trace-namespace governance applies upstream of it).
- **P6 baseline terminology** (P6-W13 telemetry/regression/handoff): the
  stage precedent for latency-measurement method, environment declaration,
  and the explicit boundary that QEMU observations are environment facts.
  Where P6 declared a latency-measurement convention, W13 reuses its
  terminology rather than coining a parallel one.

If the evidenced W09 contract lacks an event pair a metric requires, that
metric is **blocked** with the missing observable named. Extending the trace
surface is a W09/scheduler-design change, never a W13 change.

## 2. Metric definitions

Each metric is an ordered **event pair** (or count) over the W09 surface,
evaluated on one pCPU unless stated. `t(X)` is the timestamp of observable X
in the metric's declared clock domain.

| ID | Metric | Observable definition | Unit | Clock domain |
|---|---|---|---|---|
| M-01 | Preemption reaction | From the scheduler-decision event that ends vCPU V's slice on pCPU P (preemption decision per W04) to the first Guest-entry observable of the selected successor on P | time | scheduler timestamp domain |
| M-02 | Context-switch duration | From the switch-begin observable for a scheduler-initiated switch on P (previous vCPU exit/save) to the next vCPU's Guest-entry observable on P | time | scheduler timestamp domain |
| M-03 | Scheduling-decision latency | From the previous vCPU's exit observable on P to the selection-complete observable of the scheduler decision on P. Declared only if the W09 surface distinguishes decision from save/restore; otherwise recorded as derived (M-02 minus M-01 bounds) and marked derived in every summary | time | scheduler timestamp domain |
| M-04 | Idle-to-wakeup latency | From the idle-entry observable on P to the next Guest-entry observable on P, classified by wakeup source (deadline timer, physical IRQ/vIRQ, Notification/internal event per W06/W08) | time | scheduler timestamp domain |
| M-05 | Switch rate | Count of vCPU-switch observables on P (and aggregate) divided by the measurement-window length | switches/s | scheduler timestamp domain for counts; window length in the same domain |

Derived comparison (not an independent measurement): **M-06 static-vs-
scheduled delta** — per-metric difference between the scheduled configuration
and its static-basis configuration, computed per
[the comparison rules](02-scenarios-and-comparison.md) §3.

Definition rules:

- A metric value is valid only if both events of its pair occurred on the
  same pCPU within one measurement window and the pair is unambiguous in the
  trace (a run with ambiguous pairing is invalid for that metric).
- M-01/M-02/M-03 are recorded per switch kind where the W09 reason field
  distinguishes them (preemption-driven vs yield/block-driven); summaries
  never merge kinds silently.

## 3. Clock-domain rules

- The authoritative clock for all five metrics is **the scheduler timestamp
  source declared by the P6 per-pCPU timer contract as consumed by W09**. The
  concrete source is bound at implementation time and named in the record;
  W13 does not select or reprogram any timer.
- Host wall-clock is used only to record run scheduling, durations of the
  measurement session, and the environment declaration; it never enters a
  metric value.
- Guest-observable counters (architected timer reads) are not metric sources
  in P7; using them would conflate Guest-visible virtual time with scheduler
  cost. If a later design introduces such a metric, that is a new decision.
- Cross-domain derived quantities are prohibited. The known limits are stated
  in every record: in QEMU, the timestamp domain advances under the declared
  virtualization mode and does not represent hardware cycle behavior
  (ADR-003; task-book Platform Investigation row).

## 4. What is measured, and what a value means

The baseline measures the implemented scheduler's observable cost and
latency behavior under declared workloads in the declared QEMU environment.
A recorded value means: "under environment E, workload W, configuration C,
the M-xx distribution had median/minimum/maximum X over N samples collected
by method Y." It never means: the scheduler is fast/slow in general, one
algorithm is better than another (ADR-057), hardware will behave the same,
or a quality threshold is met. All of those readings are excluded by the
no-KPI rules of the parent README (decision 7).

## 5. Environment declaration (mandatory per run)

A run without the complete declaration is invalid evidence (recorded as
not-run-for-evidence with the defect named). Mandatory fields:

1. host platform: CPU model, core count, memory, kernel/OS identity;
2. host load state: the quiet rule — no competing load; observed load during
   the run is recorded;
3. QEMU identity: version, machine model, virtualization mode (TCG or KVM)
   and emulated/set CPU model, pCPU count presented;
4. hypervisor identity: source commit, build profile/features (P0 build and
   profile governance), binary identity;
5. toolchain identity: the P0-pinned toolchain version;
6. workload identity: payload programs, versions/commits, parameters;
7. configuration identity: topology, placement, time-slice-related declared
   parameters as configured (values are configuration facts of the run, not
   recommendations);
8. date/time and operator notes for anomalies.

Mode rule: TCG-mode and KVM-mode runs are separate baselines; numbers are
never merged across modes, and each record names its mode.

## 6. Warm-up, repetition, interleaving, and retention policy

Floors owned by this design (parent README decision 5); raising is allowed
with recorded rationale, lowering is a design change:

| Policy | Floor |
|---|---|
| Warm-up runs discarded per scenario-configuration | 2 |
| Measured repetitions per scenario-configuration | 5 |
| Measurement-window length per repetition | ≥ 30 s wall-clock or ≥ 10 000 switch observables on the busiest measured pCPU, whichever is later |
| Interleaving | Configurations of one comparison pair are run interleaved (alternating order recorded); all repetitions of a pair occur in one session under one environment declaration |
| Noise handling | A repetition with a recorded host anomaly is discarded and re-run; the discard and reason are recorded — silent dropping is prohibited |

Retention policy:

- Raw per-repetition data (the extracted event-pair samples, per-pCPU and
  aggregate) is retained under
  `docs/stages/p7/verification/p7-w13-performance-baseline-artifacts/` with
  the environment declaration of its session. Raw data is the evidence;
  summaries are derived views.
- Summaries report, per metric per configuration: sample count, median,
  minimum, maximum. The record may group by switch kind and wakeup source as
  declared in §2. No other statistic is required; adding percentiles is
  Reserved.
- Every summary must be recomputable from retained raw data by a reviewer;
  the extraction method (how event pairs become samples) is described in the
  record.

Limits stated in every record (method-inherent, not disclaimers added at
leisure): single-host single-session variance is not characterized beyond the
floors; QEMU timing does not model hardware; nothing here characterizes tail
latency under adversarial load (that is stress territory, P7-W11) or
throughput under saturation beyond M-05's declared windows.
