# P0-W03 AArch64 Build-Target Baseline — Implementation Record

**Status:** Implemented on branch `p0/w03-aarch64-target`; verification
evidence in [the verification
record](../verification/p0-w03-aarch64-build-target-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W03 detailed implementation
design](p0-w03-aarch64-build-target-baseline/README.md)

## Prerequisite state observed

- Baseline commit: `b82a387` (merge of PR #13, the W02 toolchain baseline).
- The W02 deliverable `rust-toolchain.toml` exists with the assumed schema
  (exact pin `1.98.1`, `profile = "minimal"`, `rustfmt` + `clippy`, empty
  `targets`); the W02 contract v0.1 is relied on unchanged.
- No workspace or member manifest existed; `hypervisor/src/.gitkeep` was the
  only tracked content under `hypervisor/`.

## Confirmed facts

- **Triple:** `aarch64-unknown-none-softfloat` (added as the first manifest
  `targets` entry).
- **Edition:** `2024` — confirmed by a successful build under the pinned
  `rustc 1.98.1`.
- **Resolver:** `3` — accepted by `cargo 1.98.1` for the virtual workspace
  with no resolver recommendation emitted at build time.

## Changed artifacts

| Artifact | Change |
|---|---|
| `Cargo.toml` (new) | Virtual workspace manifest: resolver 3, single member `hypervisor`; no profiles, features, dependencies, or patches |
| `hypervisor/Cargo.toml` (new) | Member manifest: `hypervisor` 0.1.0, edition 2024, zero dependencies |
| `hypervisor/src/main.rs` (new; replaces `hypervisor/src/.gitkeep`) | Build-chain probe: crate attributes plus the two placeholder symbols (assembly `_start` park loop, `#[panic_handler]` park), labeled with their future owners |
| `rust-toolchain.toml` | One target entry added (`aarch64-unknown-none-softfloat`); nothing else changed |
| `docs/development/build-target-baseline.md` (new) | Normative build-target policy v0.1 (classes, triple decision, `no_std` semantics, assembly rule, artifact boundary, change thresholds) |
| `docs/README.md` | One routing-table row for target/build-class work |
| `docs/stages/p0/implementation/README.md` | W03 status row updated truthfully |
| This record; the verification record | Decisions and evidence |

## Deviations from the design

None. No linker script, custom target JSON, `.cargo/` configuration, build
script, dependency, feature, profile, host member, or CI workflow was added;
the probe contains exactly the two placeholder symbols.

## Handoff notes for downstream packages

- **W07:** the target build path builds warning-free under toolchain
  defaults; gate command spelling and failure semantics are W07's.
- **W09:** a target artifact exists under
  `target/aarch64-unknown-none-softfloat/<profile>/`; the runner interface is
  W09's.
- **W16/W17:** the member `version` is a placeholder; identity and naming
  semantics are theirs and must not treat it as an identity contract.
- **W19:** the target-build entry (§6 of the policy) is one step of the
  clone-to-build path; until W08 there is no host-buildable member.
- **W20:** CI provisions toolchain and target through the W02 manifest per the
  parity rule; the build path is the required-check input.
- **P1:** reserved extension positions (custom target-spec JSON, linker
  script, build script, `.cargo/` configuration, FP/SIMD enablement) are
  reachable only through the policy's §7 thresholds; no EL2 entry, vector, or
  layout decision has been pre-taken.
