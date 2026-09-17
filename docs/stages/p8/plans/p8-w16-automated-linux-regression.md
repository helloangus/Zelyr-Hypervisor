# P8-W16 — Automated Linux regression

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan the automated QEMU and repeated-boot regression matrix for the P8 Linux outcome.

## Scope

Cover source P8.16: hypervisor/kernel/GIC/timer/CPU/userspace/marker/shutdown observations; Validation Guest, Linux 1/2/4 vCPU, RAM classes, intentional fault, and repeat execution.

## Out of scope

CI implementation, QEMU-specific machine semantics, fixed retry count, performance KPI, or real-hardware validation.

## Work sequence

1. Inspect W09–W15 and P0 test governance.
2. Define automated observable markers and expected shutdown outcomes.
3. Specify the declared scenario matrix and fixture inputs.
4. Define repeated-boot checks for stale VMID/TLB/vCPU/IRQ state and races.
5. Review failures as reproducible evidence, blocked prerequisites, or regressions.

## Acceptance and closure

P8-V21 and P8-V22 require determinate automated-matrix and repeated-boot conditions. Planned checks are not evidence.

## Handoff

W17–W20 and P9 receive regression entry points and stated limits.
