# P8-W11 — Scheduler and Linux integration

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan the P7 scheduler validation needed to run a real Linux Guest in 1:1 and declared M:N conditions.

## Scope

Cover source P8.11: preemption, timer restore, interrupt delivery, WFI, wakeup, runtime accounting, and Linux progress without dependence on static pinning.

## Out of scope

Scheduler algorithm/queue design, fairness policy, time slice values, placement API, or repairing P7.

## Work sequence

1. Inspect W10 and the evidenced P7 scheduler contract.
2. Define 1:1 and declared shared-CPU/M:N Linux scenarios.
3. Relate timer, vIRQ, WFI, and lifecycle observations to P7 handoff limits.
4. Define accounting and progress conditions without a final fairness claim.
5. Review failures as P7/P8 dependency issues rather than a scheduler redesign authorization.

## Acceptance and closure

P8-V16 requires declared Linux progress and preserved scheduler semantics under both scenarios. It does not choose scheduler policy.

## Handoff

W16–W18 receive the integration evidence requirements.
