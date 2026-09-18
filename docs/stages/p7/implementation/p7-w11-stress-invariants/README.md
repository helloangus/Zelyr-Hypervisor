# P7-W11 Scheduler Stress and Invariants — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The stress, invariant, fairness, and placement evidence plan
required by [P7-W11](../../plans/p7-w11-stress-invariants.md).  
**Owner/change context:** P7-W11 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W11. The plan requires evidence
that objectively exposes scheduler races, invariant violations, multi-VM and
overcommit faults, starvation, livelock, and placement violations when present
(P7-V24–V27). This design therefore defines what W11 delivers: four declared
stress-scenario families, an invariant-check catalog evaluated against the
P7-W09 observability contract, an objective failure-signal taxonomy,
repetition/seed/duration and determinism rules, and the evidence layout — it
does not and must not contain results, because no scenario can run before the
P0–P6 foundations and the P7-W02–W10 scheduler behavior exist.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- the scenario families and their parameters →
  [stress scenario matrices](01-stress-scenario-matrices.md);
- the invariant catalog, check integration, and failure classification →
  [invariant checks and failure signals](02-invariant-checks-and-failure-signals.md);
- the ordered execution workflow, validation matrix, and handoff →
  [workflow and acceptance](03-workflow-and-acceptance.md).

Before executing anything, the agent must also follow the Coding Guidelines
preflight, including the repository `AGENTS.md`, documentation index, ADR
baseline, P7 task book, P7-W11 plan, and the P7-W01 recorded input boundary.
This document authorizes no hypervisor source change; W11 is evidence work, and
its implementation record and verification record are separate documents
created only when that work starts.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W11 plan → this design
→ Coding Guidelines. In particular:

- ADR-016 requires evolution from static binding to preemptive M:N while
  preserving pinned/dedicated operation; the stress matrix must exercise both
  classes and must not treat overcommit failure as acceptable at 2×.
- ADR-007 treats Guest workloads as untrusted at the hypervisor boundary even
  when the Validation Guest is project-owned test code; stress scenarios must
  not require weakening any guest-input boundary to run.
- ADR-048/ADR-049 make structured telemetry and layered validation the
  evidence substrate; invariant checks are therefore specified as observations
  over the W09 contract, never as debug prints.
- ADR-003 makes QEMU `virt` the reference test environment; stress evidence is
  QEMU-environment evidence and does not define AArch64 semantics or predict
  Orange Pi 3B (RK3566) behavior.
- The plan excludes formal proof, 4× overcommit as a required gate,
  benchmarking, and performance tuning. W13 owns measurement; W11 owns
  adversarial evidence.

Classification:

- **Required:** scenario families S1–S4 and their gate scenarios; the
  invariant catalog INV-1–INV-8; the failure-signal taxonomy F1–F6;
  repetition/seed/duration minimums; determinism and classification rules;
  evidence layout; coverage review; handoff of required scenarios to W12.
- **Reserved:** additional overcommit depths beyond the declared 4×
  informational rows; longer-duration soak campaigns; fault-injection harness
  tooling beyond declared workload parameters; statistical rigor claims about
  race-detection probability. Each is activated only by a new design decision.
- **Out of Scope:** hypervisor source changes, scheduler mechanism or policy
  changes, new Guest mechanisms or Guest ABI, QEMU runner implementation
  (P0-W09) and CI mechanics, automation composition (W12), performance
  measurement (W13), formal verification artifacts, real-hardware runs.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Assemble scenarios from W08–W10 and their observable conditions | [Scenario matrices](01-stress-scenario-matrices.md) §1–§5 | P7-V24–V27 (W11-DV01) |
| Define stress repetition, race windows, and objective failure signals | [Scenario matrices](01-stress-scenario-matrices.md) §6; [checks and signals](02-invariant-checks-and-failure-signals.md) §3 | P7-V25/V26 (W11-DV03) |
| Integrate invariant checks with trace and diagnostic evidence | [Checks and signals](02-invariant-checks-and-failure-signals.md) §1–§2 | P7-V26 (W11-DV02) |
| Review coverage against P7 risks and stage validation limits | [Workflow](03-workflow-and-acceptance.md) step 5 | P7-V24–V27 (W11-DV04) |
| Record the stress-evidence plan and hand it to QEMU regression | [Workflow](03-workflow-and-acceptance.md) step 6 | W12-DV01 prerequisite (W11-DV05) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs` at
4e631ee): the repository is a P0 documentation scaffold. There is no Cargo
workspace, no Rust source, no test harness, no QEMU runner, and
`docs/stages/p7/verification/` contains only a `.gitkeep` marker. No P0–P6
package is implemented, so every runtime input W11 needs is an assumed
contract with an explicit failure boundary. Sibling P7 detailed designs
(W02–W10) are being prepared in parallel and are referenced by path and package
ID, not by content.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Scenarios assembled from W08–W10 observables | No scheduler, no observability contract, no workload suite exists | Evidenced W08 SMP/idle behavior, W09 trace/accounting contract, and W10 workload suite, each consumed through the W01 input boundary | A stress scenario can only assert against observables an evidenced contract defines | W02–W10 designs; reconciled by P7-W01 | W01 review (P7-V01); per-package verification records |
| Stress evidence exists for P7-V24–V27 | No evidence and no evidence destination | Verification record and artifact tree under `docs/stages/p7/verification/` (created at execution) | Stage validation rows are satisfied only by recorded runs, never by this design | W11 (this design fixes destinations) | W11-DV01–DV04 evidence |
| Objective failure signals | No failure-signaling convention exists in any tracked document | Failure-signal taxonomy F1–F6 bound to W09 diagnostics and the P1 fatal boundary | "Objectively reveal faults" requires named, detectable signals, not prose | W11 (plan work item 2) | W11-DV03 policy review |
| Race windows and repetition rules | Absent | Per-family repetition/seed/duration minimums and determinism rules | Race exposure requires declared sweep policy to be reviewable and repeatable | W11 (plan work item 2) | W11-DV03; per-run seed logs |
| Handoff to W12 | W12 design prepared in parallel; consumes only what W11 declares | Required-scenario list with parameters and evidence locations handed to W12 | W12 automation must compose declared scenarios, not re-derive them | W11 hands off; W12 consumes | W11-DV05; W12-DV01 |

No row requires inventing a crate, target, runner implementation, or scheduler
mechanism, so no decision blocker is outstanding for this design. The runtime
prerequisites (P1–P6 and W02–W10) are blocking-if-absent, not design gaps:
per the task book and [P7-W01](../../plans/p7-w01-entry-contract-reconciliation.md),
an absent or contradictory input blocks the affected scenario and is recorded,
never repaired inside W11.

## Resolved design decisions and their authority

1. **W11 owns scenario semantics and evidence rules; automation mechanics
   belong to W12.** The plan's handoff wording ("W12 consumes the required
   stress scenarios; evidence remains under verification") assigns scenario
   definition to W11 and composition/automation to W12. W11 therefore
   authorizes no runner, script, CI workflow, or harness implementation.
2. **Scenario identification scheme S1–S4 with stable sub-IDs.** Stage-local
   design freedom owned by this design (rationale: the task book names four
   evidence classes P7-V24–V27; stable IDs are required so that W11 evidence,
   W12 automation cases, and W14 documents cross-reference without ambiguity).
3. **Stress payloads are parameterizations of the P7-W10 workload suite.**
   Plan work item 1 assembles scenarios from W08–W10; W11 defines no new Guest
   mechanism, workload program, or Guest ABI. If a required payload does not
   exist in the W10 suite, that is a recorded blocked prerequisite for the
   affected scenario, not license to invent one here.
4. **Invariant catalog INV-1–INV-8 stated as checkable properties over the
   P7-W09 observability contract.** The checks restate the task book's P7-V04
   invariants and lifecycle/placement rules in observable form; the mechanism
   that emits the underlying trace and counters remains owned by the W09 and
   scheduler designs. Rationale: the plan says "integrate invariant checks
   with trace and diagnostic evidence," which requires an observable-level
   catalog independent of implementation mechanics.
5. **Failure-signal taxonomy F1–F6 and the determinate-classification rule.**
   A run that cannot be classified as pass or a specific failure signal is
   failed-with-artifacts, never passed by silence. Plan work item 2 requires
   "objective failure signals"; this rule is what makes them objective.
6. **Repetition/seed/duration minimums are policy floors, not tuning targets.**
   Values in [the scenario matrices](01-stress-scenario-matrices.md) §6 are
   owned by this design as minimums (rationale: a reviewable floor is needed
   before any run exists; implementation may raise a value with recorded
   rationale but never lower one without a new design decision).
7. **4× overcommit is reported separately and is not a gate.** Directly from
   the plan's out-of-scope list; the matrix marks those rows informational.
8. **Race findings do not require deterministic reproduction to stand.** A
   detected corruption, lost wakeup, or starvation event is a finding when its
   objective signal fires; reproduction attempts follow §4 of
   [the checks-and-signals file](02-invariant-checks-and-failure-signals.md).
   Rationale: demanding reproduction would convert real faults into
   unreviewable noise, contradicting the plan's goal.
9. **Evidence layout and destinations.** Evidence is recorded in
   `docs/stages/p7/verification/p7-w11-stress-invariants-verification.md` with
   per-run artifacts under
   `docs/stages/p7/verification/p7-w11-stress-invariants-artifacts/`;
   implementation decisions are recorded in
   `docs/stages/p7/implementation/p7-w11-stress-invariants-record.md`. This
   follows the [documentation index](../../../../README.md) stage
   layout (docs/README.md routing); the files are created only when work
   starts and are not part of this design.

## Work breakdown and loading order

1. Read [the scenario matrices](01-stress-scenario-matrices.md) to understand
   what each family proves, its composition inputs, and its repetition/seed/
   duration floors.
2. Read [the checks and failure signals](02-invariant-checks-and-failure-signals.md)
   to understand how a run is judged and classified.
3. Execute the steps in [the workflow](03-workflow-and-acceptance.md) in
   order; steps 1–2 fix concrete parameters and verify prerequisites, steps
   3–4 run and analyze, steps 5–6 review coverage and hand off to W12.
4. Record actual commands, environment, per-run seeds, outputs, and run/not-run
   status in the verification record; record parameter selections and
   deviations in the implementation record. Neither this design nor a written
   record may claim W11 complete; completion is claimed only in verification
   material with evidence for P7-V24–V27.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
persistent data layout, Guest-visible interface, scheduler mechanism, QEMU
runner, script, or CI workflow is designed or authorized by W11. The only
machine-facing surface this package touches is evidence it records; every
execution surface it uses (runner entry, trace namespace, workload programs)
is owned by P0-W09, P0-W13/P7-W09, and P7-W10 respectively and consumed as an
assumed contract. Declaring or implementing any of those here is a scope
conflict and must be stopped at review.

## Downstream handoff

- **P7-W12** receives the required-scenario subset (S1a, S1b, S2a–S2e core
  offsets, S3 core, S4a–S4c) with parameters, pass conditions, and evidence
  locations, per [the workflow](03-workflow-and-acceptance.md) step 6. W12 may
  compose regression cases from these declarations; it may not assume
  additional scenarios, shorter evidence, or results beyond what the
  verification record contains.
- **P7-W14** receives the stress evidence index entries, the proven/limitation
  statements, and any failed or blocked scenario findings, for the closure and
  P8 handoff review (P7-V30).
- **P8** never consumes W11 directly; it receives only what P7-W14 records as
  evidenced. Stress evidence supports containment and stability claims at the
  boundary stated in each scenario's "does not prove" column; it never becomes
  a machine-ABI or hardware-behavior commitment.
