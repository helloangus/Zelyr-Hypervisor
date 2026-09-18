# P8-W07 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W07 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| M1 GIC address map and access routing (VGM) | Decode Guest MMIO in the GICD/GICR regions to (register, vCPU-context, width, offset); enforce region bounds and alignment | None | Trapped Stage-2 MMIO fault (IPA, width, value, vCPU), VM topology | Routed access or Reject-classified fault per W05 | Does not emulate register semantics; does not own Stage-2 tables (P4) |
| M2 Distributor model (VGD) | Emulate GICD_* registers for SPIs: enables, priority, config, group, pending set/clear, typer/iidr reporting | Per-VM Distributor state (§3.1) | Routed accesses from M1; SPI INTID table from machine contract | Register values; pending-latch transitions into M5 | Does not deliver interrupts; does not own vIRQ lifecycle |
| M3 Redistributor and CPU-interface model (VGR) | Emulate per-vCPU GICR_* (WAKER, SGI/PPI control) and ICC_*_EL1 semantics (PMR, CTLR, IGRPEN1, SGI1R) | Per-vCPU Redistributor/interface state (§3.2) | Routed accesses (GICR) and system-register traps (ICC) | Register values; SGI requests into M4; interface state into M5 | Does not allocate List Registers (P6-W08); does not preserve VMCR (P6-W08 context boundary) |
| M4 SGI generation (VGS) | Decode/validate ICC_SGI1R_EL1 writes; resolve target-list against virtual topology; issue per-target injection requests | None (stateless per call) | SGI register value, issuing vCPU | One injection request per valid target (into M5 bridge) | Does not implement inter-vCPU transport; does not arbitrate |
| M5 Injection/EOI bridge (VGI) | Translate between GIC register semantics and the P6-W07 vIRQ lifecycle (pending/active/completion), P6-W08 presentation capacity, P6-W09 maintenance | None of its own — every transition is a P6 lifecycle call | M2/M3/M4 requests; LR capacity facts; maintenance events | P6 lifecycle calls; delivery outcomes back to register-visible state | Does not implement pending queues, LR selection, or maintenance policy (P6) |
| M6 Diagnostics and telemetry (VGT) | Structured events for injections, EOIs, SGIs, rejections, pressure | Bounded per-VM/per-vCPU counters | All modules | Telemetry events; W13 diagnostic context | Not a log; no unbounded buffers |

Module boundaries preserve layering: M1–M4 contain no Host-platform or QEMU
constants; every Guest-visible numeric fact enters via machine-contract data
(README decision D7).

## 2. Address map and access facts (machine-gated)

The Guest GIC layout is machine-contract data (W02 gate). This design fixes
only the structure and the architected constants:

- One Distributor region `GICD` at `<GICD-BASE>` (64 KiB per IHI 0069 GICD
  frame convention — value gated by W02).
- One Redistributor frame per vCPU: `RD_base` + `SGI_base` (two 64 KiB
  pages, the architected frame split) starting at `<GICR-FRAME-BASE>`, with
  the architected per-CPU stride; the frame order follows the machine
  contract's vCPU ordering.
- INTID allocation table (machine-gated): SGI 0–15, PPI 16–31 with the timer
  PPI per the timer policy of [W08](../p8-w08-linux-timer-integration/README.md)
  (INTIDs 26–30 are the architected arch-timer PPI positions), and the
  declared SPI set — at minimum `<CONSOLE-SPI-INTID>` for
  [W09](../p8-w09-virtual-console-single-cpu-linux/README.md). SPI count
  reported via GICD_TYPER must match the machine contract.
- Access width rules: GICD/GICR register semantics per IHI 0069 (32-bit
  words; byte access support as architected for the implemented subset);
  M1 enforces region bounds and the alignment the implemented subset
  requires; out-of-region → Stage-2 fault classification per W05.

## 3. Core objects and ownership

### 3.1 Per-VM Distributor state (owner: VM)

- `spi_table`: per SPI INTID — {enabled, priority, group1, config
  (edge/level), pending-latch, active-flag mirror}; backing for M2.
- `ctlr`: distributor enable bits (G1/G1NS presentation for the Guest's
  single secure state).
- Immutable facts: typer/iidr values reported to the Guest (machine-gated,
  consistent with topology size and SPI count).
- Created at VM creation, destroyed with the VM. All mutations occur from
  Guest MMIO paths under the VM's GIC lock (bounded).

### 3.2 Per-vCPU Redistributor/interface state (owner: vCPU)

- `sgi_ppi_table`: per INTID 0–31 — same field set as `spi_table` (SGI/PPI
  enables live in GICR_ISENABLER0 per IHI 0069).
- `waker`: GICR_WAKER state (Quiescent/NotQuiescent + RegisterWritePending
  bit presentation) — the handshake Linux uses at secondary bring-up.
- Interface registers: `pmr` (priority mask), `ctlr` (EOImode presentation —
  read-as-zero in the P8 subset, per the README's Reserved list),
  `igrpen1`.
- Created with the vCPU; carried across P7 vCPU transitions (stop/resume)
  via the quiesce/resume hooks (§5); destroyed with the vCPU.

### 3.3 What this design does NOT own

Pending-queue depth and ordering, deferred delivery, LR contents, VMCR
save/restore, and maintenance state are P6-W07/W08/W09 property. W07's
register state is the translation layer that makes P6 state visible in GIC
semantics; it must never cache a second copy of truth (a latch here is
derived from, and written back through, P6 lifecycle calls).

## 4. INTID state model (mapping, not redesign)

The GIC-visible lifecycle of one INTID maps onto the P6-W07 vIRQ lifecycle
(citation, not redefinition — README decision D4):

| GIC register-visible condition | P6-W07 vIRQ lifecycle state (consumed) |
|---|---|
| Set-pending (GICD/GICR ISPENDR write, hardware event, or SGI) | vIRQ request accepted → pending |
| Pending while disabled/masked (CTLR/PMR) | pending, presentation deferred |
| Presented in a List Register (P6-W08 capacity permitting) | presented |
| Guest ACK/EOI (ICC_EOIR1_EL1 / ICC_DIR_EL1 per EOImode) | active → completed |
| ISPENDR re-assert while pending/active | repeated-arrival policy (P6-W07 owns the coalescing semantics) |
| Over-capacity pending | preserved pending (P6-W07 deferred + P6-W09 release) |

Derived rule owned here: enable/disable transitions (ISENABLER/ICENABLER)
never destroy pending state — a disabled interrupt latches pending and is
presented only after enable; this follows IHI 0069 and is expressed as
"deferred presentation", not a new state.

## 5. Lifecycle hooks for vCPU transitions

- `vgic_quiesce(vcpu)`: called by CPU_OFF ([W06 §3](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md))
  and by the P7 stop path — withdraws any presented-but-incomplete state via
  P6-W08/P6-W09 boundaries so capacity returns, leaves pending/enable state
  intact for the future ON.
- `vgic_resume(vcpu)`: called at vCPU entry after a transition — restores
  interface-register presentation consistent with the carried vCPU state.

These hooks are the entirety of W07's coupling to the P7 lifecycle; the hook
signatures must agree with [W06](../p8-w06-psci-virtualization/README.md)
and [W10](../p8-w10-linux-smp-bringup/README.md) (workflow Step 6 there).

## 6. Concurrency and security model

- **Locks:** one VM-scoped GIC lock (Distributor state + cross-vCPU SGI
  resolution) and per-vCPU locks (interface state). Order: vCPU lock before
  VM GIC lock when both are held; never the reverse. All critical sections
  are bounded register-emulation work — no waiting on delivery.
- **VM-exit context:** every M1–M4 operation is a bounded MMIO/trap handler:
  fixed worst-case work per access, no allocation (register state is
  pre-provisioned per §3), no polling, no cross-CPU blocking. SGI fan-out to
  remote vCPUs issues requests (P6 lifecycle + P7 kick seams), it does not
  join them.
- **Untrusted-input rules (ADR §19):** region/offset/width validation in M1;
  INTID fields range-checked before any table index (SPIs ≤ machine SPI max;
  SGI/PPI < 32); SGI target-list bits masked to topology; priority/PMR
  comparisons use the architected 8-bit field; non-implemented register
  spaces answer RAZ/WI. A violation produces Reject-classified behavior per
  W05 with VM-scoped diagnostics — never a Host fault.
- **Cross-vCPU isolation:** an SGI may only target vCPUs of the issuing VM;
  PPIs are strictly per-vCPU; no register path can read or modify another
  vCPU's interface state.
- **Non-leakage:** IIDR/TYPER/IIDR-class values are machine-contract facts,
  never Host hardware reads (ADR-052/§19).

## 7. Failure boundaries of assumed prerequisite contracts

| Prerequisite | Assumed contract (cite) | Failure boundary if different |
|---|---|---|
| P6-W07 vIRQ lifecycle | `docs/stages/p6/plans/p6-w07-virtual-interrupt-core.md` — request/pending/presented/active/completed with target isolation | If states or completion semantics differ from §4's mapping, [03 §4](03-code-contracts-interrupt-flow.md) blocks; `Architecture Change Request` against P6, no local fork |
| P6-W08 presentation capacity | `docs/stages/p6/plans/p6-w08-gic-virtualization-interface.md` | If capacity/pressure behavior is absent, over-capacity scenarios (P8-V10 stress) block with a recorded gap |
| P6-W09 maintenance | `docs/stages/p6/plans/p6-w09-maintenance-interrupt.md` | If maintenance processing cannot release capacity, stress rows cannot be declared passing; blocked, not simulated |
| P4 MMIO trap + Stage-2 facts | `docs/stages/p4/plans/p4-w09-closeout-p5-handoff.md` | Without write-attribute MMIO faults, M1 has no input path; block at the P4/W07 seam |
| P7 lifecycle hooks | `docs/stages/p7/plans/p7-w07-pause-stop-fault.md` | If stop/entry hooks cannot call §5, hotplug rows block; raise at the seam |
| W02 machine gate | `docs/stages/p8/plans/p8-w02-machine-contract-governance.md` | Addresses/INTID table unapproved ⇒ no implementation may embed them (README decision placeholders) |
| W05 classification | `docs/stages/p8/plans/p8-w05-linux-cpu-virtualization.md` | ICC_SRE/trap classification is W05's; a divergent classification wins at that seam |
| W04 DTB GIC node | `docs/stages/p8/plans/p8-w04-guest-dtb-contract.md` | Node facts must equal the machine contract consumed here; mismatch fails P8-V05 review, fix in W04's contract |
