# P8-W04 — Guest DTB contract

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Specify the Guest-only DTB facts Linux must consume for the approved virtual machine.

## Scope

Cover source P8.3: CPU/topology, memory, chosen/bootargs/initrd, PSCI, timer, GIC, console, compatible/model, reserved memory, and P8 minimal devices; include consistency and host-leakage review.

## Out of scope

DTB builder mechanics, concrete node values before approval, Host DTB reuse, ACPI, Virtio device design, or Host platform description.

## Work sequence

1. Inspect W02/W03 categories and the P2 platform-information boundary.
2. Enumerate required Guest-visible DTB facts and authoritative sources.
3. Define consistency checks with approved machine and boot contracts.
4. Define negative checks for Board, SoC, physical address, IRQ, and firmware leakage.
5. Review Linux consumption without importing Host semantics.

## Acceptance and closure

P8-V05 and P8-V06 require a Guest-only, machine-consistent DTB review and host-leakage criteria. No DTB output is claimed.

## Handoff

W09–W10, W14, W15, and W16 receive contract facts and review conditions.
