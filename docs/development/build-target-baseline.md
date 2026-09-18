# Zelyr Build-Target Baseline

**Status:** Normative build-target policy.  
**Scope:** Build-class classification, the hypervisor target decision,
`no_std` build semantics with extension positions, the assembly rule, the
baseline artifact boundary, and target-change thresholds. It does not document
quality-gate commands (P0-W07), CI (P0-W20), profiles/features (P0-W04), or
toolchain policy (see the [toolchain baseline](toolchain-baseline.md)).  
**Version:** v0.1  
**Owner/change context:** P0-W03 build-target baseline.  
**Supersedes:** The absence of a build-target policy (no prior build path
existed in the repository).

## 1. Build classes

The repository defines exactly these four build classes. Nothing else is a
build class.

| Class | Target | Purpose | Owner of the class's build path | Status |
|---|---|---|---|---|
| Host-native builds | the pinned toolchain's default host target, as reported by the toolchain at build time | developer-machine programs: future host test members (P0-W08), developer tooling | P0-W08 for test members; the introducing package for any tool | Available class; no member exists yet |
| Bare-metal hypervisor | `aarch64-unknown-none-softfloat` | the freestanding hypervisor build that P0-V05 exercises and P1 brings up | P0-W03; extension via §7 | Active in P0 |
| Future guest builds | Reserved — no target named | the future Validation Guest and later guest images | the P4-stage guest design | Reserved; must not be declared, built, or built-for in P0 |
| Development tooling | the host class's target (host-native) | repository-support tools that run on developer machines, distinct from tested host programs | the package that introduces the tool | Available class; empty in P0 |

Separation rules:

- Every documented build path names its class. A build invocation without a
  class is not a repository entry point.
- Two classes must never share one target declaration "for convenience"; the
  guest class in particular must not reuse the hypervisor target entry.
- The host target is machine-dependent and is never spelled as a project pin
  in any tracked file; documents refer to "the pinned toolchain's host
  target".
- The hypervisor class never produces a host-executable artifact, and host
  classes never consume the hypervisor artifact as an input (§5).

## 2. Hypervisor target decision

- **Triple:** `aarch64-unknown-none-softfloat` — the built-in Tier-2
  bare-metal AArch64 target whose precompiled `rust-std` is distributed
  through the [pinned toolchain](toolchain-baseline.md); no `build-std` path
  and no `rust-src` component are required.
- **Rationale:** freestanding AArch64 with no OS ABI contract, matching the
  ADR's Rust-first, `no_std`, AArch64-first baseline; and, because the
  soft-float target excludes FP/SIMD from codegen, the compiled baseline
  carries no floating-point/SIMD premise. FP/SIMD context management is
  explicitly a later architecture concern; enabling it is a Reserved change
  (§7), never a silent default.
- **Boundary statement:** the target choice is a compilation-chain decision.
  It asserts nothing about CPU mode at entry, exception level, MMU state, or
  any boot protocol; those are P1 entry-design decisions.

## 3. `no_std` build semantics and extension positions

For the hypervisor class:

- `#![no_std]` and `#![no_main]` semantics: the binary is freestanding and has
  no C runtime beyond the target's self-contained linking.
- **Panic extension position:** the member's single `#[panic_handler]` symbol
  is the designated location where panic policy will later exist. Its P0
  content is a placeholder; failure-classification semantics belong to P0-W14
  and crash-information semantics to P0-W12. The placeholder must not
  implement policy.
- **Linking extension position:** P0 uses the built-in target linking with a
  linkable entry symbol and no custom linker script. When a P1 design requires
  image-layout control, that design introduces the linker script under the
  member directory and wires it through the build path; its name, content, and
  entry layout are owned by the introducing design. This policy fixes no
  layout fact.
- **Edition policy:** the baseline uses the current stable edition supported
  by the pinned toolchain (edition 2024, confirmed at implementation); an
  edition bump is routine reviewed maintenance recorded per §7.
- **Profile policy:** no `[profile.*]` sections anywhere in the workspace; the
  baseline builds with Cargo's built-in `dev` and `release` configurations
  only. Custom profiles and feature semantics are governed by
  [P0-W04](../stages/p0/plans/p0-w04-build-profile-feature-governance.md).
- **Dependency policy:** zero dependencies and zero build-dependencies. The
  first dependency enters only through P0-W18's governance.
- **Warning posture:** the member builds warning-free under toolchain
  defaults; lint and warning policy is P0-W07's.

## 4. Assembly rule

Assembly enters the hypervisor build only at necessary architectural
boundaries, as permitted by the ADR and the Coding Guidelines. In P0 the only
assembly is the probe's entry stub, whose contract forbids system-register
access, vector-table construction, MMU or cache maintenance, and console
output. A successful build of the member is itself the proof that the
Rust-and-ASM joint build path works, because the `global_asm!` input is
assembled as part of the build; disassembly inspection is an optional
informative aid, not a P0 requirement.

## 5. Baseline artifact boundary

- The hypervisor build produces an ELF image under
  `target/<triple>/<profile>/` with the member's package name as the file
  name. The concrete path is machine-generated by Cargo and is not a tracked
  contract; `target/` is ignored by the repository baseline.
- The artifact is generated output: never tracked, never committed, and never
  executed on the development host. Its arch/platform/profile/version naming
  evolution is owned by P0-W17 with identity semantics from P0-W16; this
  policy asserts only "identifiable by class, triple, and profile directory".
- All classes share the `target/` root but are segregated per target triple by
  Cargo: host-class outputs land under the host triple directories, the
  hypervisor artifact under the AArch64 triple directory. No cross-class
  consumption is permitted (§1).
- Until P0-W08 delivers a host-class member, the workspace has no
  host-buildable member; a bare host `cargo build` is therefore not a
  supported repository entry. That is a designed state, not an omission, and
  must not be "fixed" by making the hypervisor member host-buildable; the
  P0-W19 reproducible workflow documents it as such.

## 6. Build invocation

Suggested observation (not a quality gate; P0-W07 owns gate spelling):

```sh
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
```

The invocation must run in an environment restored through the
[toolchain baseline](toolchain-baseline.md); the manifest-driven provisioning
installs the target's `rust-std` together with the toolchain.

## 7. Target change thresholds

- **Routine maintenance** (ordinary PR review plus re-running the affected
  build): adding or removing a manifest target entry per an approved design
  (for example, the future guest class); the edition-bump rule of §3;
  toolchain-driven updates that follow the
  [toolchain update rules](toolchain-baseline.md).
- **Policy decision required** (a reviewed design change before the change):
  changing the hypervisor triple; introducing a custom target-spec JSON; a
  linker script, build script, or `.cargo/` configuration for the hypervisor
  class; enabling FP/SIMD codegen on the hypervisor target (coordinated with
  the P1 entry design that will own FP context); declaring the guest-class
  target.
- **ADR required:** any variant that would make unstable features or a
  nightly channel load-bearing (routed through the
  [toolchain thresholds](toolchain-baseline.md)), or that would alter an
  architectural constraint of the ADR baseline. Lifecycle and supersession
  rules are in the [ADR index](../adr/README.md).
