# Zelyr CI Baseline

**Status:** Normative CI policy.  
**Scope:** The check-mapping register (the CI view of the quality-gates
register), the trigger and run model, per-gate realization rules, the
future-class classification, failure/observability rules, the security
model, and the `main` protection requirements. It does not define gate
semantics or own gate command spelling (the [quality
gates](quality-gates.md) own both); the workflow file carries data plus a
pointer comment and no policy prose.  
**Version:** v0.1  
**Owner/change context:** P0-W20 CI baseline; implements the deferred
technical enforcement of the [integration
workflow](integration-workflow.md) without amending it.  
**Supersedes:** The absence of repository CI (`.github/workflows/` held only
a marker).

## 1. Single-source and mirror rules

- This document is the only tracked file that may state a check's
  classification, a trigger rule, or a protection requirement. The workflow
  file (`.github/workflows/ci.yml`) mirrors this document as data; no other
  tracked file may define or restate a check, trigger, or protection rule.
- The GitHub settings must mirror this document. Settings are invisible to
  pull-request review, so the mirror is enforced by observation: the
  verification record compares each setting against this document
  field-by-field, and drift is a defect fixed in the change that repairs it.
- If the workflow's behavior and the quality-gates register ever disagree,
  the register wins; the conflict goes through the register's mutation
  thresholds, never absorbed by a local workflow edit.

## 2. Check-mapping register

One register row, one GitHub check, named verbatim by the evidence label; no
aliases, prefixes, or decorations. Branch protection references the six names
exactly. The class column equals the quality-gates register's class at all
times; when that register changes through its thresholds, the workflow and
this mapping change in the same change. No check may exist for a subject
that is not a register member.

| Check name (= gate evidence label) | Gate | Class | Required on PR | Required on `main` push | Realization |
|---|---|---|---|---|---|
| `QG-FMT` | Formatting | Required | yes | yes | formatter check mode over the workspace under the pinned toolchain |
| `QG-LINT` | Lint | Required | yes | yes | whole-workspace lint analysis (host-class and bare-metal spellings) under the recorded warning policy |
| `QG-WARN` | Warning policy | Required | yes | yes | gate-bearing builds evaluated for the zero-warnings condition under the delivered enforcement configuration |
| `QG-TEST-HOST` | Host tests | Required | yes | yes | the delivered host-test execution entry, unmodified |
| `QG-BUILD-TARGET` | AArch64 target build | Required | yes | yes | the delivered target-build entry; success requires exit success and the declared artifact present |
| `QG-DOCS` | Documentation consistency | Required | yes | yes | inline workflow steps implementing the quality-gates §7 three check semantics |

Rules: each check reports its own result — two checks may execute the same
build (QG-WARN and QG-BUILD-TARGET over the target build), but each
evaluates and reports its own gate semantics, and a check must not claim a
gate it did not fully evaluate. Subjects without register rows are not CI
checks: a P0 validation subject the register does not carry (artifact
identification per P0-V14, the QEMU placeholder review per P0-V13) keeps its
evidence in its owning package's verification record and gains no check
here.

## 3. Trigger and run model

- **Triggers:** `pull_request` targeting `main`, and `push` to `main` — so
  PR-level and mainline required checks both execute and are visible.
- **No conditional skipping:** no path filter, draft-PR condition, or `if:`
  condition may prevent a required check from running on a PR. Missing,
  cancelled, skipped, failed, or timed out is not passing evidence. The one
  permitted cancellation is a superseded in-progress run on the same PR
  (only the head commit's results are evaluated).
- **Head-commit rule:** merge eligibility is determined by the required
  checks on the PR head commit; results from older commits never count.
- **Bounded runtime:** every check job carries a recorded job-level timeout
  so a hang surfaces as a failure, not as silence. Recorded values: 15
  minutes per job (see the implementation record).

## 4. Realization rules per gate

- **Provisioning (all gates):** CI installs rustup through its official
  installer invocation as documented in the [toolchain
  baseline](toolchain-baseline.md), then lets any toolchain-driven
  invocation provision the pinned toolchain, components, and targets from
  the repository `rust-toolchain.toml`. CI must not install, cache in a
  bypassing way, or select any toolchain version, component, or target
  outside the manifest (the toolchain contract's parity rule).
- **QG-FMT / QG-LINT:** the spellings recorded in the [quality
  gates](quality-gates.md) document are used verbatim; a check fails on any
  finding. Fixing is by reformatting or code change, never by weakening the
  invocation.
- **QG-WARN:** executes the gate-bearing builds (host build, host tests,
  target build) under the delivered enforcement configuration; the result is
  the zero-warnings condition. Passing builds with warnings reported
  elsewhere are not a pass for this gate.
- **QG-TEST-HOST:** executes the delivered host-test entry exactly as
  spelled in its contract; success is exit success with zero failures and
  zero unreported skips. No interpretation is added.
- **QG-BUILD-TARGET:** executes the delivered target-build entry; success is
  exit success with the declared baseline artifact present. Compilation
  success never substitutes for execution evidence; this check says nothing
  about EL2, QEMU, or hardware behavior.
- **QG-DOCS:** inline workflow steps implementing the three semantics of the
  quality-gates §7: (1) every relative link in tracked documentation
  resolves from a fresh checkout — exempting exactly the recorded class of
  forward references inside a stage's implementation designs that point at
  that stage's own future verification/record paths (the exemption the
  quality-gates contract records; any other broken link fails); (2) every
  normative development-governance document is reachable from the repository
  entry points within four link hops; (3) normative documents carry the
  status header required by the documentation baseline (presence only).
  Mechanisms are restricted to shell, Git, and text tools already available
  in the baseline environment — no new tooling dependency (dependency
  governance would apply first). If the logic outgrows inline realizability,
  extraction into a committed gate-runner script is a Reserved change
  requiring a reviewed design.

## 5. Future-class classification (no row is configured at P0)

The table exists so the absence is explicit and the enablement path is
written down.

| Subject | Class at P0 | Expected owner | Enablement conditions |
|---|---|---|---|
| QEMU runner execution (smoke/regression) | Future | QEMU runner entry contract; first implementation expected P1-W10 | a runner implementation exists and a quality-gates register row is promoted; the workflow gains the job in the same change |
| EL2 boot smoke | Future | P1 designs | same promotion path; never presented as covered before promotion |
| Linux guest regression | Future | P8+ designs | same promotion path |
| Real-hardware (Orange Pi 3B) checks | Future / manual | P15+ designs | same promotion path plus platform/runner prerequisites owned by those stages |
| Fuzz/property checks | Future | later stages per ADR-049 | same promotion path |
| Unsafe-inventory consistency | Future | unsafe-policy trigger (first `unsafe` in tree) | promotion per the quality-gates register |
| rustdoc warnings, dependency-audit reporting | Informational candidates | quality-gates / dependency governance | membership may be empty; an informational check never silently converts to required |

Presentation rules: no check name may exist whose semantics imply coverage
of a future-class subject; no green check may represent a future check.
Documents and PR descriptions may state only that such scope is **not
verified — future class**. The P0 checks prove nothing about EL2, guest,
QEMU, or hardware behavior; PR checks prove only the configured gates.
Promotion always passes through the quality-gates register's thresholds and
lands the workflow edit in the same change.

## 6. Failure, artifact, and observability rules

- **Attribution:** every failing check's visible output carries the gate
  label and the commit/PR reference it ran on, with enough output to
  diagnose without re-running.
- **Retention:** failing jobs retain their complete logs under the
  platform's default retention. A failing QG-BUILD-TARGET additionally
  prints its complete compiler/linker build log into the job log as its
  failure artifact (a dedicated upload action would be a third-party
  marketplace action, which §7 does not authorize; enabling one is a
  Reserved change through dependency governance). Artifact/log names are
  machine-processable and unique per run per the [artifact naming
  baseline](artifact-naming.md). Durable evidence lives in stage
  verification records, not in CI; no long-term artifact storage exists at
  P0.
- **Flakiness:** a required check with nondeterministic outcomes is a
  defect. A single rerun that changes the outcome is recorded in the PR with
  both results; systematic rerun-until-green is prohibited.
- **No silent degradation:** the required set may not be reduced, reordered
  into optionality, or made skippable inside a change that would benefit
  from the reduction; such changes go through §8's thresholds.

## 7. Security and permissions model

- **Least privilege:** the workflow runs with `contents: read` and no
  write-facing permission. No secret is required or used at P0. A future
  design introducing secrets must design its PR/fork exposure model first
  (Reserved trigger).
- **Supply chain:** the workflow uses only first-party steps plus the
  official rustup installer invocation; no third-party marketplace action is
  authorized at P0. Introducing one is a Reserved change governed by the
  [dependency governance](dependency-governance.md) once followed for such
  dependencies.
- **Untrusted input:** pull-request-supplied code executes in CI. At P0 no
  credential exists to leak, so the exposure is bounded to compute abuse,
  which bounded job timeouts and public-runner defaults address; the
  residual risk is recorded here rather than solved by speculative controls.

## 8. `main` protection requirements

The live GitHub settings mirror this section; the policy relationship is by
reference (the [integration workflow](integration-workflow.md) owns the
rules; this document adds the concrete settings).

1. **PR-only integration:** a pull request is required before merging;
   `main` is the sole integration branch. Direct commits, force pushes, and
   deletion of `main` are rejected.
2. **Required status checks:** exactly the six check names of §2, in strict
   mode — a check that has not reported success does not pass, so a missing,
   cancelled, skipped, failed, or timed-out required check blocks merge (the
   policy's rule, made mechanical).
3. **Bypass honesty:** no bypass allowance is granted to any role, app, or
   push rule beyond what the platform itself reserves; administrators are
   included in the restrictions where the repository settings permit. Where
   a plan or platform limitation leaves an administrator bypass capability
   in place, that residual capability is recorded in the verification
   evidence as **policy-bound, not technically bound** — the policy's rule
   that an administrator's ability to bypass is not permission to bypass
   remains the control. The evidence never describes bypass as impossible.
4. **Merge methods:** at least one enabled merge method preserves a
   reviewable connection between the merged change and its PR (merge
   commits). Methods that can sever the PR association (squash, rebase) are
   disabled. The enablement set is recorded in the implementation record.
5. **Up-to-date requirement:** "require branches to be up to date before
   merging" is **not** imposed at P0 — it serializes merges disproportionately
   for the project size, and base-branch drift is detected by the `main`-push
   runs of the same check set. Escalating to require-up-to-date is a
   design-level change under §9, taken if drift produces incidents.

## 9. Mutation thresholds

- **Recorded minor change** (ordinary PR review, noted in the implementation
  record): renaming the workflow file with its pointer updated; recording
  job-timeout values; recording failure-artifact naming; re-recording a
  bound entry spelling after the owning contract's own recorded minor
  change; recording the merge-method set.
- **Design-level change** (recorded against this document,
  reviewer-approved): adding or removing a check row; changing the trigger
  model; changing a protection requirement; changing failure-visibility or
  timeout normativity; enabling the up-to-date option.
- **ADR required:** making a hypervisor-TCB-affecting check optional,
  skippable, or advisory; weakening enforcement below the integration
  policy's rule; any change that would alter an accepted architecture
  decision per the [ADR process](../adr/README.md).
