# P8-W11 Observation and Load Contracts

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W11 detailed design](README.md).

These are normative interface obligations, not implementations. No crate,
module, file layout, or target is selected here; the realization belongs to the
consuming designs ([W16](../p8-w16-automated-linux-regression/README.md) for
execution and evidence, [W15](../p8-w15-reproducible-linux-fixture/README.md)
for Guest workload content). Names are fixed by this design for the P8 stage;
renaming requires a revision of this design.

## 1. Contract status and stability

| Contract | Kind | Realized by | Stability |
|---|---|---|---|
| `SchedulerScenarioDescriptor` | data type | W16 scenario storage | Stage-local, fixed by this design |
| `evaluate_scheduler_preservation` | function (semantic) | W16 harness logic | Stage-local; semantics fixed here |
| `SchedulerIntegrationFailureClass` + `route_scheduler_failure` | enum + function (semantic) | W16 harness logic, review process | Stage-local; routing rules fixed here |
| Guest workload programs | functional requirements | W15 fixture content | Requirements fixed here; realization by W15 |

None of these is an EL2 or Guest ABI surface. They must not appear in the
[P8-W14](../p8-w14-machine-abi-compatibility/README.md) compatibility matrix.

## 2. `SchedulerScenarioDescriptor`

```text
Name and stability: SchedulerScenarioDescriptor — validation-side data record;
  stage-local; consumed by W16 storage and review tooling.
Purpose and caller: captures one declared scenario (01 §2–§3) so that review,
  execution (W16), and evidence can share one authoritative declaration.
  Callers: W11 review steps; W16 harness; W20 closeout citation.
Inputs / outputs: not applicable (record). Fields:
  scenario_id            S11-A…S11-E (S11-R reserved; must not be scheduled)
  mode                   OneToOne | SharedMn
  topology               (vcpu_count, pcpu_count) — values recorded at
                         implementation time from evidenced facts
  placement              per-vCPU placement declarations; required only when
                         mode == OneToOne (validation rule, see below)
  workload_programs      set of workload invocations from 02 §5 with parameters
  observation_window     declared duration and sampling intent for counters
  progress_markers       the marker lines/counters that must be observed,
  progress_bounds        per-marker time bound and per-sleep slack
  preservation_set       subset of {R1..R6} this scenario must satisfy
  declared_limits        the numeric bounds making P-1..P-4 checkable
Preconditions / postconditions: descriptor validates only when mode, placement,
  and workload set are mutually consistent; an invalid descriptor fails review
  before any execution.
State and ownership change: immutable once reviewed; parameter values are
  recorded in the implementation record with their evidence sources.
Concurrency/allocation context: none at design level; W16's realization owns
  its runtime context.
Errors and failure guarantee: validation failures are review failures; they
  never cause partial scenario execution.
Security/authorization checks: SharedMn descriptors declaring dedicated
  placement are invalid (enforces "no dependence on static pinning" for M:N).
Validation: W11-DV02 (descriptor/schema review); W11-DV04 (executed instances
  match reviewed descriptors).
```

## 3. `evaluate_scheduler_preservation`

```text
Name and stability: evaluate_scheduler_preservation — semantic function fixed
  by this design; realized in W16's harness design.
Purpose and caller: turn one scenario window's observations into a verdict per
  preservation requirement R1–R6 and progress condition P-1..P-5, using only
  P7-W09-evidenced counters/trace fields and Linux-observable markers.
  Caller: W16 harness; W11 review steps 5–6.
Inputs / outputs:
  inputs  — the scenario descriptor, the window's accounting/trace
            observations (as delivered by the evidenced P7-W09 fields), and the
            observed marker sequence with timestamps
  output  — PreservationVerdict { per_requirement: Satisfied | Violated(class,
            evidence_ref), progress: P-1..P-5 each Pass | Fail(evidence_ref),
            coherent: bool }
Preconditions / postconditions: inputs are complete for the declared window
  (missing fields are a P7DependencyIssue, not a silent zero); the function is
  total — every requirement receives a verdict; verdicts cite evidence refs
  into the verification record.
State and ownership change: none; evaluation is read-only over observations.
Concurrency/allocation context: realization-owned; evaluation itself has no
  authorization to mutate hypervisor or scheduler state.
Errors and failure guarantee: missing or incoherent counter evidence yields
  Violated(P7DependencyIssue) for affected requirements, never an
  interpretation default.
Security/authorization checks: counter sources must be the EL2-evidenced
  accounting path; Guest-writable memory is never a counter source.
Logic (outline, not production code):
  for each requirement in descriptor.preservation_set:
    derive observations from the evidenced counter/trace fields per 01 §4;
    compare against declared_limits;
    on mismatch, record Violated with the routing class from 02 §4;
  for each progress marker: check presence within progress_bounds;
  check counter coherence per 01 §5 P-3;
  return the aggregated verdict.
Validation: W11-DV03 (condition review); W11-DV04 (executed verdicts);
  W11-DV05 (accounting coherence review).
```

## 4. `SchedulerIntegrationFailureClass` and `route_scheduler_failure`

```text
Name and stability: SchedulerIntegrationFailureClass — closed enum fixed by
  this design; route_scheduler_failure — total mapping.
Purpose and caller: enforce the plan's rule that failures are dependency
  issues, not redesign authorizations. Callers: W11 review steps; W16 expected-
  outcome assertions; W18 isolation mapping.
Inputs / outputs: a PreservationVerdict violation plus its diagnostic context
  (vm_id, vcpu_id, scenario_id, evidence refs) → exactly one class:
  P7DependencyIssue | P8IntegrationGap | LinuxBehaviorObservation |
  HypervisorInvariantViolation (semantics per 01 §6).
Preconditions / postconditions: total and deterministic for the fixed inputs;
  postcondition: the route's "authorized response" is the only permitted W11
  action for that class.
State and ownership change: none; produces a record entry for the class.
Errors and failure guarantee: unclassifiable input returns a block marker for
  review — never a default class.
Security/authorization checks: HypervisorInvariantViolation must not be
  downgraded to a Guest-facing class; Guest-supplied values may appear only as
  recorded observations, never as routing inputs trusted for control flow.
Validation: W11-DV06 failure-routing review.
```

## 5. Guest workload functional requirements (fixture needs)

W15 realizes these in the pinned fixture's initramfs; W16 invokes them per
scenario. Functional requirements are normative; the Guest-side implementation
belongs to W15. Marker strings are fixed here because W16's expected markers
must be stable across fixture updates.

| Program | Invocation shape | Functional requirement | Stable marker (prefix) |
|---|---|---|---|
| `spin-cpu` | `spin-cpu <seconds>` | CPU-bound loop on its vCPU for the declared duration; writes a start and an end marker; no allocation or I/O in the loop | `SCHED-MARKER spin` |
| `sleep-wake` | `sleep-wake <cycles> <interval>` | Performs the declared number of timed sleeps; writes one marker per completed cycle with elapsed time; exits after the last cycle | `SCHED-MARKER sleep` |
| `sched-fanout` | `sched-fanout <processes> <iterations>` | Forks the declared number of short-lived children that each perform declared iterations of trivial work; parent joins all children and writes one completion marker | `SCHED-MARKER fanout` |
| shell | interactive session | Demonstrates interactive responsiveness in S11-E; the operator/script drives a declared command sequence | session transcript in verification record |

Requirements and constraints:

- Programs take parameters only from the command line; they read no Guest
  memory outside their own process and write no device registers — they must
  not become an MMIO or hypercall probe (that is W12/W18 territory).
- Markers go to the console (W09 path) as plain text lines; W16 matches them
  textually. A missing marker at descriptor deadline is a P-1 failure.
- Elapsed-time values in `sleep-wake` markers are the R2/P-2 evidence source;
  their format (machine-readable, declared units) is fixed in the W15 manifest
  so W16 parsing does not guess.
- Programs are CPU/timer/process tools only; adding an IRQ-storm or memory-
  stress tool here is out of scope (W18 security scenarios and W12 memory
  workloads declare their own needs).

Validation: W11-DV02 (requirements review with W15 manifest consistency);
W11-DV04 (markers observed as declared during execution).
