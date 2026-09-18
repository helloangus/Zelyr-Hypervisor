# Zelyr Rust Toolchain Baseline

**Status:** Normative toolchain policy.  
**Scope:** Channel, version pinning, components, targets, restoration,
update/review thresholds, developer/CI parity, and unstable-feature rules for
the whole repository. It does not document build commands, quality-gate
definitions, or CI configuration, which belong to their owning work packages.  
**Version:** v0.1  
**Owner/change context:** P0-W02 toolchain baseline.  
**Supersedes:** The absence of a toolchain policy (no prior toolchain
declaration existed in the repository).

## 1. Single source of truth

The root [`rust-toolchain.toml`](../../rust-toolchain.toml) is the only
authoritative toolchain declaration for this repository. No other tracked file
— script, CI configuration, or document — may pin a toolchain version,
channel, component set, or target set. Documents reference this contract
instead of restating values.

## 2. Channel policy

The project baseline channel is **stable**, pinned by exact three-part
version. This decision keeps every toolchain update reviewable and every gate
result reproducible. No unstable feature is assumed anywhere in the
repository; a need for one is governed by §6 and §8, never adopted by
accident.

The initial pin is `1.98.1` (released 2026-09-01), selected 2026-09-18 from
the official stable distribution channel as the latest released stable
version; see the [W02 implementation
record](../stages/p0/implementation/p0-w02-rust-toolchain-baseline-record.md).

## 3. Component policy

- **Baseline-required:** `rustfmt` and `clippy`. The development quality gates
  for formatting and linting require them.
- **Reserved with triggers:** `rust-src` (only when an approved design selects
  `build-std` or otherwise requires core sources) and `llvm-tools-preview`
  (only when an approved artifact-inspection or crash-diagnostics design
  requires it). Reclassifying a reserved component is a policy decision under
  §6.
- Components are installed through the manifest, never by undocumented local
  `rustup component add` calls as a project prerequisite.

## 4. Target policy

The manifest `targets` list carries every cross-compilation target the
repository actually uses. An entry is added only through an approved
work-package design that names the exact triple and records its need; removal
follows the same path. The list starts empty; the concrete AArch64 bare-metal
target is owned by the P0-W03 design. This contract intentionally names no
triple.

## 5. Restoration from a clean environment

**Declared inputs:** Git; a host platform supported by rustup; network access
to the official Rust distribution; and rustup itself, installed from its
official installer (<https://rustup.rs>). OS-specific package-manager steps
are intentionally out of the repository.

**Provisioning behavior:** any `cargo` or `rustc` invocation inside the
repository tree causes rustup to read the root manifest and provision the
pinned toolchain, profile, and components automatically and idempotently.
No other step is required.

**Verification commands** (run inside the repository after provisioning):

```sh
rustup show active-toolchain   # reports the pinned channel/version
rustc --version                # equals the pinned version
cargo fmt --version            # rustfmt component present
cargo clippy --version         # clippy component present
rustup target list --installed # equals the manifest `targets` list
```

**Failure boundaries — what is *not* promised:** offline restoration; hosts
without rustup support; continued availability of distribution artifacts
upstream; and a pin that no longer resolves. A restoration failure is a
blocker to record and diagnose, never a reason to silently substitute another
toolchain.

Toolchains provisioned outside the pin file (a manually installed `rustc`, a
distribution package, or a different rustup directory) are **unsupported for
gate-bearing work**.

## 6. Update, review, and ADR thresholds

- **Routine maintenance** — ordinary PR review plus re-running the affected
  P0 quality gates: a pin bump to a newer released stable version; adding or
  removing a manifest target per an approved design; a component change that
  does not alter gate semantics; an expedited pin bump responding to a
  toolchain security advisory.
- **Policy decision required** — a recorded issue and owner decision before
  the change: switching the baseline channel to nightly or beta; adopting any
  unstable feature or `-Z` flag in a project build path; changing the pinning
  mechanism or provisioning manager; reclassifying a Reserved component.
- **ADR required** — a change that would make unstable features load-bearing
  for the hypervisor TCB or otherwise alter an architectural constraint. The
  lifecycle and supersession rules are in the [ADR
  index](../adr/README.md).

## 7. Developer and CI parity rule

Developers and CI must obtain the toolchain from the repository manifest via
rustup. A CI job must not install a toolchain version or component set that
diverges from the manifest, even when the manifest change has already been
merged to the CI's base branch. CI wiring and its enforcement evidence belong
to P0-W20; until then this section is the binding requirement that W20
implements.

## 8. Unstable feature rule

Unstable (`#![feature]`) code, `-Z` compiler flags, and nightly-only options
must not appear in any project build, test, or gate path while the baseline
channel is stable; the pinned toolchain enforces this by failing to compile
them. A need for unstable capability must be raised as a policy decision per
§6 with the consuming design attached. Local untracked experiments do not
change the baseline and must not be committed.

## 9. Integrity boundary (informative)

Pinning fixes the toolchain identity, not its supply chain. Distribution
integrity is delegated to rustup's signed manifests for the official channel.
Mirrors, checksum pinning, and vendored toolchains are Reserved future
governance; adopting any of them is a new decision.
