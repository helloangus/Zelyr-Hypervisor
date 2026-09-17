# P4-W06 — Guest fault isolation and diagnostics

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Make P4 Guest exits, Stage-2 translation/permission faults, and basic isolation
properties distinguishable, diagnosable, and recoverable at the correct fault
boundary.

## Scope

Cover P4-E02, P4-F01–F04, and P4-G01–G04: exit categorization, diagnostic
context, Guest-versus-Hypervisor fault distinction, address inspection, mapped
and unmapped bounds, selected Hypervisor-owned-range protection, execute and
write enforcement.

## Out of scope

Final `ExitReason` API, management error ABI, general debug monitor, full
security certification, device/DMA isolation, IOMMU, Guest interrupt delivery,
or a policy for all future VM lifecycle failures.

## Work sequence

1. Inspect W02's Stage-2 observations, W04's entry/return boundary, and W05's
   controlled Guest scenarios.
2. Produce an approved detailed design for the classification, diagnostic, and
   recovery responsibilities at the Guest/Hypervisor boundary.
3. Integrate required Guest/vCPU, PC, state, IPA, access-type, mapping, and
   reason information with the existing diagnostic/telemetry baseline.
4. Define positive and negative isolation scenarios for unmapped, permission,
   Guest-boundary, and selected Hypervisor-owned ranges.
5. Review Guest-fault containment against ADR invariant handling and ensure
   unknown synchronous conditions remain diagnosable.
6. Record implementation/evidence status and hand fault/isolation facts to W07
   and W08.

## Acceptance and closure

P4-V06–P4-V09 require evidence that defined exits are distinguishable, faults
carry required context, Hypervisor-owned access is blocked, permissions work,
and controlled Guest faults do not become unexplained Hypervisor panics. This
does not prove IOMMU/DMA isolation or all future Guest error policy.

## Handoff

W07 consumes categorized events for observability and repeatability; W08
consumes stable expected diagnostics. P5 may rely on the evidence-backed fault
boundary while retaining responsibility for its new ABI.
