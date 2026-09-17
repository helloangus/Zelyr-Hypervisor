# P0-W01 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies that it has loaded the
documents named in the parent README and reads the current tracked root tree.
Suggested read-only discovery commands are `git ls-files`, `find` for local
directories, and a Markdown-link checker if available.  The exact command is
not normative; the results below are.

If the checkout has no usable Git metadata, that is the expected trigger for
the repository-creation step below.  Do not infer clone behaviour from an
untracked local directory: initialize Git, form the initial commit, and review
the tracked-file view before claiming P0-V01.

Stop and obtain direction instead of guessing when any of the following occurs:

- no approved license text/copyright holder is available;
- an existing root document conflicts with the ADR, P0 task book, or this
  design;
- making a directory clone-safe would require a Cargo manifest, target,
  crate boundary, QEMU command, or runtime mechanism; or
- publishing a remote would require inventing a hosting service, remote URL,
  or credentials.

These are documentation/design blockers, not reasons to make a private local
workaround.

## 2. Ordered implementation steps

### Step 1 — create or verify the Git repository

Follow the version-control bootstrap contract in section 2.1 of the artifact
contract.  First determine whether the project root is already a Git work tree.
If it is not, initialize it with `main` as the initial branch.  Do not mistake
the presence of project files for an existing repository.

Complete the ignore policy before the first `git add`; then inspect the staged
name/status list before creating the first commit.  The commit contains only
approved W01 baseline artifacts.  A supplied remote URL may be configured and
pushed after owner authorization; absent that input, record remote publication
as pending and continue with the local cloneable baseline.

**Acceptance:** `HEAD` names `main`; the initial commit exists; its staged
scope was reviewed; and local-only helper files are ignored and absent from the
commit.  A remote is either verified/published with authorization or explicitly
recorded as pending, never guessed.

### Step 2 — establish the evidence inventory

Inspect the root files, the tracked-file list, intentionally empty directories,
existing links, and hard-coded paths/environment variables.  Produce a short
implementation note at
`docs/stages/p0/implementation/p0-w01-repository-baseline-record.md` when
actual work starts, recording:

- the commit or checkout identifier (when available);
- which directories need a tracked marker and why;
- every machine-specific assumption found;
- the license decision status; and
- any artifact deliberately left unchanged because its owner is a later Wxx.

**Acceptance:** every proposed modification maps to a W01 artifact contract;
no proposed modification creates a future runtime/build contract.

### Step 3 — make the root navigation authoritative

Edit `README.md` to meet section 2.2 of the artifact contract.  Preserve valid
existing content where it already satisfies the contract; do not rewrite for
style alone.  Resolve relative links from the README's directory, then verify
their target files exist in the tracked tree.

Review `AGENTS.md` and `docs/README.md` only for link consistency and
non-contradiction.  Their rules are already authoritative; duplicating their
full contents into README is prohibited.

**Acceptance:** a newcomer can start at README, reach AGENTS and docs/README,
and identify each top-level area without a machine-local instruction.

### Step 4 — preserve the intended clone layout

For each directory identified by Step 2, add the smallest allowed tracked
marker under section 2.3.  Do not populate source code or build configuration.
Use the same marker convention consistently within this W01 change.

**Acceptance:** after a fresh clone, every layout path promised by README is
either present because it contains a substantive tracked artifact or preserved
by its tracked marker.  No user has to run `mkdir` before beginning later
package work.

### Step 5 — review root configuration

Retain or make minimal corrections to `.editorconfig` and `.gitignore` to meet
section 2.4.  Verify that local developer helpers are ignored rather than
treated as project entry points.  Do not edit, execute, document, or test an
ignored local helper as part of W01.

**Acceptance:** text and generated-file policy is explicit; local developer
helpers are ignored; no tracked project entry point has a user-specific
absolute path or requires a sibling repository.

### Step 6 — install the owner-approved license

After, and only after, the owner decision is available, add root `LICENSE` and
the README link described in section 2.5.

**Acceptance:** the exact approved license is at the root, readable through a
valid README link, and no guessed legal content was added.

### Step 7 — conduct the closure review

Execute the test/review matrix below, save raw output and environment details
at `docs/stages/p0/verification/p0-w01-repository-baseline-verification.md`,
and update the implementation record with the changed artifact list and any
residual limitation.  Do not mark this design or W01 "implemented" merely
because the document itself has been written.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W01-DV01 → P0-V01 | repository bootstrap review | inspect repository root, `HEAD`, initial commit, staged-scope evidence, and ignored local-helper result | Git repository exists on `main`; initial W01 commit exists; local-only helper is ignored/untracked | versioned baseline; not remote publication or later CI policy |
| W01-DV02 → P0-V01 | fresh-clone review | clone the initial commit into a disposable directory and compare its tracked tree with README-promised paths | clone succeeds and each promised path has a substantive tracked file or marker | clone layout; not toolchain/build success |
| W01-DV03 → P0-V01 | machine-local prerequisite scan | search tracked project entry points/docs for home paths, user names, required private files, and unconditional sibling paths | none remains as a project requirement | clone safety; not external tool availability |
| W01-DV04 → P0-V09 | root-navigation link review | resolve every root README link and required docs entry link | target exists and wording does not contradict P0 scope | documentation navigation; not documentation completeness of later Wxx |
| W01-DV05 → P0-V09 | configuration-policy review | inspect `.editorconfig`, `.gitignore`, and new markers; use Git ignore inspection for local helpers | text rules are explicit; ignored paths are generated/local only; markers are tracked; helper is ignored | root convention consistency; not actual build-output validation |
| W01-DV06 → P0-V01/P0-V09 | licensing review | inspect root `LICENSE` and README link against owner decision | exact approved text/location/link; decision record present | licensing discoverability; not dependency-license compliance (W18) |
| W01-DV07 → P0-V01/P0-V09 | commit and scope review | inspect the initial commit and subsequent W01 diff against the W01 plan | changes are only repository bootstrap, docs, root conventions, markers, and license | scope discipline; not later P0 completion |

At a minimum, record each validation as **passed**, **failed**, **blocked**, or
**not run**, with command/input, environment, timestamp, and a short reason.
A successful Markdown or shell syntax check is not evidence of P0-V02 through
P0-V08 and must not be reported as such.

## 4. Error, security, and observability model

W01 has no hypervisor error model, synchronization, guest input, hardware
access, telemetry API, or `unsafe` boundary.  Its failure reporting is textual:
a missing Git initialization/commit, link, marker, license decision, or a
machine-local project requirement fails the associated review.  Remote
publication without an approved URL/authorization remains pending rather than
becoming a failed local repository bootstrap.

Implementation evidence is the observability surface: record the changed-file
list, review commands and outputs, platform/environment assumptions, and
run/not-run limitations.  Do not introduce logging dependencies, tracing
events, metrics, or a test framework for W01.

## 5. Handoff checklist

Before handing W01 to a reviewer, provide:

- the exact changed-file list;
- the license owner decision or its explicit blocking status;
- the final marker list and why each marker exists;
- P0-V01/P0-V09 evidence paths and their run status;
- confirmation that no Rust code, `unsafe`, ABI/public API, Cargo dependency,
  crate boundary, or build/target/QEMU behaviour was added; and
- open conflicts for W02, W05, W19, or W20, without trying to resolve their
  contracts inside W01.
