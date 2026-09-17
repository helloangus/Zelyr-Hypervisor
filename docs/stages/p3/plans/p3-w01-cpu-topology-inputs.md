# P3-W01 — CPU topology inputs

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Establish a reviewed contract from P2 platform discovery to P3 for physical CPU topology, hardware identity, logical identity, boot-CPU designation, and availability classification.

## Scope

Topology input review and its required classifications: present, possible, online, and failed. The package must reject assumptions that enumerated order is hardware identity, that identities are contiguous, or that present implies online.

## Out of scope

Secondary startup, CPU lifecycle transition implementation, hotplug, board-specific discovery, and guest CPU objects.

## Work sequence

1. Inspect P0–P2 contracts and actual evidence for topology authority and platform capabilities.
2. Define the P3 input/diagnostic boundary for hardware and logical CPU identity and availability.
3. Integrate boot-CPU and unavailable/disabled-CPU expectations with the P3 lifecycle consumers.
4. Review the boundary against ADR layering and the ban on QEMU/board assumptions in Core.
5. Collect topology acceptance evidence for the declared QEMU CPU counts.
6. Record the contract, limits, and consumers for bring-up and P4.

## Acceptance and closure

P3-V01 requires stable mapping and boot-CPU identification across 1/2/4/8 CPU configurations, with unavailable CPUs excluded. It does not prove hotplug or hardware-platform correctness.

## Handoff

W02–W05 and W10 consume valid topology inputs; P4 consumes the stable pCPU-identity contract. CPU-start and platform details remain P2/architecture design work.
