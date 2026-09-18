# P8-W01 Entry Contract Reconciliation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The evidence-backed P8 entry review over the P0–P7 handoffs required
by [P8-W01](../../plans/p8-w01-entry-contract-reconciliation.md).  
**Owner/change context:** P8-W01 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W01. P8-W01 is a reconciliation
package: it produces a review record, no code, no machine-ABI value, and no
Linux behavior. It converts the plan's bounded review work into one
authoritative artifact — the P8 entry reconciliation record — whose schema,
reconciliation classes, constraint register, and conflict-routing rules are
fixed here. The record links every P0–P7 prerequisite to an evidence location
or to an explicit recorded absence, names the P8 consumers and the limits of
each predecessor fact, and routes every unresolved machine or security input to
the decision gate that owns it. It deliberately does **not** repair an upstream
stage, select any `rusthv-arm-virt-v1` contract value, classify Linux CPU
behavior (that is [P8-W05](../p8-w05-linux-cpu-virtualization/README.md)), or
treat a planned predecessor document as evidence.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). W01 adds no
code, so most Coding-Guidelines implementation rules are inapplicable; the
guidelines still govern the preflight (load the governing documents, stop on
design conflicts) and the completion-report discipline. The agent then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the repository `AGENTS.md`, documentation index, ADR baseline,
P8 task book, and the P8-W01 plan. This document is the proposed detailed
design; it is not a completion record and contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W01 plan → this design.
In particular:

- The task book (§2) makes every P0–P7 entry a **dependency record, not a
  completion claim**, and permits use of a predecessor only when its factual
  handoff and evidence support the need. This design operationalizes exactly
  that rule; it does not weaken it.
- The task book (§1 and §8) routes concrete machine values to a required ADR or
  specification review. W01 records where those unresolved inputs sit; it never
  resolves one locally. Items the task book pre-classifies
  (`ADR Required / Specification Investigation`, `Architecture Change Request`)
  keep those labels in the record.
- A planned-but-unevidenced predecessor is recorded as such. It bounds what
  later P8 *designs* may assume and what P8 *implementation* may rely on; it is
  not, by itself, an `Architecture Change Request`.
- The ADR architecture invariants (§19) — guest input untrusted, capability
  authority, platform independence, versioned machine ABI, no global panic for
  guest-caused faults — are review constraints W01 registers, not items W01
  re-derives.

Classification: the reconciliation record and its review workflow are
**Required**. Tooling to automate link checking is **Reserved** (a manual
review satisfies P8-V01; automation may be added without changing this
contract). Repairing P0–P7, selecting machine ABI values, implementing Linux
support, creating verification evidence for other packages, and editing any
upstream stage document are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect ADR, task book, plans, and P0–P7 implementation and verification records | [Reconciliation contract](01-reconciliation-contract.md) §2, [workflow](02-review-workflow.md) step 1 | P8-V01 (W01-DV01) |
| Reconcile each prerequisite with an evidence location or recorded absence | [Reconciliation contract](01-reconciliation-contract.md) §3, §5 | P8-V01 (W01-DV02) |
| Identify consumers and limits of each predecessor fact | [Reconciliation contract](01-reconciliation-contract.md) §5 (record schema) | P8-V01 (W01-DV03) |
| Record Guest-untrusted, capability, platform-independence, and telemetry constraints | [Reconciliation contract](01-reconciliation-contract.md) §4 | P8-V01 (W01-DV04) |
| Classify conflicts and missing machine/security inputs without resolving them locally | [Reconciliation contract](01-reconciliation-contract.md) §3.3, [workflow](02-review-workflow.md) step 5 | P8-V01 (W01-DV05) |
| No later plan presumes undocumented behavior | [workflow](02-review-workflow.md) step 6 (consumer cross-check) | P8-V01 (W01-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs` at
`4e631ee`): the repository is a P0 documentation scaffold. There is no Cargo
workspace and no Rust source (`crates/.gitkeep`, `hypervisor/src/.gitkeep`,
`guests/validation-aarch64/.gitkeep`). Of P0–P7, only P0 has implementation
traceability: `docs/stages/p0/implementation/` contains the P0-W01 repository
baseline record and the P0-W02 toolchain design trio, and
`docs/stages/p0/verification/` contains one record (P0-W01). P1–P7 have task
books and plan indexes only; their `implementation/` and `verification/`
directories are placeholders (P1–P4 `.gitkeep`; P5–P7 an index README each,
with no package record). `docs/machine-types/README.md` and `docs/abi/README.md`
are one-line stubs. No stage in P1–P7 has a verification record, so no
P8-V01 input is currently evidenced by runtime evidence; every P1–P7 handoff
contract exists today as a plan.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every P0–P7 dependency is linked or explicitly blocked | P0: one package evidenced (W01); P1–P7: plans and task books only, no records | Reconciliation record with one row per P1–P7 handoff contract and P0 evidence links | A link-or-block review cannot exist without a fixed schema that forces a disposition per input | W01 (this design) | W01-DV02 schema and completeness review |
| Evidence status is factual, not assumed | Only P0-W01 verification exists; P1–P7 verification directories are placeholders | Reconciliation classes that separate *evidenced*, *planned-only*, and *blocked* dispositions | Conflating a plan with evidence is the exact failure P8-V01 forbids | W01 | W01-DV02 |
| Consumers and limits of each predecessor fact are identified | No document maps P1–P7 facts to P8 consumers | Record schema requires per-row consumer list and stated limits | Later packages must receive "only reconciled factual inputs and recorded limits" (plan handoff) | W01 | W01-DV03 consumer cross-check |
| Security/layering/telemetry constraints recorded | Constraints are dispersed across ADR §19, task book §2, and plans | One constraint register in the record, each entry cited to its authority | W02–W20 consume the register instead of re-deriving constraints | W01 | W01-DV04 |
| Conflicts and missing machine/security inputs classified without local resolution | Machine values are unresolved by design (ADR §18; task book §8) | Conflict register with `ADR Required` / `Architecture Change Request` labels and owning decision route | The plan forbids resolving these locally; routing them is the deliverable | W01 records; W02 / ADR process own resolution | W01-DV05 |
| Review, not runtime evidence | W01 is documentary by definition | Manual (Reserved: automated) link-and-consistency review procedure | P8-V01 is defined as a review with a passing condition | W01 | W01-DV01/DV06 |

No row above requires inventing a crate, target, tool, or machine value, so no
decision blocker is outstanding for this design. The dominant finding —
P1–P7 handoffs are planned but unevidenced — is recorded as the expected P0
stage of the roadmap, not as a defect; its consequence (implementation gating)
is carried in [the reconciliation contract](01-reconciliation-contract.md) §3.

## Resolved design decisions and their authority

1. **Deliverable artifact.** One record,
   `docs/stages/p8/implementation/p8-w01-entry-contract-reconciliation-record.md`,
   created when W01 work starts. The P8 implementation index states that
   implementation traceability lives in `docs/stages/p8/implementation/`, so
   the record — not a new top-level document — is the authoritative output.
   Rationale: preserves the task-book separation of plans, implementation
   notes, and verification.
2. **Reconciliation classes.** Four dispositions — *Evidenced*, *Planned-only*,
   *Blocked*, *Conflict* — fixed in [the reconciliation
   contract](01-reconciliation-contract.md) §3 with decision rules. Rationale:
   the plan requires "an evidence location or recorded absence" per
   prerequisite; a fixed class set makes the review mechanical and auditable.
3. **Unit of reconciliation.** The unit is the *handoff contract* (the
   P1–P7 plan that names what a consumer receives — e.g.
   [P7-W14](../../../p7/plans/p7-w14-documentation-p8-handoff.md)) plus the
   evidence its claims require, not each upstream work package individually.
   Rationale: the task book §2 table names per-stage inputs and boundaries;
   the handoff plans are the artifacts that enumerate them. Where a stage has
   no dedicated P8-facing handoff plan (P0, P1), the reconciliation unit is the
   stage's closest handoff or evidence record, per §2 of the contract.
4. **Constraint register scope.** Exactly the constraint families the plan
   work sequence names — Guest-untrusted input, capability authority,
   platform independence, telemetry — each entry carrying a normative citation.
   Rationale: bounded, checkable, and sufficient for W02–W20 without
   restating the ADR.
5. **Conflict routing.** Machine-model and security-policy inputs that P8
   needs but no authority has fixed are routed, never answered: machine-ABI
   items to the [P8-W02](../p8-w02-machine-contract-governance/README.md)
   decision route and its `ADR Required` thresholds; contradictory upstream
   architecture/security inputs to `Architecture Change Request` records per
   the task book §8 handling. W01 owns the register, not the resolution.
6. **Design-time versus implementation-time reliance.** The record states, per
   row, whether the input may be assumed by P8 *designs* (as an explicit
   assumed contract with a failure boundary) and whether P8 *implementation*
   may rely on it (only with evidence). Rationale: P8 detailed designs are
   produced while P1–P7 are unevidenced; the task book gates *use*, not
   design, on evidence.

## Work breakdown and loading order

1. Read [the reconciliation contract](01-reconciliation-contract.md) for the
   input inventory, class definitions, constraint register, and the record
   schema.
2. Apply [the review workflow](02-review-workflow.md) in order: verify
   authorities, inspect inputs, classify each, register constraints, route
   conflicts, and run the closure review.
3. Record actual findings in
   `../p8-w01-entry-contract-reconciliation-record.md` when work starts, and
   review evidence in
   `../../verification/p8-w01-entry-contract-reconciliation-verification.md`
   when the review is exercised. Neither this design nor a written record may
   claim W01 complete.

## Explicitly excluded interfaces

W01 authorizes no Rust type, function, trait, module, crate, public API, ABI,
wire format, machine-contract value, DTB fact, or test harness. It adds no
code and no configuration. The only artifact it creates is the reconciliation
record defined in [the reconciliation contract](01-reconciliation-contract.md)
§5; any additional artifact is a scope conflict.

## Downstream handoff

Per the [P8 plan index](../../plans/README.md), W01 is a prerequisite of
W02–W20. The reconciliation record is the boundary:

- **W02** receives the machine-model and security inputs that must be routed,
  with their `ADR Required` / `Specification Investigation` labels intact, and
  the recorded absence of any frozen machine contract today.
- **W03–W05** receive the per-stage input map (which P2–P7 facts the boot,
  DTB, and CPU-compatibility designs may assume, and the failure boundary if a
  predecessor delivers differently) and the constraint register.
- **W06–W12, W14** receive the same input map plus the recorded limits on PSCI,
  vGIC, timer, scheduler, and memory predecessors.
- **W13, W18** receive the recorded Guest-failure and containment constraints
  inherited from P4/P5 inputs.
- **W15–W17, W19–W20** receive the evidence-status baseline (what was
  evidenced when W01 closed) so their validation plans do not presume runtime
  facts that did not exist.

No consumer may rely on an *Evidenced* disposition beyond its stated limits,
and none may treat *Planned-only* as evidence. W20's closeout re-checks the
record against the then-current evidence state.
