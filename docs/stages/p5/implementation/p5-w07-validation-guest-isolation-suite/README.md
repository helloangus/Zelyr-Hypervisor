# P5-W07 Validation Guest Security and Isolation Suite — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The determinate P5 ABI-security Validation Guest scenario set,
including the two-context authority-isolation asset, required by
[P5-W07](../../plans/p5-w07-validation-guest-isolation-suite.md).  
**Owner/change context:** P5-W07 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W07. It turns the W02–W06
boundaries into an externally observable Guest-side evidence asset: a
scenario matrix whose every scenario has a determinate expected marker, a
two-security-context setup that proves one context cannot use another's
authority, and the harness and maintenance rules that keep the asset usable
by W09's regression and later stages. It is a validation-package design: it
defines scenarios, expected observables, pass conditions, and evidence
destinations — **not results**. No run, pass, or completion is claimed here,
and no scenario may be executed before its dependent mechanisms (W02–W06)
are implemented.

A coding agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| The P5 scenario matrix (positive, malformed, handle, authority, revocation, Guest-data, lifecycle) | [01 — scenario matrix](01-scenario-matrix.md) |
| Two-context isolation setup and the harness/evidence contract | [02 — two-context isolation and harness](02-two-context-isolation-and-harness.md) |
| Ordered implementation, validation matrix, records, handoff | [03 — implementation workflow and validation](03-implementation-workflow-and-validation.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P5
task book, and the P5-W07 plan.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → frozen contracts →
P5-W07 plan → this design → Coding Guidelines. In particular:

- The plan covers P5-T15 and T18: discovery, valid calls, valid and invalid
  references/authority/data, unknown/invalid version/flags/length/address/
  overflow/type cases, zero/max/random/stale/destroyed generations,
  insufficient/no/revoked/cross-VM authority, repeated destruction, wrong
  lifecycle state, and the two-context requirement.
- The Validation Guest remains a **maintained test asset**, inherited from
  [P4-W05](../../../p4/plans/p4-w05-validation-guest.md); W07 extends it
  with the P5 scenario set and owns that extension. It must not become a
  Guest SDK, a production Guest ABI, or a general Guest protocol.
- The P4 evidence boundary applies: QEMU is the declared validation
  environment; QEMU results do not define AArch64 semantics and do not prove
  real-hardware behavior. Every scenario row carries a
  proves / does-not-prove statement.
- W06's containment table is the semantic source for expected outcomes:
  Guest markers and dispatch-side outcomes must agree; this design consumes
  it and must not redefine outcome classes.

Classification:

- **Required:** the scenario matrix of
  [01](01-scenario-matrix.md); the two-context isolation setup and its
  negative control; the marker grammar and marker-discipline rules; the
  harness contract with determinate outcomes including non-success classes;
  scenario-maintenance ownership; evidence destinations.
- **Reserved:** additional scenario families for future object classes;
  inter-VM communication-based value passing (if a later stage adds an
  evidenced channel); automation of the suite in CI (CI policy is owned by
  P0-W20 and later stage packages; W09 wires regression, not CI policy).
- **Out of Scope:** Linux Guests, production Guest SDK/API, Control Domain
  protocol, permanent Guest machine ABI, general multi-VM orchestration, a
  scheduler, timer/IRQ mechanisms (P6), runtime evidence created before the
  dependent mechanisms exist, and any performance measurement (W08).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Discovery, valid call, valid reference/authority/data scenarios (P5-T15) | [scenario matrix](01-scenario-matrix.md) group A | P5-V11 (W07-DV02) |
| Unknown/invalid version/flags/length/address/overflow/type cases | [scenario matrix](01-scenario-matrix.md) groups B, F | P5-V11 (W07-DV03) |
| Zero/max/random/stale/destroyed generation cases; repeated destruction; wrong lifecycle state | [scenario matrix](01-scenario-matrix.md) groups C, G | P5-V11 (W07-DV04) |
| Insufficient/no/revoked authority cases | [scenario matrix](01-scenario-matrix.md) groups D, E | P5-V11 (W07-DV05) |
| Two-context cross-VM/raw-value misuse test with determinate expectations (P5-T18) | [two-context](02-two-context-isolation-and-harness.md) §2 | P5-V12 (W07-DV06) |
| Externally observable markers and expected-outcome discipline | [two-context](02-two-context-isolation-and-harness.md) §3 | W07-DV07 review |
| Test-asset maintenance ownership | [two-context](02-two-context-isolation-and-harness.md) §5 | W07-DV08 review |
| Scenario results handed to W09–W10 | [workflow](03-implementation-workflow-and-validation.md) handoff checklist | W07 closure review (W07-DV09) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p5-implementation-designs`,
`git ls-files`): `guests/validation-aarch64/` contains only a `.gitkeep`
placeholder; no Validation Guest source, no P4 `VG-*` scenario
implementation, and no QEMU runner exist in the tracked tree. P1–P4 have
plans and task books but no implementation or verification records. The W02,
W03, W05, and W06 detailed designs are parallel work on this branch (linked
by slug as forward references). No P5 scenario or marker exists in any
tracked file.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A maintained Guest asset issues P5 scenarios (P5-V11) | No Guest source exists | The P4-W05 Validation Guest asset, implemented and evidenced | Scenarios cannot exist without the asset they extend | P4-W05 (assigned prerequisite) | P4 verification evidence; W07-DV01 prerequisite review |
| Scenarios produce determinate expected results | No markers or expectations exist | The marker grammar and per-scenario expectations of [01](01-scenario-matrix.md)/[02 §3](02-two-context-isolation-and-harness.md) | "Determinate" requires a fixed observable per scenario, fixed before runs | W07 (this design) | W07-DV02–DV05 run evidence |
| Expected outcomes match dispatch behavior | W06 outcome table is design-only | W06's implemented containment table as the semantic source | A marker expecting a class dispatch never emits would be unfalsifiable noise | W06 design + implementation | W07-DV02–DV05 cross-checked with dispatch-side records |
| Two independent security contexts exist (P5-V12) | Nothing tracked; P4 foundation is single-Guest by plan | Two bootable Guest instances via the evidenced P4 boot path, each with a W05 bootstrap grant | Isolation evidence requires two authorities and one trying to misuse the other's raw value | P4 boot path + W05 grants; W07 composes them | W07-DV06 |
| Results are objectively detectable | No runner exists | The P4-W08/P0-W09 automation entry point, extended by the harness contract of [02 §4](02-two-context-isolation-and-harness.md) | Marker matching must be mechanical and determinate | P4-W08 + P0-W09 (assigned prerequisites); W07 harness rules | W07-DV07 |
| Evidence lands in governed locations | `../../verification/` holds only `.gitkeep` | Run records under `../../verification/p5-w07-validation-guest-isolation-suite-verification.md` | Task book separates evidence from design | W07 | verification record |

No row requires inventing a machine ABI, Guest protocol, or multi-VM
orchestration framework; the two-context composition uses only the P4 boot
path and W05's grant contract, with the failure boundary stated in
[02 §2](02-two-context-isolation-and-harness.md).

## Resolved design decisions and their authority

1. **Scenario identifier namespace `P5VG-nn`** (grouped ranges per matrix
   group), distinct from P4's `VG-001–VG-012`. Rationale: a distinct prefix
   prevents collision with inherited P4 scenarios and signals that P5
   scenarios never reinterpret a P4 marker. Stage-local naming freedom
   owned by this design; the IDs are test conventions, not ABI.
2. **Marker grammar: `P5VG/<scenario-id>/<expected-outcome-class>`** emitted
   on the P4-established Guest debug channel, one marker per scenario
   conclusion, optionally followed by fixed-format scenario-specific fields
   that never include addresses. Rationale: determinate machine-matchable
   evidence without pointer disclosure; machine interfaces never rely on
   free-form log text. Authority: this design within plan scope, constrained
   by the task book's no-Host-information rule and P0-W12 visibility rules.
3. **Two contexts = two bootable Validation Guest instances** on the
   evidenced P4 boot path (single vCPU each; P5 isolation evidence does not
   require SMP), each bound to its own security context by an explicit W05
   bootstrap grant. Knowing the other context's raw handle value (a
   deterministic test-bootstrap constant, see
   [02 §2](02-two-context-isolation-and-harness.md)) must not confer
   authority. Rationale: ADR-013 caller association and the task-book
   two-context requirement; a within-one-VM simulation would not satisfy
   P5-V12. Failure boundary: if the delivered P4 foundation cannot boot a
   second instance, the prerequisite is blocked and an
   `Architecture Change Request` is recorded — no weaker substitute may be
   presented as P5-V12 evidence.
4. **Negative control is mandatory:** the same raw handle value used by its
   owning context must succeed in the same boot, proving the cross-context
   denial is authority-based, not value-based. Authority: this design;
   required for the P5-V12 proof to be meaningful.
5. **Scenario semantics are consumed, never redefined:** expected outcome
   classes come from W06's containment table and W02's result vocabulary;
   the matrix maps scenarios to those classes instead of inventing new ones.
   Authority: plan prerequisites; conflict = stop and record.
6. **The asset stays maintained, bounded, and non-general:** scenarios are
   added only with a matrix row, marker, and expectation; no general Guest
   protocol, SDK, dynamic test loader, or future machine ABI is introduced.
   Authority: plan out-of-scope list.
7. **Environment is declared per run:** QEMU `virt` as the reference
   environment with declared version and CPU count; every result statement
   is QEMU-scoped; hardware-required rows, if any are ever added, must be
   labeled and cannot pass on QEMU alone. Authority: task book §1 closing
   rule and Plan-Agent detailed reference §66–§67.

## Work breakdown and loading order

1. Load [01](01-scenario-matrix.md) for the full scenario set; each row
   gives input/precondition, expected marker, pass condition, and proof
   boundary.
2. Load [02](02-two-context-isolation-and-harness.md) for the two-context
   setup, the marker grammar and discipline, the harness contract, and
   maintenance ownership.
3. Execute the ordered steps in
   [03](03-implementation-workflow-and-validation.md); record facts in
   `../p5-w07-validation-guest-isolation-suite-record.md` (created when work
   starts) and runs in
   `../../verification/p5-w07-validation-guest-isolation-suite-verification.md`.
   Neither this design nor any record may claim W07 complete; completion is
   claimed only in verification material, for what was actually run.

## Explicitly excluded interfaces

No Guest SDK or stable Guest-facing API, no general test protocol or dynamic
scenario loader, no machine ABI or management ABI surface, no Host-side
fuzzing or stress framework (W08), no telemetry counters or trace events
(W09), no CI policy, no scheduler/timer/IRQ behavior, and no P6+ mechanism.
The only Guest-visible surfaces are the W02-defined hypercall boundary and
the P4-established debug channel; any additional surface is out of scope and
must be raised as a design conflict.

## Downstream handoff

- **W09** ([telemetry/regression](../p5-w09-telemetry-safe-logging-regression/README.md))
  receives the stable marker set and scenario inventory as regression
  expectations, plus the non-success class definitions it must preserve.
- **W10** ([closeout](../p5-w10-closeout-p6-handoff/README.md)) receives the
  maintained scenario inventory and evidence links as part of the factual P5
  record.
- **P6** (through W10 only) may reuse the maintained-asset pattern and marker
  discipline for its own Validation Guest interrupt suite
  ([P6-W11](../../../p6/plans/p6-w11-validation-guest-interrupt-suite.md));
  it designs its own scenarios and must not treat P5 markers or scenario
  constants as ABI.
