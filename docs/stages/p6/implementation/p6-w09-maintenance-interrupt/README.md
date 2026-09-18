# P6-W09 Maintenance Interrupt — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Maintenance-event processing for the P6 GICv3 virtualization
interface: recognized maintenance conditions, completion correlation, reusable
presentation capacity, diagnostic handling of unexpected conditions, and
bounded progression of pending virtual interrupts under capacity pressure, per
[P6-W09](../../plans/p6-w09-maintenance-interrupt.md).  
**Owner/change context:** P6-W09 implementation handoff; this design owns only
the maintenance transition and its diagnostics inside the logical boundary
established by the P6-W08 virtualization-interface design.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W09. It converts the bounded
work-package plan into one code-bearing increment: a maintenance-interrupt
processing path that releases completed List-Register (LR) presentation state,
advances the affected vIRQ lifecycle through the P6-W07 contract, admits
bounded further pending work through the P6-W08 presentation contract, and
diagnoses — never silently repairs or panics on — unexpected maintenance
conditions.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-maintenance-contract.md](01-maintenance-contract.md) — logical module
  boundary, owned state and lifecycle, function/type contracts, invariants,
  and error model. Load before any code work (workflow steps 2–4).
- [02-workflow-and-validation.md](02-workflow-and-validation.md) — ordered
  implementation steps, validation matrix, observability model, and handoff
  checklist. Load for step execution and closure.

Before editing, the agent must also follow the Coding Guidelines preflight:
repository [AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P6 task
book](../../task-book-v0.1.md), and the [P6-W09
plan](../../plans/p6-w09-maintenance-interrupt.md). This document is the
proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W09 plan → this design
→ Coding Guidelines. In particular:

- ADR-032 makes GICv3 the AArch64 interrupt baseline with "LR/maintenance" as
  the explicitly planned evolution step; ADR-048 makes IRQ telemetry a first
  class; ADR-049 requires layered validation.
- The task book (§1 Required) demands: maintenance processing that releases
  completed presentation state; no loss, duplicate completion, or active-state
  leak under pressure (P6-V16, P6-V17); diagnosable handling of unexpected
  conditions. The task book (§8) classifies exact GIC revision, feature
  availability, and register/range interpretation as **Platform and
  Specification Investigation**: this design therefore fixes logical condition
  classes, not register bit spellings, and mandates spec reconciliation before
  code.
- The plan excludes: general interrupt scheduling policy, Guest vGIC MMIO,
  List-Register allocation internals, final fairness/coalescing policy, and
  any production IRQ-pressure guarantee. None of these may be introduced
  through the maintenance path.
- Coding Guidelines bind this design strongly: volatile MMIO access, required
  barriers and ordering on register sequences, reserved-bit handling, no
  allocation or unbounded work in interrupt context, minimal audited
  `unsafe` under the P0-W10 inventory
  ([plan](../../../p0/plans/p0-w10-unsafe-rust-governance.md)), and no
  reachable panic on contained conditions.

Classification:

| Class | Items |
|---|---|
| **Required** | Maintenance-condition recognition by logical class; completion correlation and slot release; bounded refill of freed capacity; duplicate-completion idempotency; orphan-completion escalation; unexpected-condition diagnostics; maintenance telemetry events; the P6-V16/P6-V17 acceptance scenarios. |
| **Reserved** | Exact GIC revision and maintenance register/bit names (Specification Investigation, task book §8); numeric priority or interrupt-controller configuration owned by W02/W08 designs; coalescing and fairness policy beyond "queue order, no reordering" (task book: final policy excluded); reuse of the maintenance path by P7 wakeup or P8 vGIC (consumers only). |
| **Out of Scope** | Guest vGIC Distributor/Redistributor MMIO model (P8); List-Register allocation policy internals (W08); physical-IRQ classification and spurious handling (W03); vIRQ pending-queue ownership (W07); scheduler or run-state semantics (P7); ITS/MSI/LPI and GICv2 (ADR-032/033, task book §1); real-hardware validation claims. |

## Requirement → design-location → acceptance mapping

| Plan requirement (P6-W09) | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W08 presentation/capacity contract and W03 physical IRQ lifecycle rules | [workflow](02-workflow-and-validation.md) step 1 | prerequisite reconciliation recorded in the implementation record before any code |
| Approved design for maintenance-event authority, completion correlation, error handling, pending-work continuation | [contract](01-maintenance-contract.md) §2–§4 | this design approved; contracts complete per checklist §3 |
| Integrate with vIRQ lifecycle state, telemetry, safe Host IRQ completion | [contract](01-maintenance-contract.md) §3–§5; [workflow](02-workflow-and-validation.md) steps 3–4 | code review confirms single-owner transitions, P0-W13-namespace events, W03-compliant EOI/completion |
| Over-capacity, Guest-completion, reusable-slot, unexpected-maintenance, repeat-progress acceptance scenarios | [workflow](02-workflow-and-validation.md) §3 (W09-DV01–DV05) | P6-V16, P6-V17 evidence in the verification record |
| State-leak, duplicate-completion, cross-vCPU, storm-boundary review | [contract](01-maintenance-contract.md) §4; [workflow](02-workflow-and-validation.md) steps 5 and W09-DV06/DV07 | review findings recorded; W12 consumes the orphan/storm boundary |
| Record the maintenance contract; hand to W10–W13 and P8 | [workflow](02-workflow-and-validation.md) §5 handoff checklist | handoff checklist satisfied without completion claims |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p6-implementation-designs`):
the repository is a P0 documentation scaffold — `git ls-files` shows no Cargo
workspace, no Rust sources (`hypervisor/src/.gitkeep` only), and no P6
implementation records (`docs/stages/p6/implementation/` contains only the
coordinator-owned index README). P0–P5 work packages are planned but not
implemented, so every P6 prerequisite below is an **assumed contract** known
only from its plan/handoff text. The P6-W01–W08 detailed designs are being
produced in parallel; this design references them by slug and P6-Wxx ID and
assumes only what their plans state. Each ledger row states the missing
foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Maintenance events release completed presentation state (P6-V17) | No code, no W08 design content available to this author | W08 presentation-state contract exposing, per vCPU, an LR allocation table that can name the vIRQ presented in each LR | Correlation ("which vIRQ completed?") is impossible without an authoritative in-flight record; W09 must not own a second copy of that state | W08 design (`../p6-w08-gic-virtualization-interface/README.md`, P6-W08) | W08 verification evidence; W09-DV02 exercise |
| Pending work makes bounded progress (P6-V16, P6-V17) | No pending-queue or admission contract implemented | W07 pending/deferred vIRQ lifecycle contract; W08 LR admission contract | Refill must consume from a queue with a single owner and defined capacity behavior; W09 defines neither | W07 (`../p6-w07-virtual-interrupt-core/README.md`) and W08 designs | W09-DV01/DV05 exercises on top of W07/W08 evidence |
| Maintenance IRQ is received and processed | No Host IRQ lifecycle implemented | W03 physical IRQ lifecycle classifying the maintenance PPI and dispatching to the W09 entry, with the completion (EOI) rule this path must follow | The handler cannot run outside a dispatcher, and must complete the physical IRQ through the established safe path | W03 design (`../p6-w03-physical-interrupt-lifecycle/README.md`, P6-W03) | W03 verification evidence; W09-DV02/DV03 |
| Unexpected conditions are diagnosed safely | No telemetry/diagnostic baseline implemented | P0-W12 logging and P0-W13 trace-namespace contracts; W12 impossible-state escalation path | Diagnostics must be structured and namespaced, and the escalation class must already exist or be co-designed with W12 | P0 contracts ([P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md)); W12 (`../p6-w12-fault-isolation-robustness/README.md`) | telemetry coverage review (W13, P6-V24); W09-DV04 |
| No loss, duplicate completion, or active-state leak under pressure (P6-V16) | Nothing exists to observe | The W09-owned invariants in [01-maintenance-contract.md](01-maintenance-contract.md) §4 plus W12 storm-boundary evidence | Invariants that no one designs are invariants no one tests; W09 owns them for its transition | W09 (this design); W12 for storm evidence | W09-DV01/DV05/DV07; W12 FI-D |
| Maintenance contract recorded and handed to W10–W13 and P8 | No maintenance contract exists in any tracked document | The contract sections of [01-maintenance-contract.md](01-maintenance-contract.md) plus a factual implementation record created when work starts | Downstream packages need one fixed consumable statement of maintenance semantics | W09 (this design) | implementation record + handoff checklist |

No row above requires selecting a GIC revision, crate layout, or any other
decision reserved by the task book; the spec-reconciliation boundary is
explicitly a required workflow step, not a silent default. If a prerequisite
delivers differently than its plan states (for example, W08 chooses not to
expose per-LR correlation), the affected W09 step **stops** and records an
Architecture Change Request against the owning design; W09 must not repair or
duplicate the prerequisite.

## Resolved design decisions and their authority

1. **Logical maintenance-condition classes.** The design recognizes four
   logical classes — completion/reusable-slot, under-pending, EOI-count-zero,
   and no-pending — derived from the GICv3 virtual CPU interface maintenance
   interrupt as described by the applicable architecture and GIC
   specification. Exact register names, bit positions, and applicability are
   **not** fixed here: the task book §8 routes them through Platform and
   Specification Investigation, reconciled against the W01 capability facts
   before code ([workflow](02-workflow-and-validation.md) step 2). Authority:
   task book §1 Reserved and §8.
2. **State ownership — single-writer rule.** The per-vCPU LR allocation
   table and in-flight presentation records are owned by W08. The vIRQ
   lifecycle states are owned by W07. W09 owns exactly one transition —
   "presented → completed-and-released" — expressed as calls into the W07
   completion contract and the W08 slot-release contract, plus its own
   per-vCPU diagnostic counters. No other package may perform the maintenance
   transition; W09 writes no other package's state. Authority: plan scope
   ("integrate … with vIRQ lifecycle state"); Plan Agent guidelines
   (single owner per mutable state).
3. **Correlation, idempotency, and escalation rule.** A reported completed
   slot whose in-flight record exists is reconciled exactly once; a repeated
   report for an already-reconciled slot is a counted no-op (idempotent), not
   an error. A reported completed slot with **no** in-flight record is an
   impossible internal state: it must not be silently absorbed; it is counted,
   dumped with per-vCPU presentation state, and escalated on the W12
   impossible-state path (fatal-invariant classification per
   [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md)).
   Authority: plan step 5; Coding Guidelines (no silent corruption).
4. **Bounded interrupt-context work.** The maintenance entry processes at
   most the set of completed slots reported by the controller read in that
   invocation, then admits new pending work only into capacity actually freed
   in the same invocation, and stops. Remaining pending work stays pending and
   progresses through the normal under-pending/wakeup paths (W07/W08). The
   path performs no dynamic allocation, no locking beyond the established
   W03/W08 per-pCPU ordering, and no loop whose bound is not already known
   from the controller read. Authority: Coding Guidelines (IRQ-context
   bounds); plan goal ("bounded progress").
5. **Unexpected-condition containment.** A maintenance status read whose
   recognized-class decode fails (reserved or contradictory pattern) is
   handled conservatively: no state mutation beyond what recognized portions
   justified, one diagnostic counter increment with the raw status value
   recorded, and containment — never a panic, never a speculative repair.
   Persistent recurrence is reviewed as a hardware/specification
   investigation with W12. Authority: task book §1 ("diagnosable safe
   outcomes"); P0-W14 classification (guest-adjacent/hardware diagnostics are
   not fatal invariants unless state integrity is lost).
6. **Function and type names.** The plan names no API, so this design fixes
   the logical names in [01-maintenance-contract.md](01-maintenance-contract.md)
   §3 (`maintenance_irq_entry`, `decode_maintenance_status`,
   `reconcile_completed_slots`, `refill_freed_capacity`,
   `report_unexpected_maintenance`, `MaintenanceConditions`) as **stage-local
   design freedom owned by this design**. Rationale: a coding agent needs
   stable names; the names live inside the W08-established logical module
   boundary and bind to the final Rust module path in the implementation
   record once the W08 design exists. They are internal, not public API, and
   are explicitly not frozen ABI.
7. **Telemetry event set.** Six logical maintenance events (entry, condition
   set, slot reconciled, refill admitted, unexpected condition, deferred
   remainder) are required; their concrete identifiers are registered under
   the P0-W13 trace-event namespace contract and must not be invented here.
   Authority: ADR-048; task book §1 Required (telemetry); P0-W13 ownership.

## Work breakdown and loading order

1. Read this README and the Coding Guidelines in full.
2. Load [01-maintenance-contract.md](01-maintenance-contract.md) before any
   code-bearing step; it is the sole authority for the maintenance boundary,
   contracts, invariants, and error model.
3. Execute the steps in [02-workflow-and-validation.md](02-workflow-and-validation.md)
   in order; each step names its acceptance, failure/blocker handling, and
   evidence destination.
4. Store actual commands, output, environment, and run/not-run status in
   `../../verification/p6-w09-maintenance-interrupt-verification.md` (do not
   create it until evidence exists), and record taken decisions, deviations,
   and the final module binding in
   `../p6-w09-maintenance-interrupt-record.md` only when implementation
   begins. Neither this design nor a written record may claim W09 complete;
   P6-V16/P6-V17 closure belongs to the verification record.

## Explicitly excluded interfaces

No public or stable Rust API, ABI, wire format, persistent layout, crate
boundary, or Cargo manifest is designed or authorized by W09. The contracts in
[01-maintenance-contract.md](01-maintenance-contract.md) are **internal** to
the Host GIC virtualization boundary and may be reshaped by a later approved
design without versioning. Specifically excluded: Guest vGIC MMIO handlers;
List-Register allocation policy or its data structures (W08); vIRQ pending
queue internals (W07); any scheduler-visible hook (P7); any Guest-callable
interface (P5 boundary is the only Guest control path and is untouched here);
and any QEMU-specific constant in generic Core (capability-driven selection
only, ADR-044/ADR-052).

## Downstream handoff

- **P6-W10** (`../p6-w10-interrupt-semantics/README.md`) receives the
  completed-state meaning: "completed" is defined only as post-maintenance
  reconciliation, which W10's masking/pending semantics must not contradict
  (no false completion; [§3](01-maintenance-contract.md) invariant I2).
- **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`) receives
  the Guest-observable expectations for List-Register pressure and progression
  (consumed by VG-IRQ-06): all injected vIRQs eventually complete; none is
  observed twice in one presentation epoch.
- **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`) receives the
  orphan-completion escalation contract (FI-C case) and the storm-boundary
  statement that each maintenance invocation is bounded by its controller
  read (FI-D review input).
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives the
  six logical maintenance telemetry events for the P6-V24 coverage map and the
  maintenance-frequency data class for the P6-DOC-05 performance baseline.
- **P8** receives, through W13's consumer review, the evidenced maintenance
  behavior and its explicit limits. P8 owns the Linux-visible vGIC; nothing in
  W09 freezes a vGIC MMIO semantic, an LR count, or a machine ABI (task book
  §7).
