# P2 work-package plans

Chinese readers can use the [Chinese edition](README.zh-CN.md).

**Status:** Planning index. Individual plans are bounded work-package plans,
not implementation-completion records.

## Reading order

For a P2 package, read:

1. the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md);
2. the [P2 task book](../task-book-v0.1.md);
3. this index and the selected P2-Wxx plan;
4. applicable P0/P1 records and evidence that establish listed prerequisites;
5. the mandatory guide for the activity. Code changes additionally require the
   Coding Guide and an approved detailed design.

Each plan defines its bounded outcome, acceptance, and handoff only. Record
implementation decisions under [../implementation/](../implementation/) and
commands, environments, results, limitations, and completion evidence under
[../verification/](../verification/).

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p2-w01-boot-platform-description-intake.md) | P0 host-test/diagnostic/unsafe baseline; P1 DTB handoff and image range | W02, W07, W08, W09 |
| [W02](p2-w02-platform-discovery-normalization.md) | W01 | W03, W06, W07, W09, P3, P4 |
| [W03](p2-w03-boot-memory-map-ownership.md) | W02; P1 hypervisor image range | W04, W06, W08, W09, P4 |
| [W04](p2-w04-physical-page-allocation.md) | W03 | W05, W06, W08, W09, P3, P4 |
| [W05](p2-w05-dynamic-small-allocation.md) | W04 | W06, W08, W09, P3, P4 |
| [W06](p2-w06-platform-memory-inspection.md) | W02–W05 | W09, W10, P3, P4 reviewers |
| [W07](p2-w07-offline-dtb-compatibility.md) | W01, W02 | W08, W10, platform planners |
| [W08](p2-w08-host-robustness-regression.md) | W01–W05, W07 | W09, W10 |
| [W09](p2-w09-qemu-integration-regression.md) | W01–W06, W08; P0 QEMU runner entry | W10, P3, P4 planning |
| [W10](p2-w10-p3-p4-handoff-contract.md) | W01–W09 | P3, P4, P2 completion review |

The graph is intentionally acyclic. W01–W05 establish the foundational chain;
W06 and W07 provide its observability and offline compatibility checks; W08
tests host-side robustness; W09 supplies reference-platform integration
evidence; W10 records the consumer contract without claiming the stage is done.
