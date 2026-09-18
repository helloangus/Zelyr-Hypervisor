# P3-W02 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W02 detailed design](README.md).

## 1. Validation matrix

Each item is recorded as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason in the
verification record. QEMU items appear only where W02's scope produces
them; the repeated matrix and cold-boot coverage belong to
[P3-W13](../p3-w13-qemu-smp-regression/README.md).

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W02-DV01 | Start path uses recorded P2 facts only (→ P3-V02) | boundary review | inspect `psci_cpu_on` and the capability record path | conduit/function id sourced from `StartCapabilityFacts`; no platform name or inlined identifier; single `unsafe` boundary with `SAFETY` | capability-driven call structure; not that any firmware accepts it |
| W02-DV02 | Request/sequencer correctness (→ P3-V02) | host-side unit tests | mocked call boundary + fake mailboxes: error, success, secondary-failure, timeout; isolation and no-re-dispatch assertions | exhaustive outcome mapping; per-candidate isolation; exactly one dispatch per CPU; phase-gate precondition asserted | sequencer logic; not firmware behavior |
| W02-DV03 | Entry revalidation and identity confirmation (→ P3-V02) | code review + QEMU identity evidence | review the entry checklist against the P1 contract; inspect per-CPU identity lines in bring-up captures | every check fails closed into `park_failed`; captured secondaries report the correct logical/hardware identity | the entry path holds on the reference platform; not arbitrary-firmware behavior |
| W02-DV04 | CPU/phase attribution model (→ P3-V02) | unit tests + review | outcome/phase mapping tests; review that every terminal outcome names CPU + phase (+ cause) | no terminal outcome without attribution; phases ordered; secondary phase wins over requester inference | reporting model; not diagnosability of hardware faults |
| W02-DV05 | Timeout termination (→ P3-V02) | host-side watcher tests | deterministic fake progress with the poll bound; verify terminal outcome and failure transition | no-arrival within bound yields `TimedOut` + registry failure report; no unbounded wait exists | termination property; not wall-clock timeout measurement (stated limitation, P6 Reserved) |
| W02-DV06 | Repeatable bring-up per declared count (→ P3-V02) | QEMU capture | one boot per count (1/2/4/8) via the P0 entry path; all expected secondaries `Started`; distinct stacks; correct attribution | every expected secondary reports `Started`; diagnostics complete and CPU-attributed | single-pass bring-up on the reference platform per configuration; not repeated-boot stability (W13), not hardware platforms |
| W02-DV07 | Induced failure path (→ P3-V02) | QEMU capture | CPU_ON targeting an out-of-inventory identity (absent CPU) | `RequestRejected` outcome with named PSCI error and CPU attribution; no retry; rest of bring-up unaffected | failure isolation and diagnosability for request-phase rejection; does not prove on-target timeout or firmware-denied starts on real hardware |
| W02-DV08 | Startup contract recorded (→ W02 closure) | closure review | verify implementation record and handoff checklist below | consumer contracts (W03/W04/W05/W09/W13) match implementation; no completion claim | handoff readiness; not downstream correctness |

Passing W02-DV06 once per configuration does not satisfy P3-V13
(repeatability belongs to the regression package). A bring-up capture
never proves P3-V03 lifecycle admissibility by itself; the lifecycle
invariants are P3-W03's evidence.

## 2. Error, security, and observability model

- **Error model.** Terminal outcomes, never retries: request rejection
  (named PSCI error), timeout (bounded poll), secondary-reported failure
  (named phase and cause). Individual failures isolate; only systemic
  failures (missing capability, phase-gate violation) fail closed before
  any dispatch. A CPU that cannot prove its identity writes nothing
  shared and parks.
- **Security.** The PSCI call boundary is the package's only `unsafe` and
  its audit item; conduit and function ids come from recorded platform
  facts (ADR-044), never literals or names. No guest-reachable surface
  exists; guest PSCI virtualization is P8. The context id is chosen by the
  hypervisor and cross-checked against MPIDR, so a firmware-side anomaly
  cannot inject an identity.
- **Observability.** Every candidate CPU yields at least one CPU-attributed
  diagnostic; failures carry phase and cause. Parked CPUs emit before
  parking; if a CPU fails pre-console, the requester's timeout/registry
  side still attributes it. Event catalog and contention/timing metrics
  belong to P3-W11; W02 provides the content those events carry.

## 3. Handoff checklist

Before handing W02 to a reviewer:

- the exact changed-file list, with crate/module placement traced to the
  approved workspace design and the assembly boundary identified;
- W02-DV01–DV08 evidence paths with run status, including explicit
  not-run entries (counts not reachable in the environment;
  timeout-on-target limitation; matrix deferred to W13);
- the fixed values recorded with rationale: `PROVISIONAL_STACK_SIZE`,
  `POLL_BOUND`;
- confirmation that new `unsafe` is limited to the PSCI call boundary and
  inventoried per P0 unsafe governance;
- confirmation that no lifecycle state machine, per-CPU runtime, barrier
  primitive, lock, IPI path, or guest PSCI surface was added;
- consumer contract confirmations: W03 (transition request names and
  ownership), W04 (stack ownership transfer/quarantine), W05 (dispatch
  after publication; outcome map as attempted-set input), W09 (CPU
  attribution from the first entry instruction), W13 (scenario and
  failure-input definitions);
- any recorded blocker (P2 capability facts absent for a platform; P1
  baseline reproducibility question; sibling-contract coordination) with
  its decision owner.
