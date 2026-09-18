# P0-W19 Reproducible Development Workflow — Verification Evidence

**Status:** Complete evidence recorded; W19 closure claimed.
**Date:** 2026-09-19 (Asia/Shanghai)
**Environment:** Disposable clone of branch `p0/w19-contributor-workflow` at
commit `8fff352` under `/tmp/w19-clone`; corroboration executions in the
isolated `RUSTUP_HOME`/`CARGO_HOME` sandbox (W02 restoration path).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W19-DV01 → P0-V01 | Fresh-clone walkthrough, S0 | **passed** (`reachable`) | From documents alone: root `README.md`, `AGENTS.md`, `docs/README.md` present in the clone; the routing table reaches every stage's contract. |
| W19-DV02 → P0-V02 | Fresh-clone walkthrough, S1 | **passed** (`reachable`) | The toolchain contract is in-tree with its §5 restoration inputs, verification commands, and failure boundaries; the workflow cites, not restates. Corroboration: the disposable clone's manifest provisioned the pinned toolchain (`1.98.1` active, overridden by `/tmp/w19-clone/rust-toolchain.toml`). |
| W19-DV03 → P0-V03/V04 | Fresh-clone walkthrough, S2 | **passed** (`reachable`) | The host-test baseline names the single entry and success evidence; the gate register binds it. Corroboration in the clone: entry executed, `test result: ok. 1 passed; 0 failed; 0 ignored`. |
| W19-DV04 → P0-V05 | Fresh-clone walkthrough, S3 | **passed** (`reachable`) | The build-target contract names the invocation and artifact boundary. Corroboration in the clone: target build finished; ELF produced. |
| W19-DV05 → P0-V13 | Fresh-clone walkthrough, S4 | **passed** (`reachable`) | The runner entry contract is in-tree; §8's placeholder boundary is stated verbatim in the workflow's S4 and §3.2; no execution implied. |
| W19-DV06 → P0-V09 | Boundary and citation review | **passed** | Workflow-document links all resolve in the clone (link pass: none broken); host/target non-interchangeability, artifact identification, and the enforcement status are stated as designed; no command spelling, flag, triple, workflow name, or policy paraphrase appears in the document. |
| W19-DV07 → P0-V08(scout)/W19 closure | Branch-to-PR walkthrough | **passed** | Every S6 lifecycle step classified with an owner (evidence below); the policy is cited without divergence; no step claims enforcement that does not exist. |

## Fresh-clone walkthrough outcome table (per §3.1 procedure)

| Stage | Outcome | Basis |
|---|---|---|
| S0 | reachable | entry docs present; routing table complete |
| S1 | reachable | toolchain contract §5 carries inputs/verification/failure boundaries |
| S2 | reachable | host-test baseline §3–§4 + gate register §1/§9 |
| S3 | reachable | build-target baseline §6 + artifact boundary §5 |
| S4 | reachable (placeholder) | runner entry contract §8; boundary stated |
| S5 | reachable | metadata §1–§3 + naming §3/§5–§6 |
| S6 | reachable | integration policy in-tree; S6 §4 adds contributor-side entry only |

Task-book ID mapping: P0-V01→S0, P0-V02→S1, P0-V03/V04→S2, P0-V05→S3,
P0-V13→S4 — all `reachable`; no `failed` outcome. Corroboration executions
(S1/S2/S3 in the disposable clone) are **personal-environment corroboration
only**; they add no claim beyond the owning packages' recorded evidence.

## Branch-to-PR walkthrough classification (per §3.2 procedure)

| Lifecycle step (S6 §4) | Classification | Owner / note |
|---|---|---|
| 1. Entry from the stage chain (validated tree before PR) | `human-process` | contributor; W19-added content |
| 2. Branch creation per policy namespace | `human-process` | contributor; policy cited, not restated |
| 3. Local validation before push (development set) | `human-process` | contributor; gates/host-test/target contracts cited |
| 4. Push and PR with policy-required description | `human-process` | contributor; remote configured and PR route in active use (17 merged PRs at walkthrough time) |
| 5. Required online checks | `pending-W20` | the required-check set does not exist yet; §3.4 states this truth and is retired by W20's delivery, not by an edit |
| 6. Merge by authorized maintainer via PR | `human-process` | merge method preserves the PR connection (merge commits, as practiced); direct pushes prohibited by policy |

No step claims an unimplemented check as configured or passing. P0-V08 is
not claimed by this walkthrough; its evidence is W20's.

## Not run / not proved

- **No CI/branch-protection configuration exists** (W20 scope); the
  enforcement-status statement stands until W20 delivers.
- **S4 execution:** impossible in P0 by design (placeholder boundary).
- **Walkthroughs prove discoverability, not execution**: corroboration runs
  are labelled as such and add no package-level claim.
