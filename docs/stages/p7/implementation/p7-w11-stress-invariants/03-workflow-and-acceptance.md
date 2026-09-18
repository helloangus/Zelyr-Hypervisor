# P7-W11 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W11 detailed design](README.md).

## 1. Preconditions and failure boundary

Before executing any step, the implementer verifies it has loaded the documents
named in the parent README and confirms, through the P7-W01 recorded input
boundary, that the following are evidenced rather than merely planned:

- P1–P6 runtime, diagnostics, timer/event, and notification contracts;
- P7-W02–W08 scheduler behavior contracts and P7-W09 observability contract;
- the P7-W10 workload suite and its markers;
- the P0-W09 QEMU runner entry (the single governed execution surface).

Stop and record a blocker instead of improvising when any of the following
occurs:

- a prerequisite contract is absent, contradictory, or unevidenced — the
  affected scenario is **blocked** and recorded; W11 does not repair upstream
  scope;
- a required W09 observable or diagnostic item is missing — the affected
  invariant is recorded as blocked with the gap named;
- a required payload or marker does not exist in the W10 suite — the affected
  scenario is blocked; W11 does not author Guest code;
- running appears to require changing hypervisor source, the runner, or CI —
  that is a scope violation; stop and record;
- a maintainer decision would be needed to change a floor, a taxonomy entry,
  or the class of any scenario row — follow the design-change path of this
  design, not a local shortcut.

## 2. Ordered implementation steps

### Step 1 — bind observables and fix concrete parameters

Target: implementation record
(`docs/stages/p7/implementation/p7-w11-stress-invariants-record.md`, created
in this step).

Work: bind the INV catalog entries of
[the checks file](02-invariant-checks-and-failure-signals.md) to the concrete
trace fields, counters, and diagnostic items the evidenced W09 contract
declares, and fix the concrete repetition counts, durations, window sizes, and
seed sequences for every scenario row, each at or above the floors in
[the matrices](01-stress-scenario-matrices.md) §6. Record any value raised
above a floor with rationale.

**Acceptance:** the record contains a complete binding table (INV → observable)
and a per-scenario parameter table; no floor is lowered.  
**Failure/blocker:** a missing observable or an unresolvable parameter
conflict is a recorded blocker for the affected scenario; nothing is silently
dropped from the matrix.

### Step 2 — verify per-scenario runnability

Target: implementation record.

Work: for each scenario row, confirm the composition is realizable from W10
payloads, W03 placement semantics, and the runner entry, and classify each row
runnable or blocked with the named missing input.

**Acceptance:** every row has an explicit runnable/blocked status before any
execution; blocked rows name the owner of the missing input.  
**Failure/blocker:** a runnable classification that later proves wrong during
execution reverts the row to blocked with the discovery recorded.

### Step 3 — execute the scenario matrix

Target: verification record
(`docs/stages/p7/verification/p7-w11-stress-invariants-verification.md`) and
the artifact directory.

Work: run each runnable row per its parameters through the P0-W09 runner
entry, in the recorded schedule. Capture per run: environment declaration,
parameters and seeds, serial capture, trace extract, checker evaluations, and
outcome classification (pass or F-signal per
[the taxonomy](02-invariant-checks-and-failure-signals.md) §3).

**Acceptance:** every executed run has a complete artifact set and a
determinate classification; every floor of §6 is met for gate rows.  
**Failure/blocker:** F6 outcomes follow the taxonomy rules; infrastructure
faults are recorded and re-run, never counted as scheduler failures.

### Step 4 — analyze and classify

Target: verification record.

Work: for each row, evaluate the pass condition, record failures as findings
with the artifact set of §2 of the checks file, perform reproduction attempts
per §4 of the checks file, and write the per-row verdict (passed / failed /
blocked / not run) with the reason.

**Acceptance:** the verification record states, per row, what was run, what
passed or failed, what it proves and does not prove, using the statements of
[the matrices](01-stress-scenario-matrices.md).  
**Failure/blocker:** a gate-row failure (S1a/S1b, S2 required rows, S3, S4
required rows) is a P7 stage finding to escalate through W14 closure; it is
never reclassified as informational locally.

### Step 5 — coverage review against P7 risks and stage limits

Target: verification record (review section).

Work: review the executed matrix against the task-book validation rows
P7-V24–V27, the plan's risk list (races, invariants, multi-VM/overcommit,
fairness, placement), and the stage limits (no formal proof, no benchmark
claim, no hardware claim). Name explicitly which risk classes the evidence
does not cover.

**Acceptance:** the review states coverage and non-coverage per P7-V24–V27
and flags any row that cannot satisfy its stage row as blocked/failed with
reason.  
**Failure/blocker:** a coverage gap that only a design change could close is
an open question for the closure owner (W14), not a silent omission.

### Step 6 — hand off required scenarios to W12

Target: implementation record (handoff section).

Work: emit the required-scenario handoff: S1a, S1b rows, S2a–S2e, S3a/S3b,
S4a–S4c with their fixed parameters, pass conditions, F-signals, and evidence
locations, as the consumable input for
[P7-W12](../p7-w12-qemu-regression/README.md) automation composition.

**Acceptance:** the handoff lists exactly the scenarios W12 may compose and
points to their evidence; it grants W12 nothing beyond it.  
**Failure/blocker:** a scenario that ended blocked is handed off as blocked
with its blocker; W12 composes only runnable, evidenced scenarios.

## 3. Validation matrix

Every row is planned evidence with an objective condition; none asserts that
a run has happened. Each is recorded as **passed**, **failed**, **blocked**,
or **not run** with command/input, environment, timestamp, and reason.

| ID | Review or test | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W11-DV01 → P7-V24 | S1 matrix execution per [matrices](01-stress-scenario-matrices.md) §2 | S1a/S1b runs at floors with INV checks active | No F1–F6 on gate rows; completion and accounting closure within declared duration | M:N and 2× overcommit sustained at gate configuration in QEMU; not formal correctness, not hardware, not 4× (informational) |
| W11-DV02 → P7-V25 | S2 matrix execution | Race sweeps per §3 with seeds recorded | No lost wakeup, duplicate running, or pause-race violation across swept windows | Window coverage exercised; not race-freedom beyond the sweep, not hardware timing |
| W11-DV03 → P7-V25/V26 | Signal-policy and repetition review | Inspect per-run records against the taxonomy and floors of [checks](02-invariant-checks-and-failure-signals.md) §3–§4 and [matrices](01-stress-scenario-matrices.md) §6 | Every run determinately classified; floors met; seeds recorded | Evidence discipline holds; not that more repetition would not find faults |
| W11-DV04 → P7-V26/V27 | S3/S4 execution and coverage review | Workflow step 5 | Invariants hold under load; no starvation window; placement compliance holds | Invariant and placement evidence at declared durations; not fairness quality beyond no-starvation |
| W11-DV05 → W12 prerequisite | Handoff review | Workflow step 6; read as W12 would | Required scenarios, parameters, and evidence locatable and consumable | Handoff completeness; not that W12 has run |

No validation here satisfies P7-V28 (W12), P7-V29 (W13), or P7-V30 (W14), and
none may be reported as doing so.

## 4. Error, security, and observability model

W11 adds no hypervisor error path. Its security posture is negative: stress
scenarios must never require weakening a guest-input boundary, disabling a
check in a production path, or granting the harness authority beyond a normal
Guest (ADR-007). If a scenario appears to need such a weakening, that is a
stop condition, not a harness feature. Containment itself is under test
(INV-8): an injected Guest fault is expected to stay within its VM context.

Observability is the deliverable: per-run environment declarations, seeds,
checker evaluations, and the F-signal taxonomy are the only accepted proof
surface. A fault that escapes to F1 must arrive with the P1 fatal diagnostic
boundary's output; W11 never substitutes prose for a diagnostic artifact.

Documentation-grade failure reporting: a blocked prerequisite, an incomplete
environment declaration, or a lowered floor fails the associated review and is
recorded as such.

## 5. Handoff checklist

Before handing W11 results to a reviewer, provide:

- the implementation record with the binding table and per-scenario parameters;
- the verification record with per-row verdicts and run/not-run entries for
  the full matrix, including explicit blocked rows and reasons;
- artifact-set completeness for every F1–F6 failure (diagnostic, serial,
  parameters/seeds, checker report);
- confirmation that no hypervisor source, runner, script, CI workflow, or
  Guest-code change was made by W11;
- the W12 handoff (step 6) and the W14 items: findings, limitations,
  non-coverage statements, and open questions — without resolving W12/W14
  contracts here.
