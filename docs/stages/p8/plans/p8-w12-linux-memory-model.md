# P8-W12 — Linux memory-model validation

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan validation that Stage-2 and the approved Guest map can carry Linux memory-management behavior.

## Scope

Cover source P8.12: RAM initialization, allocators, page tables, map/unmap, TLB, COW, kernel/user switching, small/normal/larger RAM cases, reserved/MMIO boundaries, and Stage-2 diagnostics.

## Out of scope

Stage-2 redesign, page-table implementation, concrete RAM sizes, memory overcommit, ballooning, snapshot, or Host allocator policy.

## Work sequence

1. Inspect W02/W05/W10 and P2/P4 memory ownership facts.
2. Define Linux memory behavior and Guest-map evidence categories.
3. Establish small/normal/larger fixture classes without selecting capacities.
4. Define boundary and negative cases for reserved, MMIO, Host memory, and Stage-2 faults.
5. Review results against ownership and Guest-untrusted constraints.

## Acceptance and closure

P8-V17 requires a memory-size and isolation matrix with actionable diagnostics. It does not prove advanced memory features.

## Handoff

W16 and W18 receive memory validation requirements and limits.
