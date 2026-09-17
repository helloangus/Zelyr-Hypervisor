# P5 work-package plans

**Status:** Planning index. Individual plans are bounded work-package plans,
not implementation-completion records.

## How to use this directory

For a P5 package, read in this order:

1. the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md);
2. the [P5 task book](../task-book-v0.1.md), including its ABI and security routing;
3. this index and the selected P5-Wxx plan;
4. the mandatory agent guide for the activity: Plan Guide for detailed design,
   or Coding Guide plus an approved detailed design for code changes;
5. prerequisite package records and available P0–P4 implementation and
   verification evidence.

Each plan defines only a package outcome, bounded work, acceptance, and
handoff. It does not prescribe commands, crates/modules, function signatures,
data layouts, register assignments, error values, algorithms, or locks. ABI
and security documents demanded by P5 are written as factual implementation
artifacts only after their approved detailed design and compatibility analysis.
Put implementation traceability in `../implementation/` and evidence in
`../verification/`.

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p5-w01-entry-contract-reconciliation.md) | ADR; P0–P4 handoffs and evidence | W02–W10 |
| [W02](p5-w02-hypercall-abi-error-boundary.md) | W01; ABI/security routing | W05–W07, W09–W10, P6+ |
| [W03](p5-w03-guest-data-safety.md) | W01; P2 ownership and P4 Stage-2/fault facts | W06–W08, W10 |
| [W04](p5-w04-handle-lifecycle-type-safety.md) | W01; ADR-013 | W05–W08, W10, P6+ |
| [W05](p5-w05-capability-rights-bootstrap-revocation.md) | W02, W04; ADR-013 | W06–W08, W10, P6+ |
| [W06](p5-w06-dispatch-permission-containment.md) | W03–W05; P4 exception boundary | W07–W10 |
| [W07](p5-w07-validation-guest-isolation-suite.md) | W02, W03, W05, W06; P4 Validation Guest | W09–W10, P5 regression users |
| [W08](p5-w08-host-fuzz-stress-smp-baseline.md) | W03–W06; P0 host-test and P3 SMP inputs | W09–W10 |
| [W09](p5-w09-telemetry-safe-logging-regression.md) | W07, W08; P0 diagnostics/QEMU inputs | W10, P6+ regression users |
| [W10](p5-w10-closeout-p6-handoff.md) | W01–W09 and their actual records | P6 planning and stage review |

The table is acyclic. W02, W03, and W04 may proceed after W01 when their
approved detailed designs remain compatible. W06 is the integrated boundary;
W07 and W08 create complementary Guest-side and host-side evidence, then W09
integrates observability and regression before W10 records factual closure.
A plan is not complete until its linked validation has real evidence in
`../verification/`.
