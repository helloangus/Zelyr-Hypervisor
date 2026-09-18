# P0-W02 Rust Toolchain Baseline — Implementation Record

**Status:** Implemented on branch `p0/w02-toolchain`; verification evidence in
[the verification record](../verification/p0-w02-rust-toolchain-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W02 detailed implementation design](p0-w02-rust-toolchain-baseline/README.md)

## Selected toolchain pin

- **Version:** `1.98.1`
- **Release date:** 2026-09-01
- **Resolution source:** official stable distribution channel manifest
  (`https://static.rust-lang.org/dist/channel-rust-stable.toml`, manifest dated
  2026-09-03, `[pkg.rust] version = "1.98.1 (48a229cea 2026-09-01)"`)
- **Selection date:** 2026-09-18
- **Selection rule applied:** latest released stable three-part version at
  implementation time; no approved downstream design declared a higher minimum.

## Changed artifacts

| Artifact | Change |
|---|---|
| `rust-toolchain.toml` (new) | Pin manifest: channel `1.98.1`, `profile = "minimal"`, components `rustfmt` + `clippy`, empty `targets`, pointer comment per contract §2 |
| `docs/development/toolchain-baseline.md` (new) | Normative toolchain contract v0.1 with all §3-required sections |
| `docs/README.md` | One routing-table row for toolchain setup/restoration/update |
| `docs/stages/p0/implementation/README.md` | W02 status row updated truthfully |
| This record | Implementation decisions |

## Deviations from the design

None. The manifest, contract document, routing row, and records follow the
approved design without local substitution. No Cargo manifest, target triple,
crate, Rust source, CI workflow, or QEMU artifact was added.

## Handoff notes for downstream packages

- **W03:** the manifest `targets` mechanism is ready; the triple is
  intentionally undefined here.
- **W07:** `rustfmt` and `clippy` are guaranteed present under the pinned
  toolchain; gate command spelling is W07's.
- **W19:** restoration inputs and verification commands in contract §5 are the
  toolchain step of the clone-to-build path.
- **W20:** contract §7 is the binding parity rule; CI wiring and enforcement
  evidence belong to W20.
- **W05:** may re-home or re-version this contract document; semantic
  ownership stays with the document.
