# P3-W07 — Cross-CPU notification

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Provide a minimal, safe physical-CPU event primitive for later coordination.

## Scope

Targeted CPU-to-CPU notification with minimal type information, defined self-notification, concurrent sender behavior, and defined invalid/offline-target outcomes.

## Out of scope

General message queues, RPC, guest interrupts, vIRQ delivery, scheduler policy, and detailed interrupt-controller mechanics.

## Work sequence

1. Inspect physical lifecycle, per-CPU reception state, and synchronization rules.
2. Define notification delivery, accounting, targeting, and error/failure outcomes.
3. Integrate the event boundary with TLB transport, telemetry, stress, and regression consumers.
4. Review host-only scope and platform-capability boundaries.
5. Collect targeted, self, concurrent, invalid, and offline-target evidence.
6. Record the reusable event contract and explicit non-RPC limit.

## Acceptance and closure

P3-V07 requires defined and recoverable target behavior with explainable arrival/type accounting. It does not prove a general communication facility.

## Handoff

W08, W11–W15, and P4 consume the event primitive; each future feature defines its own higher-level protocol.
