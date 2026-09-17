# P3-W08 — TLB shootdown transport

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Establish the cross-CPU request, acknowledgement, and completion transport that future TLB invalidation work can use.

## Scope

Single-target and mask/broadcast selection, acknowledgement/completion, invalid/offline exclusion, concurrent-request behavior, and timeout/failure diagnostics.

## Out of scope

Stage-2 address spaces, IPA/VMID selection, TLBI operation choice, guest TLB semantics, and any assertion that translation invalidation has been implemented.

## Work sequence

1. Inspect lifecycle, synchronization, and notification contracts.
2. Define transport-level target, request, acknowledgement, completion, and timeout expectations.
3. Integrate delivery and accounting with observability, stress, regression, and P4 handoff.
4. Review the transport boundary to ensure it does not predesign Stage-2.
5. Collect request/acknowledgement and failure-path acceptance evidence.
6. Record the P4-facing transport contract and explicit semantic gap.

## Acceptance and closure

P3-V08 passes when target/mask, acknowledgement, completion, timeout, invalid/offline exclusion, and concurrent-request behavior are diagnosable. It proves no Stage-2 TLBI semantics.

## Handoff

P4 consumes transport only and designs the address-space-specific operation. W11–W15 consume its observability and acceptance boundary.
