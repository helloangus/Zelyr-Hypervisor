# P1-W01 — Reference boot contract

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); consumes P0 build/QEMU entry and feeds W02, W03, W09, W10.

## Goal

Define one reviewable QEMU `virt` boot path with explicit EL2 entry assumptions
and bounded rejection behavior.

## Scope

Entry point, image/loading conditions, Non-secure EL2 requirement, boot CPU,
MMU/cache assumptions, boot parameters, DTB treatment, minimum memory and
canonical invocation/documentation.

## Out of scope

UEFI/bootloader framework, EL3/TF-A, Orange Pi boot, Guest boot protocol and
general DTB/platform discovery.

## Work sequence

1. Inspect P0's target, image and QEMU entry contracts.
2. Record supported entry state, inputs, ranges and explicit assumptions.
3. Define normal continuation and fail-fast outcomes for invalid entry state.
4. Reconcile the contract with ADR layering and P1 reserved scope.
5. Define acceptance evidence for normal and unsupported reference boots.
6. Hand off the stable entry contract to runtime and automation packages.

## Acceptance and closure

P1-V01 and P1-V02: the canonical path reaches validated Non-secure EL2, and a
disallowed environment has an explicit diagnostic and no normal continuation.
Review is acceptable for contract content; executable evidence belongs to the
later implementation/verification records.

## Handoff

W02 may rely on the declared entry state; W10 may use the canonical invocation.
DTB discovery, board support and Guest entry remain unimplemented.
