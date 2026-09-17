# P8-W19 — Validation Guest dual-track regression

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Keep the Rust Validation Guest as the mechanism-level suite while Linux supplies OS-integration coverage.

## Scope

Cover source P8.19: retained HVC, Stage-2 fault, timer, SGI, vIRQ, MMIO, SMP, and scheduler-interaction scenarios and their relationship to Linux regression.

## Out of scope

Replacing the Validation Guest with Linux, extending its implementation, redefining P4–P7 semantics, or treating Linux success as mechanism proof.

## Work sequence

1. Inspect W01, W09–W10, W15–W16, and existing Validation Guest evidence.
2. Inventory the retained mechanism scenarios and their expected observables.
3. Map Linux integration checks separately from precise Validation Guest checks.
4. Define regression coexistence and fixture-maintenance expectations.
5. Review any lost mechanism coverage as a dependency block.

## Acceptance and closure

P8-V25 requires a dual-track regression plan showing Linux is not the sole mechanism test. It is not a test result.

## Handoff

W20 and P9+ receive the maintained regression split and evidence route.
