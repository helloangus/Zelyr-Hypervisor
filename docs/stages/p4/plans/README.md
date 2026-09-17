# P4 work-package plans

**Status:** Planning index. Individual plans are bounded work-package plans,
not implementation-completion records.

## How to use this directory

For a P4 package, read in this order:

1. the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md);
2. the [P4 task book](../task-book-v0.1.md);
3. this index and the selected P4-Wxx plan;
4. upstream P0–P3 handoff records and any available implementation/verification
   evidence;
5. the mandatory agent guide for the activity: the Plan Guide for planning, or
   the Coding Guide plus an approved detailed design before code changes.

Plans define bounded outcomes, substantial work steps, acceptance, and
handoff. They intentionally do not specify module trees, APIs, data layouts,
algorithms, assembly boundaries, commands, or verification results. Put
detailed design and implementation traceability under `../implementation/` and
real evidence under `../verification/`.

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p4-w01-entry-contract-reconciliation.md) | ADR, P4 task book, P0–P3 handoff evidence | W02–W09 |
| [W02](p4-w02-stage2-address-space.md) | W01; P2 memory and P3 TLB-transport contracts | W04, W06–W09, P5 |
| [W03](p4-w03-guest-memory-image.md) | W01; P2 allocation/ownership contract | W04–W09, P5 |
| [W04](p4-w04-vcpu-entry-exit.md) | W02, W03; P1 EL2 exception contract; P3 CPU-local contract | W05–W09, P5 |
| [W05](p4-w05-validation-guest.md) | W03, W04 | W06, W08–W09, P5 |
| [W06](p4-w06-fault-isolation-diagnostics.md) | W02, W04, W05 | W07–W09, P5 |
| [W07](p4-w07-repeatability-telemetry.md) | W04, W06 | W08–W09, P5 |
| [W08](p4-w08-qemu-integration-regression.md) | W05–W07; P0 QEMU automation contract | W09, P5 regression users |
| [W09](p4-w09-closeout-p5-handoff.md) | W01–W08 and their actual implementation/verification records | P5 planning and stage review |

The table is acyclic. W02 and W03 may progress after W01 only when their
detailed designs agree on the P2 ownership boundary. A plan is not complete
until its linked validation has real evidence in `../verification/`.
