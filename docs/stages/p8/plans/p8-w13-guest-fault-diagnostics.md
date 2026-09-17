# P8-W13 — Guest fault diagnostics

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Establish the diagnostic classification and context needed to debug Linux integration without global failure.

## Scope

Cover source P8.13: kernel panic, synchronous exception, Stage-2 fault, unsupported sysreg, invalid MMIO, PSCI, vGIC, timer, vCPU-state, and Hypervisor-invariant distinctions; require VM/vCPU/PC/PSTATE/syndrome/address/exit/trace context.

## Out of scope

Full crash-dump system, fault-handler APIs, final trace encoding, or treating a Guest fault as permission to panic EL2.

## Work sequence

1. Inspect W05–W10 and existing diagnostic boundaries.
2. Classify expected Guest-facing and Hypervisor-invariant failure classes.
3. Define the minimum actionable context and recent-trace requirement.
4. Relate diagnostics to console, VM containment, and automated regression observables.
5. Review unclassified failures as blocks or design investigations.

## Acceptance and closure

P8-V18 requires declared diagnostic coverage for listed fault classes without claiming complete crash analysis.

## Handoff

W16, W18, and W20 receive diagnostic requirements and limitation statements.
