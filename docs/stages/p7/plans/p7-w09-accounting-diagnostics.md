# P7-W09 — Accounting, Trace, and Diagnostics

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Make scheduler behavior measurable and diagnosable at vCPU, pCPU, and VM levels.

## Scope

Required runtime/switch/preemption/block/wakeup/CPU-change counters; pCPU and VM aggregation; trace fields; switch reasons; and failure context.

## Out of scope

Metric backend, buffer encoding, crash-dump framework, management protocol, or performance conclusions.

## Work sequence

1. Inspect lifecycle, switch, wakeup, and pause observability inputs.
2. Define required accounting and aggregation observations.
3. Define trace/reason and scheduler-failure diagnostic content.
4. Review compatibility with P0 telemetry governance.
5. Plan evidence and hand off observability to stress, performance, and closeout.

## Acceptance and closure

P7-V19–V21: counters remain coherent, events identify required context, and failures are diagnosable without claiming full crash dumping.

## Handoff

W11, W13, and W14 consume the observability contract.
