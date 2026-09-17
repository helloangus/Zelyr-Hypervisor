# P1-W09 — Initialization sequencing

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W01–W08 and feeds W10–W12 and P2.

## Goal

Make the early lifecycle explicit from boot entry to stable EL2 runtime, with
visible prerequisites and failure results.

## Scope

Boot Entry → Minimal Runtime → Early Diagnostics → Architecture Validation →
Exception Environment → EL2 Baseline → Host Stage-1 → Stable Runtime; stage
markers, prerequisite ownership and failure boundaries.

## Out of scope

Scheduler or runtime state machines, SMP lifecycle, VM lifecycle, Guest entry,
dynamic resource reclamation and later-stage service startup.

## Work sequence

1. Collect package contracts and order their required conditions.
2. Define lifecycle states, legal transitions and failure outcomes.
3. Integrate markers and diagnostic availability at every transition.
4. Review hidden dependencies, partial-init behavior and reserved boundaries.
5. Define lifecycle acceptance review and repeat-boot evidence expectations.
6. Hand off the ordered boot contract to regression and stage documentation.

## Acceptance and closure

P1-V15: prerequisites, order and failure results are reviewable, and no hidden
initialization dependency is accepted as a P1 assumption.

## Handoff

W10/W11 can target stable lifecycle states and P2 can start from the declared
stable environment. This is not a general service lifecycle design.
