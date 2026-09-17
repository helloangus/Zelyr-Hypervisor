# P8-W07 — Linux-compatible virtual GICv3

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Define the Linux-SMP-required virtual GICv3 compatibility and evidence boundary.

## Scope

Cover source P8.6: Linux GIC initialization, CPU interfaces, timer IRQ, SGI, necessary SPI, masking, pending/active semantics, and multi-CPU interrupt stress.

## Out of scope

ITS, MSI, LPI, PCIe, final List Register strategy, interrupt-controller APIs, or throughput optimization.

## Work sequence

1. Inspect W01/W02 and P6 interrupt facts.
2. Identify Guest-visible GIC behavior required by the declared Linux scenarios.
3. Relate per-vCPU interrupt state to P7 lifecycle and P8 machine routing.
4. Define boot, secondary, mask/unmask, SGI, timer, and stress acceptance scenarios.
5. Review against Host-independent and Guest-fault containment constraints.

## Acceptance and closure

P8-V10 requires a declared matrix for required Linux GIC behavior; it makes no claim about advanced GIC features.

## Handoff

W09–W10, W16, and W18 receive the required behavior and evidence scope.
