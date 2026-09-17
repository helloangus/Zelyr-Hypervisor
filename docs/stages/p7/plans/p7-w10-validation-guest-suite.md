# P7-W10 — Validation Guest Scheduler Suite

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Maintain a Validation Guest workload suite that exercises P7 scheduler behavior.

## Scope

Single-vCPU regression and multi-vCPU CPU-bound, periodic WFI, virtual timer, SGI/IRQ, shared-memory, and HVC-heavy scenarios.

## Out of scope

Linux boot, guest machine ABI, Guest module/function design, or permanent guest SMP policy.

## Work sequence

1. Inspect P4/P6 Validation Guest assets and W04–W07 behavior.
2. Define P7 workload scenarios and expected observable outcomes.
3. Integrate scenario needs with scheduler telemetry and failure boundaries.
4. Review that guest workloads do not redefine platform contracts.
5. Plan regression evidence and hand off to stress automation.

## Acceptance and closure

P7-V22–V23: existing single-vCPU behavior remains stable and declared scheduler workloads exercise their intended conditions.

## Handoff

W11 receives maintained scenarios; P8 does not receive a machine-ABI commitment.
