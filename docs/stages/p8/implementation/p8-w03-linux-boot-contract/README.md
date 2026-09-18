# P8-W03 Linux Boot Contract — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewable Linux Image, DTB, optional-initramfs, and bootargs
boot-input and lifecycle contract required by
[P8-W03](../../plans/p8-w03-linux-boot-contract.md).  
**Owner/change context:** P8-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W03. P8-W03 is a
contract-definition package: it produces one versioned, Guest-facing contract
document — the Linux boot contract — that fixes which boot facts must exist,
what each fact must state, which authority it binds to, and how it is tested,
without freezing any artifact address, loader mechanism, Linux configuration,
or machine-ABI value. It deliberately does **not** implement an image loader,
choose artifact addresses, define UEFI or filesystem loading, select a Linux
version or configuration (that is
[P8-W15](../p8-w15-reproducible-linux-fixture/README.md)), or publish the
machine ABI (governed by [P8-W02](../p8-w02-machine-contract-governance/README.md)).

The boot contract is a versioned external-facing contract in the sense of the
Coding Guidelines: every representation-bearing fact must state its
representation, width, endianness, version binding, and compatibility
behavior, and no fact may be defined as "the layout of a Rust struct". For
the boot contract this discipline is satisfied by binding Guest-visible
boot-state facts to pinned revisions of external standards — the Linux
AArch64 boot protocol documentation — rather than by inventing a Zelyr wire
format.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). W03
produces documentation, not code; the guidelines apply through the preflight,
the ABI-representation rules, and the completion-report discipline. The agent
then loads only the linked supporting file needed for its assigned step.
Before editing it must also follow the repository `AGENTS.md`, documentation
index, ADR baseline, P8 task book, and the P8-W03 plan. This document is the
proposed detailed design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W03 plan → this
design, with [P8-W02](../p8-w02-machine-contract-governance/README.md)
governance binding the contract family. In particular:

- ADR-021 fixes the boot evolution: ELF bare-metal first, Linux
  Image+DTB+initramfs next, UEFI later. The boot contract covers the Linux
  stage only; UEFI is out of scope by ADR.
- ADR-022 fixes Guest EL1 with no virtual EL2; ADR-008 keeps the hypervisor
  out of EL3. The boot contract states these as permanent Guest-visible
  facts, citing the ADRs, not as new decisions.
- The task book §1 leaves concrete machine values routed: no address,
  register encoding choice beyond the external protocol, PSCI subset, or
  device model is fixed here. The plan's out-of-scope list (image-loader
  implementation, artifact addresses, UEFI, filesystem loading, Linux
  configuration selection, formal machine ABI) is enforced at every step.
- [P8-W01](../p8-w01-entry-contract-reconciliation/README.md) supplies the
  P4/P5 execution and failure inputs the boot lifecycle facts assume
  (planned-only at design time, with stated failure boundaries);
  [P8-W02](../p8-w02-machine-contract-governance/README.md) supplies the
  fact-class and route machinery every fact row uses.

Classification: the boot-contract document and its fact catalog
([01](01-boot-contract-facts.md)), the fixture and shutdown-evidence
requirements ([02](02-fixture-and-shutdown-evidence.md)), and the review
workflow ([03](03-workflow-and-validation.md)) are **Required**. A
machine-readable boot-configuration schema and an automated contract-vs-fixture
consistency checker are **Reserved**. Image loading, artifact addresses,
UEFI/ACPI, filesystem loading, Linux version/configuration selection, the
machine specification itself, PSCI mechanism design (owned by
[P8-W06](../p8-w06-psci-virtualization/README.md)), and DTB content design
(owned by [P8-W04](../p8-w04-guest-dtb-contract/README.md)) are **Out of
Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W01/W02 and evidenced Guest EL1/Stage-2 lifecycle inputs | [workflow](03-workflow-and-validation.md) step 1 | P8-V04 (W03-DV01) |
| Define boot-input and lifecycle facts later detailed design must establish | [boot-contract facts](01-boot-contract-facts.md) §3–§8 | P8-V04 (W03-DV02/DV03) |
| Relate artifact ownership and reserved regions to Guest isolation | [boot-contract facts](01-boot-contract-facts.md) §7, [fixture and evidence](02-fixture-and-shutdown-evidence.md) §2 | P8-V04 (W03-DV04) |
| Specify fixture reproducibility and shutdown evidence needs | [fixture and evidence](02-fixture-and-shutdown-evidence.md) §3–§4 | P8-V04 (W03-DV05) |
| Review against Guest EL1 and no-host-leakage constraints | [workflow](03-workflow-and-validation.md) step 5 | P8-V04 (W03-DV06) |
| Facts testable without selecting implementation details | [workflow](03-workflow-and-validation.md) step 6 closure review | P8-V04 (W03-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`
at `4e631ee`): no Linux boot contract, machine specification, boot fixture,
Linux Image, initramfs, or bootargs artifact exists anywhere in the tree;
`docs/machine-types/` is a stub. The only Guest-boot facts in the repository
are the planned P4 Validation Guest packages (entry/exit in
[P4-W04](../../../p4/plans/p4-w04-vcpu-entry-exit.md), image construction in
[P4-W03](../../../p4/plans/p4-w03-guest-memory-image.md)), all without
implementation or verification records. No P8 predecessor is evidenced, so
every lifecycle input this contract assumes is a planned-only assumed
contract with a failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A reviewable Linux boot contract exists | No boot-contract document anywhere | Boot-contract document with the fact catalog of [01](01-boot-contract-facts.md) | A "contract" that is not a fixed, reviewable artifact cannot be consumed by W04/W09/W10/W15/W16 | W03 (this design) | W03-DV02 |
| Boot-vCPU state and entry facts | Nothing defines the Linux entry state; P4 defines Validation-Guest entry only (planned) | Fact group B2 binding the entry state to a pinned Linux AArch64 boot-protocol revision | The hypervisor's entry obligations must cite an external standard, not be invented | W03 catalog; values gated by W02 review | W03-DV03 |
| DTB transfer facts | No DTB contract exists; W04 owns content | Fact group B4 defining transfer and placement categories, deferring content to the W04 contract by reference | Boot transfer and DTB content are different contracts; conflating them would pre-empt W04 | W03 (transfer); W04 (content) | W03-DV03; W04 cross-review |
| Boot-artifact regions/lifetime and isolation | No artifact ownership rules exist | Fact group B7 lifetime model + [02](02-fixture-and-shutdown-evidence.md) §2 isolation requirements | Artifacts that alias host memory or outlive their owner would break Stage-2 isolation (ADR §19) | W03; backing facts assumed from P2/P4 contracts | W03-DV04 |
| Guest EL/MMU expectations, secondary state | EL1 fact exists in ADR-022 only; secondary behavior undefined | Fact groups B3 and B6 | Linux booting assumes a defined CPU-start model; P8 must state what exists at entry | W03; mechanism to W06 | W03-DV03 |
| Shutdown transition | No terminal-state boot fact exists; ADR §4.1 defines VM lifecycle states | Fact group B8 + [02](02-fixture-and-shutdown-evidence.md) §4 shutdown evidence | P8-V04 names shutdown facts explicitly | W03; PSCI mechanism to W06 | W03-DV05 |
| Reproducible fixture boundary | No fixture, no pinning rule | [02](02-fixture-and-shutdown-evidence.md) §3 pinning requirements | A boot contract whose tests cannot be reproduced is not testable (P8-V20 depends on it) | W03 defines requirements; W15 owns the fixture | W03-DV05 |

No row requires selecting a Linux version, address, or mechanism, so no new
decision blocker arises from this design. The standing routed items (artifact
address values, PSCI subset, console device) appear in the fact catalog with
their routes intact.

## Resolved design decisions and their authority

1. **Artifact and location.** One contract document,
   `docs/machine-types/linux-boot-contract-v0.1.md`, with the status header
   required by `docs/README.md`. Rationale: the boot contract is Guest-visible
   virtual-machine model behavior, so `docs/machine-types/` (machine-model
   contracts) is its authoritative home per the documentation layout; it is
   not a hypervisor management ABI, which is what `docs/abi/` holds. Moving
   it later would be a documentation relocation, not an ABI change.
2. **Family-contract binding.** The contract binds explicitly to the
   `rusthv-arm-virt-v1` machine identity and carries its own version and
   compatibility section, per the W02 governance §8.3. Rationale: a boot
   contract that silently applied to all machines would defeat ADR-024/040
   versioning.
3. **External-standard binding discipline.** Every Guest-visible boot-state
   fact binds to a pinned, named revision of the Linux AArch64 boot protocol
   documentation (and, for the DTB, to the Devicetree specification via the
   W04 contract); Zelyr states its hypervisor-side obligations and, where the
   standard permits freedom, fixes the chosen option through the W02 category
   review — never by local choice. Rationale: the task book §8 routes
   boot-state detail through specification investigation of AArch64/Linux
   sources, and the Coding Guidelines forbid implicit ABI representation.
4. **Fact catalog form.** The contract's content is the fact catalog of
   [01](01-boot-contract-facts.md): fact groups B1–B8, each fact with class
   (W02 §5), authority basis, and testability. Rationale: P8-V04 requires
   "testable" facts; a catalog with per-fact testability makes the review
   mechanical.
5. **Artifact lifetime posture.** All boot artifacts (Image, DTB, initramfs)
   are placed by the hypervisor into Guest RAM before entry and are owned by
   the Guest VM from entry onward; the hypervisor neither reclaims them while
   the VM runs nor relies on their contents after entry, and all boot artifacts
   are backed only by allocatable Host pages per the P2/P4 ownership
   contracts. Rationale: this is the only posture consistent with Guest
   isolation (ADR §19) and the P4-W03 ownership facts; it is stated as a
   Guest-visible fact through the W02 gate.
6. **Secondary-CPU boot posture.** At boot entry exactly one vCPU (the boot
   vCPU) executes Linux; secondary vCPUs exist in a hypervisor-held state and
   start only through the PSCI path owned by
   [P8-W06](../p8-w06-psci-virtualization/README.md). The boot contract owns
   the boot-time observable; W06 owns the start mechanism. Rationale: the plan
   assigns secondary *state* to W03 and PSCI lifecycle to W06; the split
   follows the task book work map.
7. **Shutdown binding.** Guest-initiated shutdown is expressed through the
   PSCI system-level function routed to W06; the boot contract owns the
   resulting lifecycle evidence — the VM reaches a defined terminal lifecycle
   state (ADR §4.1) with diagnostics and no host interference. Rationale:
   separates contract fact from mechanism, per decision 6.

## Work breakdown and loading order

1. Read [the boot-contract facts](01-boot-contract-facts.md) for the fact
   groups, classes, authority bindings, and routes.
2. Read [fixture and shutdown evidence](02-fixture-and-shutdown-evidence.md)
   for the reproducibility requirements and the isolation/shutdown review
   criteria.
3. Apply [the workflow](03-workflow-and-validation.md) in order: verify
   inputs, draft the contract, review, wire discovery, record evidence.
4. Store the contract in
   `../p8-w03-linux-boot-contract-record.md` when work starts, and review
   evidence in
   `../../verification/p8-w03-linux-boot-contract-verification.md` when the
   review is exercised. Neither this design nor any record may claim W03
   complete or the contract frozen.

## Explicitly excluded interfaces

W03 authorizes no Rust type, function, trait, module, crate, public API, or
runtime artifact; no artifact address, Image header value, register encoding
beyond the cited external protocol, PSCI function list, or DTB node value; no
image-loader design; and no Linux version or configuration choice. The only
machine-facing surface it creates is the boot-contract document itself. A
fact row that cannot be written without fixing such a value states the value's
route instead. Any code, schema, or fixture appearing in a W03 artifact is a
scope conflict (fixture ownership is W15's) and must be stopped at review.

## Downstream handoff

Per the [P8 plan index](../../plans/README.md), W03 feeds W04, W09–W10, and
W15–W16, and interacts with W06/W12 through the W02 categories:

- **W04** receives the DTB transfer and placement fact group (B4) and the
  rule that DTB content authority is its own contract; its consistency review
  must cross-check against B1–B4.
- **W09** receives the boot-input facts (B1–B5) it boots Linux against and
  the shutdown evidence needs (B8, [02](02-fixture-and-shutdown-evidence.md)
  §4) for the single-vCPU path to interactive userspace.
- **W10** receives the secondary-state fact (B6): secondaries are
  hypervisor-held at boot and start only via the W06 PSCI path.
- **W15** receives the pinning requirements of
  [02](02-fixture-and-shutdown-evidence.md) §3: which fixture categories must
  be version-pinned or reproducibly generated for the contract to be testable.
- **W16** receives the boot markers and shutdown outcomes the contract's
  testability section defines as evidence expectations for the automated
  matrix.
- **W06** (via the W02 firmware/DTB category) receives the constraint that
  the PSCI mechanism must realize the shutdown and secondary-start facts of
  B6/B8 without redefining them; **W12** receives the artifact-region
  categories as memory-model boundary inputs.

No consumer may treat the contract as frozen before the W02 gate passes, and
none may add a Guest-visible boot fact outside this contract's change rules.
