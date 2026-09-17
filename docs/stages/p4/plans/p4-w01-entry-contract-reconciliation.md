# P4-W01 — Entry contract reconciliation

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Establish a reviewable P4 starting boundary from the P0–P3 handoffs before any
Stage-2 or Guest implementation is accepted.

## Scope

Inspect the upstream engineering, EL2, platform-memory, and SMP/TLB contracts;
record the P4 assumptions they actually support; align planned validation and
unsafe-review inputs; and identify blocked or conflicting prerequisites.

## Out of scope

Repairing P0–P3, defining P4 module/API design, implementing Stage-2 or Guest
behavior, or claiming that an upstream stage has completed.

## Work sequence

1. Inspect the ADR, P4 task book, plan index, and available P0–P3 handoff,
   implementation, and verification records.
2. Reconcile the EL2 exception, Host memory ownership, allocation, SMP, and
   TLB-transport assumptions required by the later P4 packages.
3. Record the authority, security, layering, telemetry, and test-environment
   constraints that bound P4 work.
4. Map each P4 prerequisite to the evidence or unresolved dependency that a
   detailed design must consume.
5. Review missing, incompatible, or ambiguous upstream inputs against the ADR
   and classify them without silently redesigning an upstream stage.
6. Record the entry review and hand the accepted contract set to W02–W09.

## Acceptance and closure

P4-V01 requires a linked review showing every P0–P3 prerequisite, its evidence
location or absence, its P4 consumer, and any block. Passing means no P4
package assumes an undocumented upstream behavior. This is a documentation
review, not evidence that any runtime mechanism exists.

## Handoff

W02–W09 may rely on an explicit list of compatible inputs and recorded gaps.
Stage-2, Guest memory, vCPU, and runtime behavior remain unimplemented.
