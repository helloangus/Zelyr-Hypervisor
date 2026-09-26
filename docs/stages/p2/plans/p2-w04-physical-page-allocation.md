# P2-W04 — Physical-page allocation foundation

Chinese readers can use the [Chinese edition](p2-w04-physical-page-allocation.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Provide a safe, observable host physical-page allocation and release capability
that never returns a range protected by W03.

## Scope

P2-E01–E08: allocation/free, page alignment, multiple RAM regions, explicit
OOM, protected-page safety, debug error detection, and managed/free/used/
reserved accounting.

## Out of scope

Selecting buddy, bitmap, or another allocator; large-page policy; concrete
allocator APIs; Guest-memory ownership transfer; or SMP concurrency design.

## Work sequence

1. Confirm W03's normalized map identifies the sole permitted allocation
   domain and all permanently excluded ranges.
2. Define the package outcome for allocation, release, alignment, exhaustion,
   multi-region handling, and accounting.
3. Establish validation/debug expectations for invalid free, duplicate free,
   unmanaged-range free, and accounting inconsistency.
4. Review the safety boundary so no convenience path reintroduces firmware,
   DTB, image, boot-artifact, or metadata pages.
5. Specify allocation/free/OOM and protected-page evidence across fragmented
   memory configurations.
6. Hand off the allocation contract to small allocation, inspection, P3, P4,
   and regression consumers.

## Acceptance and closure

P2-V06 requires evidence of explicit allocation/free/alignment/multi-region/
OOM/debug/accounting behavior and zero protected-page returns. P2-V10 and
P2-V11 extend this with stress and QEMU integration evidence.

## Handoff

W05 may obtain its backing foundation from this package. P3/P4 may rely on the
documented capability and limitations, not a selected allocator implementation.
