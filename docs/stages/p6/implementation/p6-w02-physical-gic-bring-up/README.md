# P6-W02 Physical GIC Bring-up — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The bounded Host GICv3 bring-up outcome required by
[P6-W02](../../plans/p6-w02-physical-gic-bring-up.md): a safe global
Distributor state and independent local Redistributor and CPU-interface
readiness for each online pCPU.  
**Owner/change context:** P6-W02 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W02. It converts the plan into
a register-access foundation, a one-owner global Distributor initialization,
a per-pCPU local (Redistributor + system-register CPU-interface) readiness
sequence integrated with the P3 pCPU lifecycle, and the acknowledged
readiness/failure ledger that W03–W05 and the validation packages consume.
It deliberately does **not** design IRQ dispatch policy, classification,
handler APIs, timer PPI enablement, virtual-interrupt behavior, Guest GIC
MMIO, or scheduler behavior; those belong to W03, W05, W07/W08, and later
owners.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) — for this
package the MMIO, system-register, barrier, reserved-bit, and `unsafe`
boundary rules are load-bearing — then loads only the linked supporting file
needed for its assigned step. Before editing it must also follow the Coding
Guidelines preflight, including the repository
[AGENTS.md](../../../../../AGENTS.md), [documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P6 task book](../../task-book-v0.1.md), and the P6-W02 plan. This document is
the proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W02 plan → this
design → Coding Guidelines. In particular:

- ADR-032 (GICv3 mainline), ADR-008 (Non-secure EL2, firmware-owned EL3),
  ADR-015 (early SMP), ADR-041–ADR-045 (capability-driven platform layering,
  no board-name branches), and ADR-006 (controlled `unsafe` with audited
  boundaries) govern the mechanism.
- [P6-W01](../p6-w01-gic-capability-discovery/README.md) owns the capability
  decision; W02 consumes it and its probe contracts. W02 does not re-derive
  capabilities and does not start without W01's record plus entry evidence
  (task book P6-ENTRY gating).
- The P3 contracts — online pCPU lifecycle, per-CPU state, boot rendezvous,
  synchronization semantics, CPU-local exception/interrupt foundations — are
  assumed inputs
  ([p3-w03](../../../p3/plans/p3-w03-physical-cpu-lifecycle.md),
  [p3-w04](../../../p3/plans/p3-w04-per-cpu-runtime.md),
  [p3-w05](../../../p3/plans/p3-w05-smp-boot-synchronization.md),
  [p3-w06](../../../p3/plans/p3-w06-concurrency-synchronization.md)); the
  failure boundary if they deliver differently is in
  [scope and foundations](01-scope-and-foundations.md) §1.
- The plan excludes "detailed register ordering" from the plan level and
  reserves it to detailed design; this design fixes the sequences in
  [03](03-code-contracts-register-access.md)–[05](05-code-contracts-local-gic.md)
  under the P6 supported posture, with numeric bit positions taken from the
  GIC specification revision pinned at implementation time (W01 step 1
  record). The design states architectural names and ordering; it is not a
  specification substitute.

Classification: the register-access surface, Distributor bring-up, per-pCPU
local bring-up, readiness ledger, residual-state policy, and failure
boundaries are **Required** (see
[01](01-scope-and-foundations.md) §3). Extended-range configuration,
EOImode-split (deactivate via DIR), maintenance/timer PPI enablement, and
priority/preemption policy tuning are **Reserved** with recorded triggers.
IRQ dispatch, interrupt classification, SGI send/route policy, virtual
interrupts, List Registers, Guest GIC MMIO, ITS/LPI/GICv2, scheduler
behavior, and Orange Pi 3B hardware support are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| W01 + P3 contract inspection | [Scope and foundations](01-scope-and-foundations.md) §1 | P6-V02 (W02-DV01) |
| Approved design for global/local readiness, failure boundaries, lifecycle ownership | [Architecture and state](02-architecture-and-state.md), [distributor](04-code-contracts-distributor.md), [local GIC](05-code-contracts-local-gic.md) contracts | P6-V02 (W02-DV02) |
| Bring-up sequencing integrated with capabilities and pCPU lifecycle; no BSP-local reuse on APs | [Distributor](04-code-contracts-distributor.md) §3, [local GIC](05-code-contracts-local-gic.md) §2–§3 | P6-V02/V03 (W02-DV03, DV05) |
| Normal, partial-discovery, local-readiness, residual-state acceptance scenarios | [Validation and handoff](07-validation-and-handoff.md) §2 | P6-V02/V03 (W02-DV04–DV08) |
| Layering, controlled unsafe, telemetry, unsupported-platform review | [Register access](03-code-contracts-register-access.md) §2, §6; [workflow](06-implementation-workflow.md) step 8 | P6-V02 (W02-DV09, DV10) |
| Factual status and handoff to W03–W05 | [Validation and handoff](07-validation-and-handoff.md) §3–§4 | W02 closure (W02-DV11) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is the P0 documentation
scaffold — no Cargo workspace, no Rust sources, no target configuration, no
runtime. P1–P5 plans exist as documents only; no P1 EL2 runtime, no P3 SMP
lifecycle, no P2 PlatformInfo implementation exists. The P6 capability
decision ([W01](../p6-w01-gic-capability-discovery/README.md)) and the W02
design itself are proposed, not implemented. Every upstream contract below is
assumed, with a failure boundary, and W02 code may only exist after the
W01/W02 entry review finds the task-book P6-ENTRY conditions evidenced.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Safe global Distributor state" | No GIC code or capability decision exists | W01 decision + expected identity (W01 record); register-access surface with reserved-bit and barrier rules ([03](03-code-contracts-register-access.md)) | Safety is a property of the access discipline plus a confirmed-safe platform posture, both of which must exist before the first write | W01 supplies decision; W02 owns access surface | W02-DV02/DV03; W01 record |
| "Independent local Redistributor and CPU-interface readiness for each online pCPU" | No per-CPU bring-up path exists; P3 lifecycle is a plan | Per-pCPU local sequence owned by the running pCPU, invoked from the P3 local-init point, with its own readiness state machine ([05](05-code-contracts-local-gic.md)) | "Independent" means no shared BSP state and one owner per local transition; that requires an explicit per-pCPU lifecycle | P3 supplies invocation point; W02 owns the sequence | W02-DV05 (AP-local evidence) |
| Acknowledged readiness/failure | No readiness ledger exists | Per-pCPU readiness/failure records with atomic publication ([02](02-architecture-and-state.md) §4) | Consumers (W03/W04/W05) must be able to observe readiness without racing bring-up | W02 | W02-DV06 |
| Diagnostic integration with P3 lifecycle facts | P3 telemetry/diagnostics are plans | Failure outcomes mapped to P3 lifecycle states (failed-local, not global panic) and W02 telemetry events | A local GIC failure must be attributable to a pCPU in P3's vocabulary | P3 vocabulary; W02 mapping | W02-DV07 |
| Residual-state acceptance scenario | Nothing exists; firmware residuals are expected | Residual-state policy (survey, log, quiesce) in the Distributor and local sequences | Task book P6-V02 requires "residual state handled diagnostically" | W02 | W02-DV08 |
| No BSP-local reuse on APs | n/a (nothing exists) | Structural rule: local state lives in per-CPU storage; BSP path is one caller of the same local sequence | Reuse by sharing is the defect class the plan names; the rule must be structural, not conventional | W02 + P3 per-CPU storage contract | W02-DV05 review |

No row requires inventing a crate boundary, target, or platform constant; the
GIC frames come from P2 facts, and register bit positions come from the pinned
specification. No decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Two-phase bring-up with one owner per phase:** Phase A (global
   Distributor) runs once on the boot pCPU during global initialization;
   Phase B (Redistributor + CPU interface) runs per pCPU, executed by that
   pCPU at its P3 local-initialization point. Authority: task book Required
   outcome "independent local GIC readiness"; ADR-015. Rationale: a single
   global pass would make AP readiness depend on BSP execution state, which
   the plan forbids.
2. **Confirmation before configuration:** the first hardware contact is
   W01's read-only probe contract set (distributor identity, per-pCPU
   redistributor identity, virtualization-interface presence); any mismatch
   against the W01 decision stops bring-up before a single
   state-changing write. Authority: W01 design decision 2; task book
   P6-ENTRY-01/06.
3. **Disable-and-quiesce before enable:** the Distributor is disabled
   (with completion polling) before any reconfiguration; all supported SPIs
   and local SGI/PPIs start disabled with lowest default priority; residual
   pending/active state is surveyed, logged, and cleared before the
   corresponding enable. Authority: task book P6-V02 "residual state handled
   diagnostically"; Coding Guidelines hardware rules. Rationale: enables
   must observe a known state; stale firmware pending must not storm a
   system with no consumers yet.
4. **SPI initial routing is explicit:** every supported SPI is routed
   (IRM=0) to the boot pCPU affinity at Phase A; "any CPU" (1-of-N) routing
   is not used in P6 because P6-V05/V06 require determinate target
   accounting. Routing *changes* remain W04-owned; W02 only fixes the
   initial determinate state.
5. **EOImode = 0 (combined priority-drop and deactivate on EOI) for P6
   physical handling;** the EOImode=1 + DIR split is Reserved for the
   virtualization packages ([W08](../p6-w08-gic-virtualization-interface/README.md))
   whose run/EOI split requires it. Authority: task book Reserved clause
   (register sequencing); W03 consumes the combined-EOI rule.
6. **No preemption within Group 1 (binary point at no-grouping) and no
   nested host IRQ handling in P6;** priority-based preemption tuning is
   Reserved (W10 semantics). Rationale: keeps the W03 lifecycle single-level
   and auditable at v0.
7. **System-register sequencing:** ICC SRE enablement is written-and-verified
   per pCPU before any ICC_CTLR/PMR/IGRPEN work; interface control writes are
   followed by instruction synchronization; the final per-pCPU action order
   is fixed (SRE → CTLR → PMR → group enable) in
   [05](05-code-contracts-local-gic.md) §4. Authority: Coding Guidelines
   system-register sequencing rule.
8. **GICR wake protocol with bounded wait:** ProcessorSleep/ChildrenAsleep
   quiesce handshakes use bounded polling; a timeout is a local failure
   (pCPU failed-local, excluded from targets), never a global panic and
   never an infinite spin. Authority: Coding Guidelines bounded-wait rule;
   P3 failed-CPU vocabulary.
9. **Readiness ledger is atomically published per pCPU:** local readiness is
   written once, release-ordered, by the owning pCPU; consumers read with
   acquire semantics ([02](02-architecture-and-state.md) §4). No global GIC
   lock protects local state; a single distributor-scoped lock serializes
   the Phase-A and any future global mutation (routing changes stay W04).

## Work breakdown and loading order

1. Read [scope and foundations](01-scope-and-foundations.md): assumed
   contracts, entry gating, scope classification, residual-state policy.
2. Read [architecture and state](02-architecture-and-state.md): modules,
   objects, readiness state machines, concurrency model.
3. Read [register access](03-code-contracts-register-access.md) before any
   hardware-facing contract: MMIO surface, reserved bits, barriers,
   `unsafe` boundary, typed ID reuse.
4. Read [distributor contracts](04-code-contracts-distributor.md) for Phase A
   and [local GIC contracts](05-code-contracts-local-gic.md) for Phase B.
5. Apply the changes in the order stated in the
   [implementation workflow](06-implementation-workflow.md).
6. Store actual commands, results, and run/not-run status in
   `../../verification/p6-w02-physical-gic-bring-up-verification.md`; record
   factual implementation status in
   `../p6-w02-physical-gic-bring-up-record.md` only when implementation
   begins. Neither this design nor a record may claim W02 complete.

## Explicitly excluded interfaces

No handler-registration API, no dispatch path, no classification routine, no
SGI send primitive, no SPI route-change API, no timer PPI enablement, no
maintenance-interrupt handling, no List-Register or virtual-interface
configuration beyond W01's read-only presence probe, no Guest-visible state,
no irqchip framework, and no platform-name branch is designed or authorized
by W02. PPI enablement for W05/W08 consumers happens through the W03
consumer-registration surface, not through W02. Any of these appearing in a
W02 change is a scope conflict to stop at review.

## Downstream handoff

- **W03** receives the acknowledged Host IRQ boundary: the enabled/quiet
  initial state, the completion rule (combined EOI, EOImode=0), and the
  readiness ledger it must consult before dispatch
  ([W03 design](../p6-w03-physical-interrupt-lifecycle/README.md)).
- **W04** receives independent per-pCPU local readiness plus the initial
  SPI routing state it may change; targeting consults the readiness ledger.
- **W05** receives the fact that the EL2 timer PPIs are *disabled* at local
  readiness and the PPI-configuration surface it must use to enable its own
  event source.
- **W08/W09** receive the virtualization-interface probe evidence location
  and the CPU-interface baseline their virtual controls extend.
- **W11–W13** receive P6-V02/V03 evidence boundaries: QEMU integration and
  repeat evidence establish the documented usable state in the reference
  environment; they do not prove real-hardware correctness (P15 owns that).
- No consumer receives a frozen register-level API, a machine IRQ layout, or
  any BSP-local shortcut.
