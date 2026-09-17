# P1-W08 — Host Stage-1 address space

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W04–W07 and feeds W09–W12 and P2.

## Goal

Define and validate a controlled EL2 Stage-1 transition covering the resources
needed by the P1 runtime, without making identity mapping a permanent contract.

## Scope

Executable code, read-only data, writable/zero data, boot stack, vectors,
boot-time data and early-console MMIO; permission, execution, memory-type and
cacheability classes; transition and post-MMU behavior.

## Out of scope

Dynamic virtual-memory manager, map/unmap service, physical allocator, Guest
Stage-2, huge pages, NUMA, memory ownership and future address-space policy.

## Work sequence

1. Inventory runtime regions and prerequisites from W02–W07.
2. Define logical mapping classes and prohibited ambiguous attributes.
3. Define controlled transition, continuity and bounded failure behavior.
4. Integrate vectors, console and crash diagnostics across the transition.
5. Review no-RWX, no-hidden-identity-map and layering constraints.
6. Define pre/post-MMU acceptance evidence and hand off the environment.

## Acceptance and closure

P1-V13 and P1-V14: required regions have explicit attributes, and execution,
console, vectors and fatal diagnostics continue after MMU enablement without an
identity-map semantic promise.

## Handoff

W09 and P2 receive a stable Host Stage-1 runtime and documented temporary
mapping assumptions. Dynamic mapping and physical memory remain P2 scope.
