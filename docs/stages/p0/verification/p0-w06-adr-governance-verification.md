# P0-W06 ADR Governance — Verification Evidence

**Status:** Complete evidence recorded; W06 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentation review and drill against branch
`p0/w06-adr-governance` (baseline: merge of PR #16). No build/gate/CI
execution is applicable.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W06-DV01 → P0-V10 | Lifecycle review | **passed** | All five states (Proposed/Accepted/Rejected/Deferred/Superseded) defined with meaning, normativity, and transitions; no transition contradicts adr-000's change rule ("已确定 ADR 若被推翻，应新增 ADR 标注 Supersedes ADR-xxx") or `AGENTS.md`; Accepted→Deferred explicitly excluded as a transition. |
| W06-DV02 → P0-V10 | Threshold review | **passed** | The §2 decision test applied to four sample changes (evidence below) classifies each unambiguously; cross-baseline thresholds are referenced, not restated. |
| W06-DV03 → P0-V10 | Traceability drill | **passed** | The §8/appendix hypothetical (Core branching on board names) walked as a reviewer would: conflicting ADRs identified (ADR-052/ADR-043), label chosen (`Architecture Change Request`, with the `ADR Required` boundary stated), §4 path walked to hypothetical acceptance including the exact permitted byte changes; no repository file changed during the drill (negative case confirmed: `git status` clean at drill time, no ADR file created). |
| W06-DV04 → P0-V09/P0-V10 | Template review | **passed** | Template carries every §9-required field with instructional comments; a trial fill-in (hypothetical ADR-061 draft, not committed) produces a complete review record via the append-only change-history section; field semantics match §1 states and §4 numbering (next ID ADR-061, verified against the register's highest ID ADR-060). |
| W06-DV05 → P0-V10 | History-preservation review | **passed** | §4's narrow-edit rule (status/supersession line; register pointer annotation) is strictly narrower than the `AGENTS.md`/`docs/README.md` mandates and consistent with adr-000's tail rule; git diff for this branch confirms adr-000 is byte-identical (untouched by W06). |
| W06-DV06 → P0-V09 | Discovery and link review | **passed** | The extended "architecture-affecting work" routing row reaches baseline and process in one link each; governance links (toolchain/build-target/build-choice baselines, integration workflow, template, adr-000) all resolve; implementation-index row truthful. |
| W06-DV07 → W06 closure | Consumability review | **passed** | Read as a P1 planner with a conflict (§2 test + §3 labels + §4 path answer it), as W21 (escalation path referenceable), as W05 (ADR-class detail now defined, pointer resolves), and as a threshold-section owner (references resolvable); both labels route identically to a Proposed ADR. |

## Threshold-review samples (W06-DV02 evidence)

| Sample | §2 verdict | Basis |
|---|---|---|
| Routine toolchain pin bump to a newer released stable | **No ADR** — routine maintenance | Toolchain baseline's own thresholds list pin bumps as routine; §2's not-required clause covers governance-following maintenance |
| Activating the Reserved `embedded` profile | **No ADR — but a policy decision required** | Build-choice governance lists activation as a policy decision, below the ADR threshold; §2.3 applies only if its ADR-required limb were crossed (it is not) |
| Core code branching on a board name (the drill proposal) | **ADR required** | §2.1: conflicts with ADR-052/ADR-043 |
| Fixing a broken documentation link | **No ADR** | Documentation baseline routine maintenance; §2's not-required clause |

## Drill byte-change analysis (W06-DV03 evidence)

Had the hypothetical ADR-061 been accepted (it was not; no file exists):

- `docs/adr/README.md` §6 index: one row appended (ADR-061, Superseded→ per
  its Supersedes targets, state Accepted) — permitted edit.
- adr-000 register rows ADR-052/ADR-043: status-line pointer annotation only
  (e.g., "否决（被 ADR-061 取代…）" style pointer), decision text untouched —
  permitted edit under rule §4(b).
- ADR-052/ADR-043 decision text, all other register rows, principles,
  invariants: byte-identical — any further change would be a review failure.
- Negative case: the drill changed nothing (`git status` clean; no
  `adr-061-*.md` exists).

## Not run / not exercised

- **No real ADR proposed, no real conflict resolved:** the drill is
  hypothetical by design; no 待定 register item was decided.
- **No CI or lint enforcement of the lifecycle:** W07/W20 scope; none added.
- **No owner ruling on rule §4(b):** applied as designed (see the
  implementation record); declining it later is a review-level change that
  touches no accepted decision.
