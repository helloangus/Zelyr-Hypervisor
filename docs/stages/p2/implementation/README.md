# P2 implementation designs and records

**Status:** W01/W02 implemented in their bounded reference scope; W03 map
implemented with host and bounded runtime evidence; W04 is unimplemented and deferred for a fresh work-package run; W05–W10 remain proposed designs; W11/W12 detailed designs are owner-approved with explicit execution gates. This is not whole-P2 completion.
**Scope:** P2 stage-local implementation material only.

Read the P2 task book and selected work-package plan before using an
item here.  For code changes, the repository `AGENTS.md` and the mandatory
[Coding Guidelines](../../../development/coding-guidelines.md) still apply.

| Work package | Detailed design / record | Status |
|---|---|---|
| P2-W01 | [Boot platform-description intake detailed implementation design](p2-w01-boot-platform-description-intake/README.md); [record](p2-w01-boot-platform-description-intake-record.md) | Implemented; see linked verification |
| P2-W02 | [Platform discovery and normalization detailed implementation design](p2-w02-platform-discovery-normalization/README.md); [record](p2-w02-platform-discovery-normalization-record.md) | Implemented; see linked verification |
| P2-W03 | [Boot memory map and ownership foundation detailed implementation design](p2-w03-boot-memory-map-ownership/README.md); [host record](p2-w03-boot-memory-map-ownership-record.md); [runtime record](p2-w03-runtime-record.md) | Storage, boot integration and bounded runtime verification delivered; W04 remains unimplemented |
| P2-W04 | [Physical-page allocation foundation design](p2-w04-physical-page-allocation/README.md) | Unimplemented; owner excluded current W04 work from submission and will restart it later |
| P2-W05 | [Dynamic small-allocation foundation detailed implementation design](p2-w05-dynamic-small-allocation/README.md) | Proposed design; see remaining admission gates |
| P2-W06 | [Platform and memory inspection detailed implementation design](p2-w06-platform-memory-inspection/README.md) | Proposed design; implementation not claimed |
| P2-W07 | [Offline DTB compatibility checking detailed implementation design](p2-w07-offline-dtb-compatibility/README.md) | Proposed design; implementation not claimed |
| P2-W08 | [Host robustness and negative regression detailed implementation design](p2-w08-host-robustness-regression/README.md) | Proposed design; implementation not claimed |
| P2-W09 | [QEMU platform integration regression detailed implementation design](p2-w09-qemu-integration-regression/README.md) | Proposed design; implementation not claimed |
| P2-W10 | [P3/P4 handoff contract detailed implementation design](p2-w10-p3-p4-handoff-contract/README.md) | Proposed design; implementation not claimed |

| P2-W11 | [Host allocated-frame mapping detailed design](p2-w11-host-allocated-frame-mapping/README.md); [plan](../plans/p2-w11-host-allocated-frame-mapping.md) | Owner-approved design, 2026-10-02; production prerequisites, implementation and runtime evidence pending |
| P2-W12 | [Detailed design](p2-w12-minimal-memory-objects/README.md); [plan](../plans/p2-w12-minimal-memory-objects.md) | Owner-approved design, 2026-10-02; production prerequisites, implementation and runtime evidence pending |

The [P2-W01/W02 prerequisite conflict](p2-w01-w02-prerequisite-conflict.md)
records the original DTB-length and Stage-1-access mismatch and its resolution
through P2-owned bounded access.

The [W03/W04 design-conflict audit](p2-w03-w04-design-conflicts.md) records
the missing writable metadata window and normalization/allocator contract
issues and their disposition at the time of that audit. W03
now has the linked implementation/verification record; W04 remains unimplemented, including mapping and target integration.
The [contract reconciliation record](p2-contract-reconciliation-record.md) links
the corrected W03–W10 designs and remaining layout/mapping/adapter gates.

Actual command logs and pass/fail evidence belong in `../verification/`, not in
this index or a detailed design.
