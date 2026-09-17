# P8-W02 — Machine-contract governance

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Create the decision route needed to establish a host-independent rusthv-arm-virt-v1 contract without prematurely defining it.

## Scope

Cover source P8.1: identity; CPU, address-category, interrupt, timer, firmware/DTB, console, permanent ABI, reservation, and compatibility categories; record decisions needed before factual freeze.

## Out of scope

Choosing addresses, slot counts, register models, CPU-feature values, PSCI subset, implementation internals, or publishing an unapproved ABI.

## Work sequence

1. Inspect ADR-024/025/040 and the ADR section-18 unresolved machine items.
2. Define the reviewable categories a machine specification must cover.
3. Separate permanent Guest-visible facts, implementation facts, and reservations.
4. Route each concrete unresolved item to ADR Required or Specification Investigation.
5. Review host independence and future-version compatibility constraints.

## Acceptance and closure

P8-V02 and P8-V03 require a complete decision route and no Host fact in the proposed contract boundary. Passing does not mean v1 is frozen.

## Handoff

W03–W20 receive categories and decision gates only; factual values await approval.
