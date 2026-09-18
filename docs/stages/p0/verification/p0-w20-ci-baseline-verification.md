# P0-W20 CI Baseline — Verification Evidence

**Status:** Complete evidence recorded; W20 closure claimed.
**Date:** 2026-09-19 (Asia/Shanghai)
**Environment:** GitHub Actions runs on helloangus/Zelyr-Hypervisor; local
dry runs in the W02-restored sandbox; GitHub REST API observations for
settings.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W20-DV01 → P0-V08 | Mapping review | **passed** | The contract's register maps 1:1 to the workflow's six jobs; labels verbatim; classes equal the W07 register; no extra check exists; the workflow carries a pointer comment and no policy prose (YAML inspected). |
| W20-DV02 → P0-V08 | Local dry-run | **passed** | Workflow parses (YAML); every gate step executed green locally before the PR (fmt; clippy both spellings with `-D warnings`; zero-warnings builds; host-test entry; target build + artifact presence; QG-DOCS inline script — which also failed correctly on injected breakage outside the recorded exemption and on missing headers, both fixed same-change). Explicitly local; not GitHub execution. |
| W20-DV03 → P0-V08 | Future-class presentation review | **passed** | No configured check implies QEMU/EL2/Linux/hardware coverage; the contract's §5 table marks all such subjects Future/Informational with enablement conditions; nothing presents them as covered. |
| W20-DV04 → P0-V08 | Settings-vs-contract mirror review | **passed** | API observation field-by-field against contract §8: six required contexts exactly; `strict:false`; `enforce_admins:true`; PR required (approving count 0); force pushes and deletion disabled; up-to-date not imposed; merge methods = merge-commit only. Residual capability: the repository owner is also the administrator — recorded as **policy-bound where the platform permits owner action outside the push path** (see the observation note below); never described as impossible. |
| W20-DV05 → P0-V08 | Real-PR green-path evidence | **passed** | PR #31 (the workflow itself) executed all six checks on GitHub; all reported success (`gh pr checks 31`: QG-BUILD-TARGET/QG-DOCS/QG-FMT/QG-LINT/QG-TEST-HOST/QG-WARN pass). PR #32 re-demonstrated the full green board after restore (all six pass, mergeStateStatus `CLEAN`). |
| W20-DV06 → P0-V08 | Red-probe and direct-push evidence | **passed** | **Red probe (PR #32):** a single-gate formatting violation made exactly `QG-FMT` report failure while the other five passed; `mergeStateStatus: BLOCKED`; the merge attempt was refused by GitHub (merge only offered via `--auto`/`--admin`, neither used). **Restore:** reverting the probe returned all six checks to pass and state `CLEAN`; both results recorded together. **Direct push:** a push of a *new, unchecked* commit to `main` was rejected — `! [remote rejected] ... (protected branch hook declined)` — in the administrator context (`enforce_admins: true`). Observation note: an earlier push whose tip was the *head of open PR #32 with all six required checks passing* was accepted by the platform (equivalent to merging that PR; no unchecked commit reached `main`); recorded as the platform's merge-equivalence behavior, with the policy rule (`main` accepts changes through PRs with required checks passing) substantively held. |
| W20-DV07 → P0-V08 | Failure-localization review | **passed** | The failing QG-FMT job's retained log carries the gate label (`QG-FMT on a9ea8b62…`), the commit reference, and the rustfmt diff locating the violation (crates/host-test-baseline/src/lib.rs:17) — diagnosable without re-running. |
| W20-DV08 → P0-V09 | Discovery, link, and reconciliation review | **passed** | Routing row reaches the CI baseline in one link; links resolve; the W19 enforcement-status statement was retired in the same change per its own rule and now cites the delivered register. |
| W20-DV09 → W20 closure | Consumability review | **passed** | W22 (the check-mapping register is map-row input), P1 planners (future-class promotion path is written), a PR author (failure semantics: gate label + commit ref + retained log; merge blocked until green) — each can act without inventing policy. |

## Enforcement-evidence timeline (protection-design §5 observations)

| # | Observation | Reference | Outcome |
|---|---|---|---|
| 1 | Green path | PR #31 checks (run 35374836828); PR #32 post-restore (run 35375601807) | six checks present, executed on GitHub, passing; merge available at `CLEAN` |
| 2 | Red probe | PR #32 probe commit `7b520c3` (run 35375149806) | exactly `QG-FMT` fail; others pass; `mergeStateStatus: BLOCKED`; merge refused |
| 3 | Restore | PR #32 revert `6d40b64` (run 35375601807) | all six pass; `CLEAN`; both probe results recorded together |
| 4 | Direct-push rejection | push of unchecked commit to `main` | `remote rejected ... (protected branch hook declined)` in administrator context; the platform's acceptance of an already-PR-headed, fully-checked tip is recorded as merge-equivalence behavior |

Probe discipline held: the probe lived only on the exercise branch, was
never merged, was fully reverted, and edited no gate or workflow.

## Not run / not proved

- **Future-class checks** (QEMU, EL2, Linux guest, hardware, fuzz, unsafe
  inventory): not configured and not claimed — "not verified — future
  class" per contract §5.
- **Gate semantics correctness:** CI proves the gates execute and block;
  the gates' meaning is the quality-gates contract's.
- **Non-admin rejection path:** all observations ran in the owner/admin
  context; rejection was nonetheless technical (`protected branch hook
  declined`, `enforce_admins: true`), so the policy-bound caveat is limited
  to owner-level settings changes outside the push path.
