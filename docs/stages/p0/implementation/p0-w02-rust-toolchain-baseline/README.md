# P0-W02 Rust Toolchain Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The pinned, recoverable, developer-and-CI-shared Rust toolchain
baseline required by [P0-W02](../../plans/p0-w02-rust-toolchain-baseline.md).  
**Owner/change context:** P0-W02 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W02. It converts the bounded
work-package plan into small, reviewable configuration and documentation
changes: one machine-consumed toolchain pin manifest, one normative toolchain
contract document, and minimal discovery wiring. It deliberately does **not**
create a Cargo workspace or manifest, choose an AArch64 target triple, add any
crate dependency, write hypervisor code, or configure CI; those belong to W03,
W18, and W20 respectively.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, ADR baseline, P0 task book, and P0-W02 plan.
This document is the proposed detailed design for those changes; it is not a
completion record and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W02 plan → this design
→ Coding Guidelines. In particular:

- The ADR requires Rust-first development with a `no_std` hypervisor target and
  permits necessary assembly and reviewed `unsafe`; it does not mandate a
  toolchain channel, so the channel choice is stage-local toolchain policy that
  this design owns within the plan's scope.
- The task book outcome for W02 is: Rust toolchain is **pinned, recoverable,
  and shared by developers and CI** (P0-V02). Sharing with CI at this stage
  means the repository declares exactly one toolchain source and mandates its
  use; the CI wiring that consumes it is W20.
- The plan forbids turning incidental nightly use into an unreviewed premise.
  This design therefore pins the stable channel and defines an explicit,
  reviewed escalation path to nightly instead of adopting it speculatively.
- The concrete AArch64 bare-metal target definition is W03 scope ("目标定义").
  W02 defines the declaration policy and manifest schema for targets; it must
  not select a triple, even a plausible built-in one.
- Crate dependencies, runtime Rust APIs, edition and profile decisions, and
  build commands remain with their owning packages (W04, W08, W18, and later).

Classification: everything in [the toolchain contract](01-toolchain-contract.md)
is **Required** for W02 closure. `rust-src` and `llvm-tools-preview`
components, offline restoration, and any supply-chain hardening beyond rustup's
distribution integrity are **Reserved** with recorded triggers. Cargo
manifests, target triples, CI workflows, QEMU, guests, EL2 mechanisms, and all
runtime code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Pinned toolchain (channel, pinning mechanism, components) | [Toolchain contract](01-toolchain-contract.md) §2 | P0-V02 (W02-DV01, DV02) |
| Recovery from a clean environment | [Toolchain contract](01-toolchain-contract.md) §3.5, [workflow](02-implementation-and-review.md) step 5 | P0-V02 (W02-DV02, DV03) |
| Developer/CI single source | [Toolchain contract](01-toolchain-contract.md) §3.7 | P0-V02 (W02-DV04); CI execution evidence arrives with W20 |
| Update, review, and ADR-threshold rules; unstable rules; low-level components | [Toolchain contract](01-toolchain-contract.md) §3.3, §3.6, §3.8 | P0-V02/P0-V09 (W02-DV05) |
| Document discoverability and coherence | [workflow](02-implementation-and-review.md) step 4 | P0-V09 (W02-DV06) |
| Downstream consumability by W03/W07/W19/W20 | [workflow](02-implementation-and-review.md) handoff checklist | W02 closure review (W02-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, `main` at 2f13ebd): no
`rust-toolchain.toml` or `rust-toolchain` file, no `Cargo.toml` or workspace,
`.github/workflows/` contains only a `.gitkeep` marker, and no tracked document
declares a toolchain restoration procedure. `.gitignore` already covers
generated output directories and does not exclude `*.toml`. Each ledger row
below states the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Toolchain is pinned | No pin declaration in any tracked file | Root `rust-toolchain.toml` with exact three-part version, profile, components | Without a machine-consumed pin file there is nothing to restore identically | W02 (this design) | W02-DV01 schema review |
| Toolchain is recoverable | No restore procedure in tracked docs | Contract document with declared inputs, steps, verification commands, and failure boundaries | Recovery must be executable from repository declarations alone | W02 | W02-DV02/DV03 restoration exercise |
| Necessary components are declared | No component declaration exists | `rustfmt` and `clippy` pinned in the manifest; reserved components defined with triggers | W07's formatting/lint gates (P0-V06/V07) require them; speculative components would be unreviewed premises | W02 policy; entries per approved designs | W02-DV01/DV05 |
| Necessary targets are declared | No target definition exists anywhere; W03 owns target definition | Manifest schema and policy for target entries; initial list empty | A target entry without an approved design would pre-empt W03; the mechanism must exist first | W02 schema; W03 first entries | W02-DV01; W03 consumes |
| Update and unstable rules exist | Absent | Contract sections with routine-maintenance vs ADR thresholds and the unstable-feature rule | The plan work sequence requires these rules, not merely a pin | W02 | W02-DV05 policy review |
| Developers and CI reference the same baseline | No CI exists (`.gitkeep` only) | Single-source mandate in the contract; CI consumption is wired by W20 | Parity is a property of having exactly one authoritative declaration | W02 contract; W20 wiring | W02-DV04; full CI proof deferred to W20 (P0-V08) |

No row above requires inventing a remote, license, crate, or runtime policy, so
no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Pinning mechanism:** the rustup-standard root `rust-toolchain.toml`,
   schema fixed in the [toolchain contract](01-toolchain-contract.md) §2. This
   is a toolchain-policy choice inside the plan's declared scope (版本固定方式).
2. **Channel:** stable, pinned by exact three-part version selected at
   implementation time from the official release channel. Rationale: no
   approved downstream design requires nightly (W03's custom-target/build-std
   decision does not exist yet), stable maximizes update reviewability, and the
   plan forbids unreviewed nightly premises. Nightly adoption follows the
   escalation rule in §3.6 of the contract and is ADR-required when it would
   make unstable features load-bearing for the hypervisor TCB.
3. **Components:** `profile = "minimal"` plus `rustfmt` and `clippy` only;
   `rust-src` and `llvm-tools-preview` are Reserved with explicit triggers.
4. **Targets:** the manifest declares a `targets` list; it starts empty and is
   populated only by an approved work-package design, first W03.
5. **Provisioning manager:** rustup is the declared restoration mechanism;
   toolchains provisioned outside the pin file are unsupported for
   gate-bearing work.
6. **Policy location:** `docs/development/toolchain-baseline.md` is the sole
   normative home of toolchain policy; the manifest carries data plus a
   pointer comment only.

## Work breakdown and loading order

1. Read the [toolchain contract](01-toolchain-contract.md) to understand the
   manifest schema, the required contract-document sections, and the mutation
   rules for each artifact group.
2. Apply the changes in the order stated in the
   [implementation workflow](02-implementation-and-review.md): select and
   record the pinned version, create the manifest, create the contract
   document, wire discovery, then run the restoration exercise.
3. Store actual commands, output, environment, and result in
   `../../verification/p0-w02-rust-toolchain-baseline-verification.md`, and
   record the selected version, changed artifacts, and any local decision in
   `../p0-w02-rust-toolchain-baseline-record.md` only when implementation
   begins. Neither this design nor a written record may claim W02 complete.

## Design-level state and lifecycle

W02 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is two tracked artifacts plus their discovery links. Their
documentary lifecycle:

```text
no toolchain declaration
  -> rust-toolchain.toml committed with exact pin
  -> toolchain contract document committed
  -> discovery links (docs/README routing row, stage index) committed
  -> clean-environment restoration exercised and evidenced
  -> later packages mutate only through the contract's rules
     (W03 adds target entries; W07 pins gate commands on this toolchain;
      W19 documents the path; W20 wires CI consumption)
```

The contract document is the owner of every policy statement; the manifest is
the owner of every machine-consumed value. A conflict between them is a review
failure, not a local choice. Any future channel switch, pinning-mechanism
change, or unstable-feature adoption follows the escalation thresholds in the
contract, not an ad-hoc edit.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format, or
persistent data layout is designed or authorized by W02; see
[the toolchain contract](01-toolchain-contract.md) §4. The only machine-facing
surface is the manifest schema, and the only human-facing procedure is the
restoration workflow, both fixed in the contract file.

## Downstream handoff

- **W03** receives the target-entry mechanism: it adds its approved AArch64
  bare-metal target entry to the manifest and records the triple decision in
  its own design. W02 must not have selected the triple.
- **W07** receives the guarantee that `cargo fmt`/`cargo clippy` run under the
  pinned toolchain with `rustfmt`/`clippy` components present; W07 owns the
  exact gate command spelling and failure semantics.
- **W19** receives the restoration inputs and verification commands as the
  toolchain step of the clone-to-build path.
- **W20** receives the single-source mandate: CI must provision through the
  repository manifest and must not pin a divergent toolchain. W20 owns the
  GitHub implementation and the evidence that enforcement is real.
- **W05** may later re-home or re-version the contract document under its
  documentation taxonomy; semantic ownership of toolchain policy stays with the
  contract document regardless of location.

W08 and W09 also depend on this package per the plan index (host tests and the
QEMU runner must run under the same pinned toolchain); they consume the same
contract through W07/W19 surfaces.
