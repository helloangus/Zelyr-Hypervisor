# P0-W03 Target Baseline Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W03 detailed design](README.md).

## 1. Logical artifact groups and ownership

W03 is build-configuration, documentation, and build-chain-probe work, so its
logical modules are authoritative artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Build-target policy document | `docs/development/build-target-baseline.md` | this design, ADR/task-book constraints, the resolved triple | the sole normative home of build classes, the hypervisor target, `no_std` build semantics, artifact boundary, extension points, and change thresholds; it does not document gate commands (W07), CI (W20), profiles/features (W04), or the toolchain policy (W02) |
| Toolchain manifest target entry | root `rust-toolchain.toml` (artifact owned by the [W02 contract](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md) §2) | the resolved triple | one entry in the `targets` list; W03 changes nothing else in the manifest and adds no second declaration mechanism |
| Workspace manifest | root `Cargo.toml` | resolved workspace shape | workspace membership and resolver only; no profiles, features, dependencies, or patch entries |
| Baseline member crate | `hypervisor/` (`Cargo.toml` + `src/main.rs`; the `src/.gitkeep` marker is removed in the same change) | resolved probe contracts | the two placeholder symbols and nothing else; it is not an EL2 skeleton and owns no runtime contract |
| Documentation routing | one row in `docs/README.md` routing table | policy document location | discoverability of the policy document; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w03-aarch64-build-target-baseline-record.md` (created when work starts) | actual decisions taken | confirmed edition/triple facts, changed artifacts, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w03-aarch64-build-target-baseline-verification.md` (created when evidence exists) | actual commands and output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Prerequisite contract consumed from W02

The manifest edit assumes the [W02 toolchain
contract](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md) as
proposed: a root `rust-toolchain.toml` with an exact version pin, `profile =
"minimal"`, baseline components, and a `targets` list that starts empty and is
populated only by an approved design naming the triple (§3.4 names W03 as the
expected first). Failure boundary: if that file does not exist, or its schema
differs materially, when W03 implementation starts, the target entry step is
blocked — record the blocker and reconcile with W02's delivered contract first.
Do not create a competing toolchain declaration, a second pin file, or a
`rustup`-outside-the-manifest procedure; those are W02-scope decisions.

## 3. Build-class target classification (required policy content)

The policy document must define exactly these four build classes and the
separation rules between them. Nothing else is a build class.

| Class | Target | Purpose | Owner of the class's build path | Status |
|---|---|---|---|---|
| Host-native builds | the pinned toolchain's default host target, as reported by the toolchain at build time | developer-machine programs: future host test members (W08), developer tooling | W08 for test members; the introducing package for any tool | Available class; no member exists until W08 |
| Bare-metal hypervisor | `aarch64-unknown-none-softfloat` | the freestanding hypervisor build that P0-V05 exercises and P1 brings up | W03 (this design); extension via §8 | Active in P0 |
| Future guest builds | Reserved — no target named | the future Validation Guest and later guest images | the P4-stage guest design | Reserved; must not be declared, built, or built-for in P0 |
| Development tooling | the host class's target (host-native) | repository-support tools that run on developer machines, distinct from tested host programs | the package that introduces the tool | Available class; empty in P0 |

Separation rules that the policy document must state:

- Every documented build path names its class. A build invocation without a
  class is not a repository entry point.
- Two classes must never share one target declaration "for convenience"; the
  guest class in particular must not reuse the hypervisor target entry.
- The host target is machine-dependent and is never spelled as a project pin in
  any tracked file; documents refer to "the pinned toolchain's host target".
- The hypervisor class never produces a host-executable artifact, and host
  classes never consume the hypervisor artifact as an input (see §4).

## 4. Hypervisor target decision (required policy content)

The policy document must record, as the normative hypervisor target:

- **Triple:** `aarch64-unknown-none-softfloat` — the built-in Tier-2 bare-metal
  AArch64 target with precompiled `rust-std` distributed through the pinned
  toolchain, so no `build-std` path and no `rust-src` component are required.
- **Rationale:** freestanding AArch64 with no OS ABI contract (matching the
  ADR's Rust-first, `no_std`, AArch64-first baseline); and, because the
  soft-float target excludes FP/SIMD from codegen, the compiled baseline
  carries no floating-point/SIMD premise. FP/SIMD context management is
  explicitly a later architecture concern; enabling it is a Reserved change
  (§8), never a silent default.
- **Boundary statement:** the target choice is a compilation-chain decision.
  It asserts nothing about CPU mode at entry, exception level, MMU state, or
  any boot protocol; those are P1 entry-design decisions.

## 5. `no_std` build semantics and extension positions (required policy content)

The policy document must fix, for the hypervisor class:

- `#![no_std]` and `#![no_main]` semantics; the binary is freestanding and has
  no C runtime beyond the target's self-contained linking.
- The panic extension position: the member's single `#[panic_handler]` symbol
  is the designated location where panic policy will later exist. Its P0
  content is a placeholder ([probe contract](02-workspace-and-probe-crate-contract.md)
  §3); failure-classification semantics belong to W14 and crash-information
  semantics to W12. The placeholder must not implement policy.
- The linking extension position: P0 uses the built-in target linking with a
  linkable entry symbol and no custom linker script. When a P1 design requires
  image layout control, the linker script is introduced under the member
  directory and wired through the build path by that design; its name, content,
  and entry layout are owned by the design that introduces them. W03 fixes no
  layout fact.
- Edition policy: the baseline uses the current stable edition supported by the
  pinned toolchain (edition 2024 as of this design), declared in the member
  manifest; an edition bump is routine reviewed maintenance recorded per the
  thresholds in §8.
- Profile policy: no `[profile.*]` sections anywhere in the workspace; the
  baseline builds with Cargo's built-in `dev` and `release` configurations
  only. Custom profiles and feature semantics are governed by
  [P0-W04](../p0-w04-build-profile-feature-governance/README.md); W03 must not
  pre-empt them.
- Dependency policy: zero dependencies and zero build-dependencies. The first
  dependency enters only through W18's governance.
- Warning posture: the member builds warning-free under toolchain defaults;
  lint and warning policy is W07's.

## 6. Assembly build capability (required policy content)

The policy document must state that assembly enters the hypervisor build only
at necessary architectural boundaries, as permitted by the ADR and Coding
Guidelines, and that in P0 the only assembly is the probe's entry stub. The
stub's contract ([probe contract](02-workspace-and-probe-crate-contract.md) §3)
forbids system-register access, vector-table construction, MMU or cache
maintenance, and console output. A successful build of the member is itself the
proof that the Rust-and-ASM joint build path works, because the `global_asm!`
input is assembled as part of the build; disassembly inspection is an optional
informative aid, not a P0 requirement.

## 7. Baseline artifact boundary (required policy content)

The policy document must fix the output boundary:

- The hypervisor build produces an ELF image located under
  `target/<triple>/<profile>/` with the member's package name as the file name
  (the concrete path is machine-generated by Cargo and is not a tracked
  contract; `target/` is already ignored by the W01 baseline).
- The artifact is generated output: never tracked, never committed, and its
  arch/platform/profile/version naming evolution is owned by W17 with identity
  semantics from W16. W03 asserts only "identifiable by class, triple, and
  profile directory".
- Relationship to host artifacts: all classes share the `target/` root but are
  segregated per target triple by Cargo; host-class outputs land under the host
  triple directories, the hypervisor artifact under the AArch64 triple
  directory. No cross-class consumption is permitted (§3).
- Until W08 delivers a host-class member, the workspace has no host-buildable
  member; a bare host `cargo build` is therefore not a supported repository
  entry, and this boundary must be stated so it is read as a designed state,
  not an omission. W19 documents it as such.

## 8. Target change thresholds (required policy content)

Mirroring the W02 contract's escalation style:

- **Routine maintenance** (ordinary PR review plus re-running the affected
  build): adding or removing a manifest target entry per an approved design
  (for example, the future guest class); the edition bump rule of §5;
  toolchain-driven updates that follow the
  [W02 update rules](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md)
  §3.6.
- **Policy decision required** (a reviewed design change before the change):
  changing the hypervisor triple; introducing a custom target-spec JSON; a
  linker script, build script, or `.cargo/` configuration for the hypervisor
  class; enabling FP/SIMD codegen on the hypervisor target (this change is
  coordinated with the P1 entry design that will own FP context); declaring the
  guest class target.
- **ADR required:** any variant that would make unstable features or a
  nightly channel load-bearing (routed through the
  [W02 thresholds](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md)
  §3.6), or that would alter an architectural constraint of the ADR baseline.
  The policy document must name the first two thresholds explicitly and give
  the ADR rule by reference to `docs/adr/README.md`.

## 9. Explicitly excluded code interfaces

No public Rust API, trait, module tree, crate boundary beyond the single
member, ABI, wire format, or persistent layout is designed or authorized. The
only code interfaces in scope are the two placeholder symbols contracted in
[the workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md)
§3. Any additional function, type, or assembly routine discovered to be
"needed" during implementation is a scope conflict: stop and record it against
the owning package (P1, W12, or W14) instead of adding it.
