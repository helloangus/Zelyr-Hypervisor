# P0-W19 Integration Path and Walkthrough Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W19 detailed design](README.md).

## 1. Authority relationship to the integration policy

The [branch and pull-request integration
workflow](../../../../development/integration-workflow.md) is adopted normative
policy. W19 does not restate, rewrite, summarize into divergence, or amend it.
The workflow document's integration stage **references** the policy's required
change lifecycle and adds only what the policy does not carry: where a new
contributor enters that lifecycle from the stage chain, what each lifecycle
step looks like from the contributor's side, and the current enforcement
status. If the policy and the workflow document ever appear to disagree, the
policy governs and the workflow document is corrected in the same change.

## 2. Integration path (stage S6 content)

The workflow document's S6 section must present the path as the **mandatory
route for every post-policy change**, with these contributor-side elements:

1. **Entry from the stage chain:** S6 begins only from an environment that has
   reached S2 (and S3 when the change touches target buildability), so that a
   PR is opened for validated work, not for an unverified tree.
2. **Branch creation:** one branch per bounded change, based on current
   `main`, named per the policy's descriptive-namespace examples — cited, not
   restated.
3. **Local validation before pushing:** the minimum pre-push validation set
   cited from the owning contracts (gates per W07, host tests per W08, target
   build per W03 as applicable to the change), so the path does not substitute
   push-time CI for local diligence.
4. **Push and pull request:** the push targets the configured GitHub remote
   and the PR targets `main`, with the PR description content required by the
   policy (work package, changed contracts, validation run and not run,
   unresolved conflicts).
5. **Required online checks:** the policy's rule cited verbatim by reference —
   every configured required check must pass, and a missing, cancelled,
   skipped, or failed required check is not passing evidence — together with
   the enforcement-status statement of
   [the stage contracts](01-workflow-stage-contracts.md) §5.4: the configured
   set does not exist until W20 delivers it.
6. **Merge:** merge by an authorized maintainer through the PR per the
   policy's merge-method rule; direct pushes to `main` are prohibited by the
   policy and the workflow document must not soften that.

The section must close with the responsibility table:

| Responsibility | Owner |
|---|---|
| Branch hygiene, local validation, PR description truthfulness | contributor |
| Policy text and its future amendments | the policy's owning change (not W19) |
| GitHub workflows, required-check classification, branch protection, enforcement evidence | P0-W20 (P0-V08) |
| Merge decision | authorized maintainer |

## 3. Walkthrough design (plan work sequence 5)

Two walkthroughs validate that every path is completable from repository
documents alone. Both are discoverability exercises: they prove that the
documentation reaches every entry point, and they prove nothing about the
entries' successful execution.

### 3.1 Fresh-clone walkthrough

Procedure, recorded step by step in the verification record:

1. Clone (or freshly re-clone) the repository into a clean directory; note the
   checkout state (commit, branch).
2. Starting only from documents inside the clone (no shell history, no
   machine-local notes), follow the workflow document stage by stage, S0
   through S5.
3. At each stage record exactly one outcome:
   - `reachable` — the stage's contract document exists in-tree, its cited
     entry point is findable, and its expected evidence is stated clearly
     enough that a reader knows what success looks like;
   - `blocked` — the stage's contract document is not yet in-tree, or its
     entry point cannot be reached from documents alone (record the expected
     path and the owning package);
   - `failed` — the path is documented but internally inconsistent (broken
     link, contradiction with the owning contract, missing failure
     attribution).
4. Separately, and clearly labelled as personal-environment corroboration, the
   implementer may actually execute stages whose contracts are in-tree (for
   example S1's restoration exercise); such execution is recorded with full
   command/output and never reported as package validation or as P0-V
   evidence beyond what its owning package already claims.
5. Map each task-book validation ID P0-V01, P0-V02, P0-V03, P0-V04, P0-V05,
   and P0-V13 to the stage whose documented entry demonstrates its
   discoverability, recording for each ID the outcome above.

Passing condition: every validation ID maps to a `reachable` stage, or to a
`blocked` stage with the missing contract named — and no `failed` outcome
remains. A `blocked` outcome does not fail the walkthrough (it records the
honest dependency state); a `failed` outcome does.

### 3.2 Branch-to-PR walkthrough

Procedure, recorded step by step:

1. Follow the workflow document's S6 section against the policy text: confirm
   each lifecycle step is either cited from the policy or carries its
   contributor-side content per §2, and that nothing diverges from the
   policy.
2. Mark each lifecycle step as one of: `human-process` (executable today by
   following documents), `pending-W20` (the step's substance is GitHub
   enforcement that W20 will configure — required checks, protection), or
   `blocked-external` (for example no configured remote yet — W01's record
   documents remote publication as pending owner inputs).
3. Verify the enforcement-status section (§5.4 of the stage contracts) states
   the current truth and is written to be retired by W20's delivery, and that
   nowhere does the path describe an unimplemented check as configured or
   passing.
4. If the owner has configured a GitHub remote by walkthrough time, the
   implementer may optionally perform a real branch push and PR draft and
   record the observable check state; this is corroboration only, and W20
   still owns enforcement evidence (P0-V08). If no remote exists, the
   walkthrough records `blocked-external` for the push/PR steps and that is a
   truthful outcome, not a failure.

Passing condition: every lifecycle step is classified with an owner, the
policy is respected without restatement, and no step claims enforcement that
does not exist.

## 4. Evidence semantics for both walkthroughs

- Outcomes are **passed**, **failed**, **blocked**, or **not run**, each with
  the observation, the exact citation or missing path, timestamp, and
  environment.
- Documentation of a path is never recorded as execution of the path;
  execution, where performed, is labelled personal-environment corroboration.
- A walkthrough cannot prove P0-V08 (CI and PR enforcement) and must not be
  reported as doing so; P0-V08's evidence is W20's.
- Where a walkthrough outcome depends on a parallel package's pending
  delivery, the record names the package and the expected document path so
  the blocked mark is actionable.

## 5. Explicitly excluded procedures

No GitHub workflow file, check configuration, protection rule, command
invention to fill a pending stage, or policy amendment is authorized by the
walkthroughs. A gap found by a walkthrough is fixed by the owning package (or,
for assembly defects, by this package's document); it is never fixed by
improvising content into the workflow document.
