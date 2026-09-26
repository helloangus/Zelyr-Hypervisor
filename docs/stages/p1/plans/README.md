# P1 work-package plans

**Status:** Planning index. Individual plans are bounded work-package plans;
implementation and validation are not claimed.

## Reading order

Read the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md),
the [P1 task book](../task-book-v0.2.md), this index and the selected plan, then
the applicable P0 records and mandatory agent guide. Code changes additionally
require the Coding Guide and an approved detailed design.

Implementation traceability belongs under [../implementation/](../implementation/);
actual commands, environments, results and limitations belong under
[../verification/](../verification/).

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p1-w01-reference-boot-contract.md) | P0 target/build/QEMU entry | W02, W03, W09, W10 |
| [W02](p1-w02-minimal-rust-el2-runtime.md) | W01 | W03–W12 |
| [W03](p1-w03-aarch64-capability-inventory.md) | W02 | W04, W09, W11, P2 |
| [W04](p1-w04-el2-architectural-state-baseline.md) | W03 | W05, W08, W09 |
| [W05](p1-w05-el2-exception-entry-baseline.md) | W02, W04 | W06, W07, W11 |
| [W06](p1-w06-early-console-logging.md) | W02, W05 | W07–W12 |
| [W07](p1-w07-fatal-crash-diagnostics.md) | W05, W06 | W08, W11, W12 |
| [W08](p1-w08-host-stage1-address-space.md) | W04–W07 | W09–W12, P2 |
| [W09](p1-w09-initialization-sequencing.md) | W01–W08 | W10–W12, P2 |
| [W10](p1-w10-qemu-boot-regression.md) | W01–W09 | W11, W12, P2 |
| [W11](p1-w11-negative-fault-validation.md) | W05–W10 | W12, P2; deferred NC6 execution to P6-W12/P6-V29 |
| [W12](p1-w12-p1-documentation-handoff.md) | W01–W11 | P1 completion review, P2 |

W06/W07 can be prepared in parallel after W05; all other dependencies are
acyclic and justify the stated handoff. No plan authorizes Guest, SMP, GIC,
general platform discovery or dynamic allocator implementation.
