# P0-W16 Version/Build Metadata Baseline — Verification Evidence

**Status:** Complete evidence recorded; W16 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch `p0/w16-version-metadata`
(baseline: merge of PR #26).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W16-DV01 → P0-V14 | Question/schema review | **passed** | All seven identity questions present with declared sources (Q1–Q7); eleven-field schema with meaning, source/authority, and required/optional/reserved status; the required subset is explicit and no produced-artifact exemption exists. |
| W16-DV02 → P0-V14 | Policy review | **passed** | Determinism-first timestamp policy (revision-derived only; wall-clock auxiliary, never identity, never in names); dirty-tree policy with clean/dirty definitions, evidence-artifact clean-tree rule, and the explicit human-practice-until-CI statement; single-source project-version declaration (`0.1.0`) with the preserved migration rule. |
| W16-DV03 → P0-V14 | Reservation review | **passed** | `schema_version` / `machine_version` / `management_abi_version` reserved with no values, formats, or mechanics; independence from `project_version` and each other stated (ADR-040); the conflation prohibition is bidirectional. |
| W16-DV04 → P0-V14 | Cross-review: W12 | **passed** | W12's delivered association property references this baseline's minimum set by name; §5.1 names exactly that set (project_version, source_revision, dirty, target_architecture, build_profile); no divergence; both records carry the mutual cross-review entry. |
| W16-DV05 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the baseline in one link; links to delivered contracts (diagnostics baseline, build-choice governance, build-target baseline) resolve; stage-index row truthful. The W12→W16 forward reference resolves as of this package. |
| W16-DV06 → W16 closure | Consumability review | **passed** | W17 (view-not-authority rule + traceability obligation armed), W19 (clean-checkout rule quotable), W20 (clean-CI-build rule), P1 crash/logging designs (minimum inline set) — each can act without inventing policy. |

## Prerequisite-boundary checks (recorded)

- `platform`: the W03-delivered target boundary names the host/hypervisor
  classes but no platform vocabulary yet; per the schema's own rule,
  `platform` is required-but-unfilled — recorded, not waived.
- `build_profile` / `capability_summary`: W04 governance is delivered but no
  profile is implemented in-tree (Reserved set); the schema's
  required-once-in-tree and reserved-slot rules apply as written.
- Member-manifest version keys: confirmed non-authoritative package-local
  placeholders per the build-target baseline's handoff note; the single
  declaration lives in contract §3.3 (see the implementation record's
  deviation resolution).

## Not run / not proved

- **No artifact with embedded identity exists**; the schema is policy until a
  producing design exists (first target builds remain compile-chain
  validations).
- **W17 mapping-table cross-review:** armed, executes at W17 closure.
- **Reproducibility:** the determinism-first policy is stated; bit-level
  reproduction of artifacts is not claimed by P0.
