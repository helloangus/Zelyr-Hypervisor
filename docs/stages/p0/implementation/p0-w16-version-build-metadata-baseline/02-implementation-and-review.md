# P0-W16 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W16 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no version/metadata declaration
document exists yet) and a search of tracked documents for existing version or
build-identity statements. The implementing machine's local tools, toolchain,
and any local build outputs are machine-local state and are never repository
prerequisites or evidence.

Stop and obtain direction instead of guessing when any of the following occurs:

- a tracked document already declares a project version or identity field set
  (a sibling package landed first) — reconcile in the same change per the
  single-source rule; do not create a second authority;
- a prerequisite contract (W04 profile semantics, W03 target boundary, W12
  identity set) has landed with different shapes than this design assumes —
  the contract document is written against this design's schema, the
  divergence is recorded as a cross-package conflict surface, and the
  cross-review step records it rather than silently rewording either side;
- closure appears to require choosing a target triple, defining an ABI or
  machine version value/format, writing embedding code, or adding a CI check —
  those belong to W03, the future ABI/machine designs, W12/W17, and W20;
  reaching for them here is a scope violation; or
- a reviewer asks to freeze a wire format for the metadata — refuse per the
  plan's out-of-scope clause; that is an owning-design decision.

## 2. Ordered implementation steps

### Step 1 — record prerequisite status assumptions

Target: implementation record (`p0-w16-version-build-metadata-baseline-record.md`,
created in this step).

Work: observe which prerequisite contracts exist in the tree at implementation
time (W02 toolchain contract, W03 target boundary, W04 profile semantics, W12
diagnostic identity set, W17 naming grammar; several may still be parallel
designs). Record for each: exists as document / exists as proposed design /
absent, and the assumption this design's contract makes about it. This record
is the failure-boundary baseline for the cross-reviews.

Suggested observation: `git ls-files` and a link-resolution pass over
`docs/development/` and `docs/stages/p0/`.

**Acceptance:** the record names each prerequisite's observable status and the
assumption taken.  
**Failure/blocker:** none expected; a surprising in-tree conflict is raised per
§1, not absorbed.

### Step 2 — create the metadata contract document

Target: `docs/development/version-build-metadata.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and all
sections required by [the metadata contract](01-metadata-contract.md) §§2–7:
the seven identity questions with sources, the field schema with the required
subset, the timestamp policy, the dirty-tree policy, the project-version
declaration (`0.1.0`) with its single-source and migration rules, the ADR-040
compatibility reservations, the W12/W17 linkage rules, and the mutation rules.

**Acceptance:** every required section present; no target triple, dependency,
crate, wire format, ABI/machine version value, build command, or CI reference
appears; the document contradicts no ADR, task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1; fix the draft against the schema, never the schema against a draft
convenience.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing artifact-identity
and build-metadata work at the new contract document, and add the W16 design
row to the stage implementation index with status "Proposed design;
implementation not claimed" (updated truthfully as work proceeds). Change
nothing else in either file.

**Acceptance:** a newcomer starting from `docs/README.md` reaches the contract
in one link; the index row reflects the real status; all new relative links
resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — run the cross-reviews

Targets: verification record (evidence), the W12 and W17 contracts as
reviewees when they exist in-tree.

Work: perform W16-DV04: check the contract's minimum diagnostic identity set
against the W12 contract's association requirement (if the W12 contract is
in-tree; otherwise record `blocked` with the [W12 design](../p0-w12-logging-diagnostic-baseline/README.md)
as the named review surface), and check
that the naming linkage leaves the field-to-name mapping entirely to W17
(record `blocked` analogously with the [W17 design](../p0-w17-artifact-naming-baseline/README.md)
if its grammar is not in-tree). Record each
review as passed/failed/blocked with what was compared and the outcome.

**Acceptance:** both reviews passed, or blocked with the conflict surface and
the owning packages named.  
**Failure/blocker:** a semantic disagreement is recorded, not reworded away;
resolution belongs to the two owning packages in one reconciling change.

### Step 5 — closure review and evidence

Targets: verification record and implementation record.

Work: run the validation matrix below; confirm the handoff checklist; verify
the package against its task-book requirement, prerequisite compatibility,
document links, and downstream handoff wording. Record every validation as
passed, failed, blocked, or not run with command, output, timestamp, and
reason. Complete the implementation record with changed artifacts, the declared
version, and any deviation.

**Acceptance:** all Required validations passed or explicitly blocked with a
named surface; W16-DV06 is recorded `not run` unless a target artifact already
exists against which association was actually checked.  
**Failure/blocker:** a failed review is evidence of a failure — record it as
failed with diagnosis; do not weaken the schema or policies to make it pass.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W16-DV01 → P0-V14 | schema review | inspect the contract document against [the contract](01-metadata-contract.md) §§2–3, §5 | all seven questions with sources; complete field table; required subset stated; three compatibility positions reserved and undefined; mutation rules present | a declared, reviewable identity baseline exists; not that any artifact carries it yet |
| W16-DV02 → P0-V14 | policy review | review §4 against the plan's timestamp/dirty requirement and ADR-040 | determinism-first timestamp rule; clean/dirty definitions with clean-only evidence rule; single-source version declaration; no wall-clock identity | identity cannot silently drift or be falsified by policy; not actual build behavior (no build exists) |
| W16-DV03 → P0-V09/P0-V14 | question-source completeness | for each question Q1–Q7, trace its answer to exactly one declared source | no question without a source; no source without a question; no second authority in any tracked file | the baseline is answerable in principle; not that tools can extract it today |
| W16-DV04 → P0-V09/P0-V14 | cross-review with W12 and W17 | compare the minimum diagnostic identity set and the naming linkage rules against the in-tree W12/W17 contracts, else record blocked with surfaces | both reviews passed, or blocked with named conflict surfaces and owners | metadata is consistently referenceable across diagnostics and naming; not that W12/W17 are implemented |
| W16-DV05 → P0-V09 | discovery and link review | resolve the new routing row, contract links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's documentation taxonomy |
| W16-DV06 → P0-V14 (artifact half) | first-artifact association check | when a target artifact exists (producing package, expected W03), verify it can be associated with every required field | artifact-to-source and artifact-to-declared-metadata association demonstrated by the producing package's evidence | the full P0-V14 outcome; expected `not run` at W16 closure — must be recorded as such, never implied |
| W16-DV07 → W16 closure | consumability review | read the contract as W17 (can I map fields into names?), W19 (can the workflow cite identity?), W12 (is the minimum set usable?), W03 (is the required subset actionable?) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the document
without DV04 and DV07 does not satisfy W16 closure. No validation here proves
P0-V01–V08 or P0-V13 and none may be reported as doing so.

## 4. Error, security, and observability model

W16 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary: a
schema deviation, a missing policy section, a second version declaration, or a
failed cross-review fails the associated validation and is recorded as such.

The security position is that build identity is a trust anchor for all later
evidence: the design fails closed. A dirty build must be marked dirty (never
clean by default); a missing required field is a review failure (never an
omitted field); an unmarked wall-clock timestamp in identity is a violation.
The dirty and clean-only-evidence rules are the control against passing local
experimentation off as reviewable baseline — the exact failure mode P0-V14
exists to prevent.

Observability is the evidence trail: the verification record's review outputs,
statuses, and timestamps are the only accepted proof surface. The metadata
contract itself is later the observable handle by which diagnostics (W12),
names (W17), and CI evidence (W20) cite what produced an artifact.

## 5. Handoff checklist

Before handing W16 to a reviewer, provide:

- the exact changed-file list;
- the declared project version and its single-source location;
- DV01–DV07 evidence paths and their run status, including explicit
  `blocked`/`not run` entries (cross-reviews pending sibling contracts,
  first-artifact association);
- the recorded prerequisite status assumptions from Step 1;
- confirmation that no build script, manifest, target triple, ABI or machine
  version value/format, wire format, Rust source, `unsafe`, or CI change was
  added; and
- open items for W03 (required subset is a production requirement),
  W04 (capability_summary activation), W12 (minimum diagnostic identity set),
  W17 (field-to-name mapping authority), W19 (identity step in the workflow),
  and W20 (candidate automated checks) — without resolving their contracts
  here.
