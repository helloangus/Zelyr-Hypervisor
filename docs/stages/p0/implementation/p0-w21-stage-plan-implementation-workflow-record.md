# P0-W21 Stage/Plan/Implementation Workflow — Implementation Record

**Status:** Implemented on branch `p0/w21-stage-workflow`; verification
evidence in [the verification
record](../verification/p0-w21-stage-plan-implementation-workflow-verification.md).
**Date:** 2026-09-19 (Asia/Shanghai)
**Design:** [W21 detailed implementation
design](p0-w21-stage-plan-implementation-workflow/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/stage-workflow.md` (new) | Normative stage workflow v0.1: seven-layer model (L1 ADR → L7 completion) with per-layer six fields (question, authority, escalation boundary, admission, consumers, failure behavior), admission decision table with corollaries, seven traceability/evidence-placement rules keyed on P\<stage\>-Wxx/P\<stage\>-Vxx, non-mixing rules |
| `docs/README.md` | "Stage planning / detailed design" routing row extended with the workflow pointer |
| `docs/stages/p0/implementation/README.md` | W21 status row updated truthfully |
| This record; the verification record | Decisions, drill evidence |

## Deviations from the design

None. The workflow document cites its authorities (ADR process,
documentation baseline, integration workflow, contributor workflow, CI
baseline) and restates none of their content; no lifecycle states, label
definitions, directory rules, merge-policy text, or template bodies were
created.

## Handoff notes for downstream packages

- **W22:** the handoff rule (§4.6 — link, never copy) is the shape the
  dependency map applies to P0's package.
- **P1 planners:** the admission table's "code-bearing change" row is the
  preflight; the traceability keys (P\<stage\>-Wxx/Vxx) carry over.
- **W06:** cited, not restated — label semantics remain the ADR process's.
