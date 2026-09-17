# P1-W03 — AArch64 capability inventory

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W02 and feeds W04, W09, W11 and P2.

## Goal

Produce a capability report that distinguishes required, optional and future
AArch64 virtualization facts and controls continuation accordingly.

## Scope

Current EL, CPU identity/affinity, architecture version, PA/VA and translation
limits, Stage-2 capability, granules, timer and relevant virtualization
extensions, plus absence and failure classification.

## Out of scope

Platform discovery, GIC initialization, Stage-2 implementation, CPU topology
bring-up, VM capability policy and board-specific behavior.

## Work sequence

1. Inventory the facts required by P1 and explicitly reserve later facts.
2. Define required-versus-optional validation and reporting categories.
3. Integrate the report with boot diagnostics and fail-fast policy.
4. Review that consumers query capabilities rather than platform names.
5. Define normal, missing-required and missing-optional acceptance evidence.
6. Hand off the capability contract to baseline, lifecycle and P2 planning.

## Acceptance and closure

P1-V05 and P1-V06: required facts are reported and missing required capability
blocks normal startup; optional absence remains distinguishable and does not
become an unrelated panic.

## Handoff

W04 can establish only the state justified by the report. P2 receives capability
knowledge, not a platform-discovery or allocator implementation.
