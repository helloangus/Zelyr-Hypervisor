# P6 work-package plans

**Status:** Planning index. Individual plans are bounded work-package plans,
not implementation-completion records.

## How to use this directory

For a P6 package, read in this order:

1. the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md);
2. the [P6 task book](../task-book-v0.1.md);
3. this index and the selected P6-Wxx plan;
4. actual P0–P5 handoff, implementation, and verification records;
5. the mandatory agent guide for the activity: the Plan Guide for planning, or
   the Coding Guide plus an approved detailed design before code changes.

Plans define bounded outcomes, substantial work steps, acceptance, and handoff.
They intentionally do not specify module trees, APIs, data layouts, register
sequences, algorithms, commands, or verification results. Put detailed design
and implementation traceability under ../implementation/ and real evidence
under ../verification/.

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p6-w01-gic-capability-discovery.md) | ADR, P6 book, P0–P5 handoff evidence, especially P2 PlatformInfo | W02–W13 |
| [W02](p6-w02-physical-gic-bring-up.md) | W01; P3 online-pCPU and local-state contract | W03–W05, W11–W13 |
| [W03](p6-w03-physical-interrupt-lifecycle.md) | W02; P1 IRQ-entry and P3 synchronization contracts | W04–W05, W07, W11–W13 |
| [W04](p6-w04-smp-interrupt-routing-sgi.md) | W02, W03; P3 notification and pCPU lifecycle contracts | W11, W13, P7 |
| [W05](p6-w05-el2-generic-timer.md) | W03; P1 timer-access and P3 per-pCPU contracts | W06, W10–W11, W13, P7 |
| [W06](p6-w06-guest-generic-timer.md) | W05; P4 Guest entry/exit and P5 error/capability contracts | W10–W13, P7–P8 |
| [W07](p6-w07-virtual-interrupt-core.md) | W03; P4 vCPU boundary and P5 authorization contracts | W08–W13, P7–P8 |
| [W08](p6-w08-gic-virtualization-interface.md) | W01, W07; evidenced virtualization capability | W09–W13, P8 |
| [W09](p6-w09-maintenance-interrupt.md) | W08 | W10–W13, P8 |
| [W10](p6-w10-interrupt-semantics.md) | W06, W09 | W11–W13, P7–P8 |
| [W11](p6-w11-validation-guest-interrupt-suite.md) | W04, W06, W10; P4 Validation Guest contract | W13, P7–P8 regression users |
| [W12](p6-w12-fault-isolation-robustness.md) | W03, W07, W09; P5 Guest-error boundary | W13, P7–P8 security review |
| [W13](p6-w13-telemetry-regression-handoff.md) | W01–W12 and their actual implementation/verification records | P7/P8 planning and stage review |

The table is acyclic. W05 and W07 may progress after W03 when their detailed
designs agree on the Host IRQ lifecycle and synchronization boundary; all other
listed edges supply behavior or evidence required by their consumers. A plan is
not complete until its linked validation has real evidence in ../verification/.
