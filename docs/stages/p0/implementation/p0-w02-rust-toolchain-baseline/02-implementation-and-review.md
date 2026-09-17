# P0-W02 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no toolchain pin, manifest, or CI
file exists yet) and a search of tracked documents for any existing toolchain
statement. The implementing machine may or may not already have rustup
installed; that is machine-local state and is never a repository prerequisite
or a substitute for the declared restoration path.

Stop and obtain direction instead of guessing when any of the following
occurs:

- no released stable version can be resolved (blocked network or distribution
  outage) — record the blocker; do not pin a partial version, alias, or
  locally built toolchain;
- an existing tracked document contradicts the channel, single-source, or
  unstable rules of this design — raise the conflict; do not edit the other
  document's authority silently;
- making restoration succeed appears to require a Cargo manifest, target
  triple, crate, or CI change — that belongs to W03/W07/W20, and reaching for
  it here is a scope violation; or
- a maintainer decision is requested to adopt nightly or any Reserved
  component — follow §3.6 of the toolchain contract, not a local shortcut.

## 2. Ordered implementation steps

### Step 1 — select and justify the pinned version

Target: implementation record (`p0-w02-rust-toolchain-baseline-record.md`,
created in this step).

Work: resolve the current released stable version of Rust from the official
release channel and select it as `<X.Y.Z>`. Selection rule: the latest
released stable three-part version at implementation time; if an approved
downstream design had declared a higher minimum, the higher would apply (none
exists today). Record the version, its release date, the resolution source,
and the date of selection.

Suggested observation: the official release channel listing or `rustup`
itself; the exact lookup command is not normative.

**Acceptance:** the record names one exact three-part version with its release
date and resolution source.  
**Failure/blocker:** an unresolvable channel is a recorded blocker (see §1);
no substitution.

### Step 2 — create the pin manifest

Target: root `rust-toolchain.toml`.

Work: write the file exactly per the schema in
[the toolchain contract](01-toolchain-contract.md) §2, substituting the
selected version. Keep the pointer comment; add no policy prose.

**Acceptance:** the file is valid TOML, matches the schema field-for-field,
and contains no target triple and no component beyond `rustfmt` and `clippy`.  
**Failure/blocker:** a schema mismatch fails review; fix the file, not the
schema (the schema changes only through a new design decision).

### Step 3 — create the toolchain contract document

Target: `docs/development/toolchain-baseline.md`.

Work: write the document with the status header and all sections required by
[the toolchain contract](01-toolchain-contract.md) §3. Content must reflect
the resolved decisions (stable channel, minimal profile, two baseline
components, empty initial target list, rustup as provisioning manager, the
three update thresholds, the unstable rule, the CI parity rule) and must not
name an AArch64 triple, crate, or gate command spelling owned by other
packages.

**Acceptance:** every required section is present with its required content;
the document contradicts no ADR, task-book, or Coding-Guidelines rule (the
Coding Guidelines' prohibition on toolchain changes without approval is
satisfied: this document *is* the approval record for the initial pin and the
process for future ones).  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording this document.

### Step 4 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing toolchain
setup/restoration/update work at the new contract document, and add the W02
design row to the stage implementation index with status "Proposed design;
implementation not claimed" (updated truthfully as work proceeds). Change
nothing else in either file.

**Acceptance:** a newcomer starting from `docs/README.md` can reach the
toolchain contract in one link; the index row reflects the real status; all
new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 5 — exercise restoration and record evidence

Targets: verification record
(`docs/stages/p0/verification/p0-w02-rust-toolchain-baseline-verification.md`)
and the implementation record.

Work: perform the restoration exercise of W02-DV02/DV03 (below) in the cleanest
available form — an isolated `RUSTUP_HOME`/`CARGO_HOME` sandbox provisioned
from the repository declaration, or an equivalent removal-and-reprovision of
the pinned toolchain. Record every command, its output, the environment
(operating system, rustup version, network state), and timestamps. Record what
was not run and why. Then complete the implementation record with the changed
artifact list, the selected version, and any deviation from this design.

**Acceptance:** the verification record shows the pinned toolchain, both
components, and the (empty) target list restored from the declaration, plus
explicit not-run entries for deferred proofs.  
**Failure/blocker:** a restoration failure is evidence of a failure — record
it as failed/blocked with diagnosis; do not widen the manifest schema or
switch toolchains to make it pass.

### Step 6 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement, prerequisite compatibility with
W01's delivered baseline, document links, and downstream handoff wording.
Completion is claimed only in the verification record, with evidence, and only
for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W02-DV01 → P0-V02 | manifest schema review | inspect `rust-toolchain.toml` against contract §2 | exact three-part pin; `profile = "minimal"`; exactly `rustfmt`+`clippy`; empty `targets`; pointer comment intact; valid TOML | a declared pin exists; not that it restores or builds |
| W02-DV02 → P0-V02 | clean-restore exercise | isolate `RUSTUP_HOME`/`CARGO_HOME` (or uninstall the pinned toolchain), invoke `cargo` inside the tree, compare `rustc --version`, component list, and installed targets with the manifest | declared toolchain and components restore identically from repository declaration plus rustup | restorability on a networked rustup host; not offline restore, host builds, or target builds |
| W02-DV03 → P0-V02 | repeated-lifecycle check | re-run provisioning; second invocation must not change versions/components (idempotent no-op) | no drift between repeated restorations | stable repeat behavior; not concurrent multi-host behavior |
| W02-DV04 → P0-V02 | single-source review | search tracked files for any other toolchain version/channel/component/target declaration | exactly one authoritative pin location exists | the same-source contract; not GitHub CI execution (W20, P0-V08) |
| W02-DV05 → P0-V02/P0-V09 | policy review | review the contract document against §3 required sections and governing documents | all required sections present; thresholds explicit; no contradiction with ADR/task book/Coding Guidelines | governance coherence; not future compliance by later changes |
| W02-DV06 → P0-V09 | discovery and link review | resolve the new `docs/README.md` row, contract links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's documentation taxonomy |
| W02-DV07 → W02 closure | consumability review | read the contract as W03 (can I add a target?), W07 (are fmt/clippy guaranteed present?), W19 (is the restore path usable?), W20 (is the parity rule binding?) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the manifest
and document without the DV02 exercise does not satisfy P0-V02. No validation
here proves P0-V03 through P0-V08 and none may be reported as doing so.

## 4. Error, security, and observability model

W02 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is textual: a
schema deviation, a missing contract section, a divergent second pin, or a
failed restoration fails the associated review and is recorded as such.

The security surface is toolchain supply chain. The design's position, stated
in the contract §3.9: the pin fixes identity; distribution integrity is
delegated to rustup's signed official-channel manifests; mirrors, checksums,
and vendoring are Reserved and would be a new decision. The unstable rule is
also a security rule: the stable pin makes unstable capabilities fail closed
rather than fail open.

Observability is the evidence trail: the verification record's commands,
outputs, environment, and run/not-run status are the only accepted proof
surface.

## 5. Handoff checklist

Before handing W02 to a reviewer, provide:

- the exact changed-file list;
- the selected `<X.Y.Z>` version, release date, and resolution source;
- DV01–DV07 evidence paths and their run status, including explicit not-run
  entries (CI consumption, offline restore, target builds);
- confirmation that no Cargo manifest/workspace, target triple, crate
  dependency, Rust source, `unsafe`, CI workflow, or QEMU artifact was added;
- confirmation that the contract document names no AArch64 triple and no gate
  command spelling; and
- open items for W03 (target entry mechanism ready, triple undefined),
  W05 (possible re-homing of the contract document), W19 (workflow linkage),
  and W20 (parity-rule wiring) — without resolving their contracts here.
