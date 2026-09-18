# P0-W04 Build Profile / Feature Governance — Verification Evidence

**Status:** Complete evidence recorded; W04 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentation review and classification drill against the
tracked tree at branch `p0/w04-build-profile-governance` (baseline: merge of
PR #14). No build, gate, or CI execution is applicable to this package.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W04-DV01 → P0-V09 | Classification review | **passed** | The governance document defines exactly three classes (§2), the runtime-policy-wins precedence rule (§2.4), and an ordered procedure whose every branch ends in a verdict (§3); ADR-047/037/046 are cited as the semantic authority and their semantics are preserved (feature = binary capability; configuration layering untouched). |
| W04-DV02 → P0-V09 | Profile registry review | **passed** | §4.1 records all six ADR-047/task-book names as Reserved with one-line purposes and applicability boundaries, marked refinable by the implementing design; §4.2 states the non-fork rule; no statement implies any profile is implemented, selected, or named in a build path. |
| W04-DV03 → P0-V09 | Prohibited-case review | **passed** | §7 prohibits all five plan-named categories (VM count/limits, memory sizing, vCPU count/topology, affinity/pinning, device selection) as features, profile-intrinsic constants, or any compile-time form; the illustrative violations are marked informative; authority is cited (ADR-047 final clause, ADR-037 layering) rather than re-derived. |
| W04-DV04 → P0-V09/P0-V15 | Representative-decision drill | **passed** | The §3 procedure applied to four hypothetical switches yields unambiguous verdicts, including two rejections; question-by-question evidence below. No sample required a rule outside the document. |
| W04-DV05 → P0-V09 | Discovery and link review | **passed** | `docs/README.md` routing row reaches the governance document in one link; its links resolve (ADR baseline, build-target and toolchain baselines); implementation-index row updated truthfully. |
| W04-DV06 → P0-V15 | Consumability review | **passed** | Read as W03 (baseline compliant by construction; future switches classify here), W07 (class vocabulary for gate matrices), W16 (profile identifier dimension owned in §4.1), W12 (visibility references §4, not ad-hoc cfg names), and a P1 designer (§3 procedure + §5 checklist decide a proposal before it becomes code); each consumer can act without inventing policy. |

## Representative-decision drill (W04-DV04 evidence)

| Sample | Procedure path | Verdict | Deciding rule |
|---|---|---|---|
| (a) "Include the experimental virtio backend in the binary" | Q1: no runtime quantity → Q2: changes what the binary can do, build-time-legitimate, owning module exists | **Class 1 — binary capability**; record as documented feature with default state and owning design | §2.1, §3 Q2 |
| (b) "A named build for constrained deployments" | Q1: no → Q2: selects capabilities rather than adding one → Q3: coherent named selection for a deployment class | **Class 2 — profile**; maps onto the registered `embedded` profile; no new profile needed | §2.2, §3 Q3, §4.1 |
| (c) "Maximum number of supported VMs" | Q1: yes — bounds a runtime count | **Class 3 — runtime resource/policy**; **rejected as a feature**; routed to the future runtime configuration model (ADR-037) | §2.3, §3 Q1; §7 category 1 |
| (d) "A profile that hard-codes a specific VM's device set" | Q1 inside the proposal: device selection for a specific VM is a runtime quantity; combined with a feature/profile form | **Prohibited combination — split or reject**; the capability part may re-enter as a feature, the device selection belongs to runtime configuration; a profile-intrinsic device set is rejected | §2.4 precedence rule; §3 final clause; §7 category 5 |

## Not run / not exercised

- **No real-switch classification:** no Cargo feature or profile exists in the
  tree; the drill used hypothetical switches only, as the design requires. A
  real classification belongs to the proposing change's review.
- **No gate/CI enforcement:** making the rules machine-enforced belongs to
  W07/W20; this package adds no lint, check, or workflow.
- **No profile build:** the Reserved set is protected on paper; no profile
  compiles, and none may in P0 without the §6 thresholds.
- **No manifest change:** W04 introduced no manifest and modified none.
