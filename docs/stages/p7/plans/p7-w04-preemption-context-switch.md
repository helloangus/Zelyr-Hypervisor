# P7-W04 — Preemption and Context-Switch Correctness

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Make CPU-bound Guest preemption and vCPU-switch isolation objectively verifiable.

## Scope

P6 timer-driven scheduler deadlines, time-slice behavior, return to EL2, scheduler reconsideration, and required context/address-space/timer/vIRQ/event preservation.

## Out of scope

Timer programming method, slice value, tick model, assembly layout, register-save strategy, or scheduling algorithm.

## Work sequence

1. Inspect W02 lifecycle and P6 timer/event contracts.
2. Define the observable preemption and re-scheduling outcome.
3. Define required switch-isolation observations across repeated vCPU rotation.
4. Review failure boundaries and observability needs with predecessor contracts.
5. Plan preemption and isolation evidence and hand off the behavior.

## Acceptance and closure

P7-V08–V09: CPU-bound preemption is distinguishable and declared per-vCPU state remains isolated across switches.

## Handoff

W05, W07, W09, and W10 can rely on this behavior; implementation mechanics remain detailed-design work.
