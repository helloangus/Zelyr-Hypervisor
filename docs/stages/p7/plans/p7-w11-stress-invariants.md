# P7-W11 — Scheduler Stress and Invariants

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Specify evidence that exposes scheduler races, invariants, multi-VM/overcommit behavior, fairness, and placement faults.

## Scope

4pCPU/8vCPU/two-VM and 2× overcommit stress; pause/block/wakeup race timing; invariant/fairness checks; and affinity/pinning stress.

## Out of scope

Formal proof, 4× overcommit as a required gate, benchmarking, or final performance tuning.

## Work sequence

1. Assemble scenarios from W08–W10 and their observable conditions.
2. Define stress repetition, race windows, and objective failure signals.
3. Integrate invariant checks with trace and diagnostic evidence.
4. Review coverage against P7 risks and stage validation limits.
5. Record the stress-evidence plan and hand it to QEMU regression.

## Acceptance and closure

P7-V24–V27: required stress matrices objectively reveal corruption, starvation, lost wakeup, livelock, and placement violations when present.

## Handoff

W12 consumes the required stress scenarios; evidence remains under verification.
