# P1-W12 — P1 documentation and P2 handoff

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W01–W11 and feeds P1 completion review and P2.

## Goal

Assemble the reviewable P1 contract and handoff set without confusing planned
scope, implementation records, verification evidence or completion claims.

## Scope

AArch64 Boot Contract, EL2 initialization contract, Host address-space
description, exception diagnostic contract, reference QEMU environment, known
limitations, P2 handoff, validation/evidence map and stage completion checklist.

## Out of scope

Stage completion evidence itself, implementation reports, P2 design, new ADR
decisions, Guest/SMP/GIC/platform/memory mechanisms and retroactive acceptance.

## Work sequence

1. Collect the accepted boundaries and open issues from W01–W11.
2. Produce the required P1 contract and limitation documents.
3. Map each task-book validation and exit criterion to its future evidence location.
4. Review links, package one-to-one coverage, dependency acyclicity and scope.
5. Define the completion-review questions and unresolved-decision treatment.
6. Hand off the stable assumptions and explicit non-goals to P2.

## Acceptance and closure

P1-V20 and P1-V21: all required contracts and limitations are internally
consistent, every package has one plan, every validation has an objective
condition, links resolve, dependencies are acyclic, and no document asserts
unperformed implementation or validation.

## Handoff

P2 may consume the documented stable boot-CPU EL2 environment, capability
knowledge, diagnostics and Host Stage-1 assumptions. P2 must still design and
validate DTB/platform discovery, physical memory and allocation; this package
does not freeze their APIs or module structure.
