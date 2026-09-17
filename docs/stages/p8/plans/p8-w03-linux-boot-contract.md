# P8-W03 — Linux boot contract

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Define a reviewable Linux Image, DTB, optional initramfs, and bootargs contract.

## Scope

Cover source P8.2: boot-vCPU state, entry, DTB transfer, boot-artifact regions/lifetime, Guest EL/MMU expectations, secondary state, shutdown transition, and reproducible fixture.

## Out of scope

Image-loader implementation, artifact addresses, UEFI, filesystem loading, Linux configuration selection, or a formal machine ABI.

## Work sequence

1. Inspect W01/W02 and evidenced Guest EL1/Stage-2 lifecycle inputs.
2. Define boot-input and lifecycle facts later detailed design must establish.
3. Relate artifact ownership and reserved regions to Guest isolation.
4. Specify fixture reproducibility and shutdown evidence needs.
5. Review against Guest EL1 and no-host-leakage constraints.

## Acceptance and closure

P8-V04 passes when required boot facts are testable and no implementation choice or address is silently frozen.

## Handoff

W04, W09–W10, W15, and W16 receive the documented boot-input boundary.
