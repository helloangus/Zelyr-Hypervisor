# P1-W04 — EL2 architectural-state baseline

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.2.md)
Prerequisites and consumers: [P1 plan index](README.md); requires W03 and feeds W05, W08, W09.

## Goal

Make the required EL2 architectural state explicit and owned by Hypervisor,
removing reliance on firmware or QEMU residue.

## Scope

Execution state, exception routing, trap policy, FP/SIMD access, debug and
performance traps, timer access, EL1/EL0 preparation and translation-control
baseline needed by later stages.

## Out of scope

Guest virtualization policy, Stage-2, GIC/timer virtualization, EL1 entry,
SMP state and complete future system-register design.

## Work sequence

1. Map required baseline categories to W03 capability facts.
2. Define the known-state boundary and continuation prerequisites.
3. Define how unsupported or unavailable baseline elements fail.
4. Review for firmware-state dependence and future-stage overcommitment.
5. Specify cold-boot consistency evidence.
6. Hand off the baseline contract to exception and MMU packages.

## Acceptance and closure

P1-V07: required routing, traps, FP/SIMD, debug/performance, timer,
EL1/EL0-preparation and translation controls are explicitly established and
logically consistent across clean reference boots.

## Handoff

W05 may install exception handling against a known baseline and W08 may prepare
Host Stage-1 against declared controls. Guest policy remains reserved.
