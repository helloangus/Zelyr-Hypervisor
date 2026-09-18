# P8-W09 Virtual Console and Single-vCPU Linux — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The non-Virtio Guest console (emulated PL011 subset, frontend and
separable backend, RX input path) and the complete one-vCPU Linux path to
interactive initramfs userspace, per
[P8-W09](../../plans/p8-w09-virtual-console-single-cpu-linux.md).  
**Owner/change context:** P8-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W09. It converts the bounded
work-package plan into two deliverable groups: (1) the console device — a
Guest-facing PL011 register frontend, a separable DeviceBackend boundary per
ADR §8, the RX input/interrupt path over
[W07](../p8-w07-linux-vgicv3/README.md), and the containment rules for
malformed access; and (2) the single-vCPU Linux boot path — the ordered
milestones from the [W03](../../plans/p8-w03-linux-boot-contract.md) boot
inputs through earlycon, kernel init, and the P8-V13 interactive-shell
criterion, with retained boot-log markers. It deliberately does not design
virtio (P9), SMP behavior ([W10](../p8-w10-linux-smp-bringup/README.md)),
the Linux fixture itself (W15), or the automation harness (W16).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned work | Load |
|---|---|
| Modules, device-state ownership, frontend/backend boundary, concurrency | [01 Architecture and state](01-architecture-and-state.md) |
| PL011 frontend register contracts | [02 Console frontend contracts](02-code-contracts-console-frontend.md) |
| Backend boundary, TX sink, RX input/interrupt, log retention | [03 Console backend and input contracts](03-code-contracts-console-backend-and-input.md) |
| Single-vCPU boot milestones and observables | [04 Single-vCPU boot path](04-single-vcpu-boot-path.md) |
| Implement in dependency order | [05 Implementation workflow](05-implementation-workflow.md) |
| Validate and hand off | [06 Validation and handoff](06-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P8
task book, P8-W09 plan, and the sibling contracts cited above. This document
proposes design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → the P8-W02
machine-contract gate → P8-W09 plan → this design → Coding Guidelines. In
particular:

- The task book requires a **non-Virtio console** for P8; ADR-026/058 keep
  virtio as the primary paravirtual ABI from P9 and allow an experimental
  EL2-local console backend meanwhile; ADR-027 sanctions device emulation.
  The console MMIO base and SPI INTID are
  [W02](../../plans/p8-w02-machine-contract-governance.md)-gated; the
  register-behavior subset is specified here by Specification Investigation
  from the ARM PL011 technical reference (task book §8 route).
- ADR §8 separates Guest-facing VirtualDevice from DeviceBackend; the
  boundary here is designed so P9 can re-home the backend without changing
  the Guest ABI.
- ADR-007/§19 make the Guest untrusted: every MMIO access and every register
  field is validated; a Guest can flood, misconfigure, or abuse the UART
  only within contained, VM-scoped outcomes
  ([W05](../../plans/p8-w05-linux-cpu-virtualization.md) classification).
- P8-V13 defines the completion bar: **interactive** initramfs userspace
  with a retained boot log — `start_kernel` alone is explicitly
  insufficient (plan work sequence 5). The bidirectional input path is
  therefore Required, not optional polish.

Classification:

- **Required:** PL011-subset frontend (register table of
  [02](02-code-contracts-console-frontend.md)); TX output path; RX input
  path with bounded queue and interrupt; PrimeCell/Peripheral ID registers
  required for AMBA enumeration; per-VM frontend state; backend boundary
  with the P8 host backend; boot-log retention with markers; single-vCPU
  boot milestones M0–M7 ([04](04-single-vcpu-boot-path.md)); containment
  scenarios; telemetry.
- **Reserved:** DMA-side PL011 registers (RAZ/WI trigger: a declared
  scenario needs them — none does, PL011 DMA is optional); flow-control
  modem signals beyond read-as-inactive; a second console instance;
  frontend reuse for the Validation Guest's debug channel (W19 decides
  reuse, this design keeps the device separable).
- **Out of Scope:** virtio-console and any virtio transport (P9); the final
  production device register model beyond the frozen v1 subset (W02 gate
  owns machine placement; this design owns behavior of the declared subset);
  userspace distribution contents (W15/W03); SMP validation (W10);
  performance tuning; Host serial-driver design (Host OS concern).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Early/kernel/userspace console | [02 Console frontend contracts](02-code-contracts-console-frontend.md) §2–§3, [04 Single-vCPU boot path](04-single-vcpu-boot-path.md) M2–M7 | P8-V12 + P8-V13 |
| Shell interaction (bidirectional) | [03 Console backend and input contracts](03-code-contracts-console-backend-and-input.md) §3, [04](04-single-vcpu-boot-path.md) M7 | P8-V13 |
| Serial containment / malformed access | [02](02-code-contracts-console-frontend.md) §5, [03](03-code-contracts-console-backend-and-input.md) §4, decisions D5/D6 | P8-V12 containment rows |
| Complete boot milestones + retained boot log | [04 Single-vCPU boot path](04-single-vcpu-boot-path.md), [03 §5](03-code-contracts-console-backend-and-input.md) | P8-V13 |
| start_kernel insufficiency rule | [04 §3](04-single-vcpu-boot-path.md), [06 Validation and handoff](06-validation-and-handoff.md) | P8-V13 passing condition |
| Console ownership and isolation review | [01 Architecture and state](01-architecture-and-state.md) §5–§6 | P8-V03-style review |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p8-implementation-designs`): documentation scaffold only — no Cargo
workspace, no Rust sources, no P1–P8 implementation records; `docs/stages/p1/`
contains plans only (the early console of
`docs/stages/p1/plans/p1-w06-early-console-logging.md` is a Host-side EL2
diagnostic channel, planned, and is not a Guest device). Every prerequisite
below is a planned contract consumed as an assumption with a failure
boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Non-Virtio console exists and serves earlycon/kernel/userspace (P8-V12) | No Guest console device exists; Host early console is P1-W06, EL2-side only | The PL011-subset frontend and backend contracts of [02](02-code-contracts-console-frontend.md)/[03](03-code-contracts-console-backend-and-input.md) | Without a standard device Linux can drive, userspace console is unreachable and P8-V13 fails | P8-W09 (this design); placement/INTID via W02 gate | P8-V12/V13 rows (future `../../verification/p8-w09-virtual-console-single-cpu-linux-verification.md`) |
| One-vCPU Linux reaches interactive initramfs (P8-V13) | No boot path exists; all foundations (W04–W08) are plans | The milestone model of [04](04-single-vcpu-boot-path.md) binding W03–W08 contracts into an ordered, observable path | "Interactive" requires the whole chain, each with its own observable marker | W03–W08 own their stages; W09 owns the integration path | P8-V13 with retained markers |
| Bidirectional shell interaction | No input path exists anywhere in plans before W09 | RX queue + interrupt contracts of [03 §3](03-code-contracts-console-backend-and-input.md) | P8-V13's "interactive" criterion is meaningless without Guest input | This design; delivery via W07 | P8-V13 interaction row |
| Malformed console access contained (P8-V12) | No device code; W05 owns classification | Per-register validation and containment table of [02 §5](02-code-contracts-console-frontend.md) | UART abuse (flooded FIFO, illegal offsets) must not affect Host serial or other VMs | W05 categories; this design applies them | P8-V12 containment rows; W18 |
| Retained boot log with markers | No retention mechanism planned before W09 | Backend log buffer contract of [03 §5](03-code-contracts-console-backend-and-input.md) | P8-V13 requires "retained boot log" as evidence material | This design (bounded buffer, Host-owned) | P8-V13 log rows |
| Machine-gated placement | Machine ABI unfrozen (ADR §18) | Placeholder tokens `<CONSOLE-MMIO-BASE>`, `<CONSOLE-SPI-INTID>` owned by the W02 gate | Values frozen outside the gate violate task-book routing | W02 gate owns values | P8-V02/V03 review rows |
| Console DTB node facts | W04 owns the node | Consumed facts list (compatible string, base, INTID) in [02 §4](02-code-contracts-console-frontend.md) | Divergence breaks Linux probing at the earliest stage | W04 | P8-V05 consistency review |

No row above selects a final machine value inside this design alone; the
outstanding decision is the W02 gate (MMIO base, INTID), placed by the task
book before implementation.

## Resolved design decisions and their authority

1. **Device: emulated PL011 subset.** The console is an ARM PL011 UART
   emulation implementing the register subset of
   [02 §2](02-code-contracts-console-frontend.md). Rationale (Specification
   Investigation route, task book §8): Linux ships the amba-pl011 driver and
   `earlycon pl011,mmio32` support; register semantics are spec-fixed in the
   PL011 TRM, so no private device ABI is invented (protecting the v1
   machine-ABI stability ADR-024 wants); alternatives were rejected —
   virtio-console is P9 and non-Virtio is required; a custom minimal MMIO
   device would need a custom kernel driver for userspace, contradicting the
   interactive-shell bar; semihosting is not a Guest-visible device.
2. **Device identity must be enumerable:** the PrimeCell/Peripheral ID
   registers return the PL011's standard values, because Linux's AMBA bus
   probe reads them from hardware, not only from DTB. Values are TRM-fixed
   constants, machine-gate-registered like all Guest-visible facts.
3. **Frontend/backend separation is a designed boundary (ADR §8).** The
   frontend owns Guest-visible register state; the backend owns the Host
   sink/source. The backend trait ([03 §2](03-code-contracts-console-backend-and-input.md))
   is the seam P9's virtio-console work and W19's Validation-Guest reuse may
   consume without touching the Guest ABI.
4. **TX is always-ready, store-and-forward.** The guest FIFO drains
   immediately into the backend; FR reports TX-not-busy. Rationale: the P8
   backend is an in-memory sink; backpressure semantics are meaningless
   before P9's real I/O and would only add failure modes. Bounded: a Guest
   write burst is bounded per MMIO access; the backend buffer policy bounds
   total ([03 §4](03-code-contracts-console-backend-and-input.md)).
5. **RX is interrupt-driven with a bounded queue.** Host-injected input
   enters a fixed-capacity queue; RX interrupt asserted via
   [W07 §3](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md) on
   the machine-gated INTID; overflow drops with a counter (never blocks the
   Host, never grows unboundedly). Input is Host-controlled data; the Guest
   can only drain it.
6. **Containment:** the Guest cannot reach Host serial hardware — MMIO in
   the console window resolves to the emulator only; malformed access is
   W05-classified; FIFO/misconfigure abuse is rate-neutral to the Host
   (store-and-forward + bounded queues); the device state is per-VM so no
   cross-VM effect exists. Serial "ownership" (P8-V12 wording) means Host
   serial stays under Host control by construction.
7. **Single-vCPU first, SMP later:** this design's boot path fixes the
   1-vCPU baseline and its markers; W10 extends to 2/4 vCPU on top. The
   design contains no SMP-specific assumptions.
8. **Naming and placement:** logical names only; placement follows the
   approved workspace decision; the frontend contains no board/SoC/QEMU
   constants (ADR-043).

## Work breakdown and loading order

1. Read this README, then [01 Architecture and state](01-architecture-and-state.md)
   for the module map and ownership.
2. Implement in the order given by
   [05 Implementation workflow](05-implementation-workflow.md), loading
   [02](02-code-contracts-console-frontend.md) (frontend),
   [03](03-code-contracts-console-backend-and-input.md) (backend/input/log),
   and [04](04-single-vcpu-boot-path.md) (boot path) as the workflow reaches
   them.
3. Record implementation decisions in
   `../p8-w09-virtual-console-single-cpu-linux-record.md` when implementation
   begins and evidence in
   `../../verification/p8-w09-virtual-console-single-cpu-linux-verification.md`
   when scenarios run. Neither file may claim W09 complete; P8-V12/V13 are
   the proof surfaces.

## Explicitly excluded interfaces

No virtio surface, custom device register outside the declared PL011 subset,
Host serial driver, userspace artifact, SMP path, or performance mechanism
is designed or authorized here. The Guest-visible surface is the
machine-gated PL011 subset this design declares; anything beyond it is a
scope conflict to stop at review (at minimum W02 for placement, W04 for DTB,
P9 for virtio, W10 for SMP).

## Downstream handoff

- **W10 (Linux SMP bring-up)** receives the verified 1-vCPU baseline, its
  markers ([04](04-single-vcpu-boot-path.md)), and the console as the
  standard observability channel for 2/4-vCPU runs.
- **W16 (automated regression)** receives the milestone/marker list and
  containment scenarios as the automated verdict basis (P8-V21/V22 consume
  the markers).
- **W19 (Validation Guest dual-track)** receives the separable backend
  boundary ([03 §2](03-code-contracts-console-backend-and-input.md)) as the
  reuse point for the Validation Guest debug channel — reuse decisions are
  W19's.
- **W15 (reproducible fixture)** receives the bootargs/earlycon expectations
  the milestones depend on ([04 §2](04-single-vcpu-boot-path.md)) as fixture
  inputs.
- **W13 (fault diagnostics)** receives the console containment diagnostic
  context; **W17** receives console telemetry events; **W18** receives the
  malformed-access scenarios.
