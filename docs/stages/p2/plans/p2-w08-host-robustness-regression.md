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
