# P5-W03 — Guest-data safety boundary

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Establish a reusable P5 Guest-data boundary so a Guest-supplied address and
range are never treated as an unchecked Hypervisor pointer.

## Scope

Cover P5-T02–T03 request/range legality: Guest IPA validity, Stage-2 presence
and access rights, checked length arithmetic, zero and maximum length,
cross-page/partial mapping, unmapped, read-only write, invalid memory type,
and forbidden-boundary treatment.

## Out of scope

Choosing copy/mapping APIs, page-walk algorithms, Host pointer representation,
shared-memory IPC, final maximum sizes, P4 Stage-2 redesign, or a permanent
machine-memory ABI.

## Work sequence

1. Inspect W01's P2 ownership and P4 Stage-2/Guest-fault evidence inputs.
2. Produce an approved detailed design for the Guest-data validation boundary,
   access authority, failure results, and lifetime/side-effect constraints.
3. Integrate planned request validation with all future HVC buffer consumers
   while retaining Host/Guest address and ownership separation.
4. Define normal and negative scenarios for all required range, mapping,
   permissions, partial-page, and overflow cases.
5. Review Guest data as untrusted input and ensure no scenario exposes Host
   addresses or treats a QEMU behavior as a Core rule.
6. Record implementation/evidence status and hand the controlled boundary to
   W06–W08.

## Acceptance and closure

P5-V03, V09, V10, and V13 require evidence that required valid and invalid
Guest-data cases receive controlled outcomes without unchecked Host access or
an unexplained Hypervisor failure. This does not prove IPC or general shared
memory.

## Handoff

W06 consumes a validated parameter boundary; W07 consumes expected negative
scenarios; W08 consumes host-testable range and parsing properties.
