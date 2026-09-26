# P2-W02 — Platform discovery and normalization

Chinese readers can use the [Chinese edition](p2-w02-platform-discovery-normalization.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Produce one normalized, capability-driven representation of P2-required host
platform facts from validated DTB input.

## Scope

P2-B01–B08 and P2-C01–C03: CPU inventory and boot-CPU relation; RAM and
reservations; GICv3, timer, PSCI and console/chosen discovery; capability
summary; and the distinction between absent, unsupported, and usable facts.

## Out of scope

GIC initialization, timer virtualization, PSCI execution, AP startup, UART
drivers, board-specific Core behavior, and concrete `PlatformInfo` APIs.

## Work sequence

1. Consume W01's validated input contract and enumerate the P2 facts that each
   discovery outcome must establish.
2. Establish the normalized semantic outcome for CPU, memory, reservations,
   GIC, timer, PSCI, and chosen/console facts.
3. Define capability-report expectations that preserve missing, unsupported,
   absent, and supported states rather than collapsing diagnostics.
4. Review the result against ADR platform layering and capability-driven
   behavior for QEMU and RK3566 fixture inputs.
5. Specify discovery and normalization evidence, including deterministic
   repeated-input behavior, without prescribing parser internals.
6. Hand off stable semantic facts to map, inspection, checker, QEMU, P3, and
   P4 consumers.

## Acceptance and closure

P2-V03 and P2-V04 require evidence that declared required facts are collected,
normalization preserves diagnostic states, and Core has no board-name decision.
P2-V09 and P2-V11 later exercise this outcome on fixtures and QEMU.

## Handoff

W03 may consume RAM/reservation facts; W06/W07 may report/check the normalized
result. P3/P4 may consume only the documented semantic contract, not a
particular type or implementation.
