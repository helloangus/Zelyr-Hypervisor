# P8-W06 — PSCI virtualization

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Establish the standard firmware lifecycle boundary Linux needs for virtual CPUs.

## Scope

Cover source P8.5: PSCI version/features, CPU_ON/OFF, affinity/status needs, system-off and secondary-vCPU startup without a private HVC dependency.

## Out of scope

PSCI call encoding, function subset choice before approval, lifecycle implementation, management policy, or Linux driver modifications.

## Work sequence

1. Inspect W02/W05 and P3/P4/P7 lifecycle inputs.
2. Define required standard PSCI semantic categories and decision gates.
3. Relate CPU start/stop outcomes to vCPU lifecycle and diagnostics.
4. Define normal and malformed-parameter acceptance scenarios.
5. Review that authority and Host CPU facts do not leak into Guest semantics.

## Acceptance and closure

P8-V09 requires Linux secondary start and required shutdown paths to use declared standard PSCI behavior; it does not prove an implementation.

## Handoff

W10, W14, W16, and W18 receive the reviewed PSCI boundary.
