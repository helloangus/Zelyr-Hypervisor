# P3-W12 — SMP stress and failure tests

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Define repeatable evidence for concurrency, failure, and exceptional paths beyond a one-time successful SMP boot.

## Scope

Concurrent allocate/free, notification storms, atomic/shared-counter stress, repeated rendezvous, secondary timeout, invalid/offline targets, and concurrent logging. Evidence must make accounting, hangs, corruption, and diagnostics assessable.

## Out of scope

Performance certification, exhaustive formal verification, real-hardware proof, CI-script implementation, and future guest stress workloads.

## Work sequence

1. Inspect synchronization, lifecycle, event, transport, exception, audit, and observability contracts.
2. Define bounded test categories, fault stimuli, success conditions, accounting, and known test limits.
3. Integrate test evidence requirements with QEMU matrix and stage-closure consumers.
4. Review that tests exercise Host SMP only and do not assert guest/Stage-2 behavior.
5. Run or collect applicable repeatable stress/failure evidence when implementation exists.
6. Record results in verification records, distinguishing planned, run, failed, and blocked states.

## Acceptance and closure

P3-V12 requires repeated tests with explainable accounting and no corruption, permanent hang, or undiagnosed failure within declared limits. A plan alone is not evidence.

## Handoff

W13–W15 and P4 acceptance review consume the stress contract and actual verification records when available.
