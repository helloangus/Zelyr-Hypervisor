# P3-W13 — QEMU SMP regression matrix

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Establish a repeatable QEMU `virt` Host SMP regression matrix for 1, 2, 4, and 8 CPUs and repeated cold boots.

## Scope

For each CPU count define boot, online-set, stress, cross-CPU event, and allocator-stress expectations; include repeated cold boots to expose timing/rendezvous/allocator races and retain the P0 QEMU/CI contract.

## Out of scope

Concrete runner implementation, hardware validation, QEMU behavior as an architectural specification, and performance certification.

## Work sequence

1. Inspect P0 QEMU/CI contracts and W02/W05–W12 acceptance boundaries.
2. Define matrix rows, repetition policy, diagnostic capture, and result classification.
3. Integrate matrix evidence with stress/failure cases and P4 entry review.
4. Review QEMU limits and platform-independence constraints.
5. Run or collect matrix evidence when implementation exists, recording each result honestly.
6. Hand off the regression contract, evidence location, and hardware-validation gap.

## Acceptance and closure

P3-V13 passes with declared boot/online/event/stress/allocator criteria for 1/2/4/8 CPUs over repeated cold boots. It does not prove Orange Pi or other hardware correctness.

## Handoff

W14–W15 and P4 consume the matrix and available evidence; P15 retains real-hardware SMP responsibility.
