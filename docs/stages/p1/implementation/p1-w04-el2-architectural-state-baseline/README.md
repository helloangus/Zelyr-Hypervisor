# P1-W04 EL2 Architectural-State Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The explicit EL2 architectural-state baseline required by
[P1-W04](../../plans/p1-w04-el2-architectural-state-baseline.md): execution
state, exception routing and masking, trap policy, FP/SIMD access, debug and
performance traps, timer controls, EL1/EL0 preparation, and translation
controls — each owned, written, read-back verified, and declared to
consumers.  
**Owner/change context:** P1-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W04. W04 makes the statement
"the Hypervisor owns its EL2 state" true: every control the stage depends on
is written to a recorded value, verified by read-back, and published as a
declaration that W05, W08 and W09 build on instead of firmware residue. It
deliberately does **not** install vectors (W05), enable the MMU or define
mapping classes (W08), sequence the lifecycle (W09), enter EL1, implement
Stage-2, or design the complete future system-register surface.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-architecture-and-state.md](01-architecture-and-state.md) — the
  baseline category model, the register-ownership matrix (one owner per
  register, stage-wide), the write-once establishment lifecycle, the
  read-back model, the declared-controls API, and the assumed-contract
  table. Load this first for any step.
- [02-code-contracts-control-writes.md](02-code-contracts-control-writes.md) —
  per-register write specifications: values, masks, rationale, consumer,
  and the establishment entry's contract, with pseudocode.
- [03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md) —
  read-back verification, the baseline error type and route, and the
  declared-controls API consumed by W05/W08/W09.
- [04-implementation-and-review.md](04-implementation-and-review.md) — ordered
  workflow, validation matrix, error/security/observability model, and
  handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W04
plan](../../plans/p1-w04-el2-architectural-state-baseline.md). This document
is the proposed detailed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W04 plan → this
design → Coding Guidelines. In particular:

- The task book requires an "explicit EL2 architectural-state baseline" with
  P1-V07: routing, traps, FP/SIMD, debug/performance, timer, EL1/EL0
  preparation and translation controls explicitly established and logically
  consistent across clean reference boots; ADR P1 names
  `HCR_EL2`/`CPTR_EL2`/`CNTHCTL_EL2` as the minimum security baseline.
- The plan scopes execution state, exception routing, trap policy, FP/SIMD
  access, debug and performance traps, timer access, EL1/EL0 preparation and
  translation-control baseline — and explicitly excludes Guest
  virtualization policy, Stage-2, GIC/timer virtualization, EL1 entry, SMP
  state and the complete future register design.
- The [W03](../p1-w03-aarch64-capability-inventory/README.md) report is the
  only legitimate source of the facts this baseline maps to (its plan work
  seq 1); the [W02](../p1-w02-minimal-rust-el2-runtime/README.md) runtime
  and [W09](../p1-w09-initialization-sequencing/README.md) lifecycle are
  accepted sibling contracts consumed directly: the baseline runs inside
  W09's `el2-baseline` phase and its failures route via W02's panic route
  (the matrix's `el2-baseline` row).

Classification: the baseline categories, the register-ownership matrix, the
write specifications, the read-back verification, and the declared-controls
API are **Required**. Extension of the baseline for later packages' needs,
re-tuning of recorded values when a later design demands one, and the
optional-element guards' evolution are **Reserved** with recorded triggers.
Guest virtualization policy (HCR_EL2 VM/TGE *policy* beyond the documented
P1 values), Stage-2 enablement, GIC or timer virtualization, EL1 entry, SMP
state, and any permanent identity-map or MMU promise are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Map baseline categories to W03 capability facts (work seq 1) | [Architecture](01-architecture-and-state.md) §2; [workflow](04-implementation-and-review.md) step 1 | W04 closure review (W04-DV01) |
| Define the known-state boundary and continuation prerequisites (work seq 2) | [Architecture](01-architecture-and-state.md) §3–§5; [write specs](02-code-contracts-control-writes.md) | P1-V07 (W04-DV02) |
| Define how unsupported or unavailable baseline elements fail (work seq 3) | [Architecture](01-architecture-and-state.md) §6; [readback contracts](03-code-contracts-readback-and-declaration.md) §1–§2 | P1-V07 (W04-DV03) |
| Review for firmware-state dependence and future-stage overcommitment (work seq 4) | [Workflow](04-implementation-and-review.md) step 4 | P1-V07 (W04-DV04) |
| Specify cold-boot consistency evidence (work seq 5) | [Workflow](04-implementation-and-review.md) §3 (W04-DV05/DV06) | P1-V07 (W04-DV05, DV06) |
| Hand off the baseline contract to exception and MMU packages (work seq 6) | Downstream handoff below; [workflow](04-implementation-and-review.md) §5 | W04 closure review (W04-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs`
at `4e631ee`): no system-register access exists anywhere in the tree; no
baseline, no declaration, no consumer. The W02 runtime, W03 report, and W09
phase body this package runs between exist as accepted designs in this
branch, not as code; W05/W08 (the named consumers) are parallel designs.
Every executing prerequisite is a contract, not a present artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Required EL2 state is explicitly owned (P1-V07) | No baseline, no ownership matrix, no write | The category model, ownership matrix, and write specifications of [01-architecture-and-state.md](01-architecture-and-state.md) §2–§3 and [02-code-contracts-control-writes.md](02-code-contracts-control-writes.md) | "Explicitly owned" needs a per-register owner, value, and rationale before any write exists | W04 (this design) | W04-DV02 |
| State consistent across clean boots (P1-V07) | Nothing to verify | The read-back verification of [03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md) §1 plus the deferred repeat-boot execution | Consistency must be checked at boot, not asserted; repetition belongs to W10's regression | W04 (read-back); W10 (repetition) | W04-DV03; execution via W10 |
| Unsupported/unavailable elements fail or are recorded (plan work seq 3) | No failure mechanism | The error type, route, and optional-element guard rules of [03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md) §1–§3 | A baseline that cannot fail is firmware residue with extra steps | W04 (mechanism); W02 (route); W09 (matrix row) | W04-DV03 |
| Categories mapped to capability facts (work seq 1) | Facts exist only in W03's design | The mapping table of [01-architecture-and-state.md](01-architecture-and-state.md) §2 | ADR-044: guards derive from queried facts, not platform names | W04 (mapping); W03 (facts) | W04-DV01 |
| No firmware-state dependence (work seq 4) | Nothing to audit | The residue-removal rule and its review checklist | The absence must be demonstrated register-by-register | W04 review procedure | W04-DV04 |
| Baseline declared to W05/W08/W09 (plan handoff) | No declaration exists | The declared-controls API of [03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md) §4 | Consumers must build on recorded values, not on assumptions about them | W04 (API); consumers (use) | W04-DV07 |

No row requires designing a later-stage mechanism; W04 writes only the
controls P1's own stage needs and records — never promises — their limits.

## Resolved design decisions and their authority

1. **One register, one owner, stage-wide.** The ownership matrix of
   [01-architecture-and-state.md](01-architecture-and-state.md) §3 assigns
   every touched register exactly one writing package; W02's establishment
   writes (`SPSel`, `DAIF`) are asserted by W04's read-back but never
   rewritten, and `VBAR_EL2` belongs to W05 alone. A second writer is an
   architecture violation, because a control with two owners has no
   invariant. Authority: Plan Agent ownership rules; W09 single-owner
   principle.
2. **Every write is read-modify-write with a recorded mask or a full-value
   write to a fully specified constant — never a bare "leave as is".**
   Rationale: RES1/RES0 fields make blind full writes on EL1 registers
   incorrect, while "leave as is" is precisely the firmware residue the
   stage exists to remove; the two allowed forms keep every recorded value
   meaningful. Authority: Coding Guidelines (reserved-bit handling); task
   book T04.
3. **Deny-by-default trap posture: FP/SIMD and debug/performance accesses
   that P1 does not use are trapped, not left enabled.** `CPTR_EL2.TFP=1`
   (with `CPACR_EL1.FPEN=00` at EL1/EL0) and the debug/PMU trap bits make
   any unexpected use a loud, diagnosable fault once W05's vectors exist,
   instead of silent firmware-convenience behavior. The binding complement
   is the stage-wide build guarantee that W02/W03/W04 code contains no
   FP/SIMD (recorded dependency on the P0 target semantics; verified by
   review, with the toolchain inspection reservation noted). Authority:
   ADR P1 "最小安全基线"; task book P1-V07.
4. **The MMU stays off and no mapping promise is made: `SCTLR_EL2.M/C/I=0`
   is written and verified; enabling is W08's transition alone.** W04
   establishes the pre-MMU baseline W08 requires; it neither enables nor
   constrains W08's post-MMU values. Authority: plan scope;
   [W08](../p1-w08-host-stage1-address-space/README.md) plan.
5. **Stage-2 controls are zeroed as residue removal only:** `VTCR_EL2=0`,
   `VTTBR_EL2=0`, and `HCR_EL2.VM=0` keep Stage-2 architecturally disabled;
   this is a translation-control baseline, not Stage-2 design — no
   VMID, no `VTCR` tuning, no Stage-2 semantics. Authority: plan out-of-scope
   list; task book §2.
6. **Every write is verified by masked read-back at boot; a mismatch is a
   fatal baseline error routed via the panic route.** Rationale: P1-V07's
   "explicitly established" becomes mechanically checkable at every boot
   instead of review-only; the route is established (W02's panic route, two
   phases earlier — W09 H4 holds). Authority: task book P1-V07; W09 matrix
   `el2-baseline` row.
7. **Optional elements are guarded by W03 facts, skipped with a recorded
   skip — never guessed.** The EL2 virtual timer exists only when the VH
   fact is present; writing it unconditionally would be a facts violation,
   skipping silently would hide a divergence. Authority: ADR-044; W03
   contracts.
8. **The declaration API reports recorded status, not live re-reads.**
   Consumers query what the baseline established and verified; re-reading
   controls would create second sources of truth for state W04 owns.
   Authority: Plan Agent ownership rules; W03 consumer-conduct precedent.

## Work breakdown and loading order

1. Load [01-architecture-and-state.md](01-architecture-and-state.md) for the
   category model, ownership matrix, lifecycle, and assumed contracts.
   Every step depends on it.
2. Load [02-code-contracts-control-writes.md](02-code-contracts-control-writes.md)
   when implementing the writes, and
   [03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md)
   when implementing verification, the error route, and the declaration.
3. Execute the steps in the order given in
   [04-implementation-and-review.md](04-implementation-and-review.md):
   fact mapping, write specifications, read-back and route, declaration
   wiring, residue and scope reviews, evidence and handoff.
4. Record implementation decisions and deviations in
   `../p1-w04-el2-architectural-state-baseline-record.md` when
   implementation begins, and validation commands, environments, and
   outcomes in
   `../../verification/p1-w04-el2-architectural-state-baseline-verification.md`
   when evidence exists. Neither file may exist yet, and neither this design
   nor a record may claim W04 complete.

## Explicitly excluded interfaces

No vector table, no `VBAR_EL2` write, no exception-entry code (W05); no
mapping class, page-table format, MMU enablement, or cache-maintenance
policy (W08); no GIC, timer virtualization, PSCI, or SMP register work; no
EL1 entry; no allocator. No public ABI, wire format, or persistent layout is
introduced; the declaration API is a P1-internal boot-scope surface. W04
consumes the W03 report; it never re-reads an identification register to
re-derive a fact, and any such re-read is a review failure (W04-DV04).

## Downstream handoff

- **[P1-W05](../p1-w05-el2-exception-entry-baseline/README.md)** receives the
  known-state boundary its plan work seq 3 integrates with: DAIF masked,
  trap/deny posture active, `VBAR_EL2` unowned and free to install, FP/SIMD
  denied (its save areas must not assume FP state), and the declaration API
  to assert the baseline it builds on.
- **[P1-W08](../p1-w08-host-stage1-address-space/README.md)** receives the
  verified pre-MMU baseline (`SCTLR_EL2.M/C/I=0`, `HCR_EL2.VM=0`,
  `VTCR/VTTBR=0`) and the translation-limit context recorded in the C8
  category; where its accepted design needs the underlying facts directly,
  it names that W03 query dependency in its own design. It carries the
  obligation to preserve the diagnostic path across its transition (its
  plan work seq 4) without rewriting W04-owned controls.
- **[P1-W09](../p1-w09-initialization-sequencing/README.md)** receives the
  `el2-baseline` phase body (the establishment entry its `el2_baseline_step`
  adapter calls) and the failure boundary its matrix row already names.
- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives the
  deny-by-default posture as scenario context (unexpected FP/SIMD or
  debug access behaves as a loud fault, not silent residue) and the
  baseline error vocabulary.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  baseline record (categories, values, rationale) for the EL2
  initialization contract document and the recorded limitations.
- **P2** receives the declared baseline as the known state its boot-path
  extensions build on; guest virtualization policy remains explicitly
  reserved.

A coding agent completing W04 must leave the handoff checklist in
[04-implementation-and-review.md](04-implementation-and-review.md) answerable
without inspecting W04 source code.
