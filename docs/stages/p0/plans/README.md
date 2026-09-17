# P0 work-package plans

**Status:** Planning index. Individual plans are bounded work-package plans,
not implementation-completion records.

## How to use this directory

For a P0 package, read in this order:

1. the [architecture baseline](../../../adr/adr-000-architecture-baseline-v0.1.md);
2. the [P0 task book](../task-book-v0.1.md);
3. this index and the selected P0-Wxx plan;
4. the mandatory agent guide for the activity: Plan Guide for design, Coding
   Guide plus an approved detailed design for code changes;
5. prerequisite package records and any available implementation/verification
   evidence.

Each plan defines only the work package's required outcome, bounded steps,
acceptance evidence, and handoff. It intentionally does not prescribe commands,
crate/module structure, function signatures, data layouts, or later-stage
runtime design. Put implementation traceability under ../implementation/ and
validation evidence under ../verification/.

## Dependency and execution map

| Plan | Prerequisites | Primary consumers |
|---|---|---|
| [W01](p0-w01-repository-baseline.md) | ADR, task book | W02, W05, W19, W20 |
| [W02](p0-w02-rust-toolchain-baseline.md) | W01 | W03, W07, W19, W20 |
| [W03](p0-w03-aarch64-build-target-baseline.md) | W01, W02 | W07, W09, W19, W20, P1 |
| [W04](p0-w04-build-profile-feature-governance.md) | W01, ADR | W03, W07, W16, P1+ |
| [W05](p0-w05-documentation-baseline.md) | W01, ADR | W06, W10–W22, P1+ |
| [W06](p0-w06-adr-governance.md) | W05, ADR | every later plan/stage |
| [W07](p0-w07-development-quality-gates.md) | W02–W05, W08 | W20, P1+ |
| [W08](p0-w08-host-side-testing-baseline.md) | W01, W02 | W07, W19, W20, P1+ |
| [W09](p0-w09-qemu-automation-entry-baseline.md) | W02, W03, W05 | W19, W20, P1+ |
| [W10](p0-w10-unsafe-rust-governance.md) | W05, W06 | P1+ low-level work |
| [W11](p0-w11-platform-portability-guardrails.md) | W05, W06, ADR | P1+ platform work |
| [W12](p0-w12-logging-diagnostic-baseline.md) | W04–W06 | W13, W16, P1+ |
| [W13](p0-w13-trace-event-namespace-baseline.md) | W05, W12 | P1+ telemetry |
| [W14](p0-w14-panic-failure-classification.md) | W05, W06, ADR | W10, W12, P1+ |
| [W15](p0-w15-address-identifier-type-safety.md) | W05, ADR | P1+ designs |
| [W16](p0-w16-version-build-metadata-baseline.md) | W03, W04, W12 | W17, W19, P1+ |
| [W17](p0-w17-artifact-naming-baseline.md) | W05, W16 | W19, W20, P1+ |
| [W18](p0-w18-dependency-governance.md) | W05, W06, W10 | P1+ dependency decisions |
| [W19](p0-w19-reproducible-development-workflow.md) | W01–W03, W07–W09, W16–W17 | W20, P1 onboarding; branch/PR contributors |
| [W20](p0-w20-ci-baseline.md) | W02, W03, W07–W09, W19 | P0 completion, P1+; GitHub PR and `main` protection |
| [W21](p0-w21-stage-plan-implementation-workflow.md) | W05, W06 | every later stage |
| [W22](p0-w22-stage-dependency-map.md) | W01–W21 | P0 completion, P1 planning |

Packages in a row may proceed only when their listed prerequisites provide the
required contract. Evidence of implementation belongs to its package record;
this index does not claim that any plan has been implemented.
