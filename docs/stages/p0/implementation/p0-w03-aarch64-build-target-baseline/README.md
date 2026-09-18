# P0-W03 AArch64 Build Target Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The build-class target classification, the AArch64 bare-metal
`no_std` build path, and the baseline artifact boundary required by
[P0-W03](../../plans/p0-w03-aarch64-build-target-baseline.md).  
**Owner/change context:** P0-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W03. It converts the bounded
work-package plan into small, reviewable build-configuration, documentation,
and build-chain-probe changes: one target-policy document, one target entry in
the toolchain pin manifest, a minimal Cargo workspace with exactly one
`no_std` member, and a target-build validation exercise. It deliberately does
**not** define an EL2 entry point, exception vectors, linker layout, MMU or
console code, any runnable hypervisor behavior, host-side tests, QEMU
automation, CI, features, profiles, or dependencies; those belong to P1, W04,
W07, W08, W09, W18, and W20 respectively.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, ADR baseline, P0 task book, and P0-W03 plan.
This document is the proposed detailed design for those changes; it is not a
completion record and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W03 plan → this design
→ Coding Guidelines. In particular:

- The ADR requires an AArch64-first, Rust-first hypervisor with a `no_std`
  target, permits necessary assembly, and requires a Cargo workspace from the
  first day without over-fine-grained crate splitting (ADR-002, ADR-006,
  ADR-046). It does not name a target triple, so the triple choice is stage-local
  build policy that this design owns within the plan's scope (目标定义).
- The task book outcome for W03 is: an AArch64 bare-metal/`no_std` target can
  produce a **baseline hypervisor artifact**, with host, hypervisor, guest, and
  tooling targets **distinct** (P0-V05). Success is the compilation chain only;
  the task book explicitly makes P0 not an EL2 bring-up stage.
- The plan's out-of-scope list (no EL2 entry, no linker layout details, no
  exception vectors, no runnable hypervisor behavior) bounds how much the
  buildable member may contain. This design therefore introduces a minimal,
  explicitly labeled build-chain probe, not a hypervisor skeleton.
- Profile and feature semantics are governed by
  [P0-W04](../p0-w04-build-profile-feature-governance/README.md). This design
  defines no custom profile and no feature, which is trivially compliant with
  W04's governance; the baseline builds use only Cargo's built-in `dev` and
  `release` configurations.
- The toolchain pin and its target-entry mechanism are governed by the
  [W02 toolchain contract](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md)
  §3.4; W03 adds the first manifest target entry and must not introduce a second
  toolchain declaration mechanism.

Classification: everything in
[the target-baseline contract](01-target-baseline-contract.md) and
[the workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md)
is **Required** for W03 closure. A custom target-spec JSON, a linker script, a
build script, `.cargo/` configuration, the guest build class, and FP/SIMD
codegen on the hypervisor target are **Reserved** with recorded triggers and
owners. CI wiring (W20), quality gates (W07), host test members (W08), the QEMU
runner (W09), dependency introduction (W18), version/metadata semantics (W16),
artifact naming beyond the baseline (W17), and all EL2/VM/memory/IRQ/device
mechanisms (P1+) are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Distinct target boundaries for host, hypervisor, guest, tooling | [Target-baseline contract](01-target-baseline-contract.md) §3 | P0-V05 (W03-DV01, DV05) |
| `no_std` + linking + controlled-ASM target build path with reserved layout extension points | [Target-baseline contract](01-target-baseline-contract.md) §4–§6, [workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md) §2–§3 | P0-V05 (W03-DV02, DV04) |
| Identifiable baseline artifact boundary and its relationship to host artifacts | [Workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md) §4 | P0-V05 (W03-DV02, DV03) |
| Target build validation that proves only the compilation chain | [Implementation workflow](03-implementation-and-review.md) steps 5, 7 and validation matrix | P0-V05 (W03-DV02, DV03, DV07) |
| Document discoverability and downstream consumability | [Implementation workflow](03-implementation-and-review.md) steps 6–7 | P0-V09 (W03-DV05, DV06, DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch `docs/p0-implementation-designs`
at merge 4e631ee): the repository is a documentation scaffold. There is no
`Cargo.toml` anywhere, no `rust-toolchain.toml` (the [W02
design](../p0-w02-rust-toolchain-baseline/README.md) is proposed, not
implemented), no target declaration, no `.cargo/` configuration, no Rust
source, no linker script, and no CI workflow. `crates/`, `hypervisor/src/`,
`scripts/`, and `tests/` hold only tracked `.gitkeep` markers from the W01
baseline. W01 is completed (record and verification evidence exist); no other
prerequisite or consumer package has delivered anything. Each ledger row below
states the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A bare-metal AArch64 artifact builds (P0-V05) | No Cargo workspace, member, or target path exists | Root workspace manifest plus exactly one `no_std` member that compiles for the chosen AArch64 target and links to an ELF artifact | P0-V05 is a build result; with nothing buildable it cannot be attempted | W03 (this design) | W03-DV02 build exercise |
| A target exists to build for | No target declaration anywhere; W02's manifest `targets` list starts empty | One pinned target entry added to `rust-toolchain.toml` per the W02 mechanism, plus a policy document owning the choice | A build needs a target identity; a pin without a policy home would drift | W02 owns the mechanism; W03 owns the entry and the policy | W03-DV01 declaration review |
| Host, hypervisor, guest, and tooling targets are distinct | No build-class concept exists in any tracked document | Normative four-class target classification with separation rules | "Distinct" is a classification contract, not an accident of commands | W03 | W03-DV01/DV05 policy review |
| `no_std` + linking + controlled ASM capability | No Rust or ASM build path exists | Member-level `#![no_std]`/`#![no_main]` semantics, a linking entry symbol, and one assembly translation unit in the build | The plan requires the joint Rust/ASM build capability to be established, and a successful build of `global_asm!` input is its proof | W03 | W03-DV02/DV04 |
| Panic/linking extension positions reserved | No extension points declared | Explicitly designated extension points: the member's panic handler symbol and the future linker-script/target-JSON positions, each with an owning design | The plan requires reserved expansion locations without fixing layout | W03 designates; P1 and W14/W12 own future content | W03-DV04 scope review |
| Validation proves compilation chain only, not EL2 | Nothing built | Validation matrix with explicit proof boundaries and not-run entries | The plan requires the boundary to be recorded, not merely implied | W03 | W03-DV02/DV07 |
| Downstream consumers can use the entry (W07, W09, W16, W19, W20, P1) | No entry exists | Documented build path, artifact location, and extension points | Consumers gate, run, identify, document, and extend this build | W03 delivers; consumers own their contracts | W03-DV07 consumability review |

Two prerequisite boundaries follow. First, W02 is a prerequisite and is only
proposed: the manifest edit in this design assumes the W02 toolchain contract
as proposed (a root `rust-toolchain.toml` with a `targets` list). If W02 has
not delivered when W03 implements, adding a target entry is blocked — the
blocker is recorded and the W02 flow is completed or reconciled first; W03 must
not inline a parallel toolchain mechanism. Second, W04 governance does not gate
this design: the baseline defines no feature and no custom profile, so no W04
decision is needed to proceed.

## Resolved design decisions and their authority

1. **Build-class taxonomy.** Four classes — host-native builds, the bare-metal
   hypervisor build, future guest builds, and development tooling — are defined
   in [the target-baseline contract](01-target-baseline-contract.md) §3. This
   is the plan's first work item (目标边界) and is stage-local policy.
2. **Hypervisor target triple.** `aarch64-unknown-none-softfloat`, the built-in
   Tier-2 bare-metal target whose precompiled `rust-std` ships through the
   pinned toolchain. Rationale: bare-metal AArch64 with no OS contract, no
   build-std requirement (so W02's `rust-src` stays Reserved), and — because
   the soft-float target disables FP/SIMD codegen — no unreviewed FP/SIMD
   premise in the compiled baseline. FP/SIMD context management is a later
   architecture concern (ADR §5 defers lazy FP/SIMD); enabling it is a Reserved,
   reviewed change, not a default. P1 may revisit the choice through the
   target-change thresholds in §8 of the contract.
3. **No custom target-spec JSON now.** The task book lists a target JSON among
   P0-level tools; the plan bounds W03 to reserving extension positions
   (保留后续布局扩展位置). The built-in target satisfies P0-V05, so the custom
   target-spec JSON, linker script, build script, and `.cargo/` configuration
   are Reserved extension points introduced only by a reviewed design change
   (first expected in P1).
4. **Workspace shape.** A root virtual `[workspace]` manifest with exactly one
   member, package name `hypervisor`, located at the existing tracked
   `hypervisor/` directory (replacing its `src/.gitkeep` marker in the same
   change). This matches the ADR's illustrative code-layout section without
   asserting final crate boundaries; the task-book exit rule (support later
   multi-crate work without asserting final boundaries) is preserved by adding
   members only through future designs.
5. **Edition.** The current stable edition supported by the pinned toolchain
   (edition 2024 as of this design; the exact value is confirmed at
   implementation against the pin and recorded). W02 deferred edition policy to
   the owning package; the workspace manifest that introduces it is W03's
   artifact, so the initial edition is stage-local build policy fixed here.
6. **Build-chain probe, not a hypervisor skeleton.** The member contains only
   the two symbols a freestanding `no_std` binary needs to link — an assembly
   `_start` stub that parks the core and a `#[panic_handler]` placeholder that
   parks the core — both explicitly labeled placeholders. It defines no EL2
   entry, no vector table, no sysreg access, no console, and no memory setup;
   P1 owns the real entry, W14 owns panic semantics, W12 owns crash
   information. This satisfies "Rust 与必要 ASM 联合构建能力" and "panic/链接
   扩展位置" without entering the plan's out-of-scope list.
7. **Probe naming and ADR-054.** The package name `hypervisor` follows the
   existing tracked directory and requires no crate-prefix decision; the
   pending project-name/crate-prefix question (ADR-054, register 待定) is not
   resolved by this design. If the owner freezes a formal name, renaming is
   routine policy maintenance recorded in the implementation record.
8. **Version field.** The member manifest needs a Cargo `version` for the
   baseline artifact; `0.1.0` is a placeholder value whose governance (identity
   semantics, sources, format) belongs to W16 and W17. W03 asserts no identity
   contract for it.
9. **Dependencies stay at zero.** The member has no dependencies and no
   build-dependencies. Adding the first dependency requires W18's evaluation
   path, which does not exist yet.
10. **Policy location.** `docs/development/build-target-baseline.md` is the
    sole normative home of target classification, the triple decision, build
    semantics, artifact boundary, and change thresholds; the workspace and
    member manifests carry data and pointers only. This mirrors the approved
    W02 pattern (policy in `docs/development/`, discovery via one
    `docs/README.md` routing row).

## Work breakdown and loading order

1. Read [the target-baseline contract](01-target-baseline-contract.md) for the
   build-class rules, the target policy document's required sections, the
   manifest edit, and the change thresholds.
2. Read [the workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md)
   for the workspace/member manifest schemas and the exact contracts of the two
   build-chain placeholder symbols.
3. Apply the changes in the order stated in the
   [implementation workflow](03-implementation-and-review.md): verify the W02
   prerequisite, create the workspace and probe member, add the manifest target
   entry, run the target-build exercise, write the policy document, wire
   discovery, then close.
4. Store actual commands, output, environment, and result in
   `../../verification/p0-w03-aarch64-build-target-baseline-verification.md`,
   and record changed artifacts, the confirmed edition, and any deviation in
   `../p0-w03-aarch64-build-target-baseline-record.md` only when implementation
   begins. Neither this design nor a written record may claim W03 complete.

## Design-level state and lifecycle

W03 adds no runtime state, registry, lock, allocation, or executable behavior
beyond a parked core. The authoritative state is four tracked artifact groups
plus their discovery links. Their documentary lifecycle:

```text
no build capability
  -> workspace manifest + probe member committed (marker replaced by source)
  -> rust-toolchain.toml carries the hypervisor target entry (W02 mechanism)
  -> target build exercised: ELF artifact produced under target/<triple>/
  -> build-target policy document committed and routed
  -> later packages mutate only through the policy's thresholds
     (W07 gates the build; W09 consumes the artifact path; W16 identifies it;
      W19 documents the path; W20 wires CI; P1 owns entry/layout extensions)
```

The policy document owns every target-classification statement; the manifests
own every machine-consumed value; the probe member owns nothing beyond its two
placeholder symbols. A conflict between policy and manifests is a review
failure, not a local choice. Any triple change, extension-point activation, or
FP/SIMD enablement follows the thresholds in the contract, not an ad-hoc edit.

## Explicitly excluded interfaces

No public Rust API, trait surface, module tree, ABI, wire format, persistent
data layout, or runtime interface is designed or authorized by W03 beyond the
two placeholder symbols contracted in
[the workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md)
§3, which are internal, non-ABI, and scheduled for replacement by P1, W12, and
W14 designs. No EL2 entry contract, exception-vector contract, linker-layout
contract, or memory-model statement is made; any such statement found in a W03
artifact is a scope violation to be raised at review.

## Downstream handoff

- **W07** receives a runnable target-build path and its policy document; W07
  owns the required-gate command spelling, failure semantics, and warning
  policy. The probe must build warning-free under defaults so the gate starts
  from a clean baseline.
- **W09** receives the fact that a target artifact exists at a documented
  location under `target/<triple>/`; the QEMU runner's interface, parameters,
  and evidence collection remain W09's contract.
- **W16** receives the initial identity dimensions that exist today (target
  triple, Cargo profile, placeholder package version); W16 owns metadata
  semantics and must not treat the placeholder version as an identity contract.
- **W19** receives the documented target-build entry as one step of the
  clone-to-build path, including the boundary that the workspace has no
  host-buildable member until W08 adds one.
- **W20** receives the same build path for required-check wiring; CI must
  provision the toolchain and target through the W02 manifest per the W02
  parity rule.
- **P1** receives the reserved extension positions (custom target-spec JSON,
  linker script, build script, `.cargo/` configuration, FP/SIMD enablement),
  each reachable only through the contract's change thresholds, and the
  guarantee that no EL2 entry, vector, or layout decision has been pre-taken.
- **W08** is not a listed consumer but adds the first host-class member; its
  host members must follow the classification in the policy document and must
  not alter the hypervisor member's target semantics.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
