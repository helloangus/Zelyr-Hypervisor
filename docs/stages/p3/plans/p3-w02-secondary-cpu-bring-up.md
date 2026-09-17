# P3-W02 — Secondary CPU bring-up

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Make each expected secondary physical CPU reach a defined and diagnosable EL2 bring-up result.

## Scope

The boot CPU requests startup using P2-provided platform input; a secondary reaches EL2, establishes its initial local environment, confirms identity, and reports success, timeout, or failure with CPU and phase attribution.

## Out of scope

Runtime CPU hotplug/restart, guest PSCI, guest SMP, a detailed PSCI/GIC mechanism, and policy for continuing after a degraded platform result.

## Work sequence

1. Confirm topology, P1 EL2 entry, and P2 CPU-start prerequisites.
2. Define the bounded secondary-start success, timeout, and failure outcomes.
3. Integrate secondary entry with lifecycle, local-runtime, and boot-synchronization consumers.
4. Review failure isolation and Core/Arch/platform boundaries against the ADR.
5. Collect repeatable bring-up evidence for expected CPU counts and an induced failure path.
6. Record diagnostics and hand off the resulting startup contract.

## Acceptance and closure

P3-V02 passes when every expected secondary either reaches its defined result or reports a CPU/phase-specific failure without making the SMP state opaque. It does not prove runtime hotplug.

## Handoff

W03–W05, W09, and W13 consume the bring-up result. The detailed start mechanism remains an approved design decision.
