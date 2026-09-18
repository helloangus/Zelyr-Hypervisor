# P8-W09 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W09 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| M1 Console frontend (CNF) | Emulate the PL011 register subset for one VM; produce TX bytes; expose RX bytes | Per-VM frontend state (§3.1) | Trapped MMIO accesses (IPA, width, value) | TX byte stream to M2; RX bytes to Guest reads; RX IRQ requests to M4 | Does not touch Host serial; does not decide input content; does not route IRQs (W07 does) |
| M2 Backend boundary (CNB) | The separable DeviceBackend seam: sink TX, source RX | None of its own (delegates to the bound backend instance) | Frontend TX bytes; backend input callbacks | Backend calls; state consistency | Does not emulate registers; the P8 host backend's internals are §3.3's |
| M3 Host console backend (CNH) | The P8 EL2-local backend: in-memory retained log + Host stdout sink + fixture input source | Retained log buffer (bounded), input queue (bounded) — as backend-owned state behind the §2 seam | Frontend TX; Host/fixture input injection | Log reads (evidence), input events into M4 | Not a logging subsystem (P0 telemetry rules still apply to Hypervisor logs); not a terminal emulator |
| M4 RX path (CNI) | Move input events into frontend RX state and assert the RX interrupt per PL011 semantics | Per-VM RX pending bookkeeping (inside frontend state) | Backend input events | `vgic_request_irq` calls ([W07](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md)) | Does not own the INTID (machine-gated); does not reorder Guest reads |
| M5 Containment and diagnostics (CND) | Apply W05-classified outcomes to malformed access; emit telemetry; keep W13 diagnostic context | Bounded per-VM counters and one diagnostic slot | All M1 accesses | Contained outcomes; telemetry; diagnostics | Not a fault taxonomy (W13 owns classification) |
| M6 Boot-path observer (BPO) | Map boot console markers to milestone state for evidence and automation ([04](04-single-vcpu-boot-path.md)) | Per-boot milestone record (bounded) | Retained log content | Milestone/marker report for P8-V13/W16 | Not a test harness (W16 automates; this observes) |

Layering: the frontend is a Guest-ABI device model — no board/SoC/QEMU
constants; placement (base, INTID) is machine-gated data (README decision
D8). The backend boundary keeps Host-side behavior replaceable (ADR §8,
decision D3).

## 2. The frontend/backend boundary (decision D3)

```text
Guest EL1                Hypervisor EL2                         Host side
  PL011 MMIO write  -->  M1 frontend (register state machine) --> M2 seam::send(bytes)
  PL011 MMIO read   <--  M1 (DR from RX state, FR from state) <-- M2 seam::recv() /
                          M4 RX path  <-- input event ---------- backend input source
  RX IRQ          <----  M4 via W07 injection on <CONSOLE-SPI-INTID>
```

The seam is exactly two operations plus lifecycle (bound/unbind) —
[03 §2](03-code-contracts-console-backend-and-input.md) fixes it. P9 or W19
may supply a different backend without frontend changes; the Guest ABI
never changes.

## 3. Core objects and ownership

### 3.1 Per-VM frontend state (owner: the VM's device set)

- `rx_queue`: fixed-capacity FIFO of received bytes (capacity is a design
  constant owned here, declared in [03 §3](03-code-contracts-console-backend-and-input.md);
  it is not machine-contract data — it is Guest-invisible).
- `tx`: no FIFO (store-and-forward, decision D4) — only error/latch state.
- Register image: control registers the subset declares writable (CR, IBRD/
  FBRD, LCR_H, IFLS, IMSC), read-only status (FR, RIS/MIS, flag latches),
  and the ID registers (read-only constants).
- Created with the VM (device presence is a machine fact), destroyed with
  the VM. All mutation from Guest MMIO under the VM's device lock.

### 3.2 Backend-owned state (owner: the bound backend, behind the seam)

- Retained boot log: bounded ring/string buffer with documented capacity and
  drop-on-overflow counter (Host evidence material; never Guest-readable).
- Input source: the fixture/Host side enqueues bounded input chunks; the
  backend forwards them into M4; the Guest cannot write this state.
- The P8 host backend is EL2-local (ADR-058 experimental allowance); its
  internals stay behind the seam so they impose nothing on the Guest ABI.

### 3.3 Explicit non-ownership

The Guest cannot allocate, grow, or address any Hypervisor structure: the
RX queue capacity, log capacity, and counters are fixed at creation. No
Guest register write changes Host resource usage beyond the fixed footprint
(containment basis for P8-V12).

## 4. Device lifecycle

```text
VM creation:  frontend instantiated from machine facts (base, INTID, IDs);
              backend bound (P8: host backend); log buffer initialized
boot:         Guest earlycon/kernel/userspace drive the registers (milestones
              M0–M7 of [04](04-single-vcpu-boot-path.md))
VM stop/off:  (SYSTEM_OFF, [W06]) frontend quiesced; log buffer retained as
              evidence until VM destruction
VM destroy:   frontend, backend binding, and log buffer dropped; no global
              residue; counters finalized into telemetry
```

Backend unbind/rebind (fault recovery shape, ADR §9) is Reserved for P9+
service backends; the P8 lifecycle binds once.

## 5. Concurrency model

- **Guest-side (M1/M4/M5):** all Guest-triggered paths run in VM-exit
  context — bounded per access, no allocation (fixed state, §3.1), no
  polling, no waiting. One VM device lock orders frontend mutations; lock
  order: device lock after the VM GIC lock is released (delivery via W07
  issues requests, it does not hold locks across the call).
- **Host-side (M3):** input injection and log reads are Host-context
  operations that queue events to M4 (bounded, coalescing drops) rather than
  touching frontend state directly; log reads snapshot under the backend's
  own lock. No Host path blocks on the Guest, and no Guest path blocks on
  the Host.
- **Cross-vCPU:** the console is machine-declared single-instance; Linux
  serial drivers serialize port access; the frontend needs no cross-vCPU
  protocol — accesses from any vCPU of the VM serialize on the device lock.

## 6. Security model

- **Untrusted input (ADR §19):** every MMIO access is bounds/width/alignment
  checked ([02 §5](02-code-contracts-console-frontend.md)); register fields
  are masked; RX/TX data bytes are payload, never control (the frontend's
  control decisions depend only on register state).
- **Isolation:** the console window maps to the emulator for this VM only;
  there is no code path from any Guest register value to Host serial
  hardware, other VMs, or Host files (the log is a Hypervisor-owned buffer;
  its export is a Host/evidence concern, [03 §5](03-code-contracts-console-backend-and-input.md)).
- **Flooding:** TX flooding is bounded by the backend policy (drop +
  counter); RX flooding is impossible from the Guest (input is
  Host-controlled); interrupt storms are bounded by the RX-empty condition
  (the IRQ asserts only while input is pending and unmasked).
- **Diagnostic leakage:** W13's diagnostic slot records {vCPU, IPA, access
  class}; it never contains Guest buffer contents.

## 7. Failure boundaries of assumed prerequisite contracts

| Prerequisite | Assumed contract (cite) | Failure boundary if different |
|---|---|---|
| W07 SPI injection | `../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md` §3 | Without `vgic_request_irq`, RX interrupts cannot be delivered; block at the W07/W09 seam (polling fallback is explicitly not authorized — it would change the Guest-visible device behavior) |
| W03 boot inputs | `docs/stages/p8/plans/p8-w03-linux-boot-contract.md` | Without boot input facts (Image/DTB/initramfs/bootargs, boot vCPU state), [04](04-single-vcpu-boot-path.md) has no start point; block |
| W04 DTB console node | `docs/stages/p8/plans/p8-w04-guest-dtb-contract.md` | Node must match machine placement and the PL011 compatible facts ([02 §4](02-code-contracts-console-frontend.md)); mismatch fails P8-V05 review, fix in W04's contract |
| W05 classification | `docs/stages/p8/plans/p8-w05-linux-cpu-virtualization.md` | MMIO fault outcomes are W05-classified; divergence wins at that seam |
| W08 time semantics | `../p8-w08-linux-timer-integration/README.md` | Milestone timing assertions (sleep-based tests) assume I1–I4; divergence blocks those rows only |
| W02 machine gate | `docs/stages/p8/plans/p8-w02-machine-contract-governance.md` | Unapproved base/INTID ⇒ no implementation may embed them |
| P4 MMIO trap path | `docs/stages/p4/plans/p4-w09-closeout-p5-handoff.md` | Without write-fault attribution, M1 has no input path; block at the P4/W09 seam |
| W06 VM shutdown | `../p8-w06-psci-virtualization/README.md` | SYSTEM_OFF quiesce hook shape must match §4; mismatch stops integration |
