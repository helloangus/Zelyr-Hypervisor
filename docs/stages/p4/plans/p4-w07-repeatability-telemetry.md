# P4-W07 — Repeatability and Stage-2 telemetry

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Make the P4 execution boundary repeatable and observably attributable across
same-session and declared cold-boot scenarios.

## Scope

Cover P4-H01–H03 and P4-I01–I02: create/run/stop/reinitialize/run behavior,
cold-boot consistency, no dependence on residual Guest memory, required event
categories, enter/exit/fault counts, and VM/vCPU/PC/IPA fault correlation.

## Out of scope

Final telemetry API, production metrics backend, scheduler statistics,
cross-pCPU Stage-2 shootdown proof, fault-tolerant VM orchestration, or final
VM create/destroy lifecycle semantics.

## Work sequence

1. Inspect W04's lifecycle boundary and W06's categorized diagnostic events.
2. Produce an approved detailed design for repeat scenarios, state reset
   expectations, and P4 telemetry evidence without prescribing its internals.
3. Integrate repeatability checks with Guest memory initialization and required
   vCPU/Stage-2 event context.
4. Define evidence expectations for same-session restart, repeated cold boot,
   counts, and fault correlation.
5. Review that telemetry is structured and that temporary P4 behavior is not
   represented as final lifecycle or policy semantics.
6. Record implementation/evidence status and hand repeatable markers and event
   expectations to W08 and W09.

## Acceptance and closure

P4-V10–P4-V12 require real evidence for deterministic reinitialization,
declared cold-boot consistency, observable required event categories, counts,
and fault context. Passing does not prove all hardware behavior or a production
telemetry service.

## Handoff

W08 uses the repeat and event expectations for automation. W09 records actual
limitations and evidence locations; P5 receives only the factual observability
baseline.
