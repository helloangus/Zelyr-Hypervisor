# P8-W02 Machine-Contract Governance — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The versioned machine-contract decision route and freeze gate for
`rusthv-arm-virt-v1` required by
[P8-W02](../../plans/p8-w02-machine-contract-governance.md).  
**Owner/change context:** P8-W02 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W02. P8-W02 creates the
governance that lets a host-independent machine contract exist without
prematurely defining it: one normative governance document that fixes the
categories a `rusthv-arm-virt-v1` specification must cover, the class of every
statement such a specification may contain (permanent Guest-visible fact,
implementation fact, reservation), the decision route for every still-unfixed
value, the host-independence rule, the compatibility and versioning rules, and
the gate a specification must pass before it may be published as frozen. It
deliberately does **not** choose an address, slot count, register model,
CPU-feature value, PSCI subset, console device model, or any other Guest ABI
detail; does not publish the v1 specification; and does not design any
implementation internals. Those belong to the routed decisions and to the
packages that consume the governance ([P8-W03](../p8-w03-linux-boot-contract/README.md)
and later).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). W02 adds no
code; the guidelines apply through the preflight and completion-report
discipline. The agent then loads only the linked supporting file needed for
its assigned step. Before editing it must also follow the repository
`AGENTS.md`, documentation index, ADR baseline, P8 task book, and the P8-W02
plan. This document is the proposed detailed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W02 plan → this
design. In particular:

- ADR-024 requires a versioned Generic ARM64 VM machine independent of the
  host SoC; ADR-040 requires independent versioning of schema, machine, and
  management ABI; ADR §18 and the task book §1 leave the concrete v1 IPA map,
  GIC/PCI windows, and virtio slot count **undecided**, and the task book §8
  labels those items and the freeze authority itself `ADR Required /
  Specification Investigation`. The governance document preserves those
  labels; it is the route, not the decision.
- The task book §2 Required item bounds W02: "a formally reviewed, versioned
  machine-contract process ... CPU, Guest physical-address categories,
  interrupts, timer, firmware/DTB, console, permanent ABI, reserved space,
  and compatible versus incompatible change rules. Concrete values require the
  stated routing." The governance category set is exactly this enumeration.
- The task book §2 Reserved item permits reservations (future virtio-MMIO and
  PCI capacity, later machine versions, ACPI/UEFI, additional CPU features,
  advanced GIC features, migration/snapshot compatibility) but forbids a
  reservation from defining the unimplemented protocol, window values, data
  format, runtime policy, or mechanism. The governance document encodes that
  limit as a checkable rule.
- [P8-W01](../p8-w01-entry-contract-reconciliation/README.md) supplies the
  recorded constraint register and the conflict routes W02 inherits; W02 does
  not re-derive them.

Classification: the governance document, its category and fact-class
definitions, decision-route table, host-independence and compatibility rules,
and freeze gate are **Required**. A specification template generated from the
governance (skeleton without values) and automated consistency tooling are
**Reserved**. Publishing the v1 specification, selecting any concrete value,
defining any device register model or PSCI subset, and all implementation
internals are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect ADR-024/025/040 and ADR §18 unresolved machine items | [Governance contract](01-governance-contract.md) §2, [workflow](02-review-workflow.md) step 1 | P8-V02 (W02-DV01) |
| Define the reviewable categories a machine specification must cover | [Governance contract](01-governance-contract.md) §4 | P8-V02 (W02-DV02) |
| Separate permanent Guest-visible facts, implementation facts, and reservations | [Governance contract](01-governance-contract.md) §5 | P8-V02 (W02-DV03) |
| Route each concrete unresolved item to ADR Required or Specification Investigation | [Governance contract](01-governance-contract.md) §6 | P8-V02 (W02-DV04) |
| Review host independence and future-version compatibility constraints | [Governance contract](01-governance-contract.md) §7–§8, [workflow](02-review-workflow.md) step 5 | P8-V02, P8-V03 (W02-DV05) |
| Freeze gate and ownership without publishing v1 | [Governance contract](01-governance-contract.md) §9, [workflow](02-review-workflow.md) step 6 | P8-V02/P8-V03 (W02-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs` at
`4e631ee`): `docs/machine-types/README.md` and `docs/abi/README.md` are
one-line stubs with no contract document; no file contains a
`rusthv-arm-virt-v1` specification, governance process, or category
definition; ADR §18 explicitly lists the v1 IPA map, virtio slot count, and
GIC/PCI windows among "questions awaiting ADR freeze". The ADR and task book
are the only sources of machine-model constraints today.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A decision route exists for establishing the contract without prematurely defining it | No machine-contract document or process exists anywhere in the tree | Governance document with fixed categories, fact classes, and per-item routes | A route is a normative artifact; with none, every consumer would improvise its own machine assumptions | W02 (this design) | W02-DV02/DV04 review |
| Machine identity is governable | Name `rusthv-arm-virt-v1` appears in ADR-024 and the task book; no version/identity rules exist | Identity and versioning rules section (name authority, machine_version semantics per ADR-040) | Identity is the first category every later contract and test references | W02; name authority = ADR/task book | W02-DV02 |
| Unresolved values stay escalated | ADR §18 and task book §8 name the open items; nothing tracks them as routes | Decision-route table assigning each open item an owner route and status | The plan requires routing "each concrete unresolved item", not a prose list | W02 records; ADR process owns resolution | W02-DV04 |
| No Host fact enters the contract boundary | No contract exists to violate the rule | Host-independence rule with checkable criteria (no QEMU/RK3566 name, address, IRQ, or firmware fact as a Guest-visible statement) | P8-V03 is defined as this negative review | W02 | W02-DV05 |
| Compatible-versus-incompatible change rules exist | Absent | Compatibility section: what may change within v1, what forces v2, what requires an ADR | ADR-024/040 make versioned evolution the point of the machine model | W02 | W02-DV05 |
| v1 is not published by W02 | Nothing to publish; freeze authority itself is `ADR Required` per task book §8 | Freeze gate defining what approval evidence a specification needs before publication, with the authority routed | Governance must bound its own scope: process now, values later | W02 defines gate; ADR owns freeze authority decision | W02-DV06 |

No row above selects a Guest-visible value, so no decision blocker arises from
this design. The standing blocker — concrete v1 values and the freeze
authority — is the routed item the governance exists to carry; it is recorded
as `ADR Required` per the task book, not resolved here.

## Resolved design decisions and their authority

1. **Artifact and location.** One normative governance document,
   `docs/machine-types/machine-contract-governance-v0.1.md`, with the status
   header required by `docs/README.md`. Rationale: `docs/machine-types/` is
   the designated home of guest-visible virtual-machine model contracts
   (documentation index layout); the governance process governs exactly that
   contract family and belongs beside it, versioned like any normative
   document. Placing it in the stage directory instead would hide a
   cross-stage process from the machine-model consumers (P9+).
2. **Category set.** Exactly the task book §2 enumeration — identity; CPU;
   Guest physical-address categories; interrupts; timer; firmware/DTB;
   console; permanent ABI; reserved space; compatible-versus-incompatible
   change rules — as ten reviewable categories with required content per
   category ([governance contract](01-governance-contract.md) §4). Rationale:
   the task book fixes the category list; adding or dropping one would alter
   P8 scope.
3. **Fact classes.** Three — *Permanent Guest-visible fact*, *Implementation
   fact*, *Reservation* — with the assignment rule and the reservation
   prohibition copied from the task book §2 Reserved item. Rationale: the
   plan's work-sequence item 3 requires exactly this separation.
4. **Route labels.** The task book §8 labels — `ADR Required`,
   `Specification Investigation`, `Implementation Choice`,
   `Platform Investigation`, `Architecture Change Request` — are adopted
   verbatim as the route vocabulary; W02 invents no new label. Rationale:
   route naming is task-book authority.
5. **Specification ownership.** The future v1 specification document
   (`docs/machine-types/rusthv-arm-virt-v1-spec-v0.1.md`, name fixed here as
   stage-local design freedom) is *described* by the governance — required
   sections, per-category content, gate — but is not created by W02 and
   contains no value until its routed decisions resolve. Rationale: keeps the
   plan's "does not mean v1 is frozen" closure statement literal.
6. **Consumer contracts route through categories.** W03–W20 consume
   categories and gates only ([governance contract](01-governance-contract.md)
   §10); any package needing a value routes through §6. Rationale: the plan's
   handoff wording.

## Work breakdown and loading order

1. Read [the governance contract](01-governance-contract.md) for the artifact
   groups, category definitions, fact classes, route table, and freeze gate.
2. Apply [the review workflow](02-review-workflow.md) in order: verify the
   ADR inputs, draft the governance document, review it for host independence
   and completeness, wire discovery, and record evidence.
3. Store the document and its record in
   `../p8-w02-machine-contract-governance-record.md` when work starts, and
   review evidence in
   `../../verification/p8-w02-machine-contract-governance-verification.md`
   when the review is exercised. Neither this design nor any record may claim
   W02 complete or v1 frozen.

## Explicitly excluded interfaces

W02 authorizes no Rust type, function, trait, module, crate, public API,
wire format, or runtime artifact, and no Guest ABI value of any kind: no IPA
number, interrupt number, slot count, register encoding, CPU-feature value,
PSCI function list, DTB node value, or console register model. The only
machine-facing surface it creates is the governance process itself; the v1
specification it describes remains unwritten until its routed decisions exist.
Adding any Guest-visible value to a W02 artifact is a scope conflict and must
be stopped at review.

## Downstream handoff

Per the [P8 plan index](../../plans/README.md), W02 feeds W03–W20 and P9+:

- **W03** receives the firmware/DTB and permanent-ABI categories and the gate
  its boot contract must pass; the boot contract document registers under the
  governance's contract-family rules.
- **W04** receives the firmware/DTB and identity categories its DTB contract
  must be consistent with, and the host-independence rule its negative review
  enforces.
- **W05–W09** receive the CPU, interrupt, timer, firmware/PSCI, and console
  categories: each designs against a category, never a value, and routes its
  proposed values through §6 before any specification text carries them.
- **W12** receives the Guest physical-address categories for the memory-model
  boundary; **W14** receives the compatibility rules its compatibility-test
  policy enforces; **W20** receives the freeze gate as the closeout criterion
  for the machine-documentation route.
- **P9+** receives the reservation rules: reserved virtio/PCI capacity is a
  bounded, documented reservation without protocol or window values, per the
  task book §2 Reserved item.

No consumer may publish or implement a value whose route is unresolved; the
governance document is the arbitration reference when a consumer disputes
whether a value is still open.
