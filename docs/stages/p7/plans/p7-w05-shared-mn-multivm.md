# P7-W05 — Shared M:N and Multi-VM Scheduling

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Establish correct shared-CPU M:N and multi-VM progress with basic no-starvation fairness.

## Scope

Required 1/2, 2/4, 4/8 matrices, two-VM competition, runnable progress, and first-scheduler fairness conditions.

## Out of scope

Proportional fairness, weights, quotas, policy switching, advanced balancing, or performance optimization.

## Work sequence

1. Inspect placement and preemption contracts.
2. Define the required M:N and multi-VM scenario boundary.
3. State progress/fairness conditions and prohibited duplicate/non-runnable execution.
4. Review constraints against ADR-016 and ADR-057.
5. Plan matrix and fairness evidence, then hand off to SMP and Validation Guest work.

## Acceptance and closure

P7-V10–V12: declared scenarios progress without prohibited execution and every equal-class continuously runnable vCPU has sustained nonzero execution.

## Handoff

W08, W10, and W11 consume shared-scheduler semantics; no final policy is selected.
