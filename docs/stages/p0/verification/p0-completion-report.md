# P0 Completion Report

Chinese readers can use the [Chinese edition](p0-completion-report.zh-CN.md).

**Status:** P0 stage completion claim — the only document authorized to make
it (stage workflow L7).  
**Date:** 2026-09-19 (Asia/Shanghai)  
**Basis:** every P0 package's [implementation records](../implementation/README.md)
and [verification records](p0-w22-stage-dependency-map-verification.md) (W01–W22), the [P0 handoff
map](../p0-handoff-map.md), and the [P0 task book](../task-book-v0.1.md).

## 1. Validation matrix status

Every P0-Vxx ID, per the completion-report contract in the handoff map:

| ID | Validation | Status | Evidence (primary) |
|---|---|---|---|
| P0-V01 | Fresh-clone review | **verified** | [W01 verification](p0-w01-repository-baseline-verification.md); re-demonstrated by the [W19 fresh-clone walkthrough](p0-w19-reproducible-development-workflow-verification.md) (disposable clone, S0 reachable) |
| P0-V02 | Toolchain restoration | **verified** | [W02 verification](p0-w02-rust-toolchain-baseline-verification.md) (isolated-sandbox restore, idempotence, single-source scan) |
| P0-V03 | Host build | **verified** | [W08 verification](p0-w08-host-side-testing-baseline-verification.md) (entry compiles all host-target members); CI `QG-TEST-HOST` executes it on every PR and `main` push |
| P0-V04 | Host tests | **verified** | [W08 verification](p0-w08-host-side-testing-baseline-verification.md) (1 passed/0 failed; failure-visibility probe); CI `QG-TEST-HOST` green on PR #31/#32/#33/#34 |
| P0-V05 | AArch64 target build | **verified** | [W03 verification](p0-w03-aarch64-build-target-baseline-verification.md) (build, incremental no-op, clean rebuild, ELF identification); CI `QG-BUILD-TARGET` green |
| P0-V06 | Formatting | **verified** | [W07 verification](p0-w07-development-quality-gates-verification.md) (`QG-FMT` dry run); CI `QG-FMT` green — and proven *required* by the red probe (PR #32: `QG-FMT` failure blocked merge) |
| P0-V07 | Lint | **verified** | [W07 verification](p0-w07-development-quality-gates-verification.md) (`QG-LINT` both spellings; `QG-WARN` zero-warning builds); CI `QG-LINT`/`QG-WARN` green |
| P0-V08 | CI and PR integration | **verified** | [W20 verification](p0-w20-ci-baseline-verification.md): six required checks configured and enforced on `main` (`enforce_admins: true`); red probe blocked merge; restore returned `CLEAN`; direct push of an unchecked commit rejected ("protected branch hook declined") |
| P0-V09 | Documentation review | **verified** | [W05 verification](p0-w05-documentation-baseline-verification.md) plus every package's discovery/link review entries; CI `QG-DOCS` (links, ≤4-hop reachability, status headers) green on every PR and push |
| P0-V10 | ADR governance review | **verified** | [W06 verification](p0-w06-adr-governance-verification.md) (lifecycle, thresholds, drill, template; adr-000 untouched) |
| P0-V11 | Unsafe governance review | **verified** | [W10 verification](p0-w10-unsafe-rust-governance-verification.md) (policy + inventory usable for a first unsafe change; zero-unsafe tree confirmed) |
| P0-V12 | Portability review | **verified** | [W11 verification](p0-w11-platform-portability-guardrails-verification.md) (no rule permits board/QEMU dependencies in Core or Board dependencies in Arch) |
| P0-V13 | QEMU entry review | **verified** | [W09 verification](p0-w09-qemu-automation-entry-baseline-verification.md) (exactly one documented runner interface, recorded as P0 placeholder; grammar walkthrough) |
| P0-V14 | Artifact/build identity review | **verified** | [W16 verification](p0-w16-version-build-metadata-baseline-verification.md) (identity schema, policies, reservations) and [W17 verification](p0-w17-artifact-naming-baseline-verification.md) (grammar, identity mapping, cross-review completed both directions) |
| P0-V15 | P1 handoff review | **verified** | [W22 verification](p0-w22-stage-dependency-map-verification.md) (handoff map register + consumption mapping) and the [W21 P1 drill](p0-w21-stage-plan-implementation-workflow-verification.md) (P1-W10's consumed inputs findable with distinguishable status) |

No ID is `unverified` or `blocked`.

## 2. Exit criteria (task book §7), answered from §1

1. **Clean environment from clone through toolchain, build, host test, and
   AArch64 target build** — holds: W02 restoration contract, W19 stage chain
   (all stages `reachable`), corroborated by execution in a disposable clone
   (S1/S2/S3) and enforced in CI.
2. **Workspace and quality gate support later multi-crate, multi-platform
   work without asserting final crate boundaries** — holds: virtual
   workspace (membership changes only through approved designs), the
   four-class target model, the gate register's promotion thresholds.
3. **ADR/document/stage governance, unsafe policy, portability guardrails,
   and diagnostic semantics available before low-level code begins** —
   holds: W06/W05/W21/W10/W11/W12–W15 delivered with zero `unsafe` in the
   tree.
4. **`main` accepts post-policy development changes only through GitHub PRs
   with required online checks passing** — holds: W20's protection is
   configured and evidenced; the enforcement exercise recorded refusal of
   both a failing required check and an unchecked direct push.
5. **P1 has a single QEMU runner entry, identifiable build artifacts, and a
   discoverable handoff package** — holds: W09 contract (P1-W10 implements
   it), W16/W17 identity and naming, this report plus the handoff map as the
   discoverable package.
6. **No P0 deliverable encodes future EL2, VM, memory, IRQ, device, or guest
   implementation decisions** — holds: every package's scope review
   (each verification record's not-run/does-not-prove entries; the probe's
   two-symbol boundary; the runner's interface-only status).

## 3. Open issues

| Issue | Owner | Tracking location |
|---|---|---|
| ADR-054 (project formal name) 待定 — P0 used the working tokens `zelyr` / `hypervisor`; the artifact-naming migration path is recorded | owner, before public protocol freeze | adr-000 register; artifact-naming §4.1 |
| ADR-055–ADR-058 待定 items | owning stages (P5/P6/P8 per register notes) | adr-000 register (untouched by P0) |
| ADR governance narrow-edit rule §4(b) (register pointer annotations) applied by design; owner may decline | maintainer | W06 implementation record |
| W16 `platform` field required-but-unfilled (platform vocabulary not yet named) | the design that names the platform vocabulary (P2 platform-discovery scope) | W16 verification record prerequisite notes |
| W16 `capability_summary` reserved until W04 profile semantics land in-tree | the first profile-implementing design | W16 verification record |
| W14 management-domain untrusted-input final class assignment | P5 hypercall/management-ABI error-boundary design | failure-classification §2 (recorded open item) |
| P8 proposed designs contain five forward references to their own future records | P8 implementing packages | W05 verification record (recorded observation); exempted by the recorded QG-DOCS class |

## 4. Scope honesty

- Reserved and out-of-scope boundaries: see the [P0 task book §1](../task-book-v0.1.md)
  (restated by pointer, per the single-home rule).
- **What P0 evidence does not prove:** no EL2 execution, no guest boot, no
  QEMU execution, no hardware behavior — host results prove host-side logic
  only ([host-test proof boundary](../../../testing/host-test-baseline.md));
  the runner entry is interface-only ([placeholder marking](../../../testing/qemu-runner-entry.md));
  CI checks prove only the configured gates, never future-class scope
  ([CI baseline §5](../../../development/ci-baseline.md)); the target build
  proves the compilation chain only ([build-target baseline](../../../development/build-target-baseline.md)).

## 5. P1 handoff package (assembled by linking, per the handoff rule)

Per task book §7, the package consists of:

- This task book: [task-book-v0.1.md](../task-book-v0.1.md) and this
  completion report.
- Toolchain and target/build baseline: [W02](../../../development/toolchain-baseline.md)
  + `rust-toolchain.toml`; [W03](../../../development/build-target-baseline.md)
  + workspace manifests.
- Quality/CI and test baseline: [W07](../../../development/quality-gates.md);
  [W20](../../../development/ci-baseline.md) + `.github/workflows/ci.yml` +
  `main` protection; [W08](../../../testing/host-test-baseline.md).
- QEMU runner baseline: [W09](../../../testing/qemu-runner-entry.md).
- Diagnostics/metadata rules: [W12](../../../development/diagnostics-baseline.md);
  [W13](../../../development/trace-event-namespace.md);
  [W16](../../../development/version-build-metadata.md);
  [W17](../../../development/artifact-naming.md).
- Unsafe and dependency policy: [W10](../../../security/unsafe-rust-policy.md) +
  [inventory](../../../security/unsafe-inventory.md);
  [W18](../../../development/dependency-governance.md) +
  [register](../../../development/dependency-register.md).
- ADR and documentation workflow: [W06](../../../adr/README.md);
  [W05](../../../development/documentation-baseline.md).
- Platform guardrails: [W11](../../../development/platform-portability-rules.md).
- Feature/profile governance: [W04](../../../development/build-profile-governance.md).
- Failure, type-safety, and platform governance: [W14](../../../security/failure-classification.md);
  [W15](../../../development/address-identifier-type-safety.md).
- Contributor path and stage workflow: [W19](../../../development/contributor-workflow.md);
  [W21](../../../development/stage-workflow.md); integration
  [policy](../../../development/integration-workflow.md).
- The dependency map: [p0-handoff-map.md](../p0-handoff-map.md) — the entry
  point that links every row above with status and evidence.
