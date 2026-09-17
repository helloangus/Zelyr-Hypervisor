# P8-W01 — Entry contract reconciliation

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Establish an evidence-backed P8 boundary from P0–P7 before machine or Linux work.

## Scope

Inspect EL2, memory, SMP, Guest, capability, GIC/timer, scheduler, diagnostic, and validation handoffs; record available facts, gaps, authority, security/layering constraints, and ABI/machine routing.

## Out of scope

Repairing upstream stages, selecting a machine ABI, implementing Linux support, or treating a planned predecessor as evidence.

## Work sequence

1. Inspect ADR, task book, plans, and P0–P7 implementation and verification records.
2. Reconcile each prerequisite with an evidence location or recorded absence.
3. Identify consumers and limits of each predecessor fact.
4. Record Guest-untrusted, capability, platform-independence, and telemetry constraints.
5. Classify conflicts and missing machine/security inputs without resolving them locally.

## Acceptance and closure

P8-V01 passes when every P0–P7 dependency is linked or explicitly blocked and no later plan presumes undocumented behavior. This is a review, not runtime evidence.

## Handoff

Later packages receive only reconciled factual inputs and recorded limits.
