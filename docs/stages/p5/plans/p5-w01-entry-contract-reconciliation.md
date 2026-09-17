# P5-W01 — Entry contract reconciliation

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Establish a reviewable P5 starting boundary from evidenced P0–P4 handoffs
before Hypercall, object, or authority work is accepted.

## Scope

Inspect the EL2 exception, Stage-2 Guest-data, Guest lifecycle, Validation
Guest, P3 SMP, P0 testing/unsafe, ABI, and security inputs; identify the
authority order, documentation route, and missing prerequisites for P5.

## Out of scope

Repairing an upstream stage, designing HVC/ABI values or object APIs, freezing
an ABI, implementing P5 mechanisms, or claiming that an upstream stage closed.

## Work sequence

1. Inspect the ADR, P5 task book, plan index, and available P0–P4 package,
   implementation, and verification records.
2. Reconcile the P4 Guest-EL1, HVC/exception, Stage-2, Guest-memory, fault,
   diagnostics, Validation Guest, and regression facts required by P5.
3. Identify the P2 ownership/address and P3 synchronization/pCPU constraints
   that later Guest-data and authority packages must preserve.
4. Route future factual ABI, security, detailed-design, implementation, and
   verification artifacts to their governed locations.
5. Review missing, ambiguous, or contradictory inputs against the ADR and
   classify them without silently redesigning an upstream contract.
6. Record the entry review and hand the compatible inputs and blocks to W02–W10.

## Acceptance and closure

P5-V01 requires a linked review that names every prerequisite, its evidence
location or absence, its consumer, and its ABI/security routing. Passing means
no P5 package assumes undocumented upstream behavior. This is a documentation
review, not runtime evidence.

## Handoff

W02–W10 receive an explicit set of compatible P0–P4 facts and recorded gaps.
The HVC ABI, Guest-data path, handles, and capabilities remain unimplemented.
