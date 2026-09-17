# P0-W02 Toolchain Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W02 detailed design](README.md).

## 1. Logical artifact groups and ownership

W02 is configuration and documentation work, so its logical modules are
authoritative artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Toolchain pin manifest | root `rust-toolchain.toml` | resolved design decisions, selected stable version | machine-consumed pin values; it does not carry policy prose beyond a pointer comment |
| Toolchain contract document | `docs/development/toolchain-baseline.md` | this design, ADR/task-book constraints | the sole normative home of channel, component, target, restoration, update, and unstable rules; it does not document build commands, gate definitions, or CI configuration |
| Documentation routing | one row in `docs/README.md` routing table | contract document location | discoverability of the contract; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w02-rust-toolchain-baseline-record.md` (created when work starts) | actual decisions taken | selected version, changed artifacts, deviations; no command logs (those live in verification) |
| Verification record | `docs/stages/p0/verification/p0-w02-rust-toolchain-baseline-verification.md` (created when evidence exists) | actual commands and output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Pin manifest contract

Create exactly one root file `rust-toolchain.toml` with this schema:

```toml
# Zelyr pinned Rust toolchain baseline.
# Normative policy: docs/development/toolchain-baseline.md — do not duplicate it here.
[toolchain]
channel = "<X.Y.Z>"          # exact three-part stable version; never "stable", beta, or a partial pin
profile = "minimal"          # rustc + cargo + rust-std only; components are added explicitly
components = ["rustfmt", "clippy"]
targets = []                 # populated only by an approved work-package design (first: W03)
```

Field rules:

- `channel` — the full three-part released stable version. The value is
  selected at implementation time by the rule in
  [the workflow](02-implementation-and-review.md) step 1 and recorded in the
  implementation record. A partial pin (`"1.84"`), a moving alias (`"stable"`),
  or a pre-release is prohibited because it breaks identical restoration.
- `profile` — fixed to `"minimal"` so the provisioned set is exactly the
  declared set and nothing else.
- `components` — `rustfmt` and `clippy` are the only baseline-required
  components (W07's P0-V06/V07 gates need them). `rust-src` is Reserved until
  an approved design selects `build-std` or requires core sources;
  `llvm-tools-preview` is Reserved until an approved artifact-inspection or
  crash-diagnostics design requires it. Any component change is a reviewed
  manifest edit under the contract's update rules.
- `targets` — starts empty. Adding an entry requires an approved design that
  names the exact triple; W03 is the expected first. No W02 artifact may name
  an AArch64 triple.
- The file must remain valid TOML, must keep the pointer comment, and must not
  gain policy paragraphs (policy belongs to the contract document).

This file takes effect for the whole repository tree for every rustup-driven
`cargo`/`rustc` invocation from any subdirectory. It has no effect on
non-rustup toolchains, which the contract declares unsupported for
gate-bearing work.

## 3. Toolchain contract document contract

Create `docs/development/toolchain-baseline.md` as a normative document with
the status header required by `docs/README.md` (status, scope, version `v0.1`,
owner/change context, supersedes: none) and exactly the following sections.
Required content is stated per section; additional informative detail is
allowed but must not contradict a required statement.

### 3.1 Single source of truth

`rust-toolchain.toml` is the only authoritative toolchain declaration for the
repository. No other tracked file — script, CI configuration, or document —
may pin a toolchain version, channel, component set, or target set. Documents
reference the contract instead of restating values.

### 3.2 Channel policy

The project baseline channel is stable, pinned by exact version. The section
must state this decision, that it was taken to keep every toolchain update
reviewable and every gate result reproducible, and that no unstable feature is
assumed anywhere in the repository.

### 3.3 Component policy

Baseline-required: `rustfmt`, `clippy`. Reserved with triggers: `rust-src`
(only when an approved design selects `build-std` or needs core sources),
`llvm-tools-preview` (only when an approved inspection/crash-diagnostics
design requires it). Components are installed through the manifest, never by
undocumented local `rustup component add` as a project prerequisite.

### 3.4 Target policy

The manifest `targets` list carries every cross-compilation target the
repository actually uses. An entry is added only through an approved
work-package design that names the triple and records its need; removal
follows the same path. The concrete AArch64 bare-metal target is owned by
W03's design. The contract must not name any triple.

### 3.5 Restoration from a clean environment

The declared procedure must satisfy: given a machine with Git, network access,
and a rustup-capable host, a contributor reaches the exact pinned toolchain
using only repository-declared inputs. The section must contain, at minimum:

- **Declared inputs:** Git, a host platform supported by rustup, network
  access to the official Rust distribution, and rustup itself (with a pointer
  to its official installer; OS-specific package steps stay out of the
  repository).
- **Provisioning behavior:** any `cargo`/`rustc` invocation inside the
  repository causes rustup to provision the pinned toolchain, profile, and
  components automatically and idempotently.
- **Verification commands:** check the active toolchain equals the pin, that
  `rustfmt` and `clippy` are installed, and that the installed target list
  equals the manifest `targets` list.
- **Failure boundaries:** explicitly state what is *not* promised — offline
  restoration, hosts without rustup support, distribution artifacts withdrawn
  upstream, and a pin that no longer resolves. A failure to restore is a
  blocker to record, never a reason to silently substitute another toolchain.
- A statement that toolchains provisioned outside the pin file are unsupported
  for gate-bearing work.

### 3.6 Update, review, and ADR thresholds

- **Routine maintenance** (ordinary PR review plus re-running the affected
  P0 gates): a pin bump to a newer released stable version; adding or removing
  a manifest target per an approved design; a component change that does not
  alter gate semantics; an expedited pin bump responding to a toolchain
  security advisory.
- **Policy decision required** (recorded issue and owner decision before the
  change): switching the baseline channel to nightly or beta; adopting any
  unstable feature or `-Z` flag in a project build path; changing the pinning
  mechanism or provisioning manager; reclassifying a Reserved component.
- **ADR required:** a change that would make unstable features load-bearing
  for the hypervisor TCB or otherwise alter an architectural constraint
  (per the ADR baseline's change rules). The section must name the first two
  thresholds explicitly and give the ADR rule by reference to
  `docs/adr/README.md`.

### 3.7 Developer and CI parity rule

Developers and CI must obtain the toolchain from the repository manifest via
rustup. A CI job must not install a toolchain version or component set that
diverges from the manifest, even when the manifest change has already been
merged to the CI's base branch. CI wiring and its evidence belong to W20;
until then this section is the binding requirement W20 implements.

### 3.8 Unstable feature rule

Unstable (`#![feature]`) code, `-Z` compiler flags, and nightly-only options
must not appear in any project build, test, or gate path while the baseline
channel is stable; the pinned toolchain enforces this by failing to compile
them. A need for unstable capability must be raised as a policy decision per
§3.6 with the consuming design attached; local untracked experiments do not
change the baseline and must not be committed.

### 3.9 Integrity boundary (informative within the contract)

Pinning fixes the toolchain identity, not its supply chain. Distribution
integrity is delegated to rustup's signed manifests for the official channel.
Mirrors, checksum pinning, or vendored toolchains are Reserved future
governance and would be a new decision.

## 4. Explicitly excluded code interfaces

There are no Rust structs, enums, traits, functions, modules, crates, Cargo
manifests or workspaces, target triples, shell scripts, CI workflow files, or
public APIs in this design. Adding any of them is a scope conflict requiring
the applicable detailed design (at minimum W03 for targets and workspace, W07
for gate commands, W20 for CI) and must be stopped at review.
