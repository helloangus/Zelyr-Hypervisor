# P3-W14 — P4 SMP handoff

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Provide P4 an explicit, bounded Host SMP foundation contract without predesigning Stage-2 or VM mechanisms.

## Scope

The handoff states what P4 can rely on: current logical pCPU identity, CPU-local state foundation, shared-state protection, cross-CPU notification/completion, future TLB transport, CPU-attributed faults, audit/telemetry rules, regression evidence, and unresolved limits.

## Out of scope

Stage-2 API or TLBI semantics, VM/vCPU lifecycle, guest execution, scheduling, virtual interrupts, or a P4 detailed design.

## Work sequence

1. Inspect W01–W13 plans and implementation/verification records that actually exist.
2. Consolidate only evidenced P3 guarantees, known limitations, and blocking issues.
3. Integrate the handoff with the P4 planning reading order and stage-document boundaries.
4. Review the contract against host/guest separation and ADR constraints.
5. Perform a P4-consumer review for sufficiency without claiming P4 implementation.
6. Record the handoff package and items P4 must design independently.

## Acceptance and closure

P3-V14 requires a P4 planner to locate and rely on the stated Host SMP foundations without redesigning P3; it requires no Stage-2 implementation claim.

## Handoff

P4 detailed design and implementation consume this contract. W15 uses it for P3 closure traceability.
