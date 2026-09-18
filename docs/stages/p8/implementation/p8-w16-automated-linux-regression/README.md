# P8-W16 Automated Linux Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The automated QEMU scenario matrix, observable-marker contract,
harness contract, repeated-boot checks, and failure-classification rules
required by [P8-W16](../../plans/p8-w16-automated-linux-regression.md).  
**Owner/change context:** P8-W16 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W16. It converts the bounded
work-package plan into three reviewable design artifacts: a declared scenario
matrix ([01](01-regression-matrix.md)), a harness and repeated-boot contract
with an ordered execution workflow ([02](02-harness-contract-and-workflow.md)),
and this entry document. It defines *how the P8 Linux outcome is proven by
automation* and what each proof does and does not show. It is a validation
design: it defines oracles, evidence destinations, and pass conditions, and it
contains no results, no completion claim, and no performance judgment
(performance observation is [P8-W17](../p8-w17-linux-performance-baseline/README.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the linked supporting file needed for its assigned step: load
[01](01-regression-matrix.md) to author or review scenario rows and marker
expectations; load [02](02-harness-contract-and-workflow.md) to author or
review the harness contract, repeated-boot policy, failure classification, and
evidence flow. Before editing, the agent must also follow the Coding
Guidelines preflight (repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P8 task
book](../../task-book-v0.1.md), and the P8-W16 plan). This document is a
proposed design; it is not an implementation record and claims nothing has run.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W16 plan → this design
→ Coding Guidelines. In particular:

- ADR-003 makes QEMU `virt` the reference/CI platform for deterministic
  testing; ADR-049 names Linux boot regression part of the validation strategy;
  ADR-048 and ADR section 12 require structured observability; ADR section 19
  requires Guest-caused faults to stay VM-scoped. None of these makes a QEMU
  observation an AArch64-semantics or hardware fact.
- The task book binds this package to P8-V21 (automated QEMU matrix with
  determinate markers and shutdown outcomes) and P8-V22 (repeated-boot
  regression detecting stale VMID/TLB/vCPU/IRQ state, races, or uninitialized
  state). Both are *planned evidence* rows; this design defines the conditions
  under which they could pass, never that they passed.
- The plan's exclusions are binding: **no CI implementation** (CI wiring is
  owned by the P0 CI baseline package and later CI work), **no QEMU-specific
  machine semantics** (a QEMU behavior is never encoded as machine ABI; see
  [P8-W02](../p8-w02-machine-contract-governance/README.md) and
  [P8-W14](../p8-w14-machine-abi-compatibility/README.md)), **no fixed retry
  count** (repetition is a declared, evidence-recorded run parameter), **no
  performance KPI** (measurement belongs to W17), and **no real-hardware
  validation** (hardware execution of this matrix is Reserved for later
  stages).

Classification. **Required** for W16 closure: the scenario matrix, the
marker-oracle contract, the harness behavioral contract, the repeated-boot
check set, the failure-classification rules, and the evidence destinations.
**Reserved** with recorded triggers: execution inside CI pipelines (trigger:
the P0 CI baseline package wiring stage regressions into CI), execution on
Orange Pi 3B or any physical board (trigger: P15), and machine-consumable
result aggregation beyond the run record (trigger: an approved consumer
design). **Out of Scope:** implementing any hypervisor or Guest mechanism the
matrix observes; authoring fixture inputs (W15 owns the reproducible fixture);
defining marker *vocabularies* (owned by the source packages listed in the
ledger); designing isolation scenario content (W18), performance measurement
(W17), or Validation Guest scenario semantics (W19); CI workflow files; QEMU
invocation scripts (W16 fixes the harness *contract*, and leaves command
spelling to implementation, consistent with the skill's rule that incidental
command spelling is not the contract).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W09–W15 and P0 test governance (work seq 1) | README ledger; [workflow](02-harness-contract-and-workflow.md) §1 | prerequisite-contract review (W16-DV01) |
| Automated observable markers and expected shutdown outcomes (work seq 2) | [matrix](01-regression-matrix.md) §2 | P8-V21 (W16-DV02) |
| Declared scenario matrix and fixture inputs (work seq 3) | [matrix](01-regression-matrix.md) §3–§5 | P8-V21 (W16-DV03) |
| Repeated-boot checks for stale VMID/TLB/vCPU/IRQ state and races (work seq 4) | [workflow](02-harness-contract-and-workflow.md) §5 | P8-V22 (W16-DV04, DV05) |
| Failures reviewed as reproducible evidence, blocked prerequisites, or regressions (work seq 5) | [workflow](02-harness-contract-and-workflow.md) §6 | W16 closure review (W16-DV06) |
| Handoff of regression entry points and stated limits to W17–W20 and P9 | README handoff; [workflow](02-harness-contract-and-workflow.md) §9 | consumability review (W16-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
the repository is a documentation-only P0 scaffold. `crates/`,
`hypervisor/src/`, `scripts/`, `tests/`, `boards/`, `soc/`, `guests/`, and
`control/` contain only `.gitkeep` markers; there is no Cargo workspace, Rust
source, QEMU script, CI workflow, or fixture of any kind. `docs/stages/p0`
contains one implemented package (P0-W01 record and verification) and the
P0-W02 detailed design; P0-W07/W08/W09 (quality gates, host-test baseline,
QEMU runner entry governance) are plans only. `docs/stages/p8` contains plans
W01–W20, an implementation index, and a `.gitkeep`-only `verification/`
directory. Every prerequisite named below is therefore an **assumed contract**
of its owning package, and each ledger row carries an explicit failure
boundary if the prerequisite delivers differently.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P8-V21: determinate expected markers and shutdown outcomes | No marker contract exists in any tracked file | Marker-oracle contract shape (§2 of [01](01-regression-matrix.md)); vocabularies bound to their owning packages | An automated oracle requires a declared, order-sensitive observable set per scenario | Marker vocabularies: W09, W10, W11, W12, W13, W14, W19 designs; oracle shape: W16 (this design) | W16-DV02 review; vocabularies inherited when owner packages evidence them |
| P8-V21: matrix over Validation Guest, Linux 1/2/4 vCPU, RAM classes, intentional fault | No matrix exists | Scenario matrix in [01](01-regression-matrix.md) §3 with row families and a declared full-versus-smoke split | The plan requires a *declared* matrix, not an ad-hoc scenario list | W16 (this design); row inputs from W10–W14, W19 | W16-DV03 matrix review |
| P8-V21: multiple RAM sizes | No RAM-class definition exists; task book §8 leaves concrete values ADR Required | RAM *class* mechanism (small/normal/larger) consumed from W12; W16 never fixes a capacity value | Concrete values are an unresolved ADR-routed decision (task book §8 row 1) | W12 classes; W02-governed approved values | class binding present; values deferred to the approved machine contract — recorded blocker if absent |
| P8-V21: intentional fault rows | Fault-class taxonomy does not exist yet | At least one intentional-fault row per scheduled run; fault content owned by W13 classes and W18 scenarios; W16 owns the execution envelope only | The task book requires an intentional fault in the automated matrix | W13 taxonomy; W18 scenario content; W16 envelope | row presence in matrix; scenario content via W18 |
| P8-V22: repeated-boot checks for stale VMID/TLB/vCPU/IRQ state, races, uninitialized state | No repeated-run policy exists | Repeated-boot check set and cross-run comparison rules ([02](02-harness-contract-and-workflow.md) §5) | Stale-state defects only surface across repeated lifecycle cycles | W16 (this design); diagnostic classes from W13 | W16-DV04/DV05 policy review |
| Automated execution entry | P0-W09 reserved a single QEMU runner entry; it is not implemented | Harness contract binds to the P0-W09 single-entry governance as an assumed contract | Two independently maintained QEMU invocations would drift — exactly what P0-W09 forbids | P0-W09; extension parameters via W16 [02](02-harness-contract-and-workflow.md) §3 | W16-DV01; **failure boundary:** if P0-W09 delivers no parameter-carrying entry, W16 records a blocked prerequisite and stops, never forks a second runner |
| Fixture inputs for every row | W15 fixture is planned only | Fixture version/reference is a mandatory harness input ([02](02-harness-contract-and-workflow.md) §2) | A matrix row without a pinned fixture is not reproducible evidence | W15 | fixture reference present in every run record |
| Evidence destinations | `docs/stages/p8/verification/` contains only `.gitkeep` | Declared verification-record and raw-evidence paths ([02](02-harness-contract-and-workflow.md) §4) | Evidence must have one reviewable home before any run | W16 (this design) | path convention review (W16-DV01) |

No row above invents a crate, file tree, API, or address value. The concrete
v1 machine values remain an inherited `ADR Required` item routed through
[P8-W02](../p8-w02-machine-contract-governance/README.md) (task book §8); W16
matrix rows reference *classes and contracts*, never values.

## Resolved design decisions and their authority

1. **Scenario-matrix shape.** Rows are keyed by (track, vCPU count, RAM class,
   fault injection, repetition policy). Rationale: the plan requires a
   *declared* scenario matrix but no authority fixed a shape; this
   decomposition makes each plan-mandated dimension explicit and reviewable.
   Stage-local design freedom owned by this design.
2. **Marker-oracle contract.** Pass conditions are ordered textual markers on
   the Guest console plus a terminal outcome, checked by an exact-sequence
   match. Rationale: the only P8 Guest-observable channel with determinate
   semantics is the console (W09/W14); ordering gives races a chance to
   surface without timing assertions. W16 owns the contract *shape*; marker
   *vocabularies* stay owned by W09/W10/W11/W12/W13/W14/W19. Stage-local
   design freedom, bounded by plan scope.
3. **No timing-based pass conditions.** A row fails on ordered-marker mismatch,
   wrong terminal outcome, unexpected fatal-class diagnostics, or timeout; it
   never fails on duration. Rationale: plan forbids a performance KPI here;
   duration observations belong to W17 as non-judgmental baselines.
4. **Repetition as a declared run parameter.** The repeated-boot count, warm-up
   count, and per-row timeout are declared per run and recorded in the run
   record — never hardcoded as a contract constant. Rationale: the plan names
   a *fixed retry count* out of scope. Stage-local policy owned by this
   design; minimum expectation (a repeated-boot family exists and its count is
   recorded) is Required.
5. **Determinism discipline.** Matrix rows are seed-free by default; a row
   that needs randomness must declare its seed discipline in the row, and TCG
   nondeterminism is absorbed by oracle design (order and class, not
   timestamp), never by repeat-until-pass tuning.
6. **Evidence destinations.** Per-package verification record
   `docs/stages/p8/verification/p8-w16-automated-linux-regression-verification.md`
   plus raw serial logs and run records retained under
   `docs/stages/p8/verification/assets/p8-w16/`. Rationale: `docs/README.md`
   assigns `verification/` to evidence; raw logs must be retained for failure
   reproduction. Path convention is stage-local design freedom.
7. **Validation Guest rows are envelopes.** The matrix's Validation Guest row
   family references the retained mechanism suite of
   [P8-W19](../p8-w19-validation-guest-dual-track/README.md) as its content
   owner; W16 owns only their automated execution envelope. Rationale: W19's
   plan forbids redefining P4–P7 semantics, and W16's plan forbids replacing
   the Validation Guest with Linux.

## Work breakdown and loading order

1. Read [01](01-regression-matrix.md) for the marker contract and the declared
   scenario matrix; every matrix row cites its content owner.
2. Read [02](02-harness-contract-and-workflow.md) for the harness contract,
   run-record schema, repeated-boot checks, failure classification, ordered
   workflow, validation matrix, and handoff checklist.
3. Execute the ordered workflow of [02](02-harness-contract-and-workflow.md)
   §2 (steps 1–6) when implementation is authorized. Actual commands, outputs,
   environment, and run/not-run status go to
   `../../verification/p8-w16-automated-linux-regression-verification.md`;
   factual implementation decisions go to
   `../p8-w16-automated-linux-regression-record.md` — both created only when
   that work begins. Neither file exists today, and nothing in this design
   claims they will.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format, or
persistent data layout is designed or authorized by W16. The only
machine-facing surfaces defined here are (a) the Guest console marker
contract *shape* and (b) the run-record field list, both contractual outlines
in [01](01-regression-matrix.md) and [02](02-harness-contract-and-workflow.md);
their concrete serialization is an implementation choice recorded in the
implementation record. No QEMU command line, script file, CI YAML, fixture
build recipe, or hypervisor source change is designed here. Adding any of
these under W16 authority is a scope conflict to stop at review.

## Downstream handoff

Per the [plan index](../../plans/README.md) consumer map:

- **W17** consumes W16 scenario rows as the fixed measurement scenarios and
  the run record's environment/identity fields as measurement context; W17
  owns all timing interpretation.
- **W18** consumes the harness envelope, the timeout/contamination failure
  oracles, and the failure-classification rules; W18 owns isolation scenario
  content and containment expectations.
- **W19** consumes the Validation Guest row family as the retained suite's
  automated execution path; W19 owns scenario content and the lost-coverage
  block rule.
- **W20** consumes W16's validation matrix, evidence paths, and stated limits
  for the P8 evidence index and closure review.
- **P9** receives the regression entry points and their stated limits (what a
  QEMU/TCG pass does and does not prove); P9 must separately design virtio
  behavior and may not treat matrix success as a machine-ABI or hardware fact.
