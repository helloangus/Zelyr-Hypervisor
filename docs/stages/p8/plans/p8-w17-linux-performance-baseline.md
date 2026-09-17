# P8-W17 — Linux performance baseline

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan the first reproducible Linux observability baseline without making an optimization or KPI commitment.

## Scope

Cover source P8.17: boot time, exits/reasons, Stage-2 faults, timer/vIRQ, scheduler switches, Host CPU use, and basic interrupt latency with environment and limitation records.

## Out of scope

Performance optimization, targets, competitive comparisons, final metric encoding, or interpreting QEMU measurements as hardware performance.

## Work sequence

1. Inspect W10–W12, W16, and telemetry governance.
2. Define the metrics, environment, and limitation record required for comparison.
3. Relate measurements to automated fixture and regression scenarios.
4. Separate observation from a pass/fail performance objective.
5. Review host/platform dependence and diagnostic overhead disclosure.

## Acceptance and closure

P8-V23 requires a reproducible baseline record with explicit limits and no KPI claim.

## Handoff

W20 and later stages receive comparison inputs, not optimization conclusions.
