# P8-W04 Guest DTB Contract — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Guest-only DTB fact contract and machine-consistency /
host-leakage review required by
[P8-W04](../../plans/p8-w04-guest-dtb-contract.md).  
**Owner/change context:** P8-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W04. P8-W04 is a
contract-definition package: it produces one versioned, Guest-facing contract
document — the Guest DTB contract — that enumerates the facts the Guest DTB
must present to Linux (CPU/topology, memory, chosen, PSCI, timer, GIC,
console, compatible/model, reserved memory, and the P8 minimal device set),
the authoritative source of each fact, the consistency checks against the
machine and boot contracts, and the negative host-leakage criteria — without
fixing any node value, designing the DTB builder, or reusing the Host DTB.
It deliberately does **not** implement builder mechanics, choose concrete
node values before their routes resolve, design Virtio devices, describe the
Host platform, or introduce ACPI.

The DTB is a wire format, so this design applies the Coding Guidelines'
external-ABI discipline directly: the DTB's representation is the flattened
devicetree (FDT) structure of the pinned Devicetree specification — explicit
token stream, fixed big-endian 32-bit cells, pinned specification revision —
and the contract forbids defining DTB content as "the serialization of a Rust
type". Versioning and compatibility behavior are explicit: the DTB content
profile is versioned with this contract and bound to a machine identity.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). W04
produces documentation, not code; the guidelines apply through the preflight,
the ABI-representation rules, and the completion-report discipline. The agent
then loads only the linked supporting file needed for its assigned step.
Before editing it must also follow the repository `AGENTS.md`, documentation
index, ADR baseline, P8 task book, and the P8-W04 plan. This document is the
proposed detailed design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W04 plan → this
design, with [P8-W02](../p8-w02-machine-contract-governance/README.md)
governance and the [P8-W03](../p8-w03-linux-boot-contract/README.md) boot
contract binding the fact sources. In particular:

- ADR-025 makes the Guest description DTB-first (ACPI later); ADR-024 makes
  the machine host-independent, so every DTB fact must derive from a machine
  fact or a routed Zelyr decision — never from the Host DTB or host
  observation. Host DTB reuse is out of scope by plan, and the P2
  PlatformInfo boundary is used only as the *prohibition boundary*: host
  facts stop there.
- The W02 governance assigns DTB node values to the firmware/DTB category:
  content values are routed (`Specification Investigation` for which facts
  Linux needs, `ADR Required` where a value is a machine fact); the W03 boot
  contract owns transfer/placement (B4) and the bootargs-via-`/chosen` rule
  (B1-3); the PSCI, timer, GIC, and console nodes must stay consistent with
  the categories [P8-W06](../p8-w06-psci-virtualization/README.md),
  [P8-W07](../p8-w07-linux-vgicv3/README.md),
  [P8-W08](../p8-w08-linux-timer-integration/README.md), and
  [P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md) will
  implement — consistency now, content authority with those contracts.
- [P8-W01](../p8-w01-entry-contract-reconciliation/README.md) supplies the
  constraint register (guest-untrusted, platform independence) the negative
  review enforces.

Classification: the DTB fact catalog ([01](01-dtb-fact-catalog.md)), the
consistency and host-leakage review criteria ([02](02-consistency-and-host-leakage-review.md)),
and the workflow ([03](03-workflow-and-validation.md)) are **Required**. A
machine-readable DTB profile schema and an automated DTB-vs-machine-contract
checker are **Reserved** (the checker's policy belongs to
[P8-W14](../p8-w14-machine-abi-compatibility/README.md)). Builder mechanics,
concrete node values before approval, Host DTB reuse, ACPI, Virtio device
design, and Host platform description are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02/W03 categories and the P2 platform-information boundary | [workflow](03-workflow-and-validation.md) step 1 | P8-V05 (W04-DV01) |
| Enumerate required Guest-visible DTB facts and authoritative sources | [DTB fact catalog](01-dtb-fact-catalog.md) §3–§12 | P8-V05 (W04-DV02/DV03) |
| Define consistency checks with approved machine and boot contracts | [consistency and leakage review](02-consistency-and-host-leakage-review.md) §2 | P8-V05 (W04-DV04) |
| Define negative checks for Board, SoC, physical address, IRQ, and firmware leakage | [consistency and leakage review](02-consistency-and-host-leakage-review.md) §3 | P8-V06 (W04-DV05) |
| Review Linux consumption without importing Host semantics | [DTB fact catalog](01-dtb-fact-catalog.md) §13, [workflow](03-workflow-and-validation.md) step 5 | P8-V05 (W04-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`
at `4e631ee`): no DTB contract, no DTB builder, and no Guest DTB artifact
exist; `docs/machine-types/` is a stub; the only DTB-related plans are the P2
platform-discovery packages (host-side intake, planned-only) and the P8
packages. The P2 boundary is observable in the plan set: P2-W01/W02/W07
define Host DTB intake and a host-side compatibility checker — all upstream,
planned, and Host-facing; nothing in the tree defines a Guest-visible DTB.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| The Guest-only DTB facts Linux must consume are specified | No Guest-DTB document exists; DTB facts are dispersed across ADR-024/025 and the task book | DTB contract document with fact groups D1–D10 | "Linux must consume X" is a contract only when enumerated with sources and consistency rules | W04 (this design) | W04-DV02 |
| Facts have authoritative sources | No source map exists | Per-fact source column: machine-contract category, boot-contract fact, W06–W09 contract, Linux/Devicetree binding documentation, or routed value | A DTB fact without a source is either a host leak or an invented ABI value | W04; sources per W02/W03/consumer contracts | W04-DV03 |
| Machine consistency is reviewable | No machine contract exists to check against (W02 governance in flight) | Consistency criteria keyed to W02 *categories*, not values ([02](02-consistency-and-host-leakage-review.md) §2) | Checks must work before values are frozen, or the review could not pass until the ADR lands | W04; categories from W02 | W04-DV04 |
| Host-leakage criteria exist | Nothing defines them for DTBs | Negative criteria: no host address/IRQ/board/SoC/firmware fact, with checkable node-level tests ([02](02-consistency-and-host-leakage-review.md) §3) | P8-V06 is defined as this negative review | W04 | W04-DV05 |
| Linux consumption without Host semantics | No binding-source discipline exists | Binding-source rule: every node cites a published Linux/Devicetree binding; no Zelyr-private binding in v1 ([01](01-dtb-fact-catalog.md) §13) | Custom bindings would make the DTB an unreviewed ABI and couple Linux config to Zelyr internals | W04 | W04-DV06 |
| No DTB output claimed | No builder exists | Explicit non-authorization of builder output in W04 artifacts | The plan excludes builder mechanics; the contract must say so to stop scope creep | W04 | W04-DV07 |

No row requires selecting a node value, so no decision blocker arises from
this design. The standing routed items (compatible string, addresses,
interrupt specifiers, PSCI function IDs, console registers) appear in the
fact catalog with their routes intact.

## Resolved design decisions and their authority

1. **Artifact and location.** One contract document,
   `docs/machine-types/guest-dtb-contract-v0.1.md`, with the status header
   required by `docs/README.md`. Rationale: identical to the boot-contract
   placement — Guest-visible virtual-machine model behavior lives in
   `docs/machine-types/` per the documentation layout, keeping the
   machine-contract family together and separate from `docs/abi/`
   (hypervisor/management ABIs).
2. **Family-contract binding.** The contract binds to the
   `rusthv-arm-virt-v1` machine identity and carries its own version and
   compatibility section per W02 governance §8.3. Rationale: DTB content is
   Guest-visible machine presentation; its changes are machine-compatibility
   events, not free edits.
3. **Wire-format discipline.** The contract states the DTB as an FDT wire
   format under a pinned Devicetree-specification revision (structure,
   big-endian 32-bit tokens/cells), requires any future generator to emit
   that format explicitly, and forbids defining content as a Rust-type
   serialization. Rationale: Coding-Guidelines external-ABI rule; the DTB is
   the primary Guest-visible wire artifact of P8.
4. **Source-authority rule.** Every DTB fact names exactly one authoritative
   source: a W02 machine category, a W03 boot-contract fact, a consumer
   contract (W06/W07/W08/W09), a published Linux/Devicetree binding, or a
   routed value placeholder. Rationale: makes the P8-V05/V06 reviews
   mechanical and prevents drift between DTB, machine, and boot facts.
5. **Generation source rule.** The future DTB builder is a pure function of
   the machine identity facts, the VM's boot configuration, and the contract
   — never of the Host DTB, host PlatformInfo, or host observation. Host
   PlatformInfo (P2) bounds what must *not* be disclosed; it is not an input.
   Rationale: ADR-024 host independence; ADR-042/043 layering; the plan's
   host-leakage outcome.
6. **v1 device-set posture.** The v1 DTB describes the minimal P8 device
   set: the console device (category per W02 C7, device owned by
   [P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md)) plus the
   firmware/timer/GIC/CPU/memory facts. Virtio devices are not described in
   the v1 DTB; any virtio-related DTB statement is limited to machine
   reservations under the W02 C9 rules and is absent until those resolve.
   Rationale: task book out-of-scope list (virtio transport is P9); the W02
   reservation prohibition.
7. **enable-method posture.** CPU nodes reference the PSCI enable-method so
   Linux's secondary-CPU path and the
   [P8-W06](../p8-w06-psci-virtualization/README.md) contract remain
   consistent; the PSCI function values themselves are routed. Rationale:
   the boot contract's B6 facts require a single secondary-start route;
   Linux's documented binding is the authority for the expression.

## Work breakdown and loading order

1. Read [the DTB fact catalog](01-dtb-fact-catalog.md) for the document
   contract, fact groups D1–D10, the binding-source rule, and the
   generation-source rule.
2. Read [the consistency and leakage review](02-consistency-and-host-leakage-review.md)
   for the positive (P8-V05) and negative (P8-V06) review criteria.
3. Apply [the workflow](03-workflow-and-validation.md) in order: verify
   inputs, draft the contract, run consistency and leakage reviews, wire
   discovery, record evidence.
4. Store the contract in
   `../p8-w04-guest-dtb-contract-record.md` when work starts, and review
   evidence in
   `../../verification/p8-w04-guest-dtb-contract-verification.md` when the
   review is exercised. Neither this design nor any record may claim W04
   complete or any DTB produced.

## Explicitly excluded interfaces

W04 authorizes no Rust type, function, trait, module, crate, public API, or
runtime artifact; no DTB builder; no concrete node, property, register,
address, interrupt-specifier, compatible-string, or function-ID value; no
Host DTB or PlatformInfo reuse; no ACPI statement; no Virtio device
description; and no Host platform description. The only machine-facing
surface it creates is the DTB contract document itself. A fact row that
cannot be written without fixing a value carries the value's route instead.
Any builder, schema file, DTB binary, or fixture appearing in a W04 artifact
is a scope conflict and must be stopped at review.

## Downstream handoff

Per the [P8 plan index](../../plans/README.md), W04 feeds W09–W10, W14, and
W16 (and, through the W02 categories, W06–W08, W15):

- **W06** receives the PSCI-node and enable-method facts (D5, D2): the DTB
  will present whatever PSCI conduit its contract approves, and its values
  must be registered into the W02 firmware/DTB category before publication.
- **W07** receives the GIC-node fact group (D7): its vGICv3 contract and the
  DTB interrupt/register descriptions must pass the consistency review
  together.
- **W08** receives the timer-node fact group (D6) under the same rule.
- **W09** receives the console-node and `/chosen` `stdout-path` facts (D8,
  D4) and the generation-source rule: its console device is described by the
  DTB, never by a Host-derived node.
- **W10** receives the CPU/topology facts (D2) its SMP bring-up will boot
  against, with topology expression still routed.
- **W14** receives the fact-source map as the drift-detection baseline: its
  machine-ABI compatibility test must detect drift in exactly the facts this
  contract enumerates.
- **W15** receives the DTB-relevant fixture inputs (bootargs string via D4)
  and the rule that fixture-owned DTB input values are pinned by the fixture,
  not invented per boot.
- **W16** receives the consistency and leakage criteria as the DTB portion
  of the automated regression's expected properties.

No consumer may generate or patch a DTB outside this contract's source rules,
and none may treat the DTB contract as frozen before the W02 gate passes.
