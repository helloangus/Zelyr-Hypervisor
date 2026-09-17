# P1-W02 — Minimal Rust EL2 runtime

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W01 and feeds W03–W12.

## Goal

Establish a minimal Rust `no_std` EL2 runtime that reaches a controlled stable
state without depending on accidental firmware register contents.

## Scope

Execution stack, Rust entry environment, static data readiness, boot-context
retention, panic route, build/version identity and stable idle behavior.

## Out of scope

General heap/allocator, VM/vCPU, SMP, Guest execution, scheduler and platform
frameworks.

## Work sequence

1. Confirm W01's entry assumptions are sufficient for runtime startup.
2. Establish the runtime's required initialization conditions and ownership.
3. Define the transition to stable idle and bounded failure behavior.
4. Integrate build identity and the P0 panic/diagnostic baseline.
5. Review for hidden firmware-state and future-stage dependencies.
6. Define runtime acceptance evidence and hand off the stable execution context.

## Acceptance and closure

P1-V03 and P1-V04: runtime state, context, panic path and identity are ordered
and visible, and repeated normal boots reach stable EL2 without implicit initial
register assumptions.

## Handoff

W03 may collect architecture facts from a live Rust runtime; later packages may
use stable idle as the normal-stage consumer state. Dynamic memory and Guest
runtime remain out of scope.
