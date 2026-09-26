# P6-W06 — Guest generic timer virtualization

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish vCPU-owned Guest generic-timer behavior that preserves the Guest/Host time boundary through supported exits and deferred expiry.

## Scope

Cover independent vCPU timer state, Guest timer programming/masking/expiry, entry/exit preservation, eventual delivery when absent, optional evidenced pause/resume behavior, and isolation from Host timer state.

## Out of scope

Scheduler wakeup/selection, Linux time ABI, migration/snapshot time conversion, wall clock, advanced scaling, timer state representation, or system-register implementation details.

## Work sequence

1. Inspect W05, P4 Guest entry/exit/Stage-2 evidence, and P5 HVC/error/capability boundaries.
2. Produce an approved detailed design for vCPU timer ownership, lifecycle, expiry/deferred-delivery behavior, and Guest-caused error handling.
3. Integrate planned timer behavior with Host events, vIRQ consumers, and supported Guest exits without binding state to a pCPU.
4. Define single-vCPU, exit/HVC, deferred-expiry, and conditional multi-vCPU/pause-resume acceptance scenarios.
5. Review isolation, untrusted Guest controls, telemetry, and P7/P8 extension constraints.
6. Record factual status and hand the timer contract to W10–W13 and later stages.

## Acceptance and closure

P6-V09, P6-V10, and P6-V19 require Guest timer, exit-preservation, and multi-vCPU isolation evidence when the prerequisite exists; absence of that prerequisite is a documented block. Passing does not establish scheduler semantics or Linux timer ABI.

## Handoff

W10/W11 consume the defined timer behavior; P7/P8 receive only proven vCPU-owned semantics and known limitations.
