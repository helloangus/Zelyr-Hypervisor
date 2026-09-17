# P3-W03 — Physical CPU lifecycle

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Establish explicit, observable authority for physical-CPU lifecycle and online eligibility.

## Scope

The P3 lifecycle must distinguish absent, present, starting, initializing, online, and failed CPUs; may reserve offline/stopping/suspended states; and must prevent duplicate online admission or failed CPU use.

## Out of scope

Runtime offline/online transitions, scheduling policy, guest/vCPU lifecycle, and implementation-level state representation.

## Work sequence

1. Inspect W01 topology and W02 start-result contracts.
2. Define permitted P3 lifecycle outcomes, ownership, diagnostics, and online-set admission rules.
3. Integrate lifecycle consumers with per-CPU state, notifications, audit, telemetry, and regression work.
4. Review lifecycle boundaries against host-versus-guest separation and platform portability rules.
5. Collect lifecycle/online-set acceptance evidence, including failed-start handling.
6. Record the state contract and reserve later transitions explicitly.

## Acceptance and closure

P3-V03 requires that no CPU becomes usable before local initialization, no CPU is twice online, and failed CPUs remain outside the online set.

## Handoff

W04–W15 consume lifecycle authority. P4 receives physical-CPU availability semantics only; it must not infer a vCPU state machine.
