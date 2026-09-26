# P6-W03 — physical interrupt lifecycle

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish a safe, diagnosable Host physical-IRQ lifecycle from receipt through classification, dispatch outcome, and completion.

## Scope

Cover SGI, PPI, SPI, repeated and simultaneous events, acknowledgement/completion responsibility, and safe spurious, unknown, or consumerless outcomes under P1 IRQ entry and P3 synchronization constraints.

## Out of scope

SGI routing policy, device-driver framework, Guest injection, production DoS controls, detailed handler APIs, or scheduler wakeup semantics.

## Work sequence

1. Inspect W02 readiness plus the P1 exception-entry and P3 shared-state/interrupt-context contracts.
2. Produce an approved detailed design for the bounded Host IRQ lifecycle, state authority, completion responsibility, and error boundaries.
3. Integrate planned classification and dispatch outcomes with safe per-pCPU diagnostics and supported source categories.
4. Define acceptance for ordinary, repeated, simultaneous, spurious, unknown, and unconsumed events.
5. Review Guest isolation, no-invalid-index/handler behavior, storm containment expectations, and telemetry requirements.
6. Record the lifecycle contract and hand it to routing, timers, vIRQ, robustness, and validation consumers.

## Acceptance and closure

P6-V20 and P6-V22 require evidence that spurious/unknown inputs remain diagnosable and declared high-rate smoke cases preserve the documented state invariants. Passing does not establish a device framework or production DoS resistance.

## Handoff

W04, W05, W07, W11–W13 receive only the factual Host receive/classify/complete boundary and its known limits.
