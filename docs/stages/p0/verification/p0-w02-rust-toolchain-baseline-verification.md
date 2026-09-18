# P0-W02 Rust Toolchain Baseline — Verification Evidence

**Status:** Complete evidence recorded; W02 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Linux (WSL2, x86_64), rustup 1.29.1, network available; the
restoration exercise used an isolated `RUSTUP_HOME=/tmp/w02-rustup`,
`CARGO_HOME=/tmp/w02-cargo` sandbox bootstrapped with
`rustup-init --profile minimal --default-toolchain none` (no preinstalled
toolchain).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W02-DV01 → P0-V02 | Manifest schema review | **passed** | `python3 tomllib` parse: `channel = "1.98.1"`, `profile = "minimal"`, `components = ["rustfmt", "clippy"]`, `targets = []`; pointer comment intact; no triple, no extra component. |
| W02-DV02 → P0-V02 | Clean-restore exercise | **passed** | Sandbox began with `no installed toolchains`; one `cargo --version` invocation inside the repository tree auto-provisioned the pin: `rustup show active-toolchain` → `1.98.1-x86_64-unknown-linux-gnu (overridden by '.../rust-toolchain.toml')`; `rustc 1.98.1`, `rustfmt 1.9.0-stable`, `clippy 0.1.98` all present. |
| W02-DV03 → P0-V02 | Repeated-lifecycle check | **passed** | Second invocation inside the sandbox printed no provisioning output; toolchain list unchanged (`1.98.1-x86_64-unknown-linux-gnu (active)`); installed-component diff empty ("COMPONENTS UNCHANGED"). |
| W02-DV04 → P0-V02 | Single-source review | **passed** | `git grep` for channel pins and toolchain mentions across tracked TOML/YAML/shell files: no other authoritative toolchain declaration exists (search output: "no other channel pin", "no other toolchain-mentioning machine files"). |
| W02-DV05 → P0-V02/P0-V09 | Policy review | **passed** | `docs/development/toolchain-baseline.md` contains §§1–9: single source, channel, component, target policy, restoration with failure boundaries, three update thresholds (routine / policy decision / ADR), CI parity rule, unstable rule, integrity note; no contradiction with the ADR baseline, task book, or Coding Guidelines; no AArch64 triple and no gate command spelling named. |
| W02-DV06 → P0-V09 | Discovery and link review | **passed** | `docs/README.md` routing row reaches `docs/development/toolchain-baseline.md` in one link; the contract links the manifest, ADR index, and implementation record; all checked relative paths exist from the checkout; implementation-index row updated truthfully. |
| W02-DV07 → W02 closure | Consumability review | **passed** | Read as W03 (manifest `targets` mechanism ready, triple intentionally undefined), W07 (rustfmt/clippy guaranteed present under the pin), W19 (restoration inputs and verification commands in §5), W20 (§7 parity rule is binding); each consumer can act without inventing policy. |

## Commands and observed results

```text
curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
  (isolated RUSTUP_HOME/CARGO_HOME sandbox)
rustup toolchain list
  no installed toolchains
cargo --version                      # first invocation inside the repo tree
  cargo 1.98.1 (797e8a9bc 2026-08-05)
rustup show active-toolchain
  1.98.1-x86_64-unknown-linux-gnu (overridden by '.../rust-toolchain.toml')
rustc --version
  rustc 1.98.1 (48a229cea 2026-09-01)
cargo fmt --version
  rustfmt 1.9.0-stable (48a229ceae 2026-09-01)
cargo clippy --version
  clippy 0.1.98 (48a229ceae 2026-09-01)
rustup target list --installed
  x86_64-unknown-linux-gnu
cargo --version                      # second invocation (idempotency)
  cargo 1.98.1 (797e8a9bc 2026-08-05)
rustup toolchain list
  1.98.1-x86_64-unknown-linux-gnu (active)
diff components-before components-after
  COMPONENTS UNCHANGED
git grep -n -iE 'channel = "(stable|nightly|beta|[0-9.]+)"' (excluding manifest)
  no other channel pin
python3 tomllib parse of rust-toolchain.toml
  {'toolchain': {'channel': '1.98.1', 'profile': 'minimal',
                 'components': ['rustfmt', 'clippy'], 'targets': []}}
```

Note on the installed-target list: rustup provisions the host standard library
(`x86_64-unknown-linux-gnu`) as part of the toolchain itself; the manifest
`targets` list is empty and no cross-compilation target is installed. The
AArch64 bare-metal target entry is W03 scope.

## Not run / not proved

- **CI consumption (W02-DV04's full proof, P0-V08):** no CI workflow exists
  yet; the developer/CI parity rule is binding policy, with execution evidence
  deferred to W20 by design.
- **Offline restoration:** not exercised; explicitly not promised by contract
  §5.
- **Host build, host tests, AArch64 target build:** not run; W02 does not
  authorize workspaces, targets, or gates (W03/W07/W08 scope).
- **Non-rustup hosts and non-x86_64 hosts:** not exercised; unsupported for
  gate-bearing work per contract §5.
