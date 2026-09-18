# P7-W12 Automated QEMU Scheduler Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The automated QEMU regression boundary for declared scheduler
matrices required by [P7-W12](../../plans/p7-w12-qemu-regression.md).  
**Owner/change context:** P7-W12 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W12. The plan requires that
every declared scheduler scenario — 1/1, 1/2, 2/4, 4/4, and 4/8 CPU/vCPU
topologies plus pause/resume, wakeup, affinity, and pinning regressions —
return a **determinate result with diagnosable failure artifacts** (P7-V28).
This design defines the regression case matrix, the determinate-outcome and
result-classification contract, the runner-consumption boundary, timeout and
repetition policy, and the evidence layout. It defines no results: a case is
reported as passed only in verification material, after it has actually run,
and a passing QEMU run never proves hardware semantics.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- the regression matrix, case schema, runner contract, and result taxonomy →
  [regression matrix and runner contract](01-regression-matrix-and-runner-contract.md);
- the ordered workflow, validation matrix, and handoff →
  [workflow and acceptance](02-workflow-and-acceptance.md).

Before executing anything, the agent must also follow the Coding Guidelines
preflight, including the repository `AGENTS.md`, documentation index, ADR
baseline, P7 task book, P7-W12 plan, and the P7-W01 recorded input boundary.
This document authorizes no hypervisor source change and no CI mechanics.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W12 plan → this design
→ Coding Guidelines. In particular:

- ADR-003 makes QEMU `virt` the deterministic reference/CI platform; ADR-045's
  support-tier model keeps QEMU evidence distinct from Tier-1 hardware
  evidence. The plan states the limit directly: this package must not claim
  QEMU proves hardware semantics, must not select CI mechanics, and must not
  report tests as passed before evidence exists.
- ADR-048/ADR-049 route regression evidence through structured telemetry; case
  failure artifacts therefore include the P7-W09 diagnostic contract's output,
  not serial prose alone.
- The P0-W09 QEMU automation entry baseline establishes the single, governed
  QEMU runner entry (runner responsibility, parameter carrying, serial
  capture, timeout, exit status, evidence collection). W12 consumes that
  entry and creates no second runner; see the P0-W09 plan at
  [`../../../../stages/p0/plans/p0-w09-qemu-automation-entry-baseline.md`](../../../../stages/p0/plans/p0-w09-qemu-automation-entry-baseline.md).
- The plan's work sequence orders this package after P7-W11; the regression
  composes declared W11 scenarios and P7-W10 workload regressions. It does
  not re-derive scenario semantics.

Classification:

- **Required:** the case matrix R-01–R-14; the case schema; the result
  taxonomy (PASS / FAIL / INFRA-BLOCKED / NOT-RUN); determinate-outcome and
  flakiness rules; timeout policy floors; artifact and evidence layout; the
  coverage and QEMU-limit review; closeout inputs to P7-W14.
- **Reserved:** additional topology points beyond the declared five;
  host-side parallel execution scheduling of cases; result trending across
  commits; case-parameter auto-generation. Each requires a new design
  decision.
- **Out of Scope:** CI workflow implementation (P0-W20 scope and explicitly
  outside this plan), QEMU runner implementation and its command surface
  (P0-W09), hypervisor source changes, new Guest mechanisms (P7-W10 owns
  workloads), stress-evidence production (P7-W11), performance measurement
  (P7-W13), real-hardware validation.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect P0 runner governance and W11 scenarios | [Matrix and runner contract](01-regression-matrix-and-runner-contract.md) §1–§2 | W12-DV01 |
| Define automation inputs, determinate outcomes, timeouts, artifact/evidence locations | [Matrix and runner contract](01-regression-matrix-and-runner-contract.md) §2–§5 | P7-V28 (W12-DV02) |
| Integrate scenario diagnostics with prior telemetry requirements | [Matrix and runner contract](01-regression-matrix-and-runner-contract.md) §4 | P7-V28 (W12-DV02) |
| Review matrix coverage and the QEMU-versus-architecture limit | [Workflow](02-workflow-and-acceptance.md) step 4 | P7-V28 (W12-DV03) |
| Record acceptance evidence requirements and hand off closeout inputs | [Workflow](02-workflow-and-acceptance.md) step 5 | W14-DV prerequisite (W12-DV04) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs` at
4e631ee): the repository is a P0 documentation scaffold — no workspace, no
sources, no runner, no CI (`.github/workflows/` holds only a `.gitkeep`), and
`docs/stages/p7/verification/` holds only a `.gitkeep`. P0–P6 are planned, not
implemented; sibling P7 designs W01–W11 are being prepared in parallel and are
consumed by path and package ID only.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every declared scenario returns a determinate result | No runner, no case declarations, no results | P0-W09 runner entry (assumed contract) plus the W12 case schema and result taxonomy defined here | Determinacy requires a governed execution surface and named outcome classes | P0-W09 (runner); W12 (case semantics) | W12-DV01/DV02 |
| Diagnosable failure artifacts | No artifact convention exists | Per-case artifact set: serial capture, W09 diagnostic content, case parameters | A failure without reproducible context is not diagnosable | W12 (layout); P0-W09 (capture); P7-W09 (diagnostics) | W12-DV02 |
| Regression cases for 1/1 … 4/8 and pause/wakeup/affinity/pinning | No scenarios exist at runtime | Evidenced P7-W02–W08 behavior, W10 workload suite, W11 required scenarios | Cases assert declared scheduler behavior; without it they are unrunnable | P7-W01 boundary over W02–W11 | P7-V02–V27 evidence; W12-DV03 |
| Automation without selecting CI mechanics | No CI exists; plan excludes CI selection | Automation inputs expressed as runner-consumable case declarations, recorded via the implementation record | The regression must be runnable by a person or a later CI job without W12 owning CI | P0-W09 reserved parameter space; W12 declares data | W12-DV01 |
| Closeout inputs to W14 | None | Evidence index entries and limitation statements handed to P7-W14 | P7-V30 requires the regression evidence to be reviewable at closure | W12 hands off; W14 consumes | W12-DV04 |

No row requires inventing a crate, target, runner implementation, or CI
workflow. The runtime prerequisites are blocking-if-absent per the P7-W01
boundary; an absent or divergent runner entry (P0-W09) blocks this package.

## Resolved design decisions and their authority

1. **Single runner entry; W12 declares data, not commands.** P0-W09
   establishes the unique QEMU automation entry and reserves the parameter
   carrying space for later boot/SMP/regression parameters. W12's automation
   inputs are therefore case declarations (topology, payloads, markers,
   timeouts, repetitions) consumed by that entry; W12 fixes no command-line
   spelling. Rationale: plan out-of-scope item "selecting CI mechanics" plus
   P0-W09's single-entry governance.
2. **Case identification R-01–R-14 with a fixed case schema.** Stage-local
   design freedom owned by this design (rationale: W14's evidence index,
   P7-V28 review, and later P8 regression consumers need stable case IDs).
3. **Result taxonomy PASS / FAIL / INFRA-BLOCKED / NOT-RUN, with F-signal
   reuse from P7-W11.** Determinate means: the case ends in PASS or a named
   failure signal with artifacts — never in an unclassified state. Rationale:
   P7-V28 requires "determinate results with diagnosable failure artifacts";
   reusing the W11 F-signal taxonomy ([P7-W11 checks](../p7-w11-stress-invariants/02-invariant-checks-and-failure-signals.md))
   keeps the two evidence layers comparable instead of inventing a second
   vocabulary.
4. **Flakiness rule: non-reproducible results fail.** A case that does not
   reproduce its recorded result across its declared repetitions is FAIL with
   artifacts. Rationale: a regression entry point that sometimes passes is
   not determinate; there is no flaky-pass state in P7.
5. **Regression cases are short-form derivatives of W10/W11 scenarios.** The
   regression keeps each required scenario's pass condition and markers but
   may run fewer repetitions than the W11 floors. Rationale: the plan makes
   W12 a regression boundary, not a second stress campaign; deep evidence
   stays with W11. Any reduction is bounded by the case schema (declared
   repetition count per case) and recorded.
6. **Timeout policy as design floors.** Each case class carries a timeout
   floor; values are fixed at implementation at or above the floor and
   recorded. Rationale: determinacy requires an upper bound on every case;
   the exact value depends on the environment declared at execution time.
7. **QEMU-limit statement is part of the deliverable.** Every evidence record
   states that results hold for the declared QEMU `virt` environment and do
   not define AArch64 semantics or predict RK3566/Orange Pi 3B behavior
   (ADR-003/ADR-045; plan out-of-scope wording).
8. **Evidence layout.** Case results are recorded in
   `docs/stages/p7/verification/p7-w12-qemu-regression-verification.md` with
   per-case artifacts under
   `docs/stages/p7/verification/p7-w12-qemu-regression-artifacts/<case-id>/`;
   implementation decisions in
   `docs/stages/p7/implementation/p7-w12-qemu-regression-record.md`. Files
   are created only when work starts.

## Work breakdown and loading order

1. Read [the matrix and runner contract](01-regression-matrix-and-runner-contract.md)
   for the case list, schema, result taxonomy, and runner boundary.
2. Execute [the workflow](02-workflow-and-acceptance.md) steps in order:
   verify prerequisites, fix concrete case parameters, run the matrix,
   classify results, review coverage and limits, hand off closeout inputs.
3. Record actual runs, outcomes, and artifacts in the verification record and
   parameter selections in the implementation record. Neither this design nor
   any record may claim W12 complete; completion is claimed only in
   verification material with per-case evidence for P7-V28.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
Guest-visible interface, scheduler mechanism, QEMU runner implementation,
CI workflow, or script is designed or authorized by W12. The only artifacts
W12 creates are case declarations recorded through the implementation record,
evidence under `verification/`, and index entries consumed by P7-W14. The
execution surface (runner), capture mechanics, and trace namespace remain
owned by P0-W09 and P0-W13/P7-W09; workload programs remain owned by P7-W10.

## Downstream handoff

- **P7-W14** receives the per-case evidence index entries, the matrix
  coverage statement, the QEMU-limit statement, and any FAIL/INFRA-BLOCKED
  findings with artifacts, as closure inputs (P7-V30).
- **P8** never consumes W12 directly. Through P7-W14, P8 may rely only on the
  evidenced regression entry points and their recorded results; the P8
  automated Linux regression (P8-W16) composes its own matrix and inherits
  method, not results. No P7 regression outcome becomes a machine-ABI or
  hardware-behavior commitment.
