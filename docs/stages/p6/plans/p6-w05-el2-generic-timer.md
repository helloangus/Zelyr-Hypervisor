# P6-W05 — EL2 generic timer

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish a per-pCPU EL2 monotonic-time and deadline-event foundation for P6 event delivery and later P7 scheduling.

## Scope

Cover timer capability intake, frequency/time conversion boundary, arm/cancel/rearm behavior, per-pCPU ownership, IRQ receipt, monotonicity, and diagnostics.

## Out of scope

Guest timer virtualization, scheduler ticks or time-slice policy, wall clock, NTP, migration time conversion, exact timer-register programming, or timing API design.

## Work sequence

1. Inspect W03 plus P1 timer-access and P3 per-pCPU/lifecycle constraints.
2. Produce an approved detailed design for Host monotonic-time and local deadline-event ownership, lifecycle, and failure boundary.
3. Integrate planned timer events with the physical IRQ lifecycle and per-pCPU diagnostics.
4. Define one-shot, repeated, cancellation, rearm, entry/exit monotonicity, and multi-pCPU acceptance cases.
5. Review that the mechanism permits later scheduling without deciding a scheduler policy.
6. Record the factual timer contract and hand it to W06, W10–W11, W13, and P7.

## Acceptance and closure

P6-V07 and P6-V08 require evidence that attributable one-shot and repeated Host events satisfy their declared conditions and observed monotonic time does not regress in the tested boundary. Passing does not prove scheduling, wall-clock correctness, or hardware-wide timing performance.

## Handoff

W06 receives the Host event basis; P7 receives only an evidenced timer mechanism and its limits, not a preemption policy.
