# P8-W15 — Reproducible Linux fixture

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan a versioned or fully reproducible Linux test asset for P8 regression.

## Scope

Cover source P8.15: Linux source/version, configuration, build route, initramfs, bootargs, machine configuration, and minimal shell/CPU/memory/IRQ/timer/process/stress tools.

## Out of scope

Selecting a distribution, Host filesystem loading, package-manager instructions, performance tooling, or asserting an artifact exists.

## Work sequence

1. Inspect W03/W04 and P0 reproducibility governance.
2. Define fixture provenance and reproducibility metadata.
3. Define initramfs functional content by validation need.
4. Relate artifact inputs to boot and Guest-only DTB contracts.
5. Review versioning, licensing, and evidence-record needs without choosing implementation tooling.

## Acceptance and closure

P8-V20 requires a pinned or reproducibly generated fixture definition; it is not a built-image claim.

## Handoff

W09–W10 and W16–W19 receive stable fixture expectations.
