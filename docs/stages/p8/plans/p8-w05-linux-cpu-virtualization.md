# P8-W05 — Linux CPU virtualization compatibility

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Establish the compatibility classification and controlled-failure boundary for Linux Guest CPU behavior.

## Scope

Cover source P8.4: Guest EL, system registers, MMU/cache/TLB, barriers, WFI/WFE, timer, interrupt masking, CPU/features, and Direct/Emulate/Reject/Hidden/Unsupported classification.

## Out of scope

System-register implementation, trap tables, CPU-feature values, register encodings, code/API design, or an unsupported-operation policy beyond controlled diagnostics.

## Work sequence

1. Inspect W01/W03 and P4–P7 execution and exception facts.
2. Inventory Linux-required behavior by classification rather than mechanism.
3. Define evidence for normal Linux paths and unsupported operations.
4. Relate classified failures to VM-facing containment and diagnostics.
5. Review against Guest EL1, host independence, and no-global-panic rules.

## Acceptance and closure

P8-V07 and P8-V08 require declared normal and negative scenarios with explicit diagnostic outcome. Passing does not prove trap/emulation implementation.

## Handoff

W06, W09–W10, W13, and W18 receive only reviewed behavior categories.
