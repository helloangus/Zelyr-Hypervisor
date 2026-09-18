# Zelyr Development Quality Gates

**Status:** Normative quality-gate policy.  
**Scope:** The gate register (classification, blocking, scope, invocation
binding, evidence labels), per-gate minimum standards and failure semantics,
development/integration verification sets, check-failure handling principles,
and mutation thresholds. It does not configure CI (P0-W20) or own the bound
build/test entries (P0-W03/P0-W08 own those).  
**Version:** v0.1  
**Owner/change context:** P0-W07 development quality gates; operationalizes
the P0 task book's quality outcomes (P0-V03–V08).  
**Supersedes:** The absence of an explicit gate register.

## 1. Gate register

The register is the complete list of gates this document defines. Every row
is normative content. The evidence label is the attribution key that must
appear in check output, CI check names, and saved evidence so any gate result
can be traced to its register row and stage validation ID.

| Gate ID | Name | Class | Blocks merge | Scope | Invocation binding | Evidence label | Stage validation |
|---|---|---|---|---|---|---|---|
| `QG-FMT` | Formatting | Required | Yes | All workspace Rust sources formatted under the pinned toolchain | this document §2 | `QG-FMT` | P0-V06 |
| `QG-LINT` | Lint | Required | Yes | Clippy analysis with the warning policy applied | this document §3 | `QG-LINT` | P0-V07 |
| `QG-WARN` | Warning policy | Required | Yes | No compiler/lint warning suppressed or tolerated in gate-bearing builds | this document §4; enforcement point is the delivered build baseline | `QG-WARN` | P0-V07 |
| `QG-TEST-HOST` | Host tests | Required | Yes | Every host-target test executes and passes, independent of QEMU | [host-test baseline](../testing/host-test-baseline.md) entry | `QG-TEST-HOST` | P0-V03/V04 |
| `QG-BUILD-TARGET` | AArch64 target build | Required | Yes | The bare-metal AArch64 baseline artifact builds through the delivered entry | [build-target baseline](build-target-baseline.md) entry | `QG-BUILD-TARGET` | P0-V05 |
| `QG-DOCS` | Documentation consistency | Required | Yes | Tracked-document link integrity and entry-point reachability | this document §7; CI realization is P0-W20's | `QG-DOCS` | P0-V09 |

Register rules:

- The register contains exactly these six members at W07 closure. No required
  gate may be added or removed except through the §8 mutation thresholds.
- A gate row is complete only when all fields are filled; an unresolved
  binding is a register defect.
- Recorded invocation spellings (selected 2026-09-18 under pinned toolchain
  `1.98.1`, stable thereafter; changes tracked per §8):

```sh
# QG-FMT
cargo fmt --all -- --check
# QG-LINT — host-class members
cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings
# QG-LINT — the bare-metal member, under its own target
cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings
# QG-TEST-HOST (bound to the host-test baseline entry)
cargo test --workspace --exclude hypervisor
# QG-BUILD-TARGET (bound to the build-target baseline entry)
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
```

The `--exclude hypervisor` term in the host-class spellings is semantic, not
a filter: the hypervisor member is a bare-metal-class freestanding binary
that cannot compile for a host target (see the [host-test
baseline](../testing/host-test-baseline.md), §4). `cargo fmt` needs no exclusion
(formatting does not compile sources).

## 2. QG-FMT — formatting

- **Purpose and caller:** prove all workspace Rust sources are formatted
  under the repository-pinned formatter, so review diffs carry no formatting
  noise. Called by developers pre-push and by P0-W20 as a required check.
- **Minimum standard:** the pinned toolchain's formatter (`rustfmt` per the
  toolchain baseline) reports no violation against the tracked tree. No
  formatting allowlist, per-file exclusion, or alternate style configuration
  exists unless an approved design records one.
- **Invocation semantics:** the toolchain's standard whole-tree check mode
  over the delivered workspace; recorded spelling above.
- **Passing condition:** check exits success with no findings.
- **Failure semantics:** any finding blocks merge. Fixing is by
  reformatting, never by weakening the standard.

## 3. QG-LINT — lint

- **Purpose and caller:** catch defect classes the compiler does not, under
  the pinned toolchain's linter (`clippy` per the toolchain baseline).
- **Minimum standard:** the linter runs over all workspace members and
  targets with the §4 warning policy applied; the result is lint-clean under
  that policy. Lint configuration lives in the delivered build baseline; the
  policy — lint findings are failures and broad `allow`/`expect` suppressions
  are prohibited without a recorded, narrow, justified exception — is owned
  here, consistent with the Coding Guidelines.
- **Invocation semantics:** whole-workspace, all-targets analysis, no
  inherited allowance for suppressed lints beyond the recorded configuration;
  host-class members on the host target and the bare-metal member under its
  own target (recorded spellings above).
- **Passing condition:** analysis completes with zero policy-level findings.
- **Failure semantics:** findings block merge. A proposed suppression needs a
  scoped justification in the change under review; a blanket suppression is a
  review failure.

## 4. QG-WARN — warning policy

- **Purpose and caller:** prevent warnings from accumulating as permanent
  noise and from hiding real defects in low-level code.
- **Minimum standard:** in every gate-bearing build (host build, host tests,
  target build), compiler and linter warnings are failures. Broad suppression
  attributes, unreachable `unwrap`/`expect`/`todo!`/`unimplemented!` in
  non-test paths, and dead-code tolerances are prohibited per the Coding
  Guidelines.
- **Enforcement binding:** the policy is this document's; the mechanical
  enforcement point is the delivered build baseline. As delivered (P0-W03),
  the workspace carries no lint table and relies on toolchain defaults;
  gate-bearing builds are verified zero-warning under those defaults (see
  the W07 verification record), and the `-D warnings` term in the QG-LINT
  spellings enforces the policy at lint level. Adding a workspace lint table
  is a recorded change under the build-target baseline's thresholds; it must
  not weaken this policy.
- **Passing condition:** gate-bearing builds produce zero warnings under the
  recorded enforcement configuration.
- **Failure semantics:** a warning in any gate-bearing build blocks merge;
  suppressing to pass is a review failure even when the build succeeds.

## 5. QG-TEST-HOST — host tests

- **Purpose and caller:** execute the entire host-side test baseline,
  independent of QEMU, on every change.
- **Binding:** exactly one documented entry — the host-test execution entry
  of the [host-test baseline](../testing/host-test-baseline.md). This gate consumes that
  entry as data: entry spelling, scope, and exit semantics are the host-test
  baseline's; blocking classification and evidence attribution are this
  register's.
- **Minimum standard:** the entry executes the full host-test set on the
  pinned toolchain's host target and reports success only if every test
  passed and none were silently skipped.
- **Passing condition:** entry exit status is success, with zero failures and
  zero unreported skips.
- **Failure semantics:** any failure blocks merge. An entry that cannot run
  because its prerequisite surface is missing is recorded as blocked, not
  passing.

## 6. QG-BUILD-TARGET — AArch64 target build

- **Purpose and caller:** prove the bare-metal AArch64 baseline artifact
  still builds, so target-breaking changes cannot reach `main` unnoticed.
- **Binding:** the target-build entry of the [build-target
  baseline](build-target-baseline.md). The target triple, build semantics,
  and artifact identity are that baseline's; this gate owns classification,
  blocking, and evidence.
- **Minimum standard:** the delivered entry completes, producing its declared
  baseline artifact. Success conditions beyond compilation (running, booting,
  QEMU behavior) are explicitly not part of this gate; those belong to P1 and
  the P0-W09 runner entry.
- **Passing condition:** entry exit status is success with the declared
  artifact present.
- **Failure semantics:** any failure blocks merge. This gate never
  substitutes for an EL2 smoke test; its passing proves the compile chain
  only.

## 7. QG-DOCS — documentation consistency

- **Purpose and caller:** keep the documentation baseline navigable as the
  repository grows, so contributors and agents are never stranded by broken
  links or orphaned normative documents.
- **Minimum standard (check semantics owned here):**
  1. every relative link in tracked documentation resolves from a fresh
     checkout (forward references inside not-yet-implemented stage designs
     are exempt while their targets do not exist, and must be recorded by
     the owning stage);
  2. every normative development-governance document is reachable from the
     repository entry points (`README.md`, `AGENTS.md`, `docs/README.md`)
     within four link hops; and
  3. normative documents carry the status header required by the
     [documentation baseline](documentation-baseline.md) (that baseline owns
     what the header must contain; this gate only checks presence).
- **Realization binding:** the mechanical realization inside CI is P0-W20's.
  Until dependency governance (P0-W18) exists, only mechanisms already
  available in the baseline environment may be used.
- **Passing condition:** all three checks hold for the changed tree.
- **Failure semantics:** a broken link, unreachable normative document, or
  missing status header blocks merge.

## 8. Classification, promotion, and mutation thresholds

- **Required:** proves a property the task book assigns to P0 quality.
  Always merge-blocking, locally and in CI. A required gate that cannot run
  is not satisfied by silence; it is recorded as blocked, and the PR carrying
  the cause may not merge while the gate it disabled is load-bearing.
- **Informational:** reports a useful quality signal without blocking;
  membership may legitimately be empty at P0 (expected future members:
  rustdoc-generation warnings, dependency-audit reporting). An informational
  check never silently converts to required; that is a promotion.
- **Future:** a named, owned check that cannot be defined yet because its
  subject does not exist (EL2 smoke, Linux guest regression, hardware,
  fuzz/property, unsafe audit). Future members are non-blocking by definition
  until promoted. Promotion requires an owning approved design, a register
  row, a recorded rationale, and — where the check would become load-bearing
  for hypervisor-TCB claims — review under the ADR change path.
- **Mutation thresholds:**
  - *Recorded minor change* (ordinary PR review, recorded in the
    implementation record): adjusting a recorded spelling to track a
    delivered entry without semantic change; rewording a standard for
    clarity without changing meaning.
  - *Design-level change* (recorded against this contract,
    reviewer-approved): adding/removing a register member; changing a class
    or blocking flag; changing a passing condition.
  - *ADR-level:* any change that would let hypervisor-TCB-affecting code
    merge without a gate the baseline ADR's validation strategy (ADR-049)
    expects.

## 9. Development and integration minimum verification sets

- **Development set** (expected before every push; the contributor's local
  minimum): `QG-FMT`, `QG-LINT` (host-class spelling), `QG-WARN`,
  `QG-TEST-HOST`.
- **Integration set** (must pass before merge; enforced by P0-W20 in CI):
  all Required gates — the development set plus the bare-metal QG-LINT
  spelling, `QG-BUILD-TARGET`, and `QG-DOCS`.
- A contributor may run the integration set locally; running it locally never
  substitutes for the online required checks (integration workflow rule).
- The sets are register data; membership changes follow §8.

## 10. Check-failure handling principles

1. **Blocking is binary and visible.** A Required gate that fails, or fails
   to report a passing result (missing, cancelled, skipped, timed out),
   blocks merge. Partial output is not partial credit.
2. **No silent degradation.** A gate may not be skipped, downgraded to
   informational, or waived inside a change that needs it. Waivers are
   design-level changes with an owner, a recorded reason, and an expiry.
3. **Flakiness is a defect.** A Required gate with nondeterministic outcomes
   is treated as broken until fixed; systematic rerun-until-green is
   prohibited. A single rerun that changes the outcome must be recorded in
   the PR with both results.
4. **Failure evidence is attributed.** Every failure report carries the gate
   ID, the commit/PR it ran on, and enough output to diagnose without
   re-running.
5. **Failure handling never edits the gate to pass.** Weakening a gate inside
   the change that failed it is a review failure; gate changes go through §8
   in a separate, explicit decision.

## 11. Future-class expected members (reserved, not defined)

Named by owner only; each becomes a register member only through promotion:

- unsafe-inventory consistency gate — P0-W10 policy; activates with the first
  `unsafe` in the tree;
- QEMU-class checks — the P0-W09 entry and its P1 consumers; never a P0
  required check;
- rustdoc-generation warnings and dependency-audit reporting — informational
  candidates once the subjects exist (P0-W18);
- fuzz/property and hardware checks — later stages per ADR-049.
