# P7-W02 — Scheduler Admission and Lifecycle

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Define a scheduler-controlled normal execution admission boundary and an explicit, testable vCPU lifecycle.

## Scope

Normal Guest entry/exit scheduling, Offline/Runnable/Running/Blocked/Paused/Stopped/Faulted semantics, transition validity, and core single-running invariants.

## Out of scope

Runqueue design, state representation, locking, APIs, or preemption mechanics.

## Work sequence

1. Establish the W01 entry and authority assumptions.
2. Produce the lifecycle and scheduler-admission contract with invalid-transition behavior.
3. Integrate the contract with predecessor vCPU and failure terminology without redefining it.
4. Review invariants and plan lifecycle/property evidence.
5. Record the accepted boundary and hand it to placement, preemption, wakeup, pause, observability, and stress packages.

## Acceptance and closure

P7-V02–V04: review/test evidence shows normal execution is scheduler-controlled, transition validity is objective, and listed invariants are checkable. Evidence belongs in verification.

## Handoff

W03–W11 can rely on lifecycle authority and invariants; no implementation design is supplied.
