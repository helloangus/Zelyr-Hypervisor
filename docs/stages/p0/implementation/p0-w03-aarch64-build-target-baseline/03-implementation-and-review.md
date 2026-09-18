# P0-W03 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm the workspace, member, and policy
document do not exist yet, and that `hypervisor/src/.gitkeep` is the only
tracked content under `hypervisor/`), and a search of tracked documents for
any existing target or build statement. The implementing machine may already
have a Rust toolchain; that is machine-local state and is never a repository
prerequisite or a substitute for the W02 restoration path.

Stop and obtain direction instead of guessing when any of the following occurs:

- the W02 toolchain manifest does not exist or its delivered schema differs
  from the [assumed contract](01-target-baseline-contract.md) §2 — record the
  blocker and reconcile with W02's delivered contract; do not inline a
  parallel toolchain or target-declaration mechanism;
- the target build fails in a way that appears to need a linker script, custom
  target JSON, `.cargo/` configuration, `build-std`, a nightly channel, or an
  unstable feature — each is a Reserved extension point or a W02 threshold;
  record the blocker, do not improvise;
- the probe appears to need more than the two placeholder symbols (for example
  a BSS, a stack, or console output to "make the build realistic") — that is
  P1/W12 scope; a build that needs them is itself the blocker to record;
- an existing tracked document contradicts the classification or target rules
  of this design — raise the conflict; do not edit the other document's
  authority silently; or
- a maintainer decision is requested to change the triple or enable FP/SIMD —
  follow §8 of [the target-baseline contract](01-target-baseline-contract.md),
  not a local shortcut.

## 2. Ordered implementation steps

### Step 1 — verify the prerequisite state

Target: none (read-only verification feeding the implementation record).

Work: confirm the W02 manifest exists with the assumed schema and an empty or
W02-delivered `targets` list; confirm no workspace or member manifest exists;
confirm W01's root conventions are in place. Record the observed commit.

**Acceptance:** the implementation record's prerequisite section names the
observed state and the W02 contract version relied on.  
**Failure/blocker:** missing or divergent W02 deliverables are recorded
blockers (see §1); W03 does not proceed by substituting its own mechanism.

### Step 2 — create the workspace and probe member

Target: root `Cargo.toml`, `hypervisor/Cargo.toml`, `hypervisor/src/main.rs`;
remove `hypervisor/src/.gitkeep` in the same change.

Work: write the manifests exactly per
[the workspace and probe-crate contract](02-workspace-and-probe-crate-contract.md)
§2 and the source per its §3, substituting the edition confirmed against the
pinned toolchain. Keep the placeholder comments; add nothing beyond the two
symbols and attributes.

**Acceptance:** manifests match the schemas field-for-field; the member has no
dependencies, features, build script, or profile influence; the marker removal
and source addition appear together in the change.  
**Failure/blocker:** a schema mismatch fails review; fix the file, not the
contract (the contract changes only through a new design decision).

### Step 3 — add the target entry to the toolchain manifest

Target: root `rust-toolchain.toml` (W02 artifact).

Work: add `"aarch64-unknown-none-softfloat"` to the `targets` list per
[the target-baseline contract](01-target-baseline-contract.md) §2 and the W02
contract §3.4. Change nothing else in the file.

**Acceptance:** the file remains valid TOML with the W02 pointer comment
intact and exactly one new list element.  
**Failure/blocker:** an absent manifest is a recorded blocker (§1).

### Step 4 — run the target build exercise

Target: verification record
(`docs/stages/p0/verification/p0-w03-aarch64-build-target-baseline-verification.md`,
created in this step).

Work: restore/provision the environment through the W02 declaration (an
isolated `RUSTUP_HOME`/`CARGO_HOME` sandbox or an equivalent restoration per
the W02 contract §3.5), then run the build invocation of
[the probe contract](02-workspace-and-probe-crate-contract.md) §4 and identify
the produced artifact. Record every command, its output, the environment, and
timestamps; record what was not run and why.

**Acceptance:** the build succeeds; the artifact exists under
`target/aarch64-unknown-none-softfloat/<profile>/`; identification confirms an
AArch64 ELF.  
**Failure/blocker:** a failed build is evidence of a failure — record it as
failed/blocked with diagnosis; do not widen the probe, add extension points, or
switch targets to make it pass (§1).

### Step 5 — create the build-target policy document

Target: `docs/development/build-target-baseline.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
the required content of
[the target-baseline contract](01-target-baseline-contract.md) §3–§8: the
four-class classification with separation rules, the hypervisor target and its
rationale and boundary, `no_std` semantics with the two extension positions,
the assembly rule, the artifact boundary, and the three change thresholds. The
document must reflect the validated reality of steps 2–4 (including the
documented host-invocation boundary) and must not name gate commands, CI
configuration, features, profiles, crates beyond the baseline member, or
runtime behavior.

**Acceptance:** every required section is present with its required content;
the document contradicts no ADR, task-book, W02, W04, or Coding-Guidelines
rule.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording this document.

### Step 6 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing target/build-class
and AArch64-build-path work at the new policy document, and add/update the W03
design row in the stage implementation index with a truthful status. Change
nothing else in either file.

**Acceptance:** a newcomer starting from `docs/README.md` can reach the target
policy in one link; the index row reflects the real status; all new relative
links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 7 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement, prerequisite compatibility with
W02's delivered contract, document links, and downstream handoff wording
(W07, W09, W16, W19, W20, P1). Completion is claimed only in the verification
record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W03-DV01 → P0-V05 | declaration and classification review | inspect `rust-toolchain.toml`, both Cargo manifests, and the policy document against the contracts | exactly one new target entry; manifests match schemas; four classes defined with separation rules; no second toolchain/target declaration mechanism in tracked files | a declared, classified build exists; not that it builds or runs |
| W03-DV02 → P0-V05 | target build exercise | the invocation and identification of the probe contract §4 from a W02-restored environment | build succeeds; AArch64 ELF exists at the documented boundary; assembly input assembled as part of the build | the compilation chain (toolchain, target, `no_std` semantics, linking, ASM); not EL2 execution, QEMU, guest, or hardware behavior |
| W03-DV03 → P0-V05 | repeated-lifecycle check | re-run the build with no source change; optionally clean-build once | second build is an incremental no-op success; a clean build reproduces the artifact | stable repeat behavior; not reproducibility policy (W16/W17) |
| W03-DV04 → P0-V05 | probe scope review | read the member source against the probe contract §3 | exactly two placeholder symbols; no sysreg/vector/console/memory work; no allocation; no dependencies; placeholders labeled with their future owners | scope discipline of the build chain; not P1 entry correctness |
| W03-DV05 → P0-V09 | policy coherence review | review the policy document against the contract §3–§8 and governing documents (ADR, task book, W02, W04) | all sections present; thresholds explicit; no contradiction; no out-of-scope statement (gate spelling, CI, features, profiles) | governance coherence; not future compliance by later changes |
| W03-DV06 → P0-V09 | discovery and link review | resolve the new `docs/README.md` row, policy links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's documentation taxonomy |
| W03-DV07 → W03 closure | consumability review | read the policy document as W07 (gateable build path?), W09 (artifact source?), W16 (identity dimensions?), W19 (workflow step?), W20 (CI input?), and a P1 planner (extension points located?) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the manifests,
source, and document without the DV02 exercise does not satisfy P0-V05. No
validation here proves EL2 bring-up, QEMU execution, guest boot, or hardware
behavior, and none may be reported as doing so; that boundary statement is
itself part of the plan's fourth work item and must appear in the verification
record.

## 4. Error, security, and observability model

W03 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is textual: a
schema deviation, a failed build, a missing contract section, a divergent
second declaration, or an out-of-scope probe extension fails the associated
review and is recorded as such. The probe cannot fail at runtime; it has no
inputs.

The security surface is build supply chain and scope discipline. Distribution
integrity of the toolchain and the target's precompiled `rust-std` is
delegated to rustup's signed official-channel manifests exactly as in the
[W02 contract](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md)
§3.9; mirrors, checksums, and vendoring remain Reserved there. The probe's
zero-input, zero-I/O shape means the baseline artifact introduces no attack
surface; keeping it that way is a review rule, not a runtime control.

Observability is the evidence trail: the verification record's commands,
outputs, environment, and run/not-run status are the only accepted proof
surface. No logging, tracing, or test framework may be introduced for W03.

## 5. Handoff checklist

Before handing W03 to a reviewer, provide:

- the exact changed-file list, including the marker-to-source replacement under
  `hypervisor/`;
- the confirmed edition and the target triple as committed, with the
  implementation record noting the W02 contract version relied on;
- DV01–DV07 evidence paths and their run status, including explicit not-run
  entries (EL2 execution, QEMU, guest, host-class builds);
- confirmation that no EL2 entry, vector table, linker script, custom target
  JSON, `.cargo/` configuration, build script, dependency, feature, profile,
  host test, or CI workflow was added;
- confirmation that the policy document names no gate command spelling and no
  runtime behavior; and
- open items for W07 (gate definition over this path), W09 (artifact source),
  W16/W17 (identity/naming over the placeholder version), W19 (workflow step),
  W20 (CI wiring), and P1 (extension points) — without resolving their
  contracts here.
