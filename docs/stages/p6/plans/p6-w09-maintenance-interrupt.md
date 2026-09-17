# P6-W09 — maintenance interrupt

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish maintenance-event processing that releases completed virtual-interrupt presentation state and permits pending work to make bounded progress.

## Scope

Cover recognized maintenance conditions, completion correlation, reusable presentation capacity, diagnostic handling of unexpected conditions, and no-loss/no-duplicate progression under capacity pressure.

## Out of scope

General interrupt scheduling policy, Guest vGIC MMIO, List-Register allocation internals, final fairness/coalescing policy, or a production IRQ-pressure guarantee.

## Work sequence

1. Inspect W08 presentation and capacity contract plus W03 physical IRQ lifecycle rules.
2. Produce an approved detailed design for maintenance-event authority, completion correlation, error handling, and pending-work continuation.
3. Integrate the planned outcome with vIRQ lifecycle state, telemetry, and safe Host IRQ completion.
4. Define over-capacity, Guest-completion, reusable-slot, unexpected-maintenance, and repeat-progress acceptance scenarios.
5. Review state-leak, duplicate-completion, cross-vCPU, and storm-boundary constraints.
6. Record the maintenance contract and hand it to W10–W13 and P8.

## Acceptance and closure

P6-V16 and P6-V17 require evidence that capacity pressure preserves events and maintenance processing advances remaining work without loss, duplicate completion, or active-state leak. Passing does not prove fairness or Linux vGIC behavior.

## Handoff

W10/W11 consume completed lifecycle behavior; W13 and P8 receive evidence locations and explicit limits.
