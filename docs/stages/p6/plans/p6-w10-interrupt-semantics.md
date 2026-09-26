# P6-W10 — interrupt semantics

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Define and exercise the minimum P6 Guest-visible semantics for masking, priority, pending accumulation, repeated events, and concurrent timer/vIRQ delivery.

## Scope

Cover the required observable semantics and their boundaries with the vCPU timer and virtualization-interface contracts, including the rule that Guest masking does not imply completion.

## Out of scope

Complete GIC priority/preemption specification, Linux vGIC compatibility, scheduler run-state policy, detailed priority encoding, or final event ordering algorithm.

## Work sequence

1. Inspect W06 timer behavior, W09 maintenance behavior, W07 lifecycle semantics, and applicable Validation Guest inputs.
2. Produce an approved detailed design for the bounded masking, priority, pending, repeated-event, and timer-plus-vIRQ semantics.
3. Integrate the planned behavior with deferred delivery and Guest entry/exit without assigning scheduler responsibility to P6.
4. Define mask/pending/unmask, different-priority, multiple-pending, repeated, and concurrent-event acceptance scenarios.
5. Review Guest-untrusted controls, non-loss requirements, telemetry, and P8 machine-ABI exclusion.
6. Record the semantic contract and hand it to W11–W13 and P7/P8.

## Acceptance and closure

P6-V13 through P6-V15 require evidence that masking preserves pending work, documented priority information survives, and timer plus ordinary vIRQ work progresses. Passing does not prove full GIC specification coverage.

## Handoff

W11 receives defined scenarios; P7/P8 receive a factual semantics record, not a frozen interrupt-controller ABI.
