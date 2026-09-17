# P5-W06 — Dispatch, permission, and fault containment

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Integrate P5's ABI, Guest-data, object-reference, and authority boundaries into
one minimal service path that either performs an allowed request or contains it
as a structured Guest-facing result.

## Scope

Cover P5-T02, T11, T13–T14 and the source minimal closed loop: ABI/caller/
handle/type/rights/argument validation before operation; allowed result;
invalid, stale, denied, bad-state, unsupported, resource, and Guest-fault
outcomes; and the boundary between these outcomes and Hypervisor invariants.

## Out of scope

Implementing the final dispatcher/module/API design, broad VM management,
identity policy, scheduler, timer/IRQ mechanisms, all future error policy, or
reclassifying a genuine invariant violation as a Guest error.

## Work sequence

1. Inspect W03–W05 approved contracts and P4 exception/fault evidence.
2. Produce an approved detailed design for the integrated validation order,
   state authority, side-effect/rollback boundary, failure classification, and
   observability responsibilities.
3. Integrate one minimal permitted operation with all required checks and
   controlled results, while retaining each underlying boundary's ownership.
4. Define valid, malformed, unsupported, invalid-address, invalid-reference,
   wrong-type, no-right, revoked, and bad-state acceptance cases.
5. Review that Guest-caused input cannot panic the Hypervisor and that internal
   invariant handling remains separately diagnosable.
6. Record implementation/evidence status and hand stable scenario outcomes to
   W07–W10.

## Acceptance and closure

P5-V09 and V10 require real evidence that the minimal permitted HVC validates
the complete required chain, while malicious/malformed requests receive a
defined contained result. Passing does not prove all future hypercalls or a
complete VM-management interface.

## Handoff

W07 receives observable expected outcomes; W08 receives integrated host-side
properties; W09 receives event/result categories; W10 receives factual
implementation and containment records.
