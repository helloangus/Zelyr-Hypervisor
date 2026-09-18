# P0-W07 Gate Minimum Standards

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W07 detailed design](README.md).

Each section below is normative content of the contract document. A gate's
minimum standard fixes what the gate inspects, what counts as passing, and
what its failure means. Invocation *semantics* are fixed here; the literal
command spelling is recorded in the contract document at implementation time
by the rule given per gate, following the W02 pattern of design-fixed
selection rules.

## 2. QG-FMT — formatting

- **Purpose and caller:** prove all workspace Rust sources are formatted under
  the repository-pinned formatter, so review diffs carry no formatting noise.
  Called by developers pre-push and by W20 as a required check.
- **Minimum standard:** the pinned toolchain's formatter (W02 guarantee:
  `rustfmt` component present) reports no violation against the tracked tree.
  No formatting allowlist, per-file exclusion, or alternate style
  configuration exists unless an approved design records one.
- **Invocation semantics:** the toolchain's standard whole-tree check mode
  over the delivered workspace. Selection rule for the recorded spelling: the
  shortest standard invocation that checks every workspace member; recorded
  verbatim in the contract document; stable thereafter.
- **Passing condition:** check exits success with no findings.
- **Failure semantics:** any finding blocks merge. Fixing is by reformatting,
  never by weakening the standard.

## 3. QG-LINT — lint

- **Purpose and caller:** catch defect classes the compiler does not, under
  the pinned toolchain's linter (`clippy` component per W02).
- **Minimum standard:** the linter runs over all workspace targets with the
  warning policy of §4 applied; the result is lint-clean under that policy.
  Lint configuration lives in the delivered workspace's lint mechanism
  (W03's build baseline); the policy — lint findings are failures and broad
  `allow`/`expect` suppressions are prohibited without a recorded,
  narrow, justified exception — is owned here, consistent with the Coding
  Guidelines' prohibition on hiding warnings.
- **Invocation semantics:** whole-workspace, all-targets analysis, no
  inherited allowance for suppressed lints beyond the recorded configuration.
  Selection rule for the recorded spelling: the standard invocation that
  analyzes every workspace member and target and fails on policy-level
  findings; recorded verbatim; stable thereafter.
- **Passing condition:** analysis completes with zero policy-level findings.
- **Failure semantics:** findings block merge. A proposed suppression needs a
  scoped justification in the change under review; a blanket suppression is a
  review failure.

## 4. QG-WARN — warning policy

- **Purpose and caller:** prevent warnings from accumulating as permanent
  noise and from hiding real defects in low-level code.
- **Minimum standard:** in every gate-bearing build (host build, host tests,
  target build), compiler and linter warnings are failures. Broad suppression
  attributes (`allow` without a scoped, recorded reason), unreachable
  `unwrap`/`expect`/`todo!`/`unimplemented!` in non-test paths, and
  dead-code tolerances are prohibited per the Coding Guidelines.
- **Enforcement binding:** the policy is W07's; the mechanical enforcement
  point is the delivered build configuration (W03's build baseline, e.g.
  workspace lint table or equivalent). If W03's delivered baseline cannot
  express the policy, the gap is recorded and raised against W03's contract —
  not solved by an undocumented override here.
- **Passing condition:** gate-bearing builds produce zero warnings under the
  recorded enforcement configuration.
- **Failure semantics:** a warning in any gate-bearing build blocks merge;
  suppressing to pass is a review failure even when the build succeeds.

## 5. QG-TEST-HOST — host tests

- **Purpose and caller:** execute the entire host-side test baseline,
  independent of QEMU, on every change.
- **Binding:** exactly one documented entry — the host-test execution entry
  defined by the [P0-W08 design](../p0-w08-host-side-testing-baseline/README.md).
  This gate consumes that entry as data: entry spelling, scope, and exit
  semantics are W08's; blocking classification and evidence attribution are
  this register's.
- **Minimum standard:** the entry executes the full host-test set on the
  pinned toolchain's host target and reports success only if every test
  passed and none were silently skipped.
- **Passing condition:** entry exit status is success, with zero failures and
  zero unreported skips.
- **Failure semantics:** any failure blocks merge. An entry that cannot run
  because its prerequisite surface is missing is recorded as blocked, not
  passing (register §3).

## 6. QG-BUILD-TARGET — AArch64 target build

- **Purpose and caller:** prove the bare-metal AArch64 baseline artifact still
  builds, so target-breaking changes cannot reach `main` unnoticed.
- **Binding:** the target-build entry delivered by the
  [P0-W03 design](../p0-w03-aarch64-build-target-baseline/README.md). The
  target triple, build semantics, and artifact identity are W03's; this gate
  owns classification, blocking, and evidence.
- **Minimum standard:** the delivered entry completes, producing its declared
  baseline artifact. Success conditions beyond compilation (running, booting,
  QEMU behavior) are explicitly not part of this gate; those belong to
  P1 and the [W09 runner entry](../p0-w09-qemu-automation-entry-baseline/README.md).
- **Passing condition:** entry exit status is success with the declared
  artifact present.
- **Failure semantics:** any failure blocks merge. This gate never substitutes
  for an EL2 smoke test; its passing proves the compile chain only.

## 7. QG-DOCS — documentation consistency

- **Purpose and caller:** keep the documentation baseline navigable as the
  repository grows, so contributors and agents are never stranded by broken
  links or orphaned normative documents.
- **Minimum standard (check semantics owned by W07):**
  1. every relative link in tracked documentation resolves from a fresh
     checkout;
  2. every normative development-governance document is reachable from the
     repository entry points (`README.md`, `AGENTS.md`, `docs/README.md`)
     within a bounded number of hops; and
  3. normative documents carry the status header required by the
     documentation baseline ([W05](../p0-w05-documentation-baseline/README.md)
     owns what that header must contain; this gate only checks presence).
- **Realization binding:** the mechanical realization inside CI is W20's. If
  W20's realization introduces a tooling dependency, that dependency follows
  dependency governance ([W18](../p0-w18-dependency-governance/README.md))
  once its policy exists; until then only mechanisms already available in the
  baseline environment may be used.
- **Passing condition:** all three checks hold for the changed tree.
- **Failure semantics:** a broken link, unreachable normative document, or
  missing status header blocks merge.

## 8. Development and integration minimum verification sets

- **Development set** (expected before every push; the contributor's local
  minimum): `QG-FMT`, `QG-LINT`, `QG-WARN`, `QG-TEST-HOST`. Chosen so the
  fast, host-only gates run before review, keeping the slower target build in
  CI where the toolchain and target are guaranteed present.
- **Integration set** (must pass before merge, enforced by W20 in CI): all
  Required gates — the development set plus `QG-BUILD-TARGET` and `QG-DOCS`.
- A contributor may run the integration set locally; running it locally never
  substitutes for the online required checks (integration workflow rule).
- The sets are register data: membership changes follow the §3 mutation
  thresholds of the [register](01-gate-register.md).

## 9. Check-failure handling principles

These principles are normative contract content and align with the
[integration workflow](../../../../development/integration-workflow.md):

1. **Blocking is binary and visible.** A Required gate that fails, or fails to
   report a passing result (missing, cancelled, skipped, timed out), blocks
   merge. Partial output is not partial credit.
2. **No silent degradation.** A gate may not be skipped, downgraded to
   informational, or waived inside a change that needs it. Waivers are
   design-level changes (register §3) with an owner, a recorded reason, and an
   expiry, recorded against the contract.
3. **Flakiness is a defect.** A Required gate with nondeterministic outcomes
   is treated as broken until fixed; systematic rerun-until-green is
   prohibited. A single rerun that changes the outcome must be recorded in the
   PR with both results.
4. **Failure evidence is attributed.** Every failure report carries the gate
   ID (register evidence label), the commit/PR it ran on, and enough output to
   diagnose without re-running. Saved failure evidence follows the stage
   verification conventions ([W05](../p0-w05-documentation-baseline/README.md)
   locations).
5. **Failure handling never edits the gate to pass.** Weakening a gate inside
   the change that failed it is a review failure; gate changes go through the
   contract's thresholds in a separate, explicit decision.

## 10. Future-class expected members (reserved, not defined)

Named by owner only; each becomes a register member only through promotion:

- unsafe-inventory consistency gate — policy owner
  [W10](../p0-w10-unsafe-rust-governance/README.md); activates with the first
  `unsafe` in the tree;
- QEMU-class checks — entry owner
  [W09](../p0-w09-qemu-automation-entry-baseline/README.md) and its P1
  consumers; never a P0 required check;
- rustdoc generation warnings and dependency-audit reporting — informational
  candidates once the subjects exist ([W18](../p0-w18-dependency-governance/README.md));
- fuzz/property and hardware checks — later stages per ADR-049.
