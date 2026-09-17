# P7-W03 — Placement and Scheduling Configuration

**Status:** Planned work package; implementation not claimed
**Parent:** [P7 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P7 plan index](README.md)

## Goal

Establish static pinned equivalence and explicit affinity, pinning, dedicated/shared, and invalid-control semantics.

## Scope

Placement constraints and observability requirements, including non-silent rejection of invalid scheduling controls.

## Out of scope

Configuration ABI, Control Domain interface, affinity data representation, load balancing, or RT policy.

## Work sequence

1. Inspect W02 lifecycle and P3 eligible-pCPU contracts.
2. Define required placement behavior for pinned, affinity and dedicated/shared operation.
3. Specify invalid-control outcomes and telemetry-visible configuration state.
4. Review conformance with ADR-016 and capability-based authority boundaries.
5. Plan placement matrix evidence and hand off the constraints.

## Acceptance and closure

P7-V05–V07: declared baseline, placement matrix, and invalid-control tests objectively distinguish compliance from fallback.

## Handoff

W05, W07, W08, and W11 receive placement constraints; dynamic policy remains later work.
