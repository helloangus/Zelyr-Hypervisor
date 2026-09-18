# P1-W09 Initialization Sequencing — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The ordered early-initialization lifecycle from boot entry to stable
EL2 runtime required by [P1-W09](../../plans/p1-w09-initialization-sequencing.md).  
**Owner/change context:** P1-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W09. W09 is the only P1 package
whose deliverable is integrative code: it composes the mechanisms planned by
W01–W08 into one explicit lifecycle with visible prerequisites, markers, and
failure boundaries. It deliberately does **not** design any of those mechanisms
themselves, does not introduce a scheduler/service lifecycle, does not create a
global `Manager` object, and does not add Guest, SMP, GIC, discovery, or
allocator behavior.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- [01-init-state-machine.md](01-init-state-machine.md) — the ordered lifecycle:
  phase definitions, prerequisites, marker model, failure routing, and the
  partial-init and hidden-dependency rules. Load this first for any step.
- [02-interface-contracts.md](02-interface-contracts.md) — code contracts
  (names, preconditions, concurrency, failure guarantees, pseudocode) for the
  sequencer, phase type, tracker, and marker emission. Load for the
  implementation steps.
- [03-implementation-and-review.md](03-implementation-and-review.md) — ordered
  workflow, validation matrix, observability model, and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W09
plan](../../plans/p1-w09-initialization-sequencing.md). This document is the
proposed detailed design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W09 plan → this design
→ Coding Guidelines. In particular:

- The task book requires an "ordered early-initialization lifecycle and failure
  boundaries" and P1-V15 demands reviewable prerequisites, order, and failure
  results with no hidden initialization dependency.
- The plan fixes the thematic span Boot Entry → Minimal Runtime → Early
  Diagnostics → Architecture Validation → Exception Environment → EL2 Baseline
  → Host Stage-1 → Stable Runtime. The plan index and the W05/W06 prerequisite
  lines additionally fix the only legal construction order; this design
  reconciles the two in [Resolved decision 1](#resolved-design-decisions-and-their-authority).
- W01–W08 supply the mechanisms. Their detailed designs are being prepared in
  parallel; this design treats each as an **assumed contract** known only at
  plan level, with explicit failure boundaries. It fixes only the integration
  vocabulary (phase identifiers, tracker, adapter seams) that no other authority
  has fixed.
- The plan's handoff line states this is not a general service lifecycle design:
  the lifecycle covers the single boot CPU from firmware handoff to stable idle
  and nothing else.

Classification: the lifecycle state machine, marker model, failure-routing
matrix, tracker and adapter contracts, and the sequencer wiring are
**Required**. Retrospective marker materialization points that W06's design may
redefine, and any renaming of the W09-owned symbols for consistency with
sibling designs, are **Reserved** with recorded triggers. Guest entry, Stage-2,
SMP, GIC, platform discovery, allocators, service lifecycles, runtime policy
state machines, VM/vCPU lifecycles, and any rollback/deinitialization
infrastructure are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Collect package contracts and order their required conditions (work seq 1) | [State machine](01-init-state-machine.md) §1–§2 | P1-V15 (W09-DV01) |
| Define lifecycle states, legal transitions, failure outcomes (work seq 2) | [State machine](01-init-state-machine.md) §2, §4 | P1-V15 (W09-DV01, DV04) |
| Ordered lifecycle realized in the boot path (task book Required scope) | [Interface contracts](02-interface-contracts.md) §2–§5 | P1-V15 (W09-DV01, DV04) |
| Integrate markers and diagnostic availability at every transition (work seq 3) | [State machine](01-init-state-machine.md) §3; [contracts](02-interface-contracts.md) §4 | P1-V15 (W09-DV02); supports P1-V10 |
| Review hidden dependencies, partial-init behavior, reserved boundaries (work seq 4) | [State machine](01-init-state-machine.md) §5 | P1-V15 (W09-DV03) |
| Lifecycle acceptance review and repeat-boot evidence expectations (work seq 5) | [Workflow](03-implementation-and-review.md) steps 5–7 and validation matrix | P1-V15 (W09-DV05–DV07) |
| Hand off the ordered boot contract to regression and documentation (work seq 6) | Downstream handoff below; [workflow](03-implementation-and-review.md) handoff checklist | W09 closure review (W09-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs` at
`4e631ee`): the repository is a P0 documentation scaffold. `git ls-files`
shows no Cargo manifest or workspace, no Rust sources (`hypervisor/src/` and
`crates/` contain only `.gitkeep`), no target JSON, linker script, or build
script, no QEMU runner (`scripts/.gitkeep` only), and no CI workflow. P0 has
implementation designs for W01 and W02 only, and P0-W02's design is itself
still proposed. Every P1 plan W01–W12 exists as an approved plan; `docs/stages/
p1/implementation/` and `docs/stages/p1/verification/` contain only `.gitkeep`
markers. No P1 design named by this document other than the four being prepared
in this branch can be observed; the W01–W08 designs are therefore assumed
contracts, not inspected sources.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Early lifecycle is explicit from entry to stable runtime | No boot-path code exists; W01–W08 contracts exist only at plan level | One ordered lifecycle definition binding the eight supplying contracts into phases with prerequisites | An ordered composition cannot exist while each package states its conditions separately | W09 (this design); phase mechanisms per W01–W08 | W09-DV01 lifecycle review |
| Prerequisites and failure results are visible (P1-V15) | Plans name failure behavior thematically; no per-phase routing exists | Failure-routing matrix mapping every phase failure to an established diagnostic route | A failure result is visible only if the route that reports it already exists when the phase runs | W09; routes per W01/W02/W05/W07 contracts | W09-DV04 routing review |
| Markers and diagnostic availability at every transition (work seq 3) | W06 owns the channel but is sequenced after W03–W05; no marker vocabulary exists | Phase-label vocabulary plus deferred marker materialization through the W06 channel | Transitions that precede channel availability must still be identifiable (P1-V10) | W09 (labels, deferral); channel per W06 | W09-DV02 marker-coverage review |
| No hidden initialization dependency (P1-V15) | Nothing to audit yet | Per-phase prerequisite sets and the hidden-dependency review rule | Acceptance demands the absence be demonstrated, not asserted | W09 review procedure | W09-DV03 dependency review |
| Repeat-boot lifecycle evidence expectations (work seq 5) | No runner, no image, no evidence | Defined acceptance review and evidence expectations handed to W10/W11 | W09's plan assigns execution to the regression packages; the expectations must be defined here | W09 defines; W10/W11 execute | W09-DV05; execution evidence in W10/W11 verification |
| Stable environment declared for W10–W12 and P2 | No declared stable state exists | Terminal `stable` lifecycle state with explicit entry conditions and consumers | Downstream packages need one named state to target | W09; consumption per W10/W11/W12 and P2 plans | W09-DV07 consumability review |

No row requires inventing a crate layout, target, runner implementation, or
capability list; those remain with their owning packages. The integration seam
with W02's runtime entry ([Resolved decision
6](#resolved-design-decisions-and-their-authority)) is a coordination
requirement, not a unilateral W09 choice.

## Resolved design decisions and their authority

1. **Canonical phase order.** The lifecycle order is Entry (W01) → Runtime
   (W02) → Capabilities (W03) → EL2 baseline (W04) → Exceptions (W05) →
   Console (W06) → Fatal path (W07) → Host Stage-1 (W08) → Stable. Rationale:
   the plan's thematic span lists "Early Diagnostics" third, but the plan
   index and the W05/W06 prerequisite lines make W05 precede W06 and put W03
   before both; the dependency map is the governing statement of construction
   order, and the thematic line is read as an availability requirement
   ("diagnostics as early as possible"), satisfied by the deferred-marker
   model of decision 3. Authority: P1 plan index dependency table and the
   W05/W06 plans. If reviewers intend a different runtime order, that is a
   recorded coordination issue, not a local edit.
2. **Phase vocabulary owned by W09.** The phase identifiers `entry`, `runtime`,
   `capabilities`, `el2-baseline`, `exceptions`, `console`, `fatal-path`,
   `stage1`, `stable` and their Rust enum forms are fixed by this design as
   stage-local design freedom (no higher authority names them). They are
   internal boot-scope identifiers, not ABI, and may be renamed only through a
   revised design decision recorded in the implementation record.
3. **Deferred marker materialization.** The tracker records every phase
   transition from the first Rust-visible code; markers become visible output
   when the W06 channel reports availability, at which point the tracker
   materializes a replay of all recorded transitions before live markers
   continue. Rationale: four phases legitimately precede full console
   availability; P1-V10 requires each stage to be identifiable, which replay
   satisfies without inventing pre-console output mechanisms. Authority: W09
   plan work seq 3; W06 plan scope ("from entry through stable state").
4. **Monotone lifecycle, no rollback.** Phases only advance; every phase
   failure is terminal via its routing-matrix route; no phase de-initializes
   another. Rationale: P1 has no de-initialization infrastructure, the plans
   specify fail-fast boundaries, and the task book's staged scope leaves
   recovery to later stages. The per-phase failure column in
   [the state machine](01-init-state-machine.md) §4 states what is retained at
   failure instead of a rollback.
5. **The tracker is the single owner of lifecycle position.** One type owns
   "where is boot" for the whole stage; the W07 fatal path reads it for its
   startup-phase field, W10 verdicts read its markers, and W12 documents it.
   No other component may record lifecycle position. Rationale: duplicate
   phase state is exactly the hidden dependency P1-V15 forbids. Authority:
   W07 plan (crash reports include startup phase), W09 plan work seq 2.
6. **Sequencer integration seam.** The Rust-level composition
   `run_init_sequence()` covers phases `capabilities`..`stage1`; the W02
   runtime invokes it at the seam its design fixes, immediately after
   recording `runtime` completion. The `entry` and `runtime` records are
   written by W02-owned code on the authority of W01's transfer guarantee
   (reaching the W02 entry implies W01 validated the environment). If the W02
   design fixes a different seam or rejects a delegated `entry` record, the
   conflict is raised per the workflow's failure boundary — it is not absorbed
   by editing this decision silently.
7. **Adapter bodies fail closed.** Each phase adapter calls exactly the
   mechanism entry that the owning package's design fixes. Until that design
   exists, the adapter does not compile; no stub, default, or skipped phase is
   permitted, because a silently skipped phase would fabricate lifecycle
   success.
8. **No `Manager` object.** The sequencer is straight-line code plus a small
   tracker; it owns no policy, no registry, and no subsystem state. Rationale:
   Plan Agent guidelines forbid global-manager substitutes for boundaries.

## Work breakdown and loading order

1. Load [01-init-state-machine.md](01-init-state-machine.md) to understand the
   phase table, the marker availability model, the failure-routing matrix, and
   the hidden-dependency rules. Every implementation step depends on it.
2. Load [02-interface-contracts.md](02-interface-contracts.md) for the exact
   contracts of the types and functions named there, and implement in the
   order given in [03-implementation-and-review.md](03-implementation-and-review.md):
   contracts collection and reconciliation first, tracker and phase types,
   adapters, sequencer wiring, marker integration, then the review steps.
3. Record implementation decisions and deviations in
   `../p1-w09-initialization-sequencing-record.md` when implementation begins,
   and validation commands, environments, and outcomes in
   `../../verification/p1-w09-initialization-sequencing-verification.md` when
   evidence exists. Neither file may exist yet, and neither this design nor a
   record may claim W09 complete.

## Explicitly excluded interfaces

No Guest, VM/vCPU, Stage-2, SMP, PSCI, GIC, timer-virtualization, DTB
discovery, `PlatformInfo`, allocator, heap, or board-name mechanism is designed
or authorized. No public ABI, wire format, or persistent layout is introduced;
all W09-owned symbols are internal boot-scope Rust items. No generic lifecycle
framework, service state machine, scheduler state, or rollback/recovery API is
authorized — the lifecycle is the P1 boot sequence and nothing else. No
mechanism of W01–W08 is restyled here: their internal contracts belong to their
own designs, and this design consumes them as assumed contracts
([02-interface-contracts.md](02-interface-contracts.md) §6 lists the seams and
their failure boundaries).

## Downstream handoff

- **[P1-W10](../p1-w10-qemu-boot-regression/README.md)** receives the terminal
  `stable` state and the phase/event marker vocabulary; it selects the objective
  stable-marker token and verdict rules from them (W10 owns the token).
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives the
  failure-routing matrix as the expected diagnostic route per fault phase, and
  the phase vocabulary for scenario naming and evidence.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  lifecycle as the content basis of the EL2 initialization contract document.
- **P2** receives the declared stable environment: boot CPU in Non-secure EL2,
  established Host Stage-1 runtime, working console and fatal diagnostics, and
  the recorded phase position at handoff. P2 extends the boot path after
  `stable` per its own designs; it does not inherit the sequencer as a general
  lifecycle service.

A coding agent completing W09 must leave the handoff checklist in
[03-implementation-and-review.md](03-implementation-and-review.md) answerable
without inspecting W09 source code.
