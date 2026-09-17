# P7-W06 — Blocking and Event Wakeup

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Make WFI/WFE-style blocking and eligible event wakeup scheduler-visible without lost wakeup or duplicate running.

## Scope

Block release of pCPU capacity; timer, vIRQ, Notification and internal event wakeup; before/during/after-block races; invalid-state wakeup exclusion.

## Out of scope

P6 event delivery mechanics, wait-queue structures, wakeup APIs, and device-model design.

## Work sequence

1. Inspect W02 lifecycle and P6 event contracts.
2. Define blocked eligibility and event-to-runnable behavior.
3. Define race acceptance and invalid-state protections.
4. Review cross-CPU and pause interaction requirements.
5. Plan block/wakeup evidence and hand off to dependent packages.

## Acceptance and closure

P7-V13–V14: blocking does not busy-loop and applicable events eventually wake only eligible vCPUs without lost/duplicate/invalid wakeup.

## Handoff

W08–W11 may rely on the behavioral contract; event implementation remains outside this plan.
