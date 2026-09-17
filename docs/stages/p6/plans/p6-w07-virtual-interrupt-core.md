# P6-W07 — virtual interrupt core

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish a controller-independent vIRQ lifecycle that safely preserves authorized, specified-vCPU events until presentation and completion.

## Scope

Cover vIRQ request authorization, target isolation, pending/deferred/presented/active/completed semantic coverage, repeated arrival policy, unavailable-vCPU outcomes, and diagnostic/telemetry boundaries.

## Out of scope

GIC List Register mechanics, Linux-visible vGIC MMIO, scheduler wakeups, device IRQ passthrough, final coalescing algorithm, or object/API implementation details.

## Work sequence

1. Inspect W03, P4 vCPU execution facts, P5 capability/error boundaries, and the P6 scope classifications.
2. Produce an approved detailed design for vIRQ lifecycle authority, target validation, deferred delivery, repetition, and failure outcomes.
3. Integrate the planned lifecycle with Host event producers and future hardware-presentation consumers without equating physical and virtual IRQs.
4. Define single, multiple-pending, repeated, unavailable-vCPU, and cross-vCPU isolation acceptance scenarios.
5. Review Guest-untrusted input handling, rights checks, shared-state constraints, and telemetry requirements.
6. Record the vIRQ contract and hand it to W08–W13 and P7/P8.

## Acceptance and closure

P6-V11, P6-V12, P6-V14, and P6-V18 require evidence of authorized eventual delivery, preserved multiple/repeated pending state, and target isolation. Passing does not prove List-Register presentation, Guest MMIO, or wakeup policy.

## Handoff

W08 receives pending-event semantics; W12 receives validation/failure boundaries; P7/P8 receive no frozen vGIC or scheduler API.
