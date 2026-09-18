# P8-W07 Linux-Compatible Virtual GICv3 — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Linux-SMP-required Guest-visible GICv3 behavior — Distributor
and Redistributor register model, CPU-interface system-register semantics,
SGI generation, timer-PPI and required-SPI delivery, masking, and
pending/active semantics — per [P8-W07](../../plans/p8-w07-linux-vgicv3.md).  
**Owner/change context:** P8-W07 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W07. P6 already owns the
controller-independent virtual-interrupt lifecycle
([P6-W07](../../../p6/plans/p6-w07-virtual-interrupt-core.md)), the
List-Register presentation bridge ([P6-W08](../../../p6/plans/p6-w08-gic-virtualization-interface.md)),
and maintenance processing ([P6-W09](../../../p6/plans/p6-w09-maintenance-interrupt.md)).
This design **consumes** those contracts and adds only their Linux-facing
delta: the GICv3 register behavior Linux probes and drives during boot and
SMP operation, the per-VM/per-vCPU register state that backs it, and the
declared stress scope of P8-V10. It deliberately does not redesign the vIRQ
lifecycle, List-Register allocation, maintenance policy, or any Host IRQ
mechanism.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned work | Load |
|---|---|
| Modules, register-state ownership, INTID state model, concurrency | [01 Architecture and state](01-architecture-and-state.md) |
| Distributor/Redistributor MMIO register contracts | [02 vGIC MMIO contracts](02-code-contracts-vgic-mmio.md) |
| SGI generation, injection bridge, EOI/completion, LR integration | [03 Interrupt flow contracts](03-code-contracts-interrupt-flow.md) |
| Implement in dependency order | [04 Implementation workflow](04-implementation-workflow.md) |
| Validate and hand off | [05 Validation and handoff](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P8
task book, P8-W07 plan, and the P6 contracts cited above. This document
proposes design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → the P8-W02
machine-contract gate → P8-W07 plan → this design → Coding Guidelines. In
particular:

- ADR-032 makes GICv3 the AArch64 baseline; ADR-033 keeps the interrupt
  controller behind a replaceable backend so this design must not weld Linux
  behavior into a controller-agnostic API owned by P6.
- ADR-007/§19 make the Guest untrusted: every MMIO access (address, width,
  alignment, register fields) and every SGI target list is validated before
  use, and malformed behavior is contained to the calling VM
  ([W05](../../plans/p8-w05-linux-cpu-virtualization.md) classification
  categories).
- Task book §8 routes "timer/console/device details" through Specification
  Investigation in detailed design using GIC/AArch64/Linux sources. This
  design performs that investigation from Arm IHI 0069 (GICv3) and the Linux
  GICv3 driver's architectural requirements; it infers nothing from QEMU, and
  all Guest-visible addresses and INTID allocations are placeholders gated by
  the [W02](../../plans/p8-w02-machine-contract-governance.md) freeze.
- The plan excludes ITS, MSI, LPI, PCIe, the final List-Register strategy, and
  throughput optimization. The P8 v1 machine has no ITS (task book Out of
  scope), so no LPI/ITS register appears in the Guest model.

Classification:

- **Required:** Guest GIC address-map consumption (W02 gate); Distributor
  register subset; per-vCPU Redistributor subset including GICR_WAKER;
  CPU-interface (ICC_*_EL1) subset including PMR, CTLR, IGRPEN1, SGI
  generation via ICC_SGI1R_EL1; EOI/deactivate → vIRQ completion mapping;
  priority and masking semantics; pending/active state fidelity across
  enable/disable and vCPU transitions; per-vCPU bring-up facts for W10;
  diagnostics/telemetry.
- **Reserved:** EOImode 1 (split EOI/disable-groups 1NS handling beyond the
  minimal model) with trigger "a declared Linux scenario requires it";
  GICD_ICFGR edge/level nuance beyond the required subset; security
  (Secure-group) registers (always RAZ/WI in this Guest model); GICv4
  direct-LPI/vLPI features; MSIframe/ITS (later stages).
- **Out of Scope:** P6's vIRQ lifecycle internals, LR allocation, and
  maintenance policy (consumed); Host physical GIC driver (P6-W02/W03); DTB
  GIC node contents (W04 owns the node; this design defines the facts it must
  agree with); interrupt routing policy; Device/PCI/MSI (P9/P13); real-hardware
  GIC variants (P15).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Linux GIC initialization (Distributor + Redistributor + CPU interface) | [02 vGIC MMIO contracts](02-code-contracts-vgic-mmio.md) §2–§4, decisions D1–D3 | P8-V10 boot rows |
| Timer IRQ delivery | [03 Interrupt flow contracts](03-code-contracts-interrupt-flow.md) §4, [02](02-code-contracts-vgic-mmio.md) §4 (PPI enables) | P8-V10 timer rows (semantics with [W08](../p8-w08-linux-timer-integration/README.md)) |
| SGI behavior | [03 Interrupt flow contracts](03-code-contracts-interrupt-flow.md) §2–§3 | P8-V10 SGI rows |
| Necessary SPI (console, per machine contract) | [02 vGIC MMIO contracts](02-code-contracts-vgic-mmio.md) §3 + machine-contract INTID table | P8-V10 SPI rows |
| Masking, pending/active semantics | [01 Architecture and state](01-architecture-and-state.md) §4, [03](03-code-contracts-interrupt-flow.md) §4–§5 | P8-V10 mask/pending/active rows |
| Multi-CPU interrupt stress scope | [05 Validation and handoff](05-validation-and-handoff.md) §1 stress matrix, decisions D6 | P8-V10 stress rows |
| Host-independence and Guest-fault containment review | [01 Architecture and state](01-architecture-and-state.md) §6 | P8-V03-style review + P8-V24 (W18) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p8-implementation-designs`): documentation scaffold only — no Cargo
workspace, no Rust sources, no P6 implementation records; `docs/stages/p6/`
contains plans and a verification `.gitkeep`. Every prerequisite below is a
planned contract consumed as an assumption with a failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Linux GICv3 init works at boot and on secondaries (P8-V10 boot rows) | No vGIC register model or code exists | The register-model contracts of [02](02-code-contracts-vgic-mmio.md) (Required subset, RAZ/WI rules, WAKER semantics) | Linux's GICv3 driver probes exactly these registers; without a declared subset, "works" is untestable | P8-W07 (this design) per IHI 0069; freeze via W02 | P8-V10 boot/secondary rows (future `../../verification/p8-w07-linux-vgicv3-verification.md`) |
| vIRQ delivery exists behind the register model | P6-W07 lifecycle is planned, not implemented | Injection/EOI bridge contracts of [03](03-code-contracts-interrupt-flow.md) expressed strictly as calls into the P6-W07 lifecycle | Reimplementing delivery would fork the P6 contract | P6-W07 (cite, don't redesign) | P8-V10 delivery rows |
| Pending work survives LR pressure | P6-W08 presentation capacity is planned | Consumption-only integration: over-capacity → P6-W07 deferred state ([03 §5](03-code-contracts-interrupt-flow.md)) | Linux requires no-loss under pressure for boot reliability | P6-W08/W09 own capacity/maintenance | P8-V10 stress rows |
| Per-vCPU interrupt state ties to P7 lifecycle | P7 lifecycle planned; vCPU stop/fault is P7-W07 | Quiesce/resume hooks ([02 §5](02-code-contracts-vgic-mmio.md)) that CPU_OFF ([W06](../p8-w06-psci-virtualization/README.md)) calls | A secondary stopped and re-onlined must keep consistent vGIC state | P7-W07 seam; this design defines the hook behavior | P8-V14/W10 hotplug rows |
| Guest-visible GIC facts enter the machine contract | Machine ABI unfrozen (ADR §18) | Placeholder tokens (`<GICD-BASE>`, `<GICR-FRAME-BASE>`, SPI INTID table) owned by the W02 gate | Values frozen outside the gate would violate the task book routing | W02 gate owns values; this design owns semantics | P8-V02/V03 review rows |
| Malformed GIC access contained (P8-V24) | No validation code; W05 owns classification categories | Per-register validation and Reject/Direct classification application ([02 §6](02-code-contracts-vgic-mmio.md)) | An unvalidated MMIO offset or INTID field violates ADR §19 | W05 categories; this design applies them | W18 illegal-MMIO scenarios |

No row above selects a final machine value inside this design alone; the
outstanding decision is the W02 gate (addresses, INTID allocations), which
the task book places before implementation.

## Resolved design decisions and their authority

1. **Guest CPU-interface route: system registers.** The Guest uses ICC_*_EL1
   system registers (not MMIO CPU interface), with SGI generation via
   ICC_SGI1R_EL1, per IHI 0069 and Linux's GICv3 driver model. Rationale:
   this is the only GICv3 route Linux supports for a Guest without an MMIO
   CPU interface, and it matches ADR-032. The ICC_SRE presentation (SRE=1)
   is part of the W05 CPU-classification surface and P6-W08's VMCR
   preservation; W07 defines the Guest-visible register semantics only.
2. **Register subset is minimum-Linux.** The implemented set is exactly what
   the declared Linux scenarios (boot, secondary bring-up, timer, SGI,
   console SPI, mask/unmask, stress) require ([02 §2](02-code-contracts-vgic-mmio.md));
   everything else is RAZ/WI or reserved per IHI 0069 rules. Rationale: the
   plan excludes optimization and advanced features; a minimal declared
   subset is reviewable and testable. Additions follow the same design
   process, driven by a declared scenario.
3. **State split: per-VM Distributor state, per-vCPU Redistributor/CPU-interface
   state.** SPI state (enable, priority, config, pending latch, routing) is
   VM-scoped; SGI/PPI state and interface registers are vCPU-scoped,
   mirroring the GICv3 architecture so P7 vCPU transitions carry the right
   state ([01 §3](01-architecture-and-state.md)). This is stage-local design
   freedom within ADR §4/§7 object boundaries.
4. **Delivery, completion, and pressure are consumed, not re-owned.** Pending
   set/clear, injection, activation, and completion map 1:1 onto the P6-W07
   vIRQ lifecycle states; presentation uses P6-W08 capacity; maintenance
   reuses P6-W09. W07's contracts are the translation layer ([03 §4–§5](03-code-contracts-interrupt-flow.md)).
   Any required change there is an `Architecture Change Request` against P6,
   not a local fork.
5. **SGI generation is a system-register trap.** ICC_SGI1R_EL1 writes are
   trapped, decoded, and validated (target list filtered against virtual
   topology; IRM/range semantics per IHI 0069); invalid target sets are
   dropped for the invalid portion with a guest-visible benign outcome and
   telemetry, never a Host fault. This is the only Guest-to-Guest interrupt
   path in P8.
6. **Stress scope is declared, bounded, and failure-bounded.** The P8-V10
   stress matrix ([05 §1](05-validation-and-handoff.md)) declares the
   interrupt mixes, rates are bounded by fixture configuration (W16), and the
   pass condition is state fidelity (no lost/du pending, no cross-vCPU leak),
   not throughput (plan excludes optimization).
7. **Naming and placement:** logical names only (as in
   [W06](../p8-w06-psci-virtualization/README.md) decision D8); crate/module
   placement follows the approved workspace decision; the arch backend stays
   board/SoC-independent (ADR-043).

## Work breakdown and loading order

1. Read this README, then [01 Architecture and state](01-architecture-and-state.md)
   for ownership and the INTID state model.
2. Implement in the order given by
   [04 Implementation workflow](04-implementation-workflow.md), loading
   [02](02-code-contracts-vgic-mmio.md) for the register model and
   [03](03-code-contracts-interrupt-flow.md) for the delivery/EOI/SGI paths.
3. Record implementation decisions in
   `../p8-w07-linux-vgicv3-record.md` when implementation begins and evidence
   in `../../verification/p8-w07-linux-vgicv3-verification.md` when scenarios
   run. Neither file may claim W07 complete; P8-V10 is the proof surface.

## Explicitly excluded interfaces

No Host GIC driver API change, P6 vIRQ/lifecycle signature change, LR
allocation policy, maintenance policy, DTB node content, device model,
MSI/ITS/LPI surface, or interrupt-routing policy is designed or authorized
here. The only new Guest-visible surface is the machine-gated GIC register
model this design defines; anything beyond it is a scope conflict to stop at
review (at minimum W02 for values, P6 for lifecycle, W04 for DTB).

## Downstream handoff

- **W09 (console + single-vCPU Linux)** receives the SPI injection path and
  the machine-gated console INTID consumption point
  ([03 §4](03-code-contracts-interrupt-flow.md)) that console RX interrupts
  use.
- **W10 (Linux SMP bring-up)** receives the per-secondary bring-up register
  sequence facts (WAKER wake, SGI/PPI enables, PMR/IGRPEN) and the SGI path
  used for Linux IPIs ([02 §4](02-code-contracts-vgic-mmio.md), [03 §2–§3](03-code-contracts-interrupt-flow.md)).
- **W16 (automated regression)** receives the P8-V10 scenario matrix and
  expected markers from [05 §1](05-validation-and-handoff.md).
- **W18 (security isolation regression)** receives the illegal-MMIO/SGI
  containment scenarios ([02 §6](02-code-contracts-vgic-mmio.md)) as declared
  contained outcomes.
- **W13 (fault diagnostics)** receives the malformed-access diagnostic
  context contract ([02 §6](02-code-contracts-vgic-mmio.md)).
- **W14 (ABI compatibility)** receives the Guest-visible GIC fact list
  (regions, supported registers, typer-reported values) as compatibility
  dimensions.
