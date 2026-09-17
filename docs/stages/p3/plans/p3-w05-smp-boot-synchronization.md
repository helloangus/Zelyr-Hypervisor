# P3-W05 — SMP boot synchronization

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Define and verify boot-time ordering from one-time global initialization through secondary readiness to SMP-ready.

## Scope

The bounded rendezvous/barrier semantics must prevent secondary use of incomplete global resources and premature dependent work, while requiring local initialization for every CPU.

## Out of scope

A general barrier library, scheduler blocking behavior, runtime hotplug coordination, and a concrete primitive implementation.

## Work sequence

1. Inspect bring-up, lifecycle, and per-CPU readiness contracts.
2. Define global-init, local-init, ready, and SMP-ready ordering and failure observability.
3. Integrate the readiness contract with shared synchronization and test consumers.
4. Review once-only and per-CPU authority against lifecycle rules.
5. Collect repeated rendezvous evidence including timing/failure observations.
6. Record readiness conditions and downstream limits.

## Acceptance and closure

P3-V05 requires once-only global initialization, per-CPU local initialization, and SMP-ready only after the declared readiness condition.

## Handoff

W06, W10, W12–W15 use the ordering contract. It does not authorize a future general synchronization API.
