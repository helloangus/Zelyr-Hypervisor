# P0-W20 Implementation Workflow and Validation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W20 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tree. Useful read-only
discovery: `git ls-files` (confirm `.github/workflows/` holds only a
`.gitkeep` marker and that no CI contract document exists), and a check of the
delivered prerequisite surfaces listed in step 1. The implementing machine may
or may not have rustup, `git push` access, or administrator access; those are
machine- or role-local facts, recorded as observed, never assumed.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the delivered W07 register uses different gate IDs, labels, or membership
  than this design's mapping — follow the delivered register and reconcile in
  the same change; never map a check W07 does not define;
- a delivered entry contract (W03/W08) changed its spelling or semantics —
  re-record per the [CI contract](01-ci-contract.md) §9 minor-change rule;
  never adapt the gate by editing the workflow beyond the delivered spelling;
- the repository remote or administrator access is missing — follow
  [the protection design](02-protection-and-enforcement.md) §4 blocked states;
  do not simulate GitHub behavior or record unobserved settings;
- a required choice would weaken or skip a required check, or alter the
  integration policy — that is the ADR-threshold boundary of
  [the CI contract](01-ci-contract.md) §9, not an implementation decision.

## 2. Ordered implementation steps

### Step 1 — audit prerequisite delivered surfaces

Target: implementation record (created in this step).

Work: determine, for each consumption point, whether the owning contract has
delivered and what its delivered form says. Audit list, with expected document
paths: W07 gate register and standards (`docs/development/quality-gates.md`),
W02 manifest and contract (`rust-toolchain.toml`,
`docs/development/toolchain-baseline.md`), W03 build entry
(`docs/development/build-target-baseline.md`), W08 host-test entry
(`docs/testing/host-test-baseline.md`), W19 contributor workflow
(`docs/development/contributor-workflow.md`), W17 naming contract
(`docs/development/artifact-naming.md`). Record each as delivered / absent /
differing-from-design, with the relevant spelling or label taken from the
delivered text where present.

**Acceptance:** the record names every consumption point with its status and
the exact delivered values the workflow will use.  
**Failure/blocker:** an absent prerequisite blocks only its own check
(protection design §4); it does not authorize substituting an approximate
command.

### Step 2 — author the CI contract document

Target: `docs/development/ci-baseline.md`.

Work: write the normative document with the status header required by
`docs/README.md` (status, scope, version `v0.1`, owner/change context,
supersedes: none) and the required content of [the CI
contract](01-ci-contract.md) §2–§9: single-source and mirror rules, the
check-mapping register filled from step 1's delivered values, the trigger and
run model, per-gate realization rules, the future-class table and presentation
rules, failure/artifact/observability rules, the security and permissions
model, and the mutation thresholds. Reference the integration policy and the
W07 register; restate neither.

**Acceptance:** every required section present; no contradiction with the
policy, the register, or the delivered entries; no gate semantic redefined.  
**Failure/blocker:** a contradiction is raised per §1, not absorbed by
rewording.

### Step 3 — create the check workflow

Target: `.github/workflows/ci.yml` (the tracked `.gitkeep` marker is removed
in the same change that adds the first real workflow).

Work: implement the workflow per the contract document: the two triggers, one
job per required check named by its evidence label, provisioning through the
repository manifest, steps executing the delivered entries verbatim, inline
QG-DOCS steps implementing the three W07 §7 semantics, bounded job timeouts,
least-privilege permissions, and the pointer comment to the contract document.
No third-party action, no secret, no path filter.

**Acceptance:** the workflow realizes the register 1:1 and contains no policy
prose.  
**Failure/blocker:** a mismatch between workflow and contract fails review;
fix the workflow, not the contract (the contract changes only through §9).

### Step 4 — local dry-run and syntax validation

Target: verification record (evidence entries only).

Work: validate the workflow syntax with the available local mechanism and
execute locally every check step whose prerequisite surface exists (formatting,
lint, host tests, target build, the QG-DOCS steps). Record commands, output,
environment, and result.

**Acceptance:** every locally executable step passes on the current tree;
syntax validation reports a parseable workflow.  
**Failure/blocker:** a failing local step is a real gate failure — fix the
tree or the delivered entry, never the gate semantics. **These runs are
preliminary evidence only; the plan's acceptance wording explicitly denies
that they prove P0-V08.**

### Step 5 — configure branch protection

Target: GitHub repository settings for `main`.

Work: apply [the protection design](02-protection-and-enforcement.md) §2:
require PRs, the six required checks in strict mode, merge-method enablement
per the policy rule, direct-push/force-push/deletion rejection, administrator
inclusion where the settings permit. Record the observed settings values in
the implementation record.

**Acceptance:** settings mirror the contract document field-by-field.  
**Failure/blocker:** a blocked state (no remote, no administrator access,
platform limitation) is recorded per protection design §4; nothing is
improvised.

### Step 6 — enforcement-evidence exercise

Target: verification record.

Work: perform the four-observation exercise of
[the protection design](02-protection-and-enforcement.md) §5 on the real
GitHub repository: green path, red probe with merge refusal, restored green
path, direct-push rejection. Record every observation with its PR/push
reference, commit, timestamps, and output.

**Acceptance:** all four observations are recorded, each with an explicit
proves / does-not-prove statement; the probe branch is fully reverted and
never merged.  
**Failure/blocker:** an observation that cannot be made (blocked state) is
recorded as blocked with its cause; the package is not closable until the
blocked observation is made or the owner accepts a recorded limitation.

### Step 7 — same-change reconciliation and discovery wiring

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`,
`docs/development/contributor-workflow.md` (if delivered).

Work: add one `docs/README.md` routing row pointing CI/required-check/
protection work at the contract document; add the W20 stage-index row with
truthful status; retire the W19 enforcement-status statement (§5.4 of its
stage contracts) in the same change once enforcement is evidenced, replacing
"not yet configured" with a pointer to the CI contract. Change nothing else.

**Acceptance:** a newcomer reaches the CI contract in one link from
`docs/README.md`; the W19 document no longer claims enforcement is absent;
all new links resolve.  
**Failure/blocker:** if W19 has not delivered, the retirement is recorded as
pending with the exact location named; the routing and index rows still land.

### Step 8 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
check the package against its task-book requirement (P0-V08), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what was
actually observed.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W20-DV01 → P0-V08 | mapping review | inspect `docs/development/ci-baseline.md` and `ci.yml` against the delivered W07 register | 1:1 mapping, labels verbatim, classes equal, no extra check, no policy prose in the workflow | a truthful mapping exists; not that GitHub executes anything |
| W20-DV02 → P0-V08 | local dry-run | run each locally executable check step; syntax-validate the workflow | steps pass on the current tree; workflow parses | the steps execute the real gates locally; explicitly not GitHub execution and never sufficient for P0-V08 alone |
| W20-DV03 → P0-V08 | future-class presentation review | search workflow, contract, and documents for any check or wording implying QEMU/EL2/Linux/hardware coverage | no configured check covers future-class scope; statements say "not verified — future class"; enablement conditions written down | honesty of classification; not that future checks will work when promoted |
| W20-DV04 → P0-V08 | settings-vs-contract mirror review | observe `main` protection settings on GitHub; compare field-by-field with the contract's protection section | every requirement mirrored; residual bypass capability recorded as policy-bound where applicable | intended protection is configured; not that it blocks (DV06 does) |
| W20-DV05 → P0-V08 | real-PR green-path evidence | the §5 exercise observation 1 | all six checks present, executed, and passing on a real PR on GitHub | PR-level execution and visibility; not merge-blocking behavior |
| W20-DV06 → P0-V08 | red-probe and direct-push evidence | §5 exercise observations 2 and 4 | GitHub refuses merge while a required check fails; direct push rejected (or recorded as policy-bound) | enforcement is real; not that future checks are covered or that gates themselves are correct |
| W20-DV07 → P0-V08 | failure-localization review | inspect the red-probe failure output and retained log/artifact | output carries gate label + commit ref; log retained; build-log artifact present for QG-BUILD-TARGET failures | diagnosability of failures; not correctness of the gated code |
| W20-DV08 → P0-V09 | discovery and link review | resolve the routing row, contract links, and stage-index row from a fresh checkout; confirm W19 reconciliation | one-link reachability; truthful statuses; all links resolve; W19 statement retired or pending with named path | documentation coherence; not W05's taxonomy ownership |
| W20-DV09 → W20 closure | consumability review | read the delivered configuration as W22 (map row input), P1 planners (extension path), and a PR author (failure semantics) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command or observation, input, environment, timestamp, and reason. Local
validation without the §5 exercise does not satisfy P0-V08. No validation
here proves P0-V01–V07 or P0-V09's non-document portions and none may be
reported as doing so.

## 4. Error, security, and observability model

W20 adds no hypervisor error model, synchronization, guest input path, or
`unsafe` code. Its failure reporting is behavioral and textual: a failing
check, a settings drift, a blocked exercise observation, or a missing
prerequisite fails the associated review and is recorded as such — never
silently degraded (a skipped or weakened required check is itself the defect).

The security surface has three layers: supply chain (first-party steps and
the official rustup installer only; third-party actions Reserved behind W18
governance), privilege (least-privilege workflow, no secrets, secrets gated
behind a future exposure design), and enforcement integrity (protection
configured to the strictest available set, residual bypass recorded as
policy-bound rather than denied). The unstable/supply-chain posture mirrors
the W02 contract's fail-closed approach.

Observability is the deliverable named by work sequence 5: gate-labelled,
commit-attributed, retained failure output and artifacts, plus the
verification record's observations as the only accepted proof surface for
P0-V08.

## 5. Handoff checklist

Before handing W20 to a reviewer, provide:

- the exact changed-file list, including the workflow file and contract
  document;
- the observed protection settings, the enabled merge-method set, and the
  recorded job timeouts;
- DV01–DV09 evidence paths with run status, including explicit blocked/not-run
  entries (for example a missing remote) and the four §5 observations with
  their PR references;
- confirmation that no gate semantic was redefined, no third-party action or
  secret added, no path filter or skip condition introduced, and no policy
  text amended;
- confirmation that the probe branch was reverted and never merged;
- open items for W22 (the P0-V08 row of the handoff map), W19 (retirement
  confirmation), W07 (any register conflict raised), and P1 (the future-class
  extension path) — without resolving their contracts here.
