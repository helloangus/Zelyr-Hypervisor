# P8-W11 Scenario and Semantics Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W11 detailed design](README.md).

## 1. Upstream seams consumed and their failure boundaries

W11 observes only the following seams. Each row names the owning plan, the
fact W11 assumes, and what happens if the delivered evidence differs. "Evidenced"
means present in the P7 stage implementation/verification material referenced
by the P7-W14 handoff (`../../../p7/plans/p7-w14-documentation-p8-handoff.md`);
planned wording alone never satisfies a row.

| Seam | Owning plan | Assumed, evidenced fact | Failure boundary if different |
|---|---|---|---|
| Admission and vCPU lifecycle | P7-W02 (`../../../p7/plans/p7-w02-scheduler-admission-lifecycle.md`) | Guest entry/exit is scheduler-controlled; vCPU states Offline/Runnable/Running/Blocked/Paused/Stopped/Faulted with objective transition validity | `P7DependencyIssue`; scenarios do not run |
| Placement as configuration | P7-W03 (`../../../p7/plans/p7-w03-placement-configuration.md`) | Affinity/pinning/cpuset expressible as scenario configuration | Scenario parameters cannot be expressed; `P7DependencyIssue` |
| Preemption and switch isolation | P7-W04 (`../../../p7/plans/p7-w04-preemption-context-switch.md`) | Timer-driven preemption with declared preservation of arch context, virtual timer, vGIC state, and TLB/VMID lifecycle across switches (ADR §5) | Preemption-dependent scenarios (S11-C/D/E) do not run; `P7DependencyIssue` |
| Shared M:N scheduling | P7-W05 (`../../../p7/plans/p7-w05-shared-mn-multivm.md`) | Declared vCPU/pCPU overcommit matrices progress with basic no-starvation behavior for Validation-Guest loads | `P7DependencyIssue`; M:N Linux scenarios blocked, 1:1 scenarios may proceed |
| Block and wakeup | P7-W06 (`../../../p7/plans/p7-w06-block-wakeup.md`) | WFI/WFE-style blocking releases capacity; timer/vIRQ/Notification wakeup without lost, duplicate, or invalid wakeup | S11-D/E blocked; `P7DependencyIssue` |
| Pause/stop/fault containment | P7-W07 (`../../../p7/plans/p7-w07-pause-stop-fault.md`) | Faulted/stopped vCPUs are excluded; other scheduling unaffected | Containment assertions unusable; `P7DependencyIssue` |
| SMP reschedule and idle | P7-W08 (`../../../p7/plans/p7-w08-smp-reschedule-idle.md`) | Concurrent distinct-vCPU execution, remote reschedule, non-busy idle | S11-B and multi-pCPU M:N blocked; `P7DependencyIssue` |
| Accounting and trace | P7-W09 (`../../../p7/plans/p7-w09-accounting-diagnostics.md`) | Per-vCPU runtime/switch/preemption/block/wakeup/CPU-change counters and switch-reason trace fields, aggregated per vCPU/pCPU/VM | Verdict evaluation impossible; `P7DependencyIssue`. A missing field is never filled by new W11 instrumentation |
| Timer/vIRQ mechanics | P6-W13 (`../../../p6/plans/p6-w13-telemetry-regression-handoff.md`) | Virtual timer semantics and vIRQ injection facts, including latency reference limits | Timer-preservation verdicts degrade to Linux-observable checks only; `P7DependencyIssue` |
| Linux capability | P8-W09/W10 (sibling designs `../p8-w09-virtual-console-single-cpu-linux/README.md`, `../p8-w10-linux-smp-bringup/README.md`) | One-vCPU Linux reaches interactive shell; 2/4-vCPU Linux enumerates and starts secondaries via PSCI | Prerequisite missing; W11-DV04 is blocked, not skipped |

## 2. Scenario model

A scenario is one declared scheduler-integration experiment, expressed as a
`SchedulerScenarioDescriptor` ([02 §2](02-observation-and-load-contracts.md)).
Every scenario declares: topology (vCPU count, pCPU count, mode), workload
program set (02 §5), an observation window, progress markers, the preservation
requirements it exercises, and its declared limits (time bounds and tolerances).
Declaring a scenario fixes validation intent only; it selects no scheduler
policy and no machine ABI value.

Rules common to all scenarios:

- Linux runs with the approved machine facts (vCPU count, timer, GIC, PSCI,
  console) from [P8-W02](../../plans/p8-w02-machine-contract-governance.md)-routed
  approvals and the [P8-W03](../p8-w03-linux-boot-contract/README.md)/[P8-W04](../p8-w04-guest-dtb-contract/README.md)
  boot and DTB contracts; nothing here overrides them.
- M:N scenarios must pass without per-vCPU static pinning beyond the declared
  shared-mapping itself; "without dependence on static pinning" is the plan's
  wording and is enforced by descriptor validation (02 §2): only 1:1 scenarios
  may declare dedicated placement.
- Workload markers are validation-harness observables. They are not Guest ABI
  and must never be enumerated in the [P8-W14](../p8-w14-machine-abi-compatibility/README.md)
  compatibility matrix.

## 3. Declared scenario set

| ID | Mode and topology | Linux load | Seams exercised | Purpose |
|---|---|---|---|---|
| S11-A | 1:1, 1 vCPU on 1 dedicated pCPU | Boot to shell; `sleep-wake` steady state | Admission, timer, WFI/wakeup without sharing | Baseline: Linux progress with no shared-CPU interference |
| S11-B | 1:1, 4 vCPU on 4 dedicated pCPUs | Per-CPU `spin-cpu` + `sleep-wake` | SMP reschedule/idle, per-CPU timer/vIRQ | Linux SMP over dedicated scheduling after W10 passes |
| S11-C | Shared M:N, declared overcommit tuple from evidenced P7-W05 matrix (Linux vCPUs > pCPUs) | `spin-cpu` on every vCPU | Preemption, switch isolation, timer restore | CPU-bound preemption with timer semantics preserved under real Linux |
| S11-D | Shared M:N, same tuple class as S11-C | `sleep-wake` churn on every vCPU | Block/wakeup, timer, vIRQ wakeups | WFI-blocking and wakeup correctness under sharing |
| S11-E | Shared M:N, mixed | Shell interaction + `sleep-wake` + subset `spin-cpu` + `sched-fanout` | Accounting, switch reasons, containment | Mixed-load progress and accounting coherence; the responsiveness scenario |
| S11-R | Reserved: Linux + Validation Guest concurrently | — | P7-W05 two-VM facts | Only if [P8-W19](../p8-w19-validation-guest-dual-track/README.md) declares it; not required for W11 closure |

Exact tuples for S11-C/D/E are recorded at implementation time (README decision
2). P8 schedules at most one Linux VM at a time; multi-Linux-VM scheduling is
later-stage work and is out of scope here.

## 4. Semantic-preservation requirements

Each requirement is an observation evaluated over a scenario window from the
P7-W09-evidenced fields plus Linux-observable markers. Every requirement cites
its upstream authority; W11 adds no mechanism.

- **R1 Preemption and switch isolation (P7-W04):** under S11-C, every vCPU
  experiences declared preemptions; Linux on each vCPU continues correctly
  after each switch. Observable: vCPU progress markers advance across switch
  events; no vCPU's marker stalls for the remainder of the window.
- **R2 Timer restore (P7-W04, P6-W13):** virtual timer state is preserved
  across switches such that Linux timer behavior is unaffected: `sleep-wake`
  cycles complete with per-wakeup latencies within the scenario's declared
  slack over the requested sleep; expired-deadline behavior after a preempted
  interval matches the P6-evidenced contract (a timer that expired while the
  vCPU was not running is delivered, not lost).
- **R3 Interrupt delivery (P6-W13, P7-W04):** vIRQs pending across a switch are
  delivered to the correct vCPU exactly once; declared interrupt counts are
  not lost or duplicated (checked via accounting counters and Linux-observable
  completion markers).
- **R4 WFI/wakeup (P7-W06):** WFI-blocking releases pCPU capacity in shared
  scenarios (a sleeping vCPU's pCPU runs other work); every declared sleep
  cycle wakes exactly once; no duplicate or invalid wakeups; no busy-loop
  blocking (counter evidence, P7-W17/V18-style idle checks as evidenced).
- **R5 Lifecycle consistency (P7-W02, P8-W06):** Linux-visible CPU states
  (online/offline during boot and shutdown) remain consistent with the
  scheduler's vCPU lifecycle; the declared PSCI shutdown path terminates all
  vCPUs without scheduler-inconsistent residue.
- **R6 Switch isolation breadth (P7-W04, ADR §5):** declared per-vCPU state
  (arch context, timer, vGIC state) shows no cross-vCPU contamination in
  repeated rotation, using the switch-isolation observations P7 evidences.

## 5. Progress and accounting conditions (no final fairness claim)

All conditions are per-scenario, bound-limited observations recorded in the
implementation record; none generalizes beyond the declared scenario.

- **P-1 Progress:** every progress marker declared for the scenario is observed
  within the scenario's declared time bound.
- **P-2 Sleep correctness:** `sleep-wake` completes each declared cycle within
  requested time plus the scenario's declared slack (R2).
- **P-3 Accounting coherence (P7-W09):** per-vCPU runtime, switch, and
  block/wakeup counters are monotonic and mutually coherent for the window
  (sums agree with the declared topology within the declared tolerance); no
  negative, duplicated, or frozen counter.
- **P-4 Non-starvation within bound:** every Linux vCPU in the scenario shows
  nonzero execution within the declared bound. This is a scenario outcome, not
  a starvation-freedom or fairness property; W11 claims neither.
- **P-5 Containment:** any scenario failure routes through the §6 taxonomy and
  remains VM-scoped; no failure escalates to an EL2-level event except
  `HypervisorInvariantViolation`.

## 6. Failure model and routing

Every anomaly observed in any scenario is routed by
`route_scheduler_failure` ([02 §4](02-observation-and-load-contracts.md)) into
exactly one class:

| Class | Meaning | Authorized response from W11 |
|---|---|---|
| `P7DependencyIssue` | Behavior contradicts an evidenced P7 contract row (§1), or a required field/mechanism is absent | Stop the affected scenario; record against the P7-W14 handoff; never patch, work around, or redesign |
| `P8IntegrationGap` | P7 semantics hold but a Linux-required interaction was not evidenced (e.g., a timer-restore ordering observable only under Linux) | Record a bounded design investigation in the W11 record; if resolution would change P7 contracts, reclassify `Architecture Change Request` |
| `LinuxBehaviorObservation` | Hypervisor and scheduler behavior match contracts; Linux itself misbehaves or the fixture load is faulty | Record; route to W15 (fixture) and [W13](../p8-w13-guest-fault-diagnostics/README.md) diagnostics |
| `HypervisorInvariantViolation` | An EL2-side ADR §19-class invariant breaks (e.g., state corruption visible in EL2) | Hypervisor-level failure per P0-W14 (`../../../p0/plans/p0-w14-panic-failure-classification.md`); never absorbed as a Guest fault; hand to [W13](../p8-w13-guest-fault-diagnostics/README.md) taxonomy |

`Unclassified` is not an allowed steady state: an anomaly that fits no class is
recorded as a block per [the workflow](03-implementation-workflow-and-review.md)
§1 and reviewed at closure. No class authorizes editing scheduler code,
time-slice values, placement semantics, or P7 APIs from this package.
