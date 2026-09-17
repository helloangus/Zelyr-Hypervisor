# P3-W11 — SMP observability

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Make physical CPU state and cross-CPU SMP activity observable with governed CPU attribution.

## Scope

CPU discovered/start-requested/entered-EL2/online/failed, notification sent/received, TLB transport request/completion, synchronization contention, and boot-synchronization timing; state output includes hardware identity, logical identity, and boot/secondary role.

## Out of scope

Ad-hoc debug-output contracts, a telemetry implementation design, guest telemetry, performance guarantees, and claims based only on planned events.

## Work sequence

1. Inspect lifecycle, event, transport, audit, and P0 telemetry governance.
2. Define required observable outcomes, CPU attribution, and diagnostic compatibility constraints.
3. Integrate observability with stress, QEMU regression, P4 handoff, and evidence locations.
4. Review event naming and output semantics against governance and no-board-leakage rules.
5. Collect telemetry/log review evidence under the stated test environment.
6. Record available observations, limits, and consumers.

## Acceptance and closure

P3-V11 requires meaningful CPU attribution for lifecycle, notification, transport, contention, and boot-synchronization observations; it does not prove performance.

## Handoff

W12–W15 and P4 consume the observability contract and event evidence expectations.
