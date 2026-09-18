# P1-W01 Reference Boot Contract — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The canonical QEMU `virt` reference boot path, its entry-state
assumptions, the pre-transfer validation and rejection boundary, and the
validated-entry transfer guarantee required by
[P1-W01](../../plans/p1-w01-reference-boot-contract.md).  
**Owner/change context:** P1-W01 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W01. W01 converts the open
question "how does firmware reliably deliver the hypervisor into Non-secure
EL2?" into one reviewable contract with an enforced boundary: a single
canonical boot path, a written set of entry assumptions, an explicit
rejection behavior for disallowed environments, and a transfer guarantee that
downstream packages may rely on. It deliberately does **not** design the
runtime that the boot path delivers into (W02), parse or validate DTB content
(P2), provide a bootloader framework, or describe real-board bring-up.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-boot-contract.md](01-boot-contract.md) — the contract content itself:
  authoritative artifact groups, the canonical path, every entry-state
  assumption, the rejection-boundary semantics, the transfer guarantee, and
  the canonical invocation recipe. Load this first for any step.
- [02-entry-validation-contracts.md](02-entry-validation-contracts.md) — code
  contracts for the tiered entry validation, the rejection reporter, and the
  post-transfer routing refinement, with pseudocode and failure boundaries.
  Load for the implementation steps that touch the boot entry.
- [03-implementation-and-review.md](03-implementation-and-review.md) — ordered
  workflow, validation matrix, error/security/observability model, and handoff
  checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W01
plan](../../plans/p1-w01-reference-boot-contract.md). This document is the
proposed detailed design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W01 plan → this design
→ Coding Guidelines. In particular:

- The ADR fixes QEMU `virt` as the reference platform and Non-secure EL2 as
  the entry privilege (ADR-003, ADR-008: no EL3 firmware is implemented). The
  ADR's P1 stage text additionally asks that the QEMU boot method be determined
  *and* both the direct-kernel and U-Boot/TF-A recipes be recorded; this design
  reconciles the two statements in
  [Resolved decision 1](#resolved-design-decisions-and-their-authority).
- The P1 task book requires a "canonical QEMU boot contract and entry
  validation boundary" with P1-V01/P1-V02 evidence; unsupported entry must
  produce a reason and no normal continuation.
- The plan scopes entry point, image/loading conditions, Non-secure EL2
  requirement, boot CPU, MMU/cache assumptions, boot parameters, DTB
  treatment, minimum memory, and canonical invocation/documentation — all
  contract-level artifacts. The executable part of W01 is exactly the
  enforcement of that contract at the entry point: the pre-transfer checks and
  the rejection reporter of
  [02-entry-validation-contracts.md](02-entry-validation-contracts.md).
- The P0 target/build baseline ([P0-W03](../../../p0/plans/p0-w03-aarch64-build-target-baseline.md))
  and the P0 QEMU-runner entry ([P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md))
  are **assumed contracts** known at plan level. The image form and the
  invocation mechanics are consumed from them; a missing or contradicting
  prerequisite is an upstream defect to record, never permission to redesign
  P0 inside P1 (task book §1).

Classification: the boot contract content, the tiered entry validation, the
rejection boundary and reporter, the transfer guarantee, and the canonical
invocation documentation are **Required**. The second (U-Boot/TF-A) boot
recipe, canonical-parameter variations beyond the fixed reference values, and
any hardened/re-signed boot flow are **Reserved** with recorded triggers.
UEFI/bootloader frameworks, EL3/TF-A bring-up, Orange Pi boot, Guest boot
protocols, general DTB or platform discovery, and any memory-map or platform
data model are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect P0's target, image and QEMU entry contracts (work seq 1) | [Workflow](03-implementation-and-review.md) step 1 | W01 closure review (W01-DV01) |
| Record supported entry state, inputs, ranges and assumptions (work seq 2) | [Boot contract](01-boot-contract.md) §2–§4, §6–§7 | P1-V01 (W01-DV02) |
| Define normal continuation and fail-fast outcomes for invalid entry state (work seq 3) | [Boot contract](01-boot-contract.md) §5; [validation contracts](02-entry-validation-contracts.md) §2–§5 | P1-V02 (W01-DV03, DV05) |
| Reconcile the contract with ADR layering and P1 reserved scope (work seq 4) | [Boot contract](01-boot-contract.md) §8 | P1-V01 (W01-DV04) |
| Define acceptance evidence for normal and unsupported reference boots (work seq 5) | [Boot contract](01-boot-contract.md) §9; [workflow](03-implementation-and-review.md) §3 | P1-V01, P1-V02 (W01-DV06) |
| Hand off the stable entry contract to runtime and automation (work seq 6) | Downstream handoff below; [workflow](03-implementation-and-review.md) §5 | W01 closure review (W01-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs` at
`4e631ee`): the repository is a P0 documentation scaffold. `git ls-files`
shows no Cargo manifest or workspace, no Rust sources, no linker script, no
target JSON, no QEMU runner (`scripts/.gitkeep` only), and no CI workflow. The
P0 plans that W01 consumes (target/build baseline P0-W03, QEMU runner entry
P0-W09, build metadata P0-W16) are planned; P0 has implementation designs only
for W01/W02 and neither is implemented. No boot-contract document exists
anywhere in the tree. Every executing prerequisite of W01's evidence is
therefore a planned, not present, artifact — the ledger states who owns what.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| One reviewable canonical boot path (P1-V01) | No boot contract, recipe, or entry convention exists in any tracked file | The boot contract content of [01-boot-contract.md](01-boot-contract.md), materialized per [the workflow](03-implementation-and-review.md) | A "canonical path" cannot be reviewable while it exists only as plan prose | W01 (this design) | W01-DV02 contract review |
| Canonical boot reaches validated Non-secure EL2 (P1-V01) | No image, no entry code, no runner | Entry validation implemented in the W02-delivered entry per [02-entry-validation-contracts.md](02-entry-validation-contracts.md); execution evidence via W10 | "Reaches validated EL2" is a runtime property of the integrated boot path, not a document | W01 (contract + checks); W02 (entry module); W10 (execution) | W01-DV06 review; execution deferred to W10 |
| Disallowed environment rejected with a reason and no normal continuation (P1-V02) | No validation or rejection code exists | The tiered validation boundary, the rejection reporter, and the bounded-stop semantics of [02-entry-validation-contracts.md](02-entry-validation-contracts.md) | Rejection is only explicit if the checks, the reason vocabulary, and the stop are all fixed before implementation | W01 (this design) | W01-DV03/DV05; scenario NC1 execution belongs to W11 |
| Boot CPU, DTB, and memory assumptions are explicit | Nothing recorded | Contract sections §3 (entry state) and §4 (boot parameters, DTB, memory) | W02 must know exactly which register contents it may rely on | W01 | W01-DV02 |
| Canonical invocation is documented and reusable by automation (plan handoff) | No runner (P0-W09 planned) and no recipe | The recipe contract of [01-boot-contract.md](01-boot-contract.md) §7, with exact spelling recorded at implementation against the P0 runner | W10's regression must boot the same path W02 validates, through one runner entry | W01 (recipe fields); P0-W09 (runner); W10 (driver) | W01-DV06; W10 consumes |
| Layering and reserved-scope reconciliation (work seq 4) | Unreconciled | §8 layering statement of [01-boot-contract.md](01-boot-contract.md) | The reference boot path touches platform constants; its placement must be justified against ADR-041/043 explicitly | W01 | W01-DV04 |

No row above requires inventing a crate layout, target triple, runner
implementation, or platform-discovery mechanism; those remain with their
owning packages. No decision blocker is outstanding for this design itself.

## Resolved design decisions and their authority

1. **Canonical path is QEMU `virt` direct-kernel boot with virtualization
   enabled; the U-Boot/TF-A path is recorded as a Reserved second recipe.**
   The plan demands one reviewable path; the ADR P1 stage text asks that both
   recipes be recorded. This design satisfies both: the canonical contract is
   defined only for direct-kernel boot (the smallest deterministic bring-up
   surface), and the alternative is documented as a Reserved recipe with its
   intended role — never as a validated path. Authority: P1-W01 plan goal; ADR
   P1 stage tasks; task book scope classification.
2. **Entry privilege contract is Non-secure EL2, with the Non-secure part an
   explicitly documented assumption rather than a pre-transfer check.**
   `CurrentEL` is the only privilege observable without exception machinery
   (P1 owns no vector table before W05), so the pre-transfer check enforces
   `CurrentEL == EL2`; the Non-secure state is a property of the canonical
   recipe (ADR-008 firmware contract) recorded as an assumption with a
   defense-in-depth re-derivation by W03's required-fact check. Authority:
   ADR-008; plan scope ("Non-secure EL2 requirement").
3. **Validation is tiered: everything gateable without a Rust runtime is
   checked pre-transfer in the boot entry; everything else is either a
   documented assumption or a post-transfer recorded fact.** The pre-transfer
   tier owns exactly two checks (exception level; DTB-pointer presence). More
   would make the assembly a runtime; fewer would make the rejection boundary
   cosmetic. Authority: plan work seq 3; task book P1-V02.
4. **The rejection reporter is a fixed-token, reference-platform polling write
   followed by a bounded stop; it is not a console.** The rejection path runs
   before any W02/W06 infrastructure exists, so it uses the minimal raw
   reference-UART write owned by the entry module, emits one line of the fixed
   token vocabulary of
   [02-entry-validation-contracts.md](02-entry-validation-contracts.md) §3,
   and never returns. The early console (W06) is a different mechanism with a
   different owner; no sharing is authorized. Authority: W09 routing matrix
   (`entry` row: reason observable, bounded stop, no runtime markers).
5. **Transfer guarantee: reaching the W02 Rust entry point implies the
   pre-transfer tier passed.** Downstream packages (W02, W09) may rely on the
   entry state declared in §3 of the contract once execution reaches the Rust
   entry; W02-owned code records the `entry` lifecycle events on that
   authority (aligning with the [W09](../p1-w09-initialization-sequencing/README.md)
   design's decision 6). W01 does not implement the recording. Authority: W09
   §8 seam table; W01 plan handoff line.
6. **Boot-parameter contract is x0-only; x1–x3 are reserved and must not be
   relied upon.** The direct-kernel convention delivers the DTB pointer in x0;
   P1 retains it for P2 but validates nothing about its content (P2 owns DTB
   intake). The image consumes no other firmware register content it can
   establish itself — this phrasing is inherited by W02's runtime design.
   Authority: plan scope ("boot parameters, DTB treatment").
7. **The boot contract's implementation-time fields (reference CPU model,
   minimum memory value, image form, exact invocation spelling) are selected
   once, at implementation, from the P0 runner and artifact baselines, and
   recorded in the implementation record before the first verdict-bearing
   boot.** Fixing them here would either invent QEMU facts no authority has
   frozen or pre-empt the P0 baseline. Authority: task book §1 (missing P0
   inputs are upstream defects); W10 precedent (bounded value selected at
   implementation).
8. **Post-transfer failures of entry-relevant conditions route via the panic
   route with `entry` phase attribution — a recorded refinement of the W09
   routing matrix, not a silent divergence.** The W09 matrix has an `entry`
   row (pre-transfer rejection) and a `runtime` row; a post-transfer
   entry-condition failure sits between them. This design fixes the
   refinement in
   [02-entry-validation-contracts.md](02-entry-validation-contracts.md) §5 and
   flags it to W09 for review. Authority: W09 H4 rule (route established by an
   earlier phase) — satisfied because the panic route is established by W02
   runtime establishment before any post-transfer check runs.

## Work breakdown and loading order

1. Load [01-boot-contract.md](01-boot-contract.md) to understand what the
   contract must state, which fields are implementation-selected, and where
   the boundaries of W01 lie. Every step depends on it.
2. Load [02-entry-validation-contracts.md](02-entry-validation-contracts.md)
   when working on the boot entry itself: the tier checks, the rejection
   reporter, and the transfer-guarantee mechanics are all specified there as
   contracts to be implemented inside the W02-delivered entry module.
3. Execute the steps in the order given in
   [03-implementation-and-review.md](03-implementation-and-review.md): P0
   contract inspection first, contract content second, boundary semantics
   third, layering review fourth, evidence definitions fifth, handoff sixth.
4. Record implementation decisions and deviations in
   `../p1-w01-reference-boot-contract-record.md` when implementation begins,
   and validation commands, environments, and outcomes in
   `../../verification/p1-w01-reference-boot-contract-verification.md` when
   evidence exists. Neither file may exist yet, and neither this design nor a
   record may claim W01 complete. The contract content produced by W01 is
   materialized in the implementation record; the stage-level contract
   document that assembles it is owned by
   [P1-W12](../p1-w12-p1-documentation-handoff/README.md).

## Explicitly excluded interfaces

No console, logging, panic-handler, capability-report, register-baseline, or
vector mechanism is designed here; those belong to W02–W07. No DTB parsing,
memory map, PlatformInfo, allocator, or platform-discovery artifact is
authorized — the DTB pointer is retained unexamined. No U-Boot/TF-A recipe is
designed beyond its Reserved recording. No linker layout is invented: the
image form and entry placement are consumed from the P0 target baseline and
recorded, not designed. No public ABI, wire format, or persistent layout is
introduced; the only machine-facing surface is the rejection-token vocabulary,
which is a P1-internal boot diagnostic, not an ABI.

## Downstream handoff

- **[P1-W02](../p1-w02-minimal-rust-el2-runtime/README.md)** receives the
  declared entry state (§3 of the contract), the transfer guarantee, and the
  exact contracts for the pre-transfer tier, the rejection reporter, and the
  shared boot-entry constant — implemented inside W02's entry module per
  [02-entry-validation-contracts.md](02-entry-validation-contracts.md).
- **[P1-W03](../p1-w03-aarch64-capability-inventory/README.md)** receives the
  EL entry guarantee as the precondition of its inventory and the obligation
  to re-derive the execution level as a required fact (defense in depth).
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  transfer guarantee its `entry` records delegate to, the rejection route of
  the matrix's `entry` row, and the post-transfer routing refinement of §5
  (raised for review, per its conflict rule).
- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** receives the
  canonical invocation recipe fields and the rejection-token vocabulary as
  evidence classes; it owns verdict rules and tokens for its own marker
  classes.
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives
  scenario NC1's target: the rejection boundary, its reason vocabulary, and
  the no-continuation guarantee. The reference platform property spelling for
  the EL2-less environment is selected at implementation from the P0 runner
  contract.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  accepted contract content for the future `aarch64-boot-contract.md` assembly
  and the recorded limitations (Non-secure assumption; single-CPU canonical
  path; unvalidated second recipe).
- **P2** receives a validated Non-secure EL2 entry environment and a retained,
  unexamined DTB pointer; DTB validation and discovery remain P2 scope.

A coding agent completing W01 must leave the handoff checklist in
[03-implementation-and-review.md](03-implementation-and-review.md) answerable
without inspecting W01 source code.
