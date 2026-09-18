# P7-W13 Scenarios and Static-versus-Scheduled Comparison

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W13 detailed design](README.md).

## 1. Scenario construction rules

A measurement scenario is a declared tuple: topology, placement, payloads,
metric set, and its static-basis pairing where applicable. Payloads reuse the
maintained P7-W10 workload suite by ID and parameters — this is stage-internal
asset reuse, not consumption of W10's validation evidence; W13's evidentiary
prerequisite remains P7-W09 per the plan index. If a required payload does not
exist in the W10 suite, the scenario is blocked and named; W13 authors no
Guest code.

Time-slice and related scheduler parameters are configuration facts of a run:
each scenario records the values it ran with. Varying a parameter across runs
to search for a better value is tuning, is out of scope, and would violate the
no-KPI rules; the baseline reports the implemented configuration as found.

## 2. Scenario set

Scenario IDs are stage-local design freedom owned by this design.

| ID | Purpose (metrics) | Topology and placement | Payloads |
|---|---|---|---|
| SC-01 | Preemption reaction (M-01), preemption-driven M-02/M-03/M-05 | 1 pCPU, 1 VM, 2 vCPUs, shared | CPU-bound vCPUs (W10 CPU-bound workload); the pCPU must be oversubscribed so deadline preemption fires |
| SC-02 | Context-switch duration (M-02), yield/block-driven M-03, M-05 | 1 pCPU, 1 VM, 3 vCPUs, shared | Rapid-yield workload: vCPUs block/yield frequently (periodic short WFI), producing A→B→C→A rotation |
| SC-03 | Idle-to-wakeup latency (M-04), per wakeup source | 2 pCPU, 1 VM, 2 vCPUs pinned one per pCPU | Periodic-WFI vCPU on P0 (timer-source windows); partner vCPU injects SGI/vIRQ and Notification events on declared sub-windows |
| SC-04 | Switch rate under mixed load (M-05) | 2 pCPU, 1 VM, 4 vCPUs, shared | Mixed: CPU-bound + periodic-WFI + HVC-heavy (W10 IDs), declared proportions |
| SC-05 | Static basis (all metrics in their restricted form) | The static-basis configurations of §3 | Same payloads as the paired scheduled scenario |

Topology points are measurement parameters, not stage gates; they are chosen
to isolate the measured effect (oversubscription for preemption, rotation for
switch cost, quiesced partner for idle wakeup). Scenario rows may be added
only through a design change; a row may be blocked by a missing payload or
observable like any other.

## 3. Static-basis configurations and pairing

The static basis is the P7-W03 static-pinned equivalence configuration: each
vCPU pinned 1:1 to its own pCPU, so that scheduler selection is degenerate
and the measured quantities reduce to the fixed-binding behavior that P7 must
preserve (task-book requirement: static pinned equivalence). Pairing rules:

- SC-01 basis: 2 pCPU, 2 vCPUs pinned 1:1 (no oversubscription; preemption
  rows of M-01 are expected to be empty there and are reported as not
  applicable rather than zero).
- SC-02 basis: 3 pCPU, 3 vCPUs pinned 1:1.
- SC-03 basis: identical to SC-03 (pinned 1:1 already); wakeup-source
  classification is compared across the pair, not topology.
- SC-04 basis: 4 pCPU, 4 vCPUs pinned 1:1 with the same payload mix.
- SC-05 is the collection of basis runs, reported with the same summaries so
  the scheduled rows have a declared reference.

What the comparison may support: the observed delta of M-01–M-05 between a
scheduled configuration and its basis, under equal conditions, as a factual
description of the cost of scheduler control for these workloads in this
environment.

What it must never support: a ranking of scheduling algorithms (ADR-057 —
only one shared algorithm exists in P7 and the comparison does not pit
algorithms against each other), an optimization target, a regression gate, a
hardware prediction, or a claim that the scheduler adds no cost beyond the
delta (unmeasured paths remain unmeasured).

## 4. Comparison conditions

A comparison row is valid only when all of the following hold; otherwise the
comparison is invalid and recorded as such:

1. same payload set and parameters on both sides;
2. same environment declaration (host, QEMU identity, virtualization mode,
   binary identity, toolchain, session quiet rule);
3. interleaved repetitions of the pair within one session per
   [the method](01-measurement-method.md) §6;
4. identical metric bindings and extraction method on both sides;
5. both sides met the repetition floors.

The comparison summary reports per metric: both medians with min/max, the
sample counts, and the delta of medians — labeled with the environment and
mode. Deltas are never normalized into percentages-of-improvement narratives
in the record; the factual difference and its conditions are the content.

## 5. Proves / does not prove (scenario-level)

- SC-01 proves: deadline preemption and its reaction are observable and
  quantified in the declared oversubscribed configuration. Does not prove:
  any slice-value or tick-model adequacy, worst-case latency, or hardware
  interrupt latency.
- SC-02 proves: scheduler-initiated switch cost is observable and quantified
  under declared rotation. Does not prove: context-save correctness (that is
  P7-W04/P7-V09 evidence) or upper bounds under adversarial load.
- SC-03 proves: idle-to-wakeup latency is observable per declared wakeup
  source. Does not prove: hardware wakeup latency, timer-programming
  adequacy, or WFI semantics on physical cores.
- SC-04 proves: switch-rate behavior under the declared mixed load. Does not
  prove: saturation throughput claims or fairness (P7-W05/W11 domain).
- The M-06 comparison proves: the declared delta under equal conditions.
  Does not prove: any policy conclusion (ADR-057 review, workflow step 4,
  enforces this).
