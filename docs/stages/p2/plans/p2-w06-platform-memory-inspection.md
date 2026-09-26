# P2-W06 — Platform and memory inspection

Chinese readers can use the [Chinese edition](p2-w06-platform-memory-inspection.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Make P2 platform and memory facts observable through one inspection result
derived from the normalized, actively used state.

## Scope

P2-H01–H04: platform summary, normalized boot-map dump, allocator statistics,
and consistency between reported and active platform information.

## Out of scope

An inspection-command implementation, separate hard-coded reporting data,
driver initialization, external management ABI, or telemetry API design.

## Work sequence

1. Confirm W02–W05 supply the normalized platform, map, allocation, and
   dynamic-allocation facts that inspection must represent.
2. Establish the required summary content for architecture/EL, boot CPU/topology,
   RAM/protected/allocatable regions, GIC/timer/PSCI/console, and capabilities.
3. Define map-dump and allocator-statistics outcomes using the same authoritative
   state as allocation and discovery consumers.
4. Review the package against the ADR observability constraint without choosing
   a command format, logger API, or telemetry implementation.
5. Specify inspection evidence for host and QEMU validation, including the
   absence of a divergent hard-coded view.
6. Hand off observable P2 facts to QEMU regression, P3/P4 review, and W10.

## Acceptance and closure

P2-V08 requires review/evidence that all required inspection categories derive
from active normalized state. P2-V11 later supplies QEMU integration evidence;
planning this package provides neither runtime output nor completion evidence.

## Handoff

W09 and W10 may rely on the required inspection contract. P3/P4 reviewers may
use it to assess input readiness without treating inspection as a control API.
