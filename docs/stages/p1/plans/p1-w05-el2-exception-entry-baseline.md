# P1-W05 — EL2 exception entry baseline

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W02 and W04 and feeds W06, W07 and W11.

## Goal

Establish diagnosable EL2 entry paths for all required exception categories and
a clear recoverable-versus-fatal boundary.

## Scope

Synchronous exception, IRQ, FIQ and SError vector coverage; origin/context
classification; bounded context capture; syndrome interpretation boundary;
unexpected and unhandled paths.

## Out of scope

GIC initialization or dispatch, virtual interrupts, timer delivery, SMP
notification, Guest exception handling and a complete IRQ subsystem.

## Work sequence

1. Establish vector coverage and the applicable origin/context categories.
2. Define minimum diagnostic context and classification outcomes.
3. Integrate valid-entry behavior with W04's architectural baseline.
4. Review recursive-entry and unhandled-vector failure boundaries.
5. Define intentional synchronous and unexpected-vector acceptance evidence.
6. Hand off the exception contract to console, crash and fault validation.

## Acceptance and closure

P1-V08 and P1-V09: every category has a valid EL2 entry path, and intentional
or unexpected synchronous faults expose syndrome/location before their defined
outcome without unbounded recursive failure.

## Handoff

W06/W07 may rely on an exception path that can emit context. IRQ dispatch and
GIC behavior remain explicitly unimplemented.
