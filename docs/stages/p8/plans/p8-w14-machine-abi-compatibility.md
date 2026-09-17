# P8-W14 — Machine ABI compatibility

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan the compatibility-test route that prevents silent drift of an approved machine version.

## Scope

Cover source P8.14: Guest map, device location, interrupt assignment, DT compatible, CPU topology semantics, PSCI, timer, console, and machine identity; distinguish compatible internal change from version/ADR change.

## Out of scope

Selecting v1 values, test implementation mechanics, migration compatibility, snapshot format, or silently changing an approved contract.

## Work sequence

1. Inspect W02/W04/W06–W09 and all approved machine-contract facts.
2. Enumerate Guest-visible facts that must enter compatibility comparison.
3. Define version and incompatibility escalation conditions.
4. Relate fixtures and DTB assertions to the approved machine contract.
5. Review that QEMU observations cannot become ABI facts.

## Acceptance and closure

P8-V19 requires a policy and planned drift-detection test for the same approved configuration. It cannot pass before v1 facts are approved.

## Handoff

W16, W20, and P9+ receive the compatibility route and no unapproved values.
