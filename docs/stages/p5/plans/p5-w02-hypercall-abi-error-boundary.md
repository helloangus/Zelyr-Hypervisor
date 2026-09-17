# P5-W02 — Hypercall ABI and error boundary

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Define the bounded planning and documentation path for a versioned P5 HVC
service boundary whose compatible, unsupported, malformed, Guest-caused, and
invariant-failure outcomes remain distinguishable.

## Scope

Cover P5-T01, T13, T14, and the ABI portion of T22: discovery, version and
compatibility semantics, inputs/results, reserved-space treatment,
unknown/unsupported/malformed behavior, structured error categories, and the
separation of Guest-facing failure from Hypervisor invariants.

## Out of scope

Freezing a calling convention, HVC immediate, register/call-number/error
encoding, wire representation, public compatibility promise, module/API
design, management ABI, IPC protocol, or implementation result.

## Work sequence

1. Inspect W01 inputs, ADR versioning/capability constraints, and the ABI and
   security documentation governance.
2. Produce an approved detailed design for the minimum HVC boundary and its
   compatibility, request/result, failure, and observability responsibilities.
3. Define how the implementation will distinguish valid discovery, supported
   calls, unknown calls, unsupported features, version mismatch, malformed
   requests, Guest faults, resource conditions, and invariant failures.
4. Plan the factual ABI artifact and compatibility analysis required under
   `docs/abi/` once implementation decisions are approved and evidenced.
5. Review that this boundary neither exposes Host information nor becomes a
   management/machine ABI or an identity-based authority bypass.
6. Record implementation/evidence status and hand the planned dispatch contract
   to W05–W07 and the documentation route to W09–W10.

## Acceptance and closure

P5-V02, V06, and V15 require real review and implementation evidence that the
supported ABI behavior, controlled errors, and factual ABI/security documents
are compatible and distinguishable. Passing does not prove a public ABI or
freeze values merely planned here.

## Handoff

W05 receives an error/authority-routing boundary; W06 receives an integration
contract; W07 receives expected Guest-visible outcomes. Concrete ABI choices
remain the approved detailed design and implemented artifact.
