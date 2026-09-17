# P3-W10 — SMP safety audit

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Audit P0–P2 infrastructure and P3 foundations so every reviewed mutable state has an explicit SMP-safety classification.

## Scope

Review logging/console, physical and heap/object allocation, platform information, global registries, diagnostic state, and other mutable infrastructure. Classify each item immutable-after-boot, CPU-local, atomic, lock-protected, or boot-only; identify remediation and unresolved blockers.

## Out of scope

Claiming all code is race-free without evidence, redesigning predecessor contracts without authority, or expanding into later VM/Stage-2/scheduler audits.

## Work sequence

1. Inspect W01–W09 and available P0–P2 implementation/verification records.
2. Inventory reviewed shared and local state with its owner, access contexts, and classification.
3. Integrate findings with synchronization, exceptions, telemetry, test, and closure work.
4. Review unresolved items against P0 unsafe governance and ADR architecture boundaries.
5. Collect audit evidence and confirm each unclassified state blocks closure.
6. Record remediation/handoff boundaries without treating an audit plan as completion evidence.

## Acceptance and closure

P3-V10 passes only when every reviewed mutable state has a classification and unresolved items are visible closure blockers.

## Handoff

W11–W15 and P4 consume the audit record. P4 must audit new VM/Stage-2 state independently.
