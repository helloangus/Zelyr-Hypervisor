# P0-W03 AArch64 Build-Target Baseline — Verification Evidence

**Status:** Complete evidence recorded; W03 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Linux (WSL2, x86_64); the build exercise ran in the isolated
`RUSTUP_HOME=/tmp/w02-rustup`, `CARGO_HOME=/tmp/w02-cargo` sandbox restored
through the repository manifest per the W02 contract §3.5 (the same sandbox
was re-provisioned incrementally by rustup when the manifest gained the
target entry).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W03-DV01 → P0-V05 | Declaration and classification review | **passed** | `rust-toolchain.toml` gained exactly one `targets` entry; both Cargo manifests match the contract schemas (virtual workspace, resolver 3, single member; member with zero dependencies/features/profiles); the policy document defines the four classes with separation rules; no second toolchain/target declaration mechanism exists in tracked files. |
| W03-DV02 → P0-V05 | Target build exercise | **passed** | In the W02-restored sandbox, `cargo build --target aarch64-unknown-none-softfloat -p hypervisor` finished successfully; rustup provisioned the target's `rust-std` from the updated manifest (`info: downloading component rust-std`); `file target/aarch64-unknown-none-softfloat/debug/hypervisor` → `ELF 64-bit LSB executable, ARM aarch64, statically linked`; the `global_asm!` entry stub was assembled as part of the build (informative `readelf -s`: `_start` present). |
| W03-DV03 → P0-V05 | Repeated-lifecycle check | **passed** | Second build with no source change: `Finished ... in 0.00s` (incremental no-op success); after `cargo clean`, a clean rebuild reproduced the AArch64 ELF artifact. |
| W03-DV04 → P0-V05 | Probe scope review | **passed** | `hypervisor/src/main.rs` contains exactly the crate attributes and the two placeholder symbols; the `_start` stub is a `wfi` park loop with no sysreg/vector/MMU/cache/console work; the panic handler parks without allocation, unwinding, I/O, or payload inspection; both placeholders are labeled with their future owners (P1 entry; W14/W12 panic semantics); zero dependencies; zero warnings under toolchain defaults. |
| W03-DV05 → P0-V09 | Policy coherence review | **passed** | `docs/development/build-target-baseline.md` contains §§1–7 with all contract-required content (classes + separation rules, triple decision + rationale + boundary, `no_std` semantics + two extension positions, assembly rule, artifact boundary incl. the documented host-invocation boundary, three change thresholds); no contradiction with the ADR, task book, W02 contract, W04 governance, or Coding Guidelines; no gate command spelling, CI configuration, feature, or profile statements. |
| W03-DV06 → P0-V09 | Discovery and link review | **passed** | `docs/README.md` routing row reaches the build-target policy in one link; policy links resolve (toolchain baseline, W04 plan, ADR index); implementation-index row updated truthfully. |
| W03-DV07 → W03 closure | Consumability review | **passed** | Read as W07 (a gateable, warning-free build path exists), W09 (artifact source under `target/<triple>/<profile>/`), W16 (identity dimensions: triple, profile, placeholder version), W19 (documented build entry incl. the no-host-member boundary), W20 (build path for required checks, provisioning via the W02 manifest), and a P1 planner (extension points located, nothing pre-taken); each consumer can act without inventing policy. |

## Commands and observed results

```text
# sandbox restored per W02 contract (RUSTUP_HOME/CARGO_HOME isolated)
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
  info: component clippy is up to date
  info: component rustfmt is up to date
  info: downloading component rust-std
  Compiling hypervisor v0.1.0 (.../hypervisor)
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
file target/aarch64-unknown-none-softfloat/debug/hypervisor
  ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV),
  statically linked, with debug_info, not stripped
rustup target list --installed
  aarch64-unknown-none-softfloat
  x86_64-unknown-linux-gnu
cargo build --target aarch64-unknown-none-softfloat -p hypervisor   # re-run
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
cargo clean && cargo build --target aarch64-unknown-none-softfloat -p hypervisor
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
readelf -s target/aarch64-unknown-none-softfloat/debug/hypervisor | grep -w _start
  0000000000210180 T _start
```

## Not run / not proved

- **EL2 execution, QEMU, guest boot, hardware behavior:** not run and not
  claimed; W03 proves the compilation chain only (the P0 task book makes P0
  explicitly not an EL2 bring-up stage).
- **Host-class builds:** not run; the workspace intentionally has no
  host-buildable member until W08 (documented boundary, not an omission).
- **`release`-profile build:** not exercised; the built-in `dev` profile
  carried the validation, and profile governance is W04's.
- **Reproducibility policy:** the clean rebuild reproduces an artifact, but
  bit-level reproducibility and identity metadata are W16/W17 scope.
