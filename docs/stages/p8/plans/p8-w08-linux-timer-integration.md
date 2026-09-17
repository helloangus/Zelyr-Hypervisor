# P8-W08 — Linux timer integration

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Define the Linux time and virtual-timer compatibility boundary for single- and multi-vCPU Guests.

## Scope

Cover source P8.7: counter access, timer IRQ, monotonic time, tick/tickless operation, sleep/timeout, preemption, target-vCPU delivery, and WFI wakeup.

## Out of scope

Timer programming mechanics, counter frequency values, scheduling algorithm, hardware timing guarantees, or real-hardware conclusions.

## Work sequence

1. Inspect W01/W02 and P6/P7 timer and scheduler facts.
2. Identify time semantics Linux and the machine contract must expose.
3. Define normal, high-frequency, sleep, preemption, multi-vCPU, and WFI scenarios.
4. Relate timer delivery to vCPU state and P7 wakeup semantics.
5. Review monotonicity and Host-independent constraints without choosing values.

## Acceptance and closure

P8-V11 requires a declared timer matrix proving stated time and target-vCPU semantics; it does not prove performance.

## Handoff

W09–W10 and W16–W18 receive the timer integration boundary.
