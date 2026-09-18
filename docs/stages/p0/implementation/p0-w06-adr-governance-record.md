# P0-W06 ADR Governance — Implementation Record

**Status:** Implemented on branch `p0/w06-adr-governance`; verification
evidence in [the verification
record](../verification/p0-w06-adr-governance-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W06 detailed implementation
design](p0-w06-adr-governance/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/adr/README.md` | Expanded from the five-line index note into the normative ADR governance document v0.1: state table and transitions, decision test, label semantics, conflict/supersession/review process, history-preservation rules, register interaction, index (ADR-000 row), hypothetical worked example (labeled informative). All true statements of the previous note are preserved; adr-000 decision text untouched. |
| `docs/templates/adr-template.md` (new) | ADR template with the required field set, instructional comments, append-only change-history section; carries no project-specific decision |
| `docs/README.md` | "Any architecture-affecting work" routing row extended with the ADR process pointer (no duplicate row) |
| `docs/stages/p0/implementation/README.md` | W06 status row updated truthfully |
| This record; the verification record | Decisions, drill evidence |

## Narrow-edit rule status (design open question)

Rule §4(b) — permitting status-line pointer annotations on adr-000 register
rows after a standalone ADR decides a 待定 item — is **applied as designed**.
No annotation has been performed in P0 (none was authorized: no 待定 item was
decided). The owner may decline the rule later; declining changes no accepted
decision and requires only dropping rule (b) via review.

## Deviations from the design

None. adr-000's decision text, principles, and register rows are
byte-identical; the drill created no file; no CI/lint enforcement was added.

## Handoff notes for downstream packages

- **W21:** the `ADR Required` / escalation path is ready for its admission
  rules to reference.
- **W05:** ADR-class lifecycle detail now lives in `docs/adr/README.md`; the
  documentation baseline's pointer resolves.
- **Threshold-section owners (W02-style):** "ADR required" references in the
  governance baselines now resolve to a defined process.
- **Every later plan/stage:** the two labels are defined and route to the
  same Proposed-ADR artifact; the template is the required form.
