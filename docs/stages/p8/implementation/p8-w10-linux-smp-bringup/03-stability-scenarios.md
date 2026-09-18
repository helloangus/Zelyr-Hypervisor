# P8-W10 Stability Scenario Contracts (P8-V15)

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W10 detailed design](README.md).  
**Modules covered:** M5 (scenario engine), M6 (diagnostics integration).

All names are logical contract names (README decision D7). These are
semantic contracts: workload *content* (programs, thread counts, script
text) is fixture material owned by
[W15](../../plans/p8-w15-reproducible-linux-fixture.md); *automation* is
[W16](../../plans/p8-w16-automated-linux-regression.md). This file fixes,
for each declared scenario: purpose, workload class, injected conditions,
required observables, passing condition, and failure bound. Adding,
removing, or weakening a scenario is a revision of this design.

## 1. Common rules for all scenarios

- **Environment:** the declared vCPU count (2 or 4), the current P8
  scheduler configuration, the W15 fixture, one console ([W09](../p8-w09-virtual-console-single-cpu-linux/README.md)).
- **Verdict inputs:** Hypervisor-side telemetry streams ([01 §6](01-architecture-and-state.md))
  AND the Guest console log AND the workload's own progress markers. No
  single stream is sufficient (anti-forgery rule, README decision D6).
- **Verdict classes:** passed (all observables hold for the declared
  duration) / failed (an observable violated — a recorded failure with
  evidence) / inconclusive (environment ended early; rerun required) /
  blocked (a prerequisite stage missing). Nothing is "skipped".
- **Duration and rates:** fixture-configured (W15), bounded, and recorded
  with the evidence; they are not KPIs (W17 owns baselines; these scenarios
  prove semantics, not speed — the plan's exclusion of optimization).
- **Containment invariant (all scenarios):** no Guest behavior may degrade
  the Host, the Hypervisor, or another VM; violations are `failed` with
  severity `architecture`, not merely `failed`.

## 2. Declared scenarios

### S1 — CPU-saturating busy work

```text
Name and stability: scenario SMP-S1 (busy loops) — semantic contract
Purpose: prove concurrent per-vCPU execution with per-CPU timer/idle paths
  remains state-consistent under sustained load (P7-W08 concurrency facts)
Workload class: one spinner thread pinned per vCPU (affinity = vCPU id);
  periodic clock ticks continue (tickless off for the scenario)
Injected conditions: none beyond load
Required observables:
  O1  each vCPU's [W08] timer/periodic tick events continue at the fixture
      period (no tick starvation on any CPU)
  O2  P7-W09 per-vCPU accounting advances for all vCPUs (no zero-progress CPU)
  O3  console responsive throughout (control path alive)
Passing condition: O1–O3 hold for the declared duration
Failure bound: a violated O is `failed`; a Hypervisor-side anomaly is
  `architecture`
```

### S2 — Oversubscribed threading

```text
Name and stability: scenario SMP-S2 (threads > vCPUs)
Purpose: exercise Linux scheduler preemption over the P7 seams (context
  switches, timer-driven preemption) without lost state ([W08 I5] continuity)
Workload class: K runnable threads, K > vCPU count, mixed CPU-bound chunks
  with periodic sleeps
Injected conditions: none beyond oversubscription
Required observables:
  O1  all threads make progress (per-thread progress markers)
  O2  [W08] preemption rows: deadlines delivered exactly once across forced
      preemptions (continuity)
  O3  P7-W09 switch telemetry shows plausible, bounded switch rates
Passing condition: O1–O3 for the declared duration
Failure bound: as §1
```

### S3 — Sleep/wakeup churn

```text
Name and stability: scenario SMP-S3 (sleep/wakeup)
Purpose: stress the WFI/block-wakeup path (WFI contract: ../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md §4;
  block/wakeup owner: P7-W06) — the classic lost-wakeup detector
Workload class: per-vCPU tasks alternating short timed sleeps (WFI idle
  path) and wakeups; cross-vCPU wakeup via IPIs ([W07 SGI]
  (../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md))
Injected conditions: none beyond churn rate
Required observables:
  O1  every sleep completes within a declared latency envelope (fixture-
      configured; envelope generous — semantics, not performance)
  O2  no missed deadline (timer-assertion-without-wakeup events = 0)
  O3  [W08] WFI noop/block telemetry ratios consistent with the workload
Passing condition: O1–O3 for the declared duration
Failure bound: any missed wakeup is `failed` (severity `architecture` — it
  would contradict invariant I4 of [W08])
```

### S4 — Affinity migration

```text
Name and stability: scenario SMP-S4 (affinity)
Purpose: verify vCPU↔pCPU placement changes (P7-W03 policy changes at
  runtime) preserve time/interrupt continuity and TLB coherence
  (continuity: [W08 §5 of 03]
  (../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md);
  transport: docs/stages/p3/plans/p3-w08-tlb-shootdown-transport.md)
Workload class: migratable tasks changing CPU affinity across the Guest's
  vCPUs while timing themselves (clock_gettime deltas)
Injected conditions: deliberate vCPU placement changes driven through the
  scheduler configuration seam (P7's), at fixture-bounded rate
Required observables:
  O1  measured deltas show no time jumps ([W08 I1/I5])
  O2  no stale-execution anomalies (P3-W08 shootdown counters consistent;
    Stage-2 fault anomalies absent)
  O3  per-vCPU interrupt attribution remains exact ([W07 I3-analog])
Passing condition: O1–O3 for the declared duration and migration count
Failure bound: as §1
```

### S5 — Interrupt-heavy mix

```text
Name and stability: scenario SMP-S5 (interrupt mix)
Purpose: the P8-V10 S6 stress context at SMP scale — SGI + timer + console
  traffic concurrently on all vCPUs without loss/duplication
Workload class: concurrent IPI generators ([W07 SGI] §2), dense timer
  reprogramming ([W08] high-frequency rows), console output stream
  ([W09]); rates fixture-bounded
Injected conditions: none beyond the mix
Required observables:
  O1  [W07] state fidelity: no lost/duplicated pending or completions
      (injection/EOI outcome counters reconcile with Guest-handled counts)
  O2  [W08] no lost wakeup under the mix
  O3  Host/Hypervisor responsiveness maintained (bounded-work property)
Passing condition: O1–O3 for the declared duration
Failure bound: as §1; fidelity violations are `architecture`
```

### S6 — Fault and storm containment

```text
Name and stability: scenario SMP-S6 (fault/storm containment)
Purpose: a failing or hostile vCPU must not take down siblings, the VM's
  healthy parts, or the Host (P8-V15 stability × P8-V24 containment at SMP
  scale)
Workload class: scripted fault injection and abuse at fixture-bounded rate:
  (a) a Userspace-triggered Guest panic on one vCPU; (b) an IPI/SGI storm
  from one vCPU; (c) an infinite-loop vCPU; (d) malformed PSCI/GIC access
  bursts ([W06 S5]/[W07] sets)
Injected conditions: as listed — this scenario's workload IS the injection
Required observables:
  O1  sibling vCPUs continue making progress throughout each injection
  O2  each contained outcome matches the owning contract's declared result
      ([W06 §5 of 02](../p8-w06-psci-virtualization/02-code-contracts-psci-dispatch.md),
      [W07 §6 of 02](../p8-w07-linux-vgicv3/02-code-contracts-vgic-mmio.md))
  O3  W13 diagnostic context populated per event; Host unaffected
Passing condition: O1–O3 across the full injection script
Failure bound: any Host-visible degradation is `failed` (severity
  `architecture`)
```

## 3. Race/lost-event interpretation contract (M5)

```text
Name and stability: scenario_correlate(run_record) -> ScenarioVerdict — internal
Purpose and caller: M5's correlation of telemetry + log + markers into a
  verdict for [W16](../../plans/p8-w16-automated-linux-regression.md)
Inputs: the bounded run record for one scenario execution (telemetry
  streams, log snapshot, marker table, fixture parameters)
Outputs: verdict per §1's classes, with per-observable evidence references
Preconditions: all input streams captured for the same run window
Postconditions:
  - pure function over the record; verdicts are reproducible from the same
    record (re-correlation yields identical output — auditability rule)
  - telemetry/log disagreement resolves to `failed` with the discrepancy
    recorded (never silently reconciled)
State and ownership change: none (read-only over the record)
Concurrency/allocation: host-side analysis context; allocation permitted
Errors: missing stream -> `inconclusive` (never guessed)
Security checks: log content is untrusted data ([W09 §5 of 03]
  (../p8-w09-virtual-console-single-cpu-linux/03-code-contracts-console-backend-and-input.md));
  telemetry is trusted; disagreement is the forgery alarm
Logic: evaluate each observable's predicate; combine per scenario rules
Validation: DV07 correlation review; W16 consumes the contract directly
```

## 4. Consumer obligations recorded here

- **[W15](../../plans/p8-w15-reproducible-linux-fixture.md)** provides the
  workload content (programs/scripts/thread counts) and duration/rate
  parameters per scenario class; content must not add success criteria.
- **[W16](../../plans/p8-w16-automated-linux-regression.md)** automates
  execution and collects the run records; `scenario_correlate` is the only
  verdict path.
- **[W17](../../plans/p8-w17-linux-performance-baseline.md)** may reuse the
  telemetry of these runs for baselines, but scenario pass/fail is never a
  performance judgment.
- **[W18](../../plans/p8-w18-security-isolation-regression.md)** consumes
  S6 as its SMP-scale containment input (S5a–S5f and the W07/W08 abuse sets
  fold into S6's script).
- **[W20](../../plans/p8-w20-documentation-closure-handoff.md)** receives
  the verdict records as factual evidence input.
