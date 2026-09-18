# P0-W20 CI Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W20 detailed design](README.md).

## 1. Logical artifact groups and ownership

W20 is configuration and policy work, so its logical modules are authoritative
artifact groups, not code modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| CI contract document | `docs/development/ci-baseline.md` | this design, the delivered W07 register, the W19 stage mapping, the integration policy | the sole normative home of the check-mapping register, trigger model, protection requirements, future-class rules, and failure/evidence conventions; it does not define gate semantics, own gate command spelling, or contain workflow YAML |
| Check workflow | `.github/workflows/ci.yml` | the contract document, the delivered W02/W03/W07/W08 entries | the machine-consumed realization: triggers, jobs named by gate labels, steps executing delivered entries and inline QG-DOCS semantics; it carries a pointer comment to the contract and no policy prose |
| Branch protection configuration | GitHub repository settings for `main` | the contract document's protection requirements | the live enforcement state; not reviewable in pull requests, therefore mirrored from the contract and proven per [the protection design](02-protection-and-enforcement.md) §3 |
| Documentation routing | one row in `docs/README.md`; stage index row in `docs/stages/p0/implementation/README.md` | contract location / design status | discoverability and truthful status; no restatement |
| Implementation record | `../p0-w20-ci-baseline-record.md` (created when work starts) | decisions taken, observed protection settings, merge-method set, deviations | changed artifacts and configuration facts; no command logs (those live in verification) |
| Verification record | `../../verification/p0-w20-ci-baseline-verification.md` (created when evidence exists) | exercise observations, run outputs, settings observations | run/failed/blocked/not-run evidence per the validation matrix; the only location where completion may be claimed |

The artifact named in the second column is the sole authoritative home for the
statement in its row. The record and verification paths are future locations;
this design does not create them.

## 2. Single-source and mirror rules

- The contract document is the only tracked file that may state a check's
  classification, a trigger rule, or a protection requirement. The workflow
  file carries data plus a pointer comment, mirroring the W02 manifest
  pattern; no other tracked file may define or restate a check, trigger, or
  protection rule.
- The GitHub settings must mirror the contract document. Because settings are
  invisible to pull-request review, the mirror is enforced by observation:
  the verification record compares each setting against the contract, and any
  drift is a defect fixed in the same change that introduced it.
- If the workflow's behavior and the W07 register ever disagree, the register
  wins; the conflict is raised through the register's mutation thresholds, not
  absorbed by editing the workflow locally.

## 3. Check-mapping register (the CI view of the W07 register)

The contract document must contain this mapping as its normative register.
It consumes the [W07 gate
register](../p0-w07-development-quality-gates/01-gate-register.md) §2 as data
and adds only the CI realization columns.

| Check name (= W07 evidence label) | Gate | Class | Required on PR | Required on `main` push | Realization | Realization owner |
|---|---|---|---|---|---|---|
| `QG-FMT` | Formatting | Required | yes | yes | formatter check mode over the workspace under the pinned toolchain | W07 contract (spelling); W20 (execution) |
| `QG-LINT` | Lint | Required | yes | yes | whole-workspace lint analysis under the recorded warning policy | W07 contract (spelling); W20 (execution) |
| `QG-WARN` | Warning policy | Required | yes | yes | gate-bearing builds evaluated for zero warnings under W03's delivered enforcement configuration | W03 build baseline (mechanics); W07 (policy); W20 (execution) |
| `QG-TEST-HOST` | Host tests | Required | yes | yes | the delivered W08 host-test execution entry, unmodified | W08 host-test contract; W20 (execution) |
| `QG-BUILD-TARGET` | AArch64 target build | Required | yes | yes | the delivered W03 target-build entry; success requires exit success and the declared artifact | W03 target-baseline contract; W20 (execution) |
| `QG-DOCS` | Documentation consistency | Required | yes | yes | inline workflow steps implementing W07 §7's three check semantics | W07 (semantics); W20 (realization) |

Register rules:

- The mapping is 1:1: one register row, one GitHub check, named verbatim by
  the evidence label. No aliases, prefixes, or display decorations; branch
  protection references the six names exactly.
- The class column must equal the register's class at all times. When the
  register changes through W07's thresholds, the workflow and this mapping
  change in the same change.
- No check may exist for a subject that is not a register member. A
  future-class candidate has no row here until promoted (§6).
- Each check reports its own result. Two checks may execute the same build or
  entry (for example QG-WARN and QG-BUILD-TARGET over the target build), but
  each evaluates and reports its own gate semantics; a check must not claim a
  gate it did not fully evaluate (register §4).
- **Subjects without register rows are not CI checks.** A P0 validation
  subject that W07's register does not carry as a gate — for example artifact
  identification (P0-V14, owned by W16/W17's own validation) or the QEMU
  runner entry's placeholder review (P0-V13, owned by W09) — keeps its
  evidence in its owning package's verification record and gains no check
  here. The [W19 stage
  chain](../p0-w19-reproducible-development-workflow/01-workflow-stage-contracts.md)
  is consumed as the checklist of what CI must cover and classify; a stage
  without a register row is classified future/informational per §6, never
  silently checked.

## 4. Trigger and run model

The contract document must fix:

- **Triggers:** `pull_request` with `main` as target branch, and `push` to
  `main`. This realizes P0-V08's requirement that PR and mainline required
  checks both execute and are visible.
- **No conditional skipping:** no path filter, draft-PR condition, or `if:`
  condition may prevent a required check from running on a PR. A check that
  is missing, cancelled, skipped, failed, or timed out is not passing
  evidence (integration policy, restated as behavior). The one permitted
  cancellation is a superseded in-progress run on the same PR, because only
  the head commit's results are evaluated.
- **Head-commit rule:** merge eligibility is determined by the required
  checks on the PR head commit; results from older commits never count.
- **Bounded runtime:** every check job carries a recorded job-level timeout so
  a hang surfaces as a failure, not as silence. Timeout values are recorded in
  the implementation record at implementation time; the requirement of a
  finite bound is normative here, mirroring the W09 runner's timeout
  philosophy.

## 5. Realization rules per gate

- **Provisioning (all gates):** CI installs rustup through its official
  installer invocation as documented in the delivered W02 contract, then lets
  any toolchain-driven invocation provision the pinned toolchain, components,
  and targets from the repository `rust-toolchain.toml`. CI must not install,
  cache in a bypassing way, or select any toolchain version, component, or
  target outside the manifest (W02 parity rule §3.7).
- **QG-FMT / QG-LINT:** the command spelling recorded in the delivered W07
  contract document is used verbatim and re-recorded when W07 records a minor
  change. The check fails on any finding; fixing is by reformatting or code
  change, never by weakening the invocation.
- **QG-WARN:** executes the gate-bearing builds (host build, host tests,
  target build) under the warning-enforcement configuration delivered by
  [W03](../p0-w03-aarch64-build-target-baseline/README.md). The check's
  result is the zero-warnings condition of W07 §4; passing builds with
  warnings reported elsewhere are not a pass for this gate.
- **QG-TEST-HOST:** executes the delivered W08 host-test entry exactly as
  spelled in its contract; success is exit success with zero failures and
  zero unreported skips (W07 §5). No interpretation is added.
- **QG-BUILD-TARGET:** executes the delivered W03 target-build entry; success
  is exit success with the declared baseline artifact present (W07 §6).
  Compilation success never substitutes for execution evidence; this check
  says nothing about EL2, QEMU, or hardware behavior.
- **QG-DOCS:** inline workflow steps implementing the three semantics of W07
  §7: (1) every relative link in tracked documentation resolves from a fresh
  checkout; (2) every normative development-governance document is reachable
  from the repository entry points within a bounded hop count; (3) normative
  documents carry the status header required by the documentation baseline
  (presence only; [W05](../p0-w05-documentation-baseline/README.md) owns the
  content). Mechanisms are restricted to shell, Git, and text tools already
  available in the baseline environment — no new tooling dependency until
  W18's policy exists and is followed. If the logic outgrows inline
  realizability or gains a second consumer, extraction into a committed
  gate-runner script is a Reserved change requiring a reviewed design.

## 6. Future-class classification and enablement

The contract document must contain this classification. At P0 **no row is
configured as a check**; the table exists so the absence is explicit and the
enablement path is written down.

| Subject | Class at P0 | Expected owner | Enablement conditions |
|---|---|---|---|
| QEMU runner execution (smoke/regression) | Future | [W09](../p0-w09-qemu-automation-entry-baseline/README.md) contract; first implementation expected P1-W10 | a runner implementation exists and a W07 register row is promoted; the workflow gains the job in the same change |
| EL2 boot smoke | Future | P1 designs | same promotion path; never presented as covered before promotion |
| Linux guest regression | Future | P8+ designs | same promotion path |
| Real-hardware (Orange Pi 3B) checks | Future / manual | P15+ designs | same promotion path plus platform/runner prerequisites owned by those stages |
| Fuzz/property checks | Future | later stages per ADR-049 | same promotion path |
| Unsafe-inventory consistency | Future | [W10](../p0-w10-unsafe-rust-governance/README.md) trigger (first `unsafe` in tree) | promotion per W07 register |
| rustdoc warnings, dependency-audit reporting | Informational candidates | W07/W18 | membership may be empty; an informational check never silently converts to required (W07 register §3) |

Presentation rules (normative content of the contract document):

- No check name may exist whose semantics imply coverage of a future-class
  subject; no green check may represent a future check.
- Documents and PR descriptions may state only that such scope is
  **not verified — future class**. The P0 checks prove nothing about EL2,
  guest, QEMU, or hardware behavior; PR checks prove only the configured
  gates (integration policy boundaries section).
- Promotion always passes through the W07 register's thresholds and lands the
  workflow edit in the same change.

## 7. Failure, artifact, and observability rules

- **Attribution:** every failing check's visible output carries the gate
  label and the commit/PR reference it ran on, with enough output to diagnose
  without re-running (W07 principle 4).
- **Retention:** failing jobs retain their complete logs under the platform's
  default retention. A failing QG-BUILD-TARGET additionally uploads its
  compiler/linker output as a run artifact. Artifact names follow the
  [W17 naming contract](../p0-w17-artifact-naming-baseline/01-naming-contract.md)
  once delivered; until then they must be machine-processable and unique per
  run. Durable evidence lives in stage verification records, not in CI; no
  long-term artifact storage exists at P0.
- **Flakiness:** a required check with nondeterministic outcomes is a defect
  (W07 principle 3). A single rerun that changes the outcome is recorded in
  the PR with both results; systematic rerun-until-green is prohibited.
- **No silent degradation:** the required set may not be reduced, reordered
  into optionality, or made skippable inside a change that would benefit from
  the reduction; such changes go through §9 thresholds.

## 8. Security and permissions model

- **Least privilege:** the workflow runs with `contents: read` and no
  write-facing permission. No secret is required or used at P0. If a future
  design introduces secrets, its pull-request/fork exposure model must be
  designed before any secret enters the workflow (Reserved trigger).
- **Supply chain:** the workflow uses only first-party steps plus the
  official rustup installer invocation; no third-party marketplace action is
  authorized at P0. Introducing one is a Reserved change governed by
  [W18](../p0-w18-dependency-governance/README.md) once delivered. This is
  the CI-side counterpart of the W02 contract's fail-closed supply-chain
  position.
- **Untrusted input:** pull-request-supplied code executes in CI. At P0 no
  credential exists to leak, so the exposure is bounded to compute abuse,
  which the bounded job timeouts and public-runner defaults of the platform
  address; the residual risk is recorded here rather than solved by
  speculative controls.

## 9. Mutation thresholds

- **Recorded minor change** (ordinary PR review, noted in the implementation
  record): renaming the workflow file with its pointer updated; recording
  job-timeout values; recording the failure-artifact naming once W17
  delivers; re-recording a bound entry spelling after the owning contract's
  own recorded minor change.
- **Design-level change** (recorded against the contract document,
  reviewer-approved): adding or removing a check row; changing the trigger
  model; changing a protection requirement; changing failure-visibility or
  timeout normativity; enabling the "require branches up to date" option.
- **ADR required:** making a hypervisor-TCB-affecting check optional,
  skippable, or advisory; weakening enforcement below the integration
  policy's rule; any change that would alter an accepted architecture
  decision per the [ADR change path](../../../../adr/README.md).

## 10. Explicitly excluded interfaces

No script, program, crate, dependency, target triple, QEMU invocation,
runner, EL2/Linux/hardware test, secret store, third-party action, PR
template, or policy amendment is authorized by this contract. The only
machine-facing surfaces are the workflow file (data plus pointer comment) and
the protection settings (mirror of this document); the only human-facing
procedures are the enforcement exercise and the failure-evidence conventions,
fixed in [the protection design](02-protection-and-enforcement.md). Adding an
excluded item under W20 is a scope conflict against the owning package and
must be stopped at review.
