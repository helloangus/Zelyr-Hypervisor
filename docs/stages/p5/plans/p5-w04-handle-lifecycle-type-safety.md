# P5-W04 — Handle lifecycle and type safety

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Establish opaque P5 object-reference semantics that identify a valid object
without exposing its Host address or allowing stale or wrong-type access.

## Scope

Cover P5-T04–T06 and the Handle portion of T23: opaque identity, existence,
destruction, stale-reference protection after reuse, repeated destroy, owner
destruction, forged values, generation/type mismatch, and the explicit rule
that a handle does not grant authority.

## Out of scope

Freezing handle bits/layout, object-table representation, Rust type/API design,
garbage collection, all future object classes, capability rights, final
concurrency mechanics, or implementation evidence.

## Work sequence

1. Inspect W01 and ADR-013 constraints together with P4 lifecycle facts.
2. Produce an approved detailed design for object-reference lifetime authority,
   type checking, invalidation, destruction/reuse, and failure boundaries.
3. Define planned lifecycle and wrong-type cases covering creation, lookup,
   destruction, stale reuse, repeated destruction, and owner disappearance.
4. Integrate the reference boundary with W05's authority checks while keeping
   object identity distinct from permission.
5. Review Host-pointer non-disclosure, Guest-untrusted input, multi-pCPU
   readiness, telemetry needs, and future object-class extension.
6. Record implementation/evidence status and hand the factual reference
   semantics to W05–W08 and later P6 consumers.

## Acceptance and closure

P5-V04 and V05 require lifecycle and type-safety evidence for forged, stale,
reused, invalid-generation, and wrong-type references. Passing does not prove
the representation, a general object system, or authority enforcement.

## Handoff

W05 consumes an identity boundary for capabilities; W06/W07 consume defined
invalid-reference outcomes; W08 consumes lifecycle stress properties.
