# P2 implementation designs and records

**Status:** W01/W02 implemented in their bounded reference scope; later packages
remain proposed designs. This is not whole-P2 completion.
**Scope:** P2 stage-local implementation material only.

Read the P2 task book and selected work-package plan before using an
item here.  For code changes, the repository `AGENTS.md` and the mandatory
[Coding Guidelines](../../../development/coding-guidelines.md) still apply.

| Work package | Detailed design / record | Status |
|---|---|---|
| P2-W01 | [Boot platform-description intake detailed implementation design](p2-w01-boot-platform-description-intake/README.md); [record](p2-w01-boot-platform-description-intake-record.md) | Implemented; see linked verification |
| P2-W02 | [Platform discovery and normalization detailed implementation design](p2-w02-platform-discovery-normalization/README.md); [record](p2-w02-platform-discovery-normalization-record.md) | Implemented; see linked verification |
| P2-W03 | [Boot memory map and ownership foundation detailed implementation design](p2-w03-boot-memory-map-ownership/README.md) | Proposed design; implementation not claimed |
| P2-W04 | [Physical-page allocation foundation detailed implementation design](p2-w04-physical-page-allocation/README.md) | Proposed design; implementation not claimed |
| P2-W05 | [Dynamic small-allocation foundation detailed implementation design](p2-w05-dynamic-small-allocation/README.md) | Proposed design; implementation not claimed |
| P2-W06 | [Platform and memory inspection detailed implementation design](p2-w06-platform-memory-inspection/README.md) | Proposed design; implementation not claimed |
| P2-W07 | [Offline DTB compatibility checking detailed implementation design](p2-w07-offline-dtb-compatibility/README.md) | Proposed design; implementation not claimed |
| P2-W08 | [Host robustness and negative regression detailed implementation design](p2-w08-host-robustness-regression/README.md) | Proposed design; implementation not claimed |
| P2-W09 | [QEMU platform integration regression detailed implementation design](p2-w09-qemu-integration-regression/README.md) | Proposed design; implementation not claimed |
| P2-W10 | [P3/P4 handoff contract detailed implementation design](p2-w10-p3-p4-handoff-contract/README.md) | Proposed design; implementation not claimed |

The [P2-W01/W02 prerequisite conflict](p2-w01-w02-prerequisite-conflict.md)
records the original DTB-length and Stage-1-access mismatch and its resolution
through P2-owned bounded access.

Actual command logs and pass/fail evidence belong in `../verification/`, not in
this index or a detailed design.
