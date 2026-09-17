# P7-W12 — Automated QEMU Scheduler Regression

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Provide an automated QEMU regression boundary for declared scheduler matrices.

## Scope

Determinate 1/1, 1/2, 2/4, 4/4, and 4/8 CPU/vCPU scenarios plus pause/resume, wakeup, affinity, and pinning regressions.

## Out of scope

Claiming QEMU proves hardware semantics, selecting CI mechanics, or reporting tests as passed before evidence exists.

## Work sequence

1. Inspect P0 runner governance and W11 scenarios.
2. Define automation inputs, determinate outcomes, timeouts, and artifact/evidence locations.
3. Integrate scenario diagnostics with prior telemetry requirements.
4. Review matrix coverage and the QEMU-versus-architecture limit.
5. Record acceptance evidence requirements and hand off closeout inputs.

## Acceptance and closure

P7-V28: every declared scenario returns a determinate result with diagnosable failure artifacts. Real runs belong in verification.

## Handoff

W14 receives automation/evidence locations; hardware validation remains later work.
