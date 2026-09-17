# P7-W07 — Pause, Stop, and Fault Containment

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Ensure pause/resume and non-runnable stop/fault outcomes cooperate safely with scheduling.

## Scope

Running/runnable/blocked pause, VM pause completion, resume eligibility and pending events, stopped/faulted exclusion, remote-running-vCPU requests, and guest-fault containment.

## Out of scope

Snapshot/migration protocol, VM fault policy, management API, or scheduler implementation method.

## Work sequence

1. Inspect lifecycle, placement, and preemption inputs.
2. Define pause completion and resume preservation conditions.
3. Define stop/fault exclusion and other-VM containment boundary.
4. Review remote execution and event interaction requirements.
5. Plan acceptance evidence and hand off the contract.

## Acceptance and closure

P7-V15–V16: declared pause completion excludes Guest execution, resume preserves constraints, and stopped/faulted vCPUs do not corrupt other scheduling.

## Handoff

W08–W11 consume these semantics; snapshot and fault policy remain later stages.
