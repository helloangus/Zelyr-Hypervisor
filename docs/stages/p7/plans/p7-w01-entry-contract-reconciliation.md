# P7-W01 — Entry Contract Reconciliation

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Establish one reviewed P0–P6 input set and P7 scheduler boundary before implementation.

## Scope

Predecessor evidence, authority, scope, blocked-prerequisite handling, and the normal scheduler-controlled Guest-entry boundary.

## Out of scope

Repairing predecessor stages, selecting scheduler mechanics, or making completion claims.

## Work sequence

1. Inspect the required P0–P6 handoffs and their implementation/verification records.
2. Reconcile the timer, event, vCPU, SMP, capability, fault, and diagnostics inputs P7 consumes.
3. Record compatible assumptions, missing evidence, and any Architecture Change Request or ADR Required issue.
4. Review the resulting boundary against ADR constraints and hand it to all P7 packages.

## Acceptance and closure

P7-V01: a reviewer can locate each input and its P7 use; an absent or contradictory contract remains an explicit block. This is a review, not an executable test.

## Handoff

W02–W14 may rely on the recorded P7 input boundary only; implementation remains unperformed.
