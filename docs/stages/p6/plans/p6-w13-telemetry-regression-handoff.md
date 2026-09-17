# P6-W13 — telemetry, regression, and handoff

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Collect P6's factual observability, regression, documentation, performance-baseline, and consumer-handoff record without converting planned behavior into a completion claim.

## Scope

Cover structured Host IRQ/timer/vIRQ/maintenance telemetry, latency measurement method, QEMU integration/repeat evidence, P0–P5 regression, required future documentation records, and P7/P8 consumer review.

## Out of scope

Implementing unplanned mechanisms, asserting stage closure without evidence, defining a machine ABI, scheduler policy, raw log dumps as a substitute for records, or real-hardware support claims.

## Work sequence

1. Inspect all P6 package records, the P0 telemetry/QEMU governance, and available implementation/verification evidence.
2. Define or collect the required structured telemetry, correlation, latency-baseline, integration, repeat, and upstream-regression evidence boundaries.
3. Reconcile implemented facts, limitations, unsafe delta, capability status, and validation results into the prescribed documentation layers.
4. Review QEMU/environment limits, evidence completeness, unresolved investigations, and every P6 exit criterion.
5. Provide the P7/P8 consumer review that distinguishes proven mechanisms from reserved or excluded work.
6. Record the factual handoff; leave stage completion unclaimed until all exit evidence exists.

## Acceptance and closure

P6-V23 through P6-V28 require determinate upstream regression, telemetry, latency, QEMU integration/repeat, documentation, and consumer-review evidence. Passing in QEMU does not prove real hardware, complete machine ABI, or scheduler correctness.

## Handoff

P7 and P8 receive linked factual contracts, evidence locations, limitations, and unresolved items. No downstream stage may infer unproved P6 work or a frozen API.
