# P2-W03 — Boot memory map and ownership foundation

Chinese readers can use the [Chinese edition](p2-w03-boot-memory-map-ownership.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Establish a normalized, checked host physical-memory map that identifies every
P2-protected range before any dynamic page allocation begins.

## Scope

P2-D01–D08 and P2-G01–G03: RAM collection, hypervisor/DTB/firmware/boot
artifact protection, overlap and overflow treatment, map normalization, and a
reserved extension path for later ownership accounting.

## Out of scope

Allocator algorithm/metadata placement, P4 Guest-memory transfer, Stage-2,
`MemoryObject`/`MemoryRegion` object design, and any policy for releasing the
original DTB.

## Work sequence

1. Consume W02's normalized RAM/reservation facts and P1's authoritative image
   range without assuming one RAM bank or a QEMU address layout.
2. Define the required classification of allocatable, hypervisor-owned,
   reserved, dynamically allocated, and unavailable ranges.
3. Establish checked range, overlap, hole, conflict, and zero-size handling
   expectations, including explicit fatal outcomes for incompatible ownership.
4. Review protected-range rules against the hard safety gate and future P4
   ownership extension without designing P4 objects.
5. Specify map-normalization and conflict evidence for host and QEMU
   validation.
6. Hand off the authoritative allocatable/protected map to allocation,
   inspection, regression, and P4 consumers.

## Acceptance and closure

P2-V05 requires evidence that discovered RAM and all protected ranges normalize
with checked, unambiguous conflict treatment. P2-V10 and P2-V11 later require
negative and QEMU accounting evidence. P2-ACR-01 remains unresolved and blocks
any plan to define P4 memory objects here.

## Handoff

W04 may derive its safe allocation domain only from this map. P4 receives a
protected-range and ownership-extension foundation; no Guest-memory mechanism
is implemented.
