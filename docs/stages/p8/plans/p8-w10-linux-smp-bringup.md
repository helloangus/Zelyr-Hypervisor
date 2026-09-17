# P8-W10 — Linux SMP bring-up

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan the Linux 2- and 4-vCPU integration path over PSCI, DTB, GIC, timer, and scheduler foundations.

## Scope

Cover source P8.10: CPU enumeration, PSCI secondary start, per-CPU timer/IRQ, SGI, scheduler, idle/WFI, synchronization, and declared SMP stability workloads.

## Out of scope

Scheduler redesign, CPU topology values, Host SMP bring-up, real-board scale claims, or final stress implementation.

## Work sequence

1. Inspect W04–W09 and stated upstream lifecycle facts.
2. Define required 2- and 4-vCPU Linux observable outcomes.
3. Relate secondary start to PSCI, DTB, interrupt, timer, and scheduler contracts.
4. Define busy, thread, sleep/wakeup, affinity, interrupt-heavy, and scheduler-heavy scenarios.
5. Review race/lost-event/TLB diagnostic dependencies.

## Acceptance and closure

P8-V14 and P8-V15 require declared SMP boot and stability conditions. Passing does not assert an SMP implementation exists.

## Handoff

W11–W20 receive the specified Linux SMP evidence boundary.
