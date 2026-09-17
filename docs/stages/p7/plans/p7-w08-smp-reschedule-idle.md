# P7-W08 — SMP Reschedule and Idle Behavior

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Establish correct scheduler behavior across pCPUs, remote reschedule, and idle-to-work transitions.

## Scope

Concurrent distinct-vCPU execution, pCPU state isolation, cross-CPU wakeup/pause notification effects, placement compliance, and non-busy idle behavior.

## Out of scope

P3 notification implementation, CPU hotplug, power management, or advanced balancing.

## Work sequence

1. Inspect W03 and W05–W07 plus P3 notification contracts.
2. Define pCPU concurrency and remote scheduling outcomes.
3. Define idle entry/exit requirements and failure diagnostics.
4. Review placement and lifecycle preservation under concurrent activity.
5. Plan SMP and idle evidence and hand it to stress work.

## Acceptance and closure

P7-V17–V18: concurrent pCPU scheduling isolates state; remote work/control events cause appropriate reconsideration; idle avoids busy looping.

## Handoff

W11 consumes the proven SMP behavior; no CPU-management policy is implemented here.
