# P5-W09 — Telemetry, safe logging, and regression

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Make P5 service results observable without default Host-information disclosure,
and integrate P4 and P5 cases into a determinate long-term regression boundary.

## Scope

Cover P5-T20, T21, and T25: per-VM call/result categories, success/failure,
denied/invalid-reference/invalid-address accounting, safe debug context, no
default Host VA/PA/object-pointer/raw-sensitive-buffer disclosure, and P4/P5
positive, negative, revoke, isolation, overflow, and fuzz-smoke regression.

## Out of scope

Defining a final telemetry API/backend, a production audit service, secret
logging policy, CI-policy change, real-hardware validation, or claiming that
the QEMU runner proves architectural or hardware correctness.

## Work sequence

1. Inspect W07 scenario markers, W08 robustness limits, P0 diagnostics/QEMU
   governance, and the task-book observability requirements.
2. Produce an approved detailed design for P5 event/result attribution,
   Guest-visible versus developer-debug information, redaction, and regression
   result collection.
3. Integrate required counters and safe diagnostics with the existing runner
   and preserve P4 EL1/Stage-2 regression inputs.
4. Define determinate valid/invalid HVC, reference, authority, revoke,
   Guest-data, fuzz-smoke, and inherited-P4 regression expectations.
5. Review timeout, incomplete evidence, leaked-sensitive-information, and
   unsupported-environment outcomes as diagnosable non-success results.
6. Record implementation/evidence status and hand telemetry, regression, and
   limitation facts to W10 and P6+ regression consumers.

## Acceptance and closure

P5-V15 and V16 require evidence of the required result categories, safe default
diagnostics, factual ABI/security documentation route, and determinate P4/P5
regression outcomes in the declared environment. Passing does not prove a
production telemetry system or real-hardware correctness.

## Handoff

W10 receives the regression/evidence index and observability limits. Later
stages reuse the declared P5 regression boundary while preserving its security
and environment constraints.
