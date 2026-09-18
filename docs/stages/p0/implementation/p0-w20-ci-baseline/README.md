# P0-W20 CI Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The GitHub check execution and reporting, required-check
classification, `main` branch protection, failure and artifact visibility, and
enforcement evidence required by
[P0-W20](../../plans/p0-w20-ci-baseline.md).  
**Owner/change context:** P0-W20 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W20. It converts the bounded
work-package plan into small, reviewable configuration and documentation
changes: one normative CI contract document, one GitHub check workflow that
executes the delivered quality gates as named checks on pull requests and on
`main`, the branch-protection configuration that makes verified PR-only
integration to `main` technically enforced, and a real-GitHub enforcement
exercise that records the evidence P0-V08 demands. It deliberately does
**not** define gate semantics or classification rules
([P0-W07](../p0-w07-development-quality-gates/README.md) owns the gate
register), define the build or test entries the checks execute
([P0-W03](../p0-w03-aarch64-build-target-baseline/README.md),
[P0-W08](../p0-w08-host-side-testing-baseline/README.md)), install a toolchain
outside the repository manifest
([P0-W02](../p0-w02-rust-toolchain-baseline/README.md) owns the pin and its
parity rule), implement a QEMU runner or any EL2, Linux-guest, or hardware
check ([P0-W09](../p0-w09-qemu-automation-entry-baseline/README.md) delivers an
interface-only placeholder whose first implementation is P1), or amend the
[branch and pull-request integration
workflow](../../../../development/integration-workflow.md) — W20 implements
that policy's deferred GitHub enforcement without changing its text.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P0 task
book](../../task-book-v0.1.md), and the P0-W20 plan. This document is the
proposed detailed design for those changes; it is not a completion record and
contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W20 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W20 is: CI classifies required, informational, and
  later manual/hardware checks, and **enforces verified PR-only integration to
  `main`** (P0-V08). Task-book exit criterion 4 states the same as an exit
  condition: `main` accepts post-policy development changes only through
  GitHub PRs with required online checks passing.
- The [integration
  workflow](../../../../development/integration-workflow.md) boundaries section
  assigns this package verbatim: "P0-W20 owns GitHub workflow triggers,
  required-check classification, branch protection configuration, and evidence
  that GitHub actually enforces this policy." Its PR rule — a missing,
  cancelled, skipped, or failed required check is not passing evidence — is
  implemented as behavior, never weakened.
- The [W07 gate
  register](../p0-w07-development-quality-gates/01-gate-register.md) is the
  classification source of truth. Its §4 binds W20: each Required row maps to
  one GitHub check; a check must not claim a gate it does not fully execute
  and must not execute a gate whose register semantics it contradicts. W20
  redefines nothing; a conflict goes back through the register's mutation
  thresholds.
- The [W02 toolchain
  contract](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md) §3.7
  parity rule is binding on CI: provisioning flows through the repository
  manifest; CI never installs a divergent toolchain, channel, component set,
  or target set.
- The [W09 runner-entry
  contract](../p0-w09-qemu-automation-entry-baseline/01-runner-entry-contract.md)
  §8 binds CI presentation: until a runner implementation exists, the QEMU
  entry is treated as a future/non-P0 check and nothing may describe it as
  passing or runnable.
- The plan's acceptance wording is the evidence bar: a configuration review or
  a local workflow syntax check does **not** prove that GitHub executed the
  checks or that protection is in effect; evidence must be observed on the
  real GitHub repository.
- Administrator bypass: the policy states an administrator's ability to bypass
  protection is not permission to bypass it, and the plan forbids treating
  administrator bypass as normal process. This design configures the
  strictest protection the repository settings allow and records any residual
  bypass capability honestly as policy-bound rather than technically bound; it
  does not claim bypass is impossible.

Classification: the CI contract document, the check workflow, the branch
protection configuration, the enforcement-evidence procedure, and the
failure-visibility and future-class presentation rules (all in the supporting
files) are **Required** for W20 closure. Gate-runner script extraction,
artifact retention beyond platform defaults, reusable-workflow refactoring,
third-party workflow actions (governed by
[W18](../p0-w18-dependency-governance/README.md) once delivered), and
QEMU/hardware execution are **Reserved** with recorded triggers. Gate
semantics and register membership (W07), gate command spelling beyond the
delivered entries (W03/W08/W07), the toolchain pin (W02), target triples,
crate dependencies, QEMU invocation profiles, EL2/Linux/hardware check
implementation, PR templates, and amendments to the integration policy are
**Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| W07 gates and W19 entries mapped onto CI-reportable checks (work sequence 1) | [CI contract](01-ci-contract.md) §3 | P0-V08 (W20-DV01) |
| Required status for format, lint, host tests, AArch64 build, and documentation checks (work sequence 2) | [CI contract](01-ci-contract.md) §3–§5 | P0-V08 (W20-DV01, DV02, DV05) |
| Non-P0 classification and enablement conditions for the QEMU runner, future EL2 smoke, Linux, and real-hardware verification (work sequence 3) | [CI contract](01-ci-contract.md) §6 | P0-V08 (W20-DV03) |
| `main` protection: PR with `main` base, all configured required checks pass before merge, administrator bypass not normal process (work sequence 4) | [Protection and enforcement](02-protection-and-enforcement.md) §2–§4 | P0-V08 (W20-DV04, DV06) |
| Failure localization, artifact/metadata association, real-PR evidence (work sequence 5) | [CI contract](01-ci-contract.md) §7, [protection](02-protection-and-enforcement.md) §5, [workflow](03-implementation-and-validation.md) step 6 | P0-V08 (W20-DV06, DV07) |
| Document discoverability and coherence | [workflow](03-implementation-and-validation.md) step 7 | P0-V09 (W20-DV08) |
| Downstream handoff to P0 completion, P1+, and PR authors | [workflow](03-implementation-and-validation.md) handoff checklist | W20 closure (W20-DV09) |

## Current-state findings and goal-to-baseline ledger

Observed state (2026-09-18, worktree branch `docs/p0-implementation-designs`):
`.github/workflows/` contains only a `.gitkeep` marker — no workflow, check,
or protection setting exists. The [integration
workflow](../../../../development/integration-workflow.md) is adopted
normative policy whose technical enforcement is explicitly deferred to W20;
until now, PR-only integration is human process. No GitHub remote is
configured in this worktree; the [W01 implementation
record](../p0-w01-repository-baseline-record.md) documents remote publication
as pending owner inputs. Of W20's plan-index prerequisites, W01 is delivered;
W02, W03, W07, W08, W09, W17, and W19 have proposed designs in this tree whose
contract documents do not yet exist in the tracked tree. No gate register
document, toolchain manifest, build/test entry, or contributor workflow
document exists yet. Each ledger row below states the missing foundation the
plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Required checks are classified and visible on PR and `main` | No workflow, no check, no classification exists in CI | Check-mapping register (contract document) plus a workflow with one check per Required gate, named by the register's evidence labels | Without a machine-consumed mapping and a running workflow, no check is classifiable or visible | W20 mapping; register rows consumed from W07 | W20-DV01, DV05 |
| Checks execute the real delivered gates | Gate register and build/test entries are proposed designs, not delivered | Realization-by-reference: each check executes the delivered entry contract verbatim | A check that approximates a gate would fake the gate's evidence | W07 semantics; W02/W03/W08 entries | W20-DV02, DV05 |
| `main` accepts post-policy changes only through verified PRs | No protection exists; policy is human process only; remote unconfigured | Branch-protection configuration: PR-only, required-check set, merge-method rule, bypass honesty | Exit criterion 4 is a GitHub-enforced property, not a documented intention | W20 per the integration policy | W20-DV04, DV06 |
| Future checks are not misreported as verified | No classification exists for QEMU/EL2/Linux/hardware in CI | Future-class presentation rules and enablement conditions | The plan's out-of-scope clause forbids disguising future checks as P0 checks | W20 presentation; W07 promotion rule; W09 §8 | W20-DV03 |
| CI output helps localize failures; artifact/metadata association preserved | No CI output conventions exist | Failure-visibility rules: gate label + commit ref in output, retained logs, failure artifacts | Work sequence 5 makes diagnosability a deliverable, not an afterthought | W20; artifact naming via W17 when delivered | W20-DV07 |
| Evidence that GitHub actually enforces the policy | Nothing observed on GitHub; config review alone is disclaimed by the plan | Real-PR enforcement exercise with green, red-probe, blocked-merge, and direct-push observations | The acceptance wording names a real PR as the evidence source | W20 | W20-DV05, DV06 |

Two execution boundaries are recorded, not resolved by this design: (1) the
GitHub remote and administrator access are owner inputs — without them the
protection configuration and enforcement exercise are blocked and recorded as
such; (2) the prerequisite contracts are proposed designs — W20 implements
against their delivered forms, treats a missing prerequisite as a blocked
check (never as a stub or a silent reduction of the required set), and
reconciles a differently-delivered contract in the same change.

## Resolved design decisions and their authority

1. **Policy home:** `docs/development/ci-baseline.md` is the sole normative
   home of the check-mapping register, trigger model, protection
   requirements, future-class rules, and failure/evidence conventions.
   Rationale: GitHub settings are not reviewable in pull requests, so the
   contract document is the reviewable authority that the settings must
   mirror, and the enforcement exercise proves the mirror. This mirrors the
   approved W02/W03/W07 pattern of a development-governance contract document.
2. **Check identity:** one GitHub check per Required gate; the check (job)
   name is exactly the register evidence label (`QG-FMT`, `QG-LINT`,
   `QG-WARN`, `QG-TEST-HOST`, `QG-BUILD-TARGET`, `QG-DOCS`), with no aliases
   or decorative prefixes. Branch protection references these six names.
   Rationale: the register's evidence label is defined as the attribution key
   for check names; a 1:1 name mapping makes drift reviewable.
3. **Trigger model:** the workflow runs on `pull_request` targeting `main`
   and on `push` to `main`, satisfying P0-V08's requirement that PR and
   mainline required checks both execute and are visible. No path filters and
   no conditions may skip a required check (a skipped check is not passing
   evidence); superseded in-progress runs on the same pull request may be
   cancelled because only the head commit's results are evaluated.
4. **Realization by reference:** QG-FMT and QG-LINT use the spellings recorded
   in the delivered W07 contract; QG-TEST-HOST executes the delivered W08
   entry; QG-BUILD-TARGET executes the delivered W03 entry; QG-WARN evaluates
   gate-bearing builds under W03's delivered warning-enforcement
   configuration; QG-DOCS implements W07 §7's three check semantics as inline
   workflow steps using mechanisms already available in the baseline
   environment (W07 §7 binding: no new tooling dependency until
   [W18](../p0-w18-dependency-governance/README.md) governance exists).
   Jobs may share toolchain provisioning and caches, but each check evaluates
   and reports its own gate semantics.
5. **Provisioning rule:** CI installs rustup via its official installer and
   provisions the toolchain, components, and targets exclusively through the
   repository `rust-toolchain.toml` (W02 parity rule). No third-party
   marketplace action is used at P0; introducing one is a Reserved,
   W18-governed decision.
6. **Future-class handling:** no QEMU, EL2-smoke, Linux-guest, or hardware
   check is configured at P0. Such a check enters only when (a) the owning
   design delivers a runnable subject, (b) the W07 register row is promoted
   with all seven fields, and (c) the workflow gains the job in the same
   change as the promotion. Until then, future scope may appear only as
   explicit not-verified statements in documents and PR descriptions, never
   as a configured check.
7. **Branch protection:** require a pull request before merging with `main`
   as the sole integration branch; the six labels as required status checks
   (a check that has not reported success does not pass); no bypass
   allowances granted to roles; administrators included in the restrictions
   where repository settings permit, with any residual administrator
   capability recorded as policy-bound. At least one enabled merge method must
   satisfy the policy's reviewable-connection rule; methods that sever the PR
   association are disabled; the exact enablement set is recorded at
   implementation time. The "require branches up to date" option is not
   imposed at P0 (rationale in
   [the protection contract](02-protection-and-enforcement.md) §2); imposing
   it later is a design-level change.
8. **Enforcement evidence:** the P0-V08 proof is a real-PR exercise with four
   observations — green path, deliberate red probe with merge refusal,
   restored green path with merge permitted, and rejected direct push —
   specified in [the protection contract](02-protection-and-enforcement.md)
   §5. Local dry-runs and syntax checks are recorded as preliminary evidence
   only.
9. **Failure visibility:** every failing check's output carries the gate
   label and the commit/PR reference (W07 principle 4); failing jobs retain
   their complete logs under platform defaults; a QG-BUILD-TARGET failure
   uploads its compiler/linker log as a run artifact whose name follows the
   [W17 naming contract](../p0-w17-artifact-naming-baseline/01-naming-contract.md)
   once delivered — until then, names must be machine-processable and unique
   per run (the W09 §7 placeholder pattern). Durable evidence lives in stage
   verification records, not in CI; no long-term artifact storage exists at
   P0 (Reserved).
10. **Least privilege and supply chain:** workflows run with `contents: read`;
    no secret is required or used at P0; if a future design introduces
    secrets, its PR/fork exposure model must be designed first (Reserved
    trigger). The workflow's supply chain is first-party steps plus the
    official rustup installer; the unstable/supply-chain position mirrors the
    W02 contract's fail-closed approach.

## Work breakdown and loading order

1. Read [the CI contract](01-ci-contract.md) for the artifact groups, the
   check-mapping register, the trigger and realization rules, the future-class
   enablement conditions, and the failure/evidence conventions.
2. Read [the protection and enforcement design](02-protection-and-enforcement.md)
   for the protection requirements, the settings-to-contract mirror rule, the
   failure boundaries, and the enforcement-evidence procedure.
3. Apply the changes in the order stated in [the implementation
   workflow](03-implementation-and-validation.md): audit prerequisite
   delivered surfaces, author the contract document, create the workflow,
   dry-run locally, configure protection, run the enforcement exercise, wire
   discovery and reconcile, then close.
4. Store actual observations, output, environment, and result in
   `../../verification/p0-w20-ci-baseline-verification.md`, and record
   decisions taken, the observed protection settings, and any deviation in
   `../p0-w20-ci-baseline-record.md` only when implementation begins. Neither
   this design nor a written record may claim W20 complete.

## Design-level state and lifecycle

W20 adds no runtime state, registry, lock, allocation, or hypervisor code
path. Its subject is an enforcement state: the configured GitHub behavior plus
its documentary mirror. Two state models apply.

The operational model for a required check run:

```text
queued -> running -> success | failure
A required check on a PR head commit whose outcome is anything other than a
reported success — missing, queued, cancelled, skipped, failed, or timed out —
blocks merge. Partial output is not partial credit (W07 principle 1).
```

The documentary and enforcement lifecycle of the package:

```text
no CI; PR-only integration is human process
  -> docs/development/ci-baseline.md committed (mapping, triggers,
     protection requirements, future-class and failure rules)
  -> .github/workflows/ci.yml committed (six required checks, PR + main)
  -> branch protection configured to mirror the contract
  -> enforcement exercise observed on real GitHub (green path, red probe
     with merge refusal, restored green, rejected direct push)
  -> P0-V08 evidence recorded in the verification record
  -> W19's enforcement-status statement retired in the same change
  -> later changes mutate only through the contract's thresholds
     (W07 promotions add checks; W03/W08/W07 spelling re-records track
      delivered entries; P1+ QEMU/regression checks enter as future class
      first)
```

The contract document owns every policy statement; the workflow file owns the
machine-consumed realization; the GitHub settings must mirror the contract. A
check whose semantics contradict the register is raised back through W07's
thresholds, never redefined locally; drift between settings and contract is a
defect fixed in the same change.

## Explicitly excluded interfaces

No Rust type, function, trait, crate, Cargo manifest, target triple, gate
semantic, register member, gate command spelling of W07-fixed gates, QEMU
command line, runner program or profile, EL2/Linux/hardware test, secret
store, third-party action, PR template, or amendment to the integration
policy is designed or authorized by W20. The machine-facing surfaces are the
workflow file and the protection settings, both fixed by the supporting
files; the human-facing procedures are the enforcement exercise and the
failure-evidence conventions. Adding any excluded item is a scope conflict
requiring the applicable detailed design (at minimum W07 for register changes,
W03/W08 for entry changes, W09/P1 for QEMU execution) and must be stopped at
review.

## Downstream handoff

- **P0 completion review** (aggregated by
  [W22](../p0-w22-stage-dependency-map/README.md)) receives the enforcement
  evidence, the protection settings record, and the check-status conventions
  as the P0-V08 portion of the stage handoff.
- **P1 and later stages** receive the protection baseline and the only
  extension path for new checks: a W07 register promotion plus a same-change
  workflow edit. P1-W10's QEMU boot regression enters as future class until
  its subject and register row exist; CI at P0 never executes QEMU.
- **PR authors** receive required-check semantics (merge-blocking, no
  skipping), the failure-localization conventions, and the rule that local
  gate runs never substitute for the online required checks.
- **W07** receives the conflict rule: if a check's behavior and the register
  ever disagree, the register wins and the conflict is raised through its
  mutation thresholds.
- **W19** receives the same-change retirement of its
  [enforcement-status statement](../p0-w19-reproducible-development-workflow/01-workflow-stage-contracts.md)
  §5.4: when W20's configuration is delivered and evidenced, the contributor
  workflow's "not yet configured" caveat is updated in the same change.
- **W09 and P1** receive confirmation that the runner entry stays a future/non-P0
  classification in CI per its §8 placeholder rule, and that no P0 check may
  claim QEMU execution.
- **W02, W03, W08** receive the consumption rule: their delivered entry
  contracts are executed verbatim; a semantic change to any entry reaches CI
  only through their own contracts' change thresholds plus the contract
  document's recorded-minor re-record rule.
