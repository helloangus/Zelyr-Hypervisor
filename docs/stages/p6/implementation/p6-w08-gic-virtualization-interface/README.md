# P6-W08 GIC Virtualization Interface — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Scope:** The bounded hardware-presentation bridge required by
[P6-W08](../../plans/p6-w08-gic-virtualization-interface.md): from the
P6-W07 vIRQ lifecycle to supported GICv3 virtualization-interface /
List-Register state.
**Owner/change context:** P6-W08 implementation handoff.
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W08. It converts the bounded
work-package plan into a designed, code-bearing mechanism: capability-gated
per-pCPU virtualization-interface readiness, List-Register (LR) presentation
of W07 pending events with basic priority carriage, vCPU-context
preservation of the virtual CPU interface across Guest entry/exit, and safe
pressure behavior when pending work exceeds presentation capacity. It owns
the LR table and the maintenance-interrupt *enabling* and *state reads*;
maintenance processing policy is P6-W09, which consumes W08's
completed-presentation/reusable-capacity boundary. W08 deliberately does
**not** design maintenance policy, Guest GIC Distributor/Redistributor MMIO,
the Linux machine ABI, hardware-mapped (passthrough) interrupts, or any
crate/file layout. The physical GIC bring-up itself is the parallel W02
design's deliverable, consumed here by plan-path citation — W08 does not
re-bring-up physical GIC state.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), completes
the Coding-Guidelines preflight (repository `AGENTS.md`, documentation
index, ADR baseline, [P6 task book](../../task-book-v0.1.md), and the
[P6-W08 plan](../../plans/p6-w08-gic-virtualization-interface.md)), and then
loads only the linked supporting file needed for its assigned step:

| Assigned material | Supporting file |
|---|---|
| Baseline findings, scope classification, resolved decisions | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Logical modules, ownership, LR slot lifecycle, concurrency | [02-architecture-and-state.md](02-architecture-and-state.md) |
| Readiness, context save/restore, entry/exit sequence contracts | [03-code-contracts-interface-context.md](03-code-contracts-interface-context.md) |
| LR presentation, pressure, and maintenance-boundary contracts | [04-code-contracts-lr-presentation.md](04-code-contracts-lr-presentation.md) |
| Ordered implementation steps | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Validation matrix, error/security/observability model, handoff checklist | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Nothing in this design is an implementation or completion claim. Evidence is
recorded only in
`../../verification/p6-w08-gic-virtualization-interface-verification.md`
(created when evidence exists), and factual implementation traceability only
in `../p6-w08-gic-virtualization-interface-record.md` (created when work
starts).

## Authority, constraints, and scope classification

The governing order is Architecture ADR → P6 task book → P6-W08 plan → this
design → Coding Guidelines. Binding ADR constraints: GICv3 as the AArch64
baseline (ADR-032); capability-driven platform selection, never
QEMU/platform names (ADR-044, ADR-052); Core/Arch layering with register
access confined to the Arch domain (ADR-041–045); Guest-untrusted
boundaries (ADR-007); one owner per hardware/state datum and structured
telemetry (section 12, ADR-048); vCPU switch must handle vGIC state
explicitly (section 5). The task book §8 classifies List-Register
allocation, locking, and trace encoding as Implementation Choice resolved
here, and requires the exact supported GIC revision to be fixed only after
authoritative specification/platform review.

Classification summary:

- **Required:** virtualization-interface readiness per pCPU (capability
  verification against W01 conclusions, capacity discovery, safe enable);
  the per-pCPU LR table; presentation of W07 selections as software (hw=0)
  LRs with priority carriage; entry/exit context preservation (VMCR, active
  priority registers, LR purge/re-pend); bounded pressure behavior (fill to
  capacity, excess stays pending in W07); the maintenance-state read and
  refill primitives consumed by W09; per-pCPU diagnostics and telemetry
  hooks.
- **Reserved:** hardware-mapped (hw=1) LRs and any passthrough path;
  maintenance processing policy (W09); Guest-chosen priority semantics
  (W10); ITS/MSI/LPI and PCI interrupt virtualization (task book out of
  scope); vGIC Distributor/Redistributor MMIO modeling (P8); exact
  fair-share or quality-of-service refill ordering beyond the bounded P6
  policy.
- **Out of Scope:** physical GIC bring-up (W02 design, consumed by
  citation); vIRQ lifecycle ownership (W07); maintenance policy (W09);
  Validation Guest scenarios (W11); storm evidence (W12); telemetry
  collection/latency baseline (W13); the Linux-visible machine model
  (P8); crate/module paths beyond the stated contracts.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Virtualization-interface readiness | [foundations](01-scope-and-foundations.md) §4 (D1); [interface-context contracts](03-code-contracts-interface-context.md) §2 | W08-DV01 → P6-V01 consumption |
| Available presentation capacity | [interface-context contracts](03-code-contracts-interface-context.md) §2; [LR contracts](04-code-contracts-lr-presentation.md) §2 | W08-DV02 → P6-V16 basis |
| vCPU-context preservation | [interface-context contracts](03-code-contracts-interface-context.md) §3–§4 | W08-DV03 → P6-V11/P6-V16 basis |
| Basic priority carriage | [foundations](01-scope-and-foundations.md) §4 (D7); [LR contracts](04-code-contracts-lr-presentation.md) §3 | W08-DV05 → P6-V11 basis |
| Safe pressure behavior for pending work | [foundations](01-scope-and-foundations.md) §4 (D6); [LR contracts](04-code-contracts-lr-presentation.md) §4 | W08-DV04 → P6-V16 |
| Architecture/platform separation; unsafe boundaries; P8 non-ABI constraints | [architecture](02-architecture-and-state.md) §1–§2; [handoff](06-validation-and-handoff.md) §3 | W08-DV06 review |
| Capacity/presentation facts to W09–W13 and P8 | [handoff](06-validation-and-handoff.md) §3 | W08-DV06 |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no implemented W01/W02/W07
mechanisms, and no P6 implementation designs in this worktree yet (siblings
are being prepared in parallel on this branch). Every prerequisite is an
**assumed contract** with an entry check and failure boundary; the ledger
is in [01-scope-and-foundations.md](01-scope-and-foundations.md) §3 and the
contract table in §5. No row invents a GIC revision, LR count, register
offset, or platform behavior; all such facts are discovered or
intake-validated at implementation.

## Resolved design decisions and their authority

The numbered decisions (D1–D8), their rationale, and authority basis are in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §4. In brief:
(D1) readiness is capability-gated on the W01 virtualization conclusions and
fails precisely; (D2) all virtual-interface state is per-pCPU, manipulated
only on the pCPU running the vCPU; (D3) P6 presents only software (hw=0)
interrupts — hardware-mapped LRs are Reserved for passthrough; (D4) LR
capacity is discovered per pCPU from the virtualization-interface
identification register, never assumed; (D5) entry and exit are fixed
sequences with explicit barrier placement, and exit always returns LR
state into W07 truth before the vCPU is regarded as unloaded; (D6) pressure
policy is deterministic priority-ordered fill to capacity with the excess
left pending in W07 — no loss, no overwrite; (D7) priority carriage copies
the W07-validated priority into the LR priority field, masked to the
discovered width; (D8) the maintenance boundary is: W08 enables the
maintenance sources, exposes state reads and the refill primitive, and
W09 owns processing policy.

## Work breakdown and loading order

1. Load this README and the Coding Guidelines; complete the coding
   preflight.
2. Load [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   prerequisite contracts, failure boundaries, scope classification, and
   decisions D1–D8.
3. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   logical modules, the LR slot lifecycle, and the concurrency rules.
4. Implement in the order given in
   [05-implementation-workflow.md](05-implementation-workflow.md), loading
   [03-code-contracts-interface-context.md](03-code-contracts-interface-context.md)
   for workflow steps 3–4 and
   [04-code-contracts-lr-presentation.md](04-code-contracts-lr-presentation.md)
   for workflow steps 5–6.
5. Close with [06-validation-and-handoff.md](06-validation-and-handoff.md):
   run the validation matrix, record run/not-run evidence in the
   verification record path above, and complete the handoff checklist.

## Explicitly excluded interfaces

W08 authorizes no Guest-visible interface at all (no MMIO, no trap handler
for Guest GIC access — Guest GIC MMIO trapping is not modeled in P6); no
hardware-mapped interrupt path; no maintenance processing policy; no
scheduler or migration API; no persistent or wire representation of
interface state (snapshot/migration serialization is P16/P17-owned and must
define its own format). Its only named collaborators are: the W02 physical
GIC readiness contract (consumed, not redesigned), the W07 lifecycle
protocol (consumed), W09 (maintenance processing, consumes W08 primitives),
W06 (the vCPU entry/exit boundary it extends), and the P4 transition
context. Crate names, module paths, and file trees are not designed here;
all register access is Arch-domain `unsafe` under the audited boundary.

## Downstream handoff

Per the [plan index](../../../p6/plans/README.md) consumer map:

- **P6-W09** (`../p6-w09-maintenance-interrupt/README.md`) receives the
  completed-presentation/reusable-capacity boundary: maintenance-state
  reads, LR purge/clear primitives, and the refill operation, with the
  invariant that processing cannot lose or duplicate completion.
- **P6-W10** (`../p6-w10-interrupt-semantics/README.md`) receives the
  presentation behavior (priority carriage, load ordering) its
  masking/priority/concurrent-event semantics build on.
- **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`)
  receives the presentation facts its VG-IRQ scenarios observe.
- **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`) receives
  the pressure and unexpected-condition containment rules to stress.
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives
  capacity facts, presentation counters, and the maintenance-frequency
  inputs for the factual records and baseline.
- **P8** receives only lower-level evidenced capability and presentation
  facts. The Linux-visible vGIC Distributor/Redistributor model, Guest DTB,
  and machine ABI remain P8-owned; nothing here freezes a machine
  contract.

No consumer may treat W08 register sequences as a hardware contract beyond
the declared QEMU reference environment, and none may assume a specific LR
count or GIC revision in Core.
