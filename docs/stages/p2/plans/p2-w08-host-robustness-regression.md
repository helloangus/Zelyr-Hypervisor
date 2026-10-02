# P2-W08 — Host robustness and negative regression

Chinese readers can use the [Chinese edition](p2-w08-host-robustness-regression.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Define reproducible host-side regression evidence for P2's untrusted-input,
memory-map, allocation, and deterministic-discovery safety properties.

## Scope

P2-J01–J05: malformed DTB cases, map conflicts and range overflow, allocation
exhaustion/reuse/protected-page safety, allocation stress, and deterministic
discovery of identical inputs.

## Out of scope

QEMU integration execution, fuzzing architecture, performance benchmarking,
implementation test commands, or acceptance evidence fabricated during planning.

## Work sequence

1. Gather W01–W05 safety outcomes and W07 fixture/readiness expectations as
   the assertions under test.
2. Define the malformed-DTB cases required to demonstrate bounded rejection and
   diagnostics, including structural, encoding, truncation, and overflow input.
3. Define map-conflict and allocator-lifecycle cases, including all prohibited
   protected-page returns and reuse after exhaustion/release.
4. Establish deterministic repeated-input and pseudo-random stress evidence
   expectations with accounting/invariant checks.
5. Review that host-side results prove logic properties but do not substitute
   for QEMU or real-hardware semantics.
6. Hand off the reproducible regression suite requirements to W09 and W10.

## Acceptance and closure

P2-V10 requires real reproducible evidence for every P2-J group: bounded bad
input handling, conflict/overflow treatment, exhaustion/reuse, protected-page
safety, stress consistency, and deterministic repeated discovery. No evidence
is claimed by this plan.

## Handoff

W09 may combine the established host-side safety baseline with reference-platform
integration. W10 may record evidence locations and untested limitations.

## P2-ACR-02 planning amendment

W11 supplies additional assertions for checked Host coverage, attributes/aliases, owner/borrow lifetime, protected/outside-domain rejection, partial-map rollback and unmap/invalidation before reuse. Include these host-model cases in W08 under P2-V10/P2-V14; target evidence remains separate.

Owner-selected ADR-062 follow-up (2026-10-02): consume the
[W12 common ownership/view foundation](p2-w12-minimal-memory-objects.md) and
P2-V15 evidence at the corresponding lifetime/acceptance boundary. W08 owns
Host negative/lifetime scenarios, W09 owns actual integrated Host access, W10
owns condition-specific handoff, and W11 owns the translation adapter. W12
Host modeling precedes W11 and does not require its hardware; W04-MAP remains
independent. Formal ADR integration and approved detailed designs are still
required before affected coding; no object or runtime delivery is inferred.
