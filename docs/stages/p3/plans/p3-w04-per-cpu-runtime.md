# P3-W04 — Per-CPU runtime

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Give each online physical CPU independent execution and CPU-local state foundations.

## Scope

Independent stack, logical identity, runtime/exception/interrupt-local state, notification and TLB reception state, and telemetry basis; reserve capacity for later scheduler/current-vCPU needs without defining them.

## Out of scope

vCPU scheduling, context switching, guest state, global array shortcuts with implicit sharing, and detailed storage/API design.

## Work sequence

1. Inspect W02/W03 and the P1 exception baseline plus P2 allocation constraints.
2. Define the required local-state categories and the explicit boundary from global shared state.
3. Integrate local-state availability with boot synchronization, synchronization, notification, exceptions, and audit.
4. Review against the prohibition on implicit global current-CPU/context state.
5. Collect isolation and CPU-identity acceptance evidence.
6. Record P4-reserved storage needs without asserting a VM/vCPU implementation.

## Acceptance and closure

P3-V04 passes when online CPUs have distinct execution/local diagnostic state and no implicit global-current-CPU or global-context contract remains.

## Handoff

W05–W12 and P4 consume CPU-local foundations. Future consumers design their own local data and lifecycle.
