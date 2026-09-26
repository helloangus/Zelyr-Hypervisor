# P6-W02 — physical GIC bring-up

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish the bounded Host GICv3 bring-up outcome: a safe global Distributor state and independent local Redistributor and CPU-interface readiness for each online pCPU.

## Scope

Cover the P6 Host GIC bring-up boundary, including initial safe state, supported SPI range, per-pCPU local initialization, acknowledged readiness/failure, and diagnostic integration with P3 lifecycle facts.

## Out of scope

IRQ dispatch policy, virtual IRQs, Guest GIC MMIO, scheduler behavior, detailed register ordering, or final IRQ-controller abstraction.

## Work sequence

1. Inspect W01 and the P3 online-pCPU, CPU-local, synchronization, and diagnostic contracts.
2. Produce an approved detailed design for global and local Host GIC readiness, failure boundaries, and lifecycle ownership.
3. Integrate planned bring-up sequencing with platform capabilities and pCPU lifecycle without reusing BSP-local state on APs.
4. Define normal, partial-discovery, local-readiness, and residual-state acceptance scenarios.
5. Review layering, controlled unsafe boundaries, telemetry, and unsupported-platform handling.
6. Record factual status and hand the Host-GIC readiness boundary to W03–W05 and later validation.

## Acceptance and closure

P6-V02 and P6-V03 require evidence that the supported Distributor and every declared online pCPU local interface reach the documented usable state or produce a precise failure. Passing does not prove routing, Guest delivery, or a real-hardware support tier.

## Handoff

W03 receives the acknowledged Host IRQ boundary; W04 and W05 receive independent local readiness. Later work still owns dispatch, timer, and virtualized behavior.
