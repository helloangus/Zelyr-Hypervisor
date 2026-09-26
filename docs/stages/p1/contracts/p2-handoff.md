# P1 to P2 handoff

**Status:** P1-to-P2 handoff contract; P1 completion is bounded by the L7 report, and P2 readiness is not claimed.\
**Scope:** Named P2 consumers of P1 contracts; no P2 API, module or algorithm freeze.\
**Version:** v0.2.\
**Owner/change context:** P1-W12 and P1 L7 completion review, following [P2 task-book inputs](../../p2/task-book-v0.1.md#2-inputs-constraints-and-state) and [plan index](../../p2/plans/README.md), 2026-09-26.\
**Supersedes:** v0.1 handoff wording.

P2 may consume the bounded environment described by [boot](aarch64-boot-contract.md),
[initialization](el2-initialization-contract.md), [Host Stage-1](host-address-space.md)
and [diagnostics](exception-diagnostics-contract.md), subject to the
[P1 completion report's limits](../verification/p1-completion-report.md) and
[evidence map](stage-gate-evidence-map.md). The image physical
range and retained, unexamined DTB pointer are inputs, not a validated
platform description. Capability facts describe the current boot CPU; they
are not a DTB-to-PlatformInfo implementation.

| P2 consumer | P1 source consumed | Boundary retained in P2 |
|---|---|---|
| [W01 intake](../../p2/plans/p2-w01-boot-platform-description-intake.md) | boot contract: DTB pointer and image range | validate DTB location, content, lifetime and overlap |
| [W02 normalization](../../p2/plans/p2-w02-platform-discovery-normalization.md) | boot CPU capability context | perform actual discovery and normalization |
| [W03 boot map](../../p2/plans/p2-w03-boot-memory-map-ownership.md) | Host image/stack/table bounds and temporary map | derive checked RAM and reserved ranges |
| [W04 pages](../../p2/plans/p2-w04-physical-page-allocation.md), [W05 small allocation](../../p2/plans/p2-w05-dynamic-small-allocation.md) | stable EL2 Host runtime | design allocation and ownership |
| [W06 inspection](../../p2/plans/p2-w06-platform-memory-inspection.md) | diagnostic conventions | report actual normalized P2 state |
| [W08 robustness](../../p2/plans/p2-w08-host-robustness-regression.md) | bounded failure classes | design P2-specific negative/property scenarios |
| [W09 integration](../../p2/plans/p2-w09-qemu-integration-regression.md) | [reference runner conventions](reference-qemu-environment.md) | define P2 matrices and evidence |
| [W10 handoff](../../p2/plans/p2-w10-p3-p4-handoff-contract.md) | [limitations](known-limitations.md) and [evidence map](stage-gate-evidence-map.md) | assemble its own downstream evidence |

P2 owns DTB validation and discovery, normalized PlatformInfo, physical-memory
map, page/small allocation and their tests. P1 offers no permanent
identity-map promise, allocator, GIC/IRQ service, secondary CPU, Guest,
Stage-2 or VM. No table above fixes a P2 Rust type, parser, module, allocator
algorithm, lock or inspection command. If new evidence invalidates a P1 gate
at P2 entry, that is an upstream dependency defect, not permission to assume
a pass or reimplement P1 inside P2. P1 completion itself is not a P2
implementation/readiness claim.

Under [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md),
P1 provides reviewed structural IRQ/FIQ/SError vector coverage, not executed
asynchronous delivery. The original NC6 run remains unperformed and belongs
to P6-W12/P6-V29 after Host GIC/IRQ readiness. P2 must not interpret an
P1 completion as proof of NC6 or add a GIC/IRQ mechanism to
close it.
