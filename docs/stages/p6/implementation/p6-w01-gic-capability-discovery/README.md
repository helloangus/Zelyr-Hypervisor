# P6-W01 GIC Capability Discovery — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The capability-driven decision that the current platform can safely
support the P6 GICv3 and virtualization work, required by
[P6-W01](../../plans/p6-w01-gic-capability-discovery.md).  
**Owner/change context:** P6-W01 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W01. It converts the bounded
work-package plan into a typed capability model, a pure reconciliation of P1/P2
platform facts against the P6 interrupt requirements, and an explicit set of
acceptance, rejection, and escalation outcomes. It deliberately does **not**
touch GIC hardware, choose register sequences, initialize anything, or treat a
QEMU observation as an architectural guarantee. Read-only hardware identity
probing is *specified here as a contract* and *executed* by
[P6-W02](../p6-w02-physical-gic-bring-up/README.md) during bring-up.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
[AGENTS.md](../../../../../AGENTS.md), [documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P6 task book](../../task-book-v0.1.md), and the P6-W01 plan. This document is
the proposed detailed design for those changes; it is not a completion record
and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W01 plan → this design
→ Coding Guidelines. In particular:

- ADR-032 makes GICv3 the interrupt mainline and ADR-033 keeps GICv2 out of the
  mainline, so the supported posture fixed below is GICv3-family only.
- ADR-041–ADR-045 and ADR-052 require capability-driven layering: every
  decision in this design keys on capability facts, never on a platform name.
- The task book reserves "the exact supported GIC revision after authoritative
  specification/platform review" to detailed design; the posture in
  [scope and foundations](01-scope-and-foundations.md) §2 is that selection,
  and it binds only the P6 stage.
- The P2 PlatformInfo GIC/timer/CPU facts and the P1 CPU capability inventory
  are **assumed input contracts**
  ([p2-w02](../../../p2/plans/p2-w02-platform-discovery-normalization.md),
  [p1-w03](../../../p1/plans/p1-w03-aarch64-capability-inventory.md),
  [p2-w10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md)). If they
  deliver differently than this design assumes, the failure boundary in §1.4 of
  the same file applies; P6 does not repair or silently redesign them.
- The task book's open classification "exact supported GIC revision, feature
  availability, and platform register/range interpretation" is a Platform and
  Specification Investigation: this design names registers symbolically and
  pins the authoritative specification revision at implementation time
  ([workflow](04-implementation-workflow.md) step 1). It does not freeze a GIC
  specification revision from the design side.

Classification: the capability model, reconciliation, verdict/escalation
taxonomy, and downstream entry contract in
[01](01-scope-and-foundations.md), [02](02-architecture-and-state.md), and
[03](03-code-contracts-capability-model.md) are **Required**. Extended SPI/SGI
/PPI ranges, GICv3.1-only features, GICv4 vSGI/vLPI features, and the
virtualization-interface deep capability review consumed by W08 are
**Reserved** with recorded triggers. GIC initialization, register sequencing,
a generic irqchip framework, Guest-visible GIC state, ITS/LPI/MSI, GICv2, and
any board-name branch are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect ADR, task book, P0–P5 handoffs, actual P2 facts | [Scope and foundations](01-scope-and-foundations.md) §1–§2, [README ledger](#current-state-findings-and-goal-to-baseline-ledger) | P6-V01 (W01-DV01) |
| Reconcile GIC, Redistributor, CPU-interface, virtualization, IRQ, MMIO inputs | [Architecture and state](02-architecture-and-state.md) §2, [contracts](03-code-contracts-capability-model.md) §2–§3 | P6-V01 (W01-DV02, DV03) |
| Planned acceptance/rejection outcomes for missing, unsupported, incomplete, malformed, conflicting facts | [Scope and foundations](01-scope-and-foundations.md) §4, [contracts](03-code-contracts-capability-model.md) §4 | P6-V01 (W01-DV04) |
| Capability-driven layering, Guest-untrusted inputs, facts-vs-policy separation review | [Scope and foundations](01-scope-and-foundations.md) §5, [workflow](04-implementation-workflow.md) step 6 | P6-V01 (W01-DV05) |
| Entry contract, open investigations, downstream assumptions for W02–W13 | [Scope and foundations](01-scope-and-foundations.md) §6, [validation and handoff](05-validation-and-handoff.md) §3 | P6-V01 (W01-DV06, DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is the P0 documentation
scaffold. There is no Cargo workspace, no Rust source, no target
configuration, and no implemented P1–P5 mechanism. Tracked artifacts relevant
here are plans only: the P2 platform-discovery and handoff plans, the P1
capability-inventory plan, and the P6-W01..W13 plans; implementation records
exist for P0-W01/P0-W02 documentation work only, and no `docs/stages/p6/`
implementation or verification artifact exists beyond the stage index. Every
upstream fact this design consumes is therefore an assumed contract with the
failure boundary stated in
[scope and foundations](01-scope-and-foundations.md) §1.4.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Reviewable, capability-driven decision" that the platform supports P6 GICv3 work | No capability model, requirement table, or decision artifact exists anywhere | Typed capability inputs, requirement classes, and a reconciliation decision type ([02](02-architecture-and-state.md), [03](03-code-contracts-capability-model.md)) | A decision cannot be reviewed unless its inputs, rules, and outcomes are explicit and machine-checkable | W01 (this design) | W01-DV02 model review |
| GIC version, Redistributor coverage, CPU-interface, virtualization, IRQ, MMIO facts reconciled | P2 plans promise these facts; no fact instance exists; P1 promises CPU virtualization-extension facts | Assumed-contract fact schemas with named sources and a static reconciliation function | Reconciliation is only well-defined when each input's source, shape, and authority are fixed in advance | P2-W02/P1-W03 supply facts; W01 owns schemas and rules | W01-DV03 reconciliation-table review against plan wording |
| Rejection when unusable | No rejection taxonomy or escalation path exists | Verdict taxonomy (usable/absent/unsupported/incomplete/malformed/contradictory) plus per-verdict escalation owners | "Rejected when unusable" requires a determinate mapping from every defect class to a safe stop and an owner | W01 | W01-DV04 verdict-matrix review |
| No GIC initialization or register sequences | Nothing exists to initialize; W02 owns bring-up | Probe *contracts* (read-only identity checks) handed to W02 without executable sequencing | W02 needs expected-identity facts to confirm against; giving W02 more than expectations would cross the W01/W02 boundary | W01 defines; W02 executes | W02 record cross-reference |
| Downstream W02–W13 consumability | No entry contract exists | Published decision artifact and per-consumer assumptions ([05](05-validation-and-handoff.md) §3) | Consumers must act on the decision without re-deriving it | W01 | W01-DV07 consumer review |

No row requires inventing a crate, target, platform constant, or runtime
policy, so no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Supported GIC posture (stage-local design freedom under the task book's
   Reserved clause):** GICv3-family with the system-register CPU interface,
   affinity routing (ARE) enabled, Non-secure single-security-state
   assumption subject to runtime DS confirmation, SGI/PPI/SPI only. Details
   and rationale in [scope and foundations](01-scope-and-foundations.md) §2.
2. **Discovery is static; confirmation is delegated:** W01 reconciles declared
   P1/P2 facts only; all hardware contact is specified as read-only probe
   contracts that W02 executes against W01's expected-identity outputs.
   Rationale: the plan forbids initializing hardware in W01, and W02 already
   owns the first hardware access; a second probe path in W01 would create two
   hardware-access owners.
3. **Verdict taxonomy:** every required capability input resolves to exactly
   one of usable, absent (required/optional), unsupported, incomplete,
   malformed, contradictory; each verdict has a fixed escalation owner
   ([scope and foundations](01-scope-and-foundations.md) §4).
4. **Graded overall decision:** `ReadyForP6`, `PhysicalOnly` (physical
   interrupt work may proceed; virtualization-interface work is blocked), or
   `Rejected`. Rationale: ADR-032 makes the physical chain the mainline; the
   virtualization interface is required for W08+ but its absence must not
   orphan already-justified physical work; the graded outcome keeps P6-V01's
   rejection requirement exact instead of all-or-nothing.
5. **Interrupt-ID facts:** PPI assignments used by P6 (Host EL2 timer PPIs,
   maintenance PPI) are required declared facts sourced from P2 platform
   facts; SPI support is range-based from P2/GICD facts, never a device list.
6. **No board constants:** QEMU virt and Orange Pi 3B values may appear only
   as fixture *inputs* to host-side reconciliation tests, never as Core
   defaults (ADR-043, ADR-052, Coding Guidelines layering rule).

## Work breakdown and loading order

1. Read [scope and foundations](01-scope-and-foundations.md) for the assumed
   prerequisite contracts, the supported GIC posture, the scope classification,
   and the failure boundary when P1/P2 deliver differently.
2. Read [architecture and state](02-architecture-and-state.md) for the logical
   modules, capability objects, decision lifecycle, and the module-layering
   rules.
3. Read [code contracts — capability model](03-code-contracts-capability-model.md)
   for the exact types, the reconciliation function, probe contracts, and
   pseudocode before writing any code.
4. Apply the changes in the order stated in the
   [implementation workflow](04-implementation-workflow.md); each step names
   its acceptance and failure boundary.
5. Store actual commands, results, and run/not-run status in
   `../../verification/p6-w01-gic-capability-discovery-verification.md`, and
   record factual implementation status in
   `../p6-w01-gic-capability-discovery-record.md` only when implementation
   begins. Neither this design nor a written record may claim W01 complete.

## Explicitly excluded interfaces

No MMIO access, system-register access, hardware-init sequence, irqchip
framework, handler-registration API, Guest-visible GIC model, or platform-name
branch is designed or authorized by W01. The probe contracts in
[03](03-code-contracts-capability-model.md) §5 define *what W02 must confirm*,
not executable register code; implementing them is W02 scope. The runtime
interrupt subsystem APIs begin at W02/W03; W01 authorizes only the capability
and decision types in [03](03-code-contracts-capability-model.md).

## Downstream handoff

- **W02** receives the supported posture, the expected-identity facts, and the
  three probe contracts; its first bring-up step confirms identity against
  them and fails safe on mismatch. W02 must not extend or reinterpret the
  verdict taxonomy.
- **W03–W05** receive the usable-IRQ-range facts (SGI/PPI/SPI ranges,
  maintenance PPI, timer PPIs) and the overall decision; they may assume
  `ReadyForP6` or `PhysicalOnly` only after W02's confirmation evidence
  exists.
- **W08** receives the virtualization-interface findings and probe evidence
  location; under `PhysicalOnly` its entry condition fails with a recorded
  stage block, not a redesign.
- **W11–W12** receive the rejection/escalation taxonomy as the classification
  vocabulary for platform-caused interrupt failures.
- **W13** receives the capability report as the first entry of the P6 evidence
  chain (P6-V01) and the open-investigation list (P6-V27).
- No consumer receives a frozen API, a machine-specific IRQ layout, or a
  QEMU-derived hardware guarantee.
