# P6-W11 Validation Guest Interrupt Suite — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The maintained Guest-side verification asset that independently
observes the P6 timer and virtual-interrupt boundary: EL1 exception-vector
handling, scenario set VG-TIMER-01–VG-TIMER-04, VG-IRQ-01–VG-IRQ-06, Host-SGI
support, and conditional multi-vCPU timer/IRQ isolation, per
[P6-W11](../../plans/p6-w11-validation-guest-interrupt-suite.md).  
**Owner/change context:** P6-W11 suite handoff; this design owns the scenario
matrix, result-collection protocol, and harness contract as increments on the
P4 Validation Guest asset and P6 Host contracts — it owns no Host interrupt
mechanism.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W11. It is a validation
package: it defines what each Guest scenario proves and does not prove, how
results are collected and classified (including negative and blocked
outcomes), and where evidence lands. It produces **no results** — results
belong to the verification record after real runs.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scenario-matrix.md](01-scenario-matrix.md) — the full scenario matrix
  (trigger, expected Guest-observable, pass condition, proves/does-not-prove,
  repetition and failure-injection semantics, evidence destination), the
  result-record schema, and the BLOCKED/NOT-RUN rules. Load before writing or
  running any scenario.
- [02-harness-and-workflow.md](02-harness-and-workflow.md) — the harness
  contract, ordered implementation steps, validation matrix, and handoff
  checklist.

Before editing, follow the Coding Guidelines preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P6 task
book](../../task-book-v0.1.md), and the [P6-W11
plan](../../plans/p6-w11-validation-guest-interrupt-suite.md). This document
is proposed design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W11 plan → this
design → Coding Guidelines. In particular:

- ADR-009 makes a Rust bare-metal Validation Guest the first EL1 guest and
  the mechanism-verification instrument; ADR-049's "guest self-test" layer is
  exactly this package; the plan states its purpose: independent Guest
  observation rather than reliance on Hypervisor logs.
- The task book (§1 Required) demands the declared timer, vIRQ,
  exception-vector, and Host-SGI scenarios, with two-vCPU cases **only when
  an evidenced upstream multi-vCPU contract exists** (P6-V18/P6-V19 record a
  stage block otherwise).
- The plan excludes: Linux Guest tests, machine ABI/DTB work, production
  Guest drivers, substituting Hypervisor logs for Guest observation, and
  implementation of the underlying GIC/timer mechanisms.
- The Guest asset itself is a P4 deliverable
  ([P4-W05](../../../p4/plans/p4-w05-validation-guest.md)); W11 extends
  its scenario surface under that contract. The interrupt semantics asserted
  by the scenarios are the W10 contract
  ([§7](../p6-w10-interrupt-semantics/01-interrupt-semantics-contract.md)).

Classification:

| Class | Items |
|---|---|
| **Required** | EL1 exception-vector handling coverage; VG-TIMER-01–04; VG-IRQ-01–06; VG-SGI-01; conditional VG-SMP-01/VG-SMP-02 with BLOCKED mode; the result-record schema and PASS/FAIL/BLOCKED/NOT-RUN taxonomy; Guest-observation primacy rule; harness contract; evidence destinations. |
| **Reserved** | Temporary Validation Guest interrupt layout values (task book §1 Reserved; owned by this design within P6, never a machine ABI); scenario iteration counts and timeouts (declared at run time in evidence); additional diagnostic scenarios beyond the declared set. |
| **Out of Scope** | Linux Guest tests; machine ABI/DTB; production Guest drivers; Host-side GIC/timer mechanism implementation (W01–W09); storm/robustness cases (W12's FI matrix; W11 may only cross-cite); scheduler behavior (P7); real-hardware claims. |

## Requirement → design-location → acceptance mapping

| Plan requirement (P6-W11) | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W04, W06, W10, P4 Validation Guest facts, declared validation matrix | [workflow](02-harness-and-workflow.md) step 1 | prerequisite verdicts in the implementation record |
| Approved design for Guest-side scenario observability, result collection, negative/blocked outcomes | [matrix](01-scenario-matrix.md) §1–§3 | design approved; every scenario row complete; taxonomy explicit |
| Integrate each scenario with applicable Host SGI, timer, vIRQ, masking, maintenance contracts | [matrix](01-scenario-matrix.md) §2 prerequisite column; [workflow](02-harness-and-workflow.md) step 3 | each scenario names its contract dependencies and their evidence status |
| Expected evidence for timer, HVC/exit, vIRQ, mask/unmask, repeated, concurrent, conditional multi-vCPU cases | [matrix](01-scenario-matrix.md) §2; [workflow](02-harness-and-workflow.md) §3 (W11-DV01–DV07) | P6-V09–P6-V19 evidence produced and classified |
| Suite tests Guest-visible behavior without defining P8 Linux machine contracts | [matrix](01-scenario-matrix.md) §4; [workflow](02-harness-and-workflow.md) step 5 | exclusion review recorded; no machine-ABI artifact touched |
| Scenario status/evidence references recorded; suite handed to W13 and downstream regression users | [workflow](02-harness-and-workflow.md) step 6; §5 handoff checklist | verification record complete with status per scenario |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p6-implementation-designs`):
`guests/validation-aarch64/` contains only a `.gitkeep`; no Guest crate,
scenario code, harness, or evidence exists. P0–P5 are planned but
unimplemented, so the Validation Guest asset, its debug/result channel, and
the multi-vCPU capability are **assumed contracts** from their plans. The
P6-W01–W10 designs are being written in parallel and are referenced by slug
and ID only. Each ledger row states the missing foundation the plan outcome
necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Guest-side asset independently observes the P6 boundary | `guests/validation-aarch64/.gitkeep` only | P4-W05 Validation Guest asset: EL1 entry, exception vectors, debug/result channel | Scenarios are Guest code; without the P4 asset and channel there is nothing to extend or observe with | P4-W05 design; failure boundary in [workflow](02-harness-and-workflow.md) §1 | P4 verification evidence; step 1 verdict |
| EL1 exception-vector handling covered | No Guest vectors exist | P4/P1 exception-vector contract realized in the Guest asset; W11 adds the scenario-facing handlers | Vector behavior is both a test subject (P6-V09–V19 prerequisites) and an observation precondition | P4-W05/P1-W05 contracts; W11 for scenario handlers | VG-IRQ-01 negative-path evidence |
| VG-TIMER-01–04, VG-IRQ-01–06 produce declared evidence | No scenarios exist | W05/W06 timer contracts, W07/W08/W09 vIRQ/presentation/maintenance contracts, W10 semantics §7 | A scenario can only assert behavior an upstream contract defines | W05–W10 designs | per-scenario evidence in the verification record |
| Host-SGI support in the suite | No SGI path exists | W04 Host SGI/routing contract; W07 injection path | VG-SGI-01 exercises the Host-SGI-to-vIRQ chain Guest-visibly | W04 (`../p6-w04-smp-interrupt-routing-sgi/README.md`) | VG-SGI-01 evidence; P6-V04–V06 remain W04/W13-owned |
| Conditional multi-vCPU isolation (P6-V18/P6-V19) | No multi-vCPU capability or evidence | Evidenced upstream multi-vCPU contract (P3-W14 handoff; P4 vCPU isolation) | Task book conditions two-vCPU rows on evidenced capability; otherwise a stage block must be recorded | [P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md); P4-W09 | VG-SMP-01/02 evidence or BLOCKED record |
| Scenario status and evidence references recorded; suite reusable downstream | No verification record exists | Result taxonomy, record schema, and evidence destinations (this design) | Downstream users need classified, located, non-duplicated evidence | W11 (this design); W13 composition | verification record + handoff checklist |

No row requires inventing a machine ABI, a Linux test, or a Host mechanism;
the temporary interrupt layout is fixed in
[01-scenario-matrix.md](01-scenario-matrix.md) §1 as stage-local design
freedom with an explicit non-freeze statement.

## Resolved design decisions and their authority

1. **Scenario identity and matrix authority.** The scenario set is exactly:
   VG-TIMER-01–04, VG-IRQ-01–06, VG-SGI-01, VG-SMP-01, VG-SMP-02 — the IDs
   fixed by the plan (VG-TIMER/VG-IRQ ranges, Host-SGI support, conditional
   multi-vCPU) plus named SGI/SMP rows. [01-scenario-matrix.md](01-scenario-matrix.md)
   §2 is the sole authority for each scenario's trigger, expectation, pass
   condition, and proof boundary. Rationale: prevents suite drift and makes
   W13 composition deterministic. Authority: plan scope; task book §5
   traceability.
2. **Result taxonomy with four distinct states.** Every scenario execution
   ends PASS, FAIL, BLOCKED, or NOT-RUN, defined in
   [01-scenario-matrix.md](01-scenario-matrix.md) §3. BLOCKED is reserved for
   missing prerequisites (multi-vCPU without evidenced contract, unavailable
   L2 masking, absent upstream evidence) and is recorded as a stage block —
   never as a failure and never as a pass. Authority: plan ("negative/blocked
   outcomes"); checklist §4 (planned, run, blocked, failed evidence distinct).
3. **Guest-observation primacy.** Guest-emitted result records are the
   primary evidence; Host logs and telemetry corroborate and correlate, and
   never substitute for a missing Guest record. A scenario whose expected
   Guest record is absent cannot PASS on the strength of Host logs. Authority:
   plan goal (stated explicitly); task book §1.
4. **Conditional multi-vCPU mode.** VG-SMP-01/VG-SMP-02 run only when the
   P3-W14-evidenced multi-vCPU contract and P4 vCPU isolation are confirmed
   in step 1; otherwise they execute in BLOCKED mode and the verification
   record states the stage block per P6-V18/P6-V19 wording. Authority: plan
   scope; task book §1 Required and validation matrix.
5. **VG-SGI-01 scope boundary.** The Host-SGI scenario exercises the chain
   Host cross-pCPU SGI → EL2 handling → vIRQ injection → Guest observation,
   and supports P6-V11-style end-to-end evidence; it does **not** substitute
   for W04's target-attribution evidence (P6-V04–P6-V06), which remains
   W04/W13-owned. Authority: plan scope ("Host-SGI support"); task book
   requirement map.
6. **Temporary interrupt layout is stage-local.** The Guest-visible virtual
   timer and vIRQ identifiers/properties used by scenarios are P6-temporary,
   fixed in [01-scenario-matrix.md](01-scenario-matrix.md) §1 with a
   non-freeze statement; they define no machine ABI and no P8 layout.
   Authority: task book §1 Reserved.
7. **Harness contract.** One declared execution order, per-scenario
   repetition and timeout semantics, environment capture, and aggregate
   result classification are fixed in
   [02-harness-and-workflow.md](02-harness-and-workflow.md) §2; the harness
   changes no Host state and runs no W12 fault-injection cases (it may run
   adjacent smoke counts only where a scenario row declares them). Authority:
   plan step 2; separation from W12 scope.

## Work breakdown and loading order

1. Read this README and the Coding Guidelines in full.
2. Load [01-scenario-matrix.md](01-scenario-matrix.md) before writing,
   changing, or running any scenario; it is the sole authority for scenario
   content and classification.
3. Execute [02-harness-and-workflow.md](02-harness-and-workflow.md) in order:
   prerequisite reconciliation, Guest scenario implementation against the P4
   asset contract, harness work, runs, evidence recording.
4. Evidence lands in
   `../../verification/p6-w11-validation-guest-interrupt-suite-verification.md`
   (created when evidence exists) with raw run artifacts linked per the
   P0-W09 QEMU evidence conventions
   ([plan](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md));
   decisions and deviations go to
   `../p6-w11-validation-guest-interrupt-suite-record.md` when implementation
   begins. Neither file may claim W11 or stage completion.

## Explicitly excluded interfaces

No public/stable Rust API, ABI, machine ABI, DTB binding, wire format, or
Host-side interface is authorized by W11. The scenario descriptor and result
record are **internal** to the suite and the P4 Guest asset's module
contract; their concrete Rust realization must conform to that contract and
is bound in the implementation record. Specifically excluded: any Host
interrupt mechanism change; any Guest-visible control beyond the P5-authorized
test/control paths; any vGIC MMIO access from scenarios; any scheduler or
run-state dependency; Linux Guest tests; and production Guest drivers.

## Downstream handoff

- **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`) receives the
  suite's negative-outcome machinery (FAIL/BLOCKED classification, timeout
  handling) as the observation layer for its FI matrix, and the declaration
  that W11 runs no fault-injection cases itself.
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives
  per-scenario evidence references for the P6-DOC-04 validation matrix
  (P6-V09–P6-V19 rows), the latency-relevant Guest timestamps (VG rows'
  Guest-record fields), and the reusable-asset statement for downstream
  regression users.
- **P7/P8 regression users** may reuse the maintained asset while extending
  it under their own scope and designs; nothing in the suite freezes a
  machine ABI, a vCPU count, or an interrupt layout for them (task book §7;
  [§4](01-scenario-matrix.md) non-freeze statement).
