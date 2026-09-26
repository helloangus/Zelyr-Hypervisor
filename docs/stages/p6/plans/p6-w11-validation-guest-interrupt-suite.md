# P6-W11 — Validation Guest interrupt suite

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Maintain a Guest-side verification asset that independently observes the P6 timer and virtual-interrupt boundary rather than relying only on Hypervisor logs.

## Scope

Cover EL1 exception-vector handling; VG-TIMER-01 through VG-TIMER-04; VG-IRQ-01 through VG-IRQ-06; Host-SGI support; and conditional multi-vCPU timer/IRQ isolation when the upstream capability is evidenced.

## Out of scope

Linux Guest tests, machine ABI/DTB work, production guest drivers, substituting Hypervisor logs for Guest observation, or implementation of underlying GIC/timer mechanisms.

## Work sequence

1. Inspect W04, W06, W10, P4 Validation Guest facts, and the declared P6 validation matrix.
2. Produce an approved detailed design for maintained Guest-side scenario observability, result collection, and negative/blocked outcomes.
3. Integrate each declared scenario with the applicable Host SGI, timer, vIRQ, masking, and maintenance contracts.
4. Define expected evidence for timer, HVC/controlled-exit, vIRQ, mask/unmask, repeated, concurrent, and conditional multi-vCPU cases.
5. Review that the suite tests Guest-visible behavior without defining P8 Linux machine contracts.
6. Record scenario status/evidence references and hand the suite to W13 and downstream regression users.

## Acceptance and closure

P6-V09 through P6-V19 require Guest-observable evidence for the relevant scenarios; multi-vCPU rows require the actual prerequisite or a documented block. Passing proves only the declared Validation Guest behavior in the stated environment.

## Handoff

W13 receives scenario evidence and limitations; P7/P8 may reuse the maintained asset while extending it under their own scope.
