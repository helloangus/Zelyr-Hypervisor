# P0-W21 Stage/Plan/Implementation Workflow — Verification Evidence

**Status:** Complete evidence recorded; W21 closure claimed.
**Date:** 2026-09-19 (Asia/Shanghai)
**Environment:** Documentary review and drills against branch
`p0/w21-stage-workflow` (baseline: merge of PR #33).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W21-DV01 → P0-V09 | Layer-matrix review | **passed** | Seven layers present with all six fields each; non-mixing rules stated; authorities cited (ADR process, documentation baseline, integration workflow, contributor workflow, CI baseline), none restated. |
| W21-DV02 → P0-V09/P0-V15 | Admission-table review | **passed** | Three hypothetical cases resolve unambiguously (evidence below), each to a required-hold set and an owned escalation path. |
| W21-DV03 → P0-V09 | Traceability review (real chain: P0-W01) | **passed** | The delivered P0-W01 chain resolves: plan index → plan → design directory → implementation record → verification record; every cross-link in the traced files resolves from the checkout (link pass over the four artifacts found no broken link); the record names its design and artifacts; the verification record carries per-ID evidence with proves/does-not-prove statements and holds the only completion claim. |
| W21-DV04 → P0-V09 | Discovery and link review | **passed** | The extended routing row reaches the workflow in one link; its links resolve; stage-index row truthful. |
| W21-DV05 → P0-V15 | P0 drill | **passed** | Walking the stage workflow as a P0 agent: L1 reachable (ADR index), L2 reachable (task book), L3 reachable (plans index), L4 reachable (implementation designs), L5/L6 reachable (records/evidence locations), L7 reachable (task-book exit criteria + verification area); every layer step `reachable`, no `failed`. |
| W21-DV06 → P0-V15 | P1 drill | **passed** | Walking as the hypothetical P1-W10 planner (consumes the runner entry, toolchain, target build, gates): each consumed P0 input findable from documents alone with distinguishable status (delivered contracts vs interface-only placeholder vs future-class gate); no `failed` step. |
| W21-DV07 → W21 closure | Consumability review | **passed** | W22 (handoff rule §4.6 gives the map's shape: link, never copy), W06 (labels cited, not restated), a hypothetical P1 planner (admission row + traceability keys answer "may I proceed, and what must I hold?") — each can act without inventing rules. |

## Admission-table drills (W21-DV02 evidence)

| Hypothetical case | Resolution | Owning escalation |
|---|---|---|
| An agent holds an approved plan but no detailed design exists for the code it wants to write | Row 1: the required-hold set is incomplete — the design is a blocker to record; "design while coding" is prohibited by the corollary | Blocker recorded against the owning package; no code |
| Mid-implementation, a change conflicts with an accepted decision (ADR-043) | Row 5: label `ADR Required` (blocked) or `Architecture Change Request` (proposal) per the ADR process; the affected work stops at the boundary | Work stops; Proposed ADR via the ADR path; no local redesign |
| L7 attempts a stage completion where one package's verification record lacks a validation-ID entry | Row 7: the missing evidence item fails completion; it cannot be waived by process text | Stage stays open until the owning package's L6 record is completed |

## Not run / not proved

- **No new code-bearing package has yet been admitted under this table** (P0's
  packages were governed by their designs' own preflight rules, which this
  workflow consolidates); the rules are reviewable, applicable (DV03 traces a
  real chain), and become the standing admission rule for P1+.
- **Drills are discoverability evidence only**: they prove nothing about
  builds, tests, QEMU, hardware, or any package's completion.
