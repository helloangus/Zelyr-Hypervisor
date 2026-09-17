# P4-W05 — Rust Validation Guest test asset

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Create and maintain the Rust `no_std` Validation Guest and its explicit P4
scenario set as a reusable test asset for the Guest-EL1 boundary.

## Scope

Cover P4-D01–D08 and P4-J: a directly loadable Rust bare-metal Guest, observable
EL1 marker, CurrentEL confirmation, RAM/stack/code execution, unmapped and
permission-fault triggers, WFI/WFE behavior, controlled illegal behavior, and
the VG-001–VG-012 scenario list.

## Out of scope

Linux, libc, firmware dependency, production guest ABI, Guest DTB, virtio,
virtual interrupts/timer, a scheduler, or a decision about permanent repository
layout beyond recording factual implementation output.

## Work sequence

1. Inspect the W03 boot-input and W04 Guest-EL1 transition contracts.
2. Produce an approved detailed design for the Validation Guest's externally
   observable scenarios and its test-asset maintenance boundary.
3. Integrate scenario inputs with the established Guest RAM, entry, and exit
   contracts without selecting future runtime mechanisms.
4. Define the expected success, controlled-fault, and diagnostic markers for
   every VG-001–VG-012 scenario.
5. Review the asset against the untrusted-Guest boundary and distinguish P4
   test conventions from a frozen Guest machine ABI.
6. Record implementation/evidence status and hand the scenario contract to W06
   and W08.

## Acceptance and closure

P4-V05, P4-V06, P4-V08, and P4-V09 require real evidence for mandatory
VG-001–VG-007, VG-010, and VG-012, plus explicit review treatment for
VG-008/VG-009/VG-011 if they cannot be completed. Passing does not prove a
general guest operating environment.

## Handoff

W06 receives controlled triggers for exit and isolation diagnosis; W08 receives
stable expected markers. P5 inherits a maintained validation asset, not a
management or hypercall ABI.
