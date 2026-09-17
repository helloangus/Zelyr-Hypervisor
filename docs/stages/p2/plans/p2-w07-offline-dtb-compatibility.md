# P2-W07 — Offline DTB compatibility checking

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Define an offline DTB readiness check that exposes P2 platform-description
gaps before EL2 boot and tests common discovery semantics across fixtures.

## Scope

P2-I01–I05: offline input, checks for P2-required facts, objective readiness
reporting, QEMU `virt` fixture, and Orange Pi 3B/RK3566 fixture semantics.

## Out of scope

Full SoC-driver analysis, runtime Hypervisor startup, Orange Pi EL2 support,
board-specific Core logic, or a checker CLI/parser implementation.

## Work sequence

1. Consume W01's input-safety boundary and W02's normalized discovery
   semantics as the only basis for offline checking.
2. Establish readiness-report outcomes for CPUs, RAM, reservations, GIC,
   timer, PSCI, chosen/console, and safely ignored unrelated devices.
3. Define QEMU and RK3566 fixture expectations that exercise shared semantics
   instead of QEMU-specific layout assumptions.
4. Review report classifications so PASS/WARN/unsupported outcomes do not
   imply a runtime board-support claim.
5. Specify fixture and report evidence suitable for host-side regression.
6. Hand off fixtures/readiness expectations to W08, W10, and platform planners.

## Acceptance and closure

P2-V09 requires evidence that both fixtures receive objective P2 readiness
reports and that unrelated unsupported devices are diagnosed safely. P2-V10
uses this outcome in negative regression. Passing does not prove Orange Pi EL2
runtime support.

## Handoff

W08 may use the documented fixtures and expected semantics. Later platform work
receives a compatibility signal, not an implementation or BSP contract.
