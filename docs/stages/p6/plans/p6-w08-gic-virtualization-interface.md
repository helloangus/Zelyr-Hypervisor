# P6-W08 — GIC virtualization interface

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish the bounded hardware-presentation bridge from the P6 vIRQ lifecycle to supported GICv3 virtualization-interface/List-Register state.

## Scope

Cover virtualization-interface readiness, available presentation capacity, vCPU-context preservation, basic priority carriage, and safe pressure behavior for pending work.

## Out of scope

Maintenance-event policy, Guest GIC Distributor/Redistributor MMIO, Linux machine ABI, exact List-Register allocation algorithm, or detailed save/restore implementation.

## Work sequence

1. Inspect W01 capability conclusions, W07 lifecycle semantics, and the relevant P4 vCPU context boundary.
2. Produce an approved detailed design for hardware-presentation ownership, availability, state preservation, and pressure/failure outcomes.
3. Integrate pending vIRQ selection with the declared virtualization capability without turning QEMU behavior into the Core contract.
4. Define single-presentation, over-capacity, vCPU transition, basic-priority, and cross-vCPU-isolation acceptance cases.
5. Review architecture/platform separation, unsafe boundaries, telemetry, and P8 non-ABI constraints.
6. Record factual status and hand capacity/presentation facts to W09–W13 and P8.

## Acceptance and closure

P6-V11, P6-V16, and P6-V18 require evidence that supported presentation occurs, over-capacity pending work is preserved, and vCPU state stays isolated. Passing does not provide a Linux-visible vGIC model.

## Handoff

W09 receives the completed-presentation/reusable-capacity boundary; P8 receives only lower-level evidenced capability facts.
