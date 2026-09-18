# P0-W07 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm which prerequisite surfaces
exist), a search for any existing gate/lint/warning statement, and an
inspection of the delivered workspace structure if present.

Prerequisite-surface expectations at implementation time, per the task book's
recommended phase order (phase A completes before phase B):

- **W02** (declared prerequisite): the toolchain pin and contract exist, with
  `rustfmt`/`clippy` guaranteed. If absent, record a blocker; do not define
  gate invocations against an unpinned toolchain.
- **W03** (phase-A package, expected complete): a delivered workspace and a
  target-build entry exist so QG-FMT/QG-LINT/QG-WARN/QG-BUILD-TARGET can bind
  and dry-run. If the workspace or entry is absent, the register rows stay
  valid as policy with the binding marked "awaiting W03 entry"; execution
  evidence for those gates is recorded as blocked. If W03's plan scope turns
  out not to deliver a stable target-build entry at all, that is recorded as
  an Architecture Change Request against the plan-index dependency map — W07
  must not absorb the target definition itself.
- **W08** (declared prerequisite): its host-test entry contract exists for the
  QG-TEST-HOST binding. If W08 delivered differently or not at all, mark the
  binding blocked and record the conflict; never invent a test entry here.
- **W05** (declared prerequisite): the documentation baseline defines the
  status-header and location conventions QG-DOCS checks for presence.

Stop and obtain direction instead of guessing when: a tracked document already
claims gate policy (single-source conflict); making a gate executable appears
to require creating a script, workflow, crate, or target (W20/W03 scope);
or a maintainer asks to waive a required gate (that is a design-level change
per the register, not a local edit).

## 2. Ordered implementation steps

### Step 1 — record prerequisite-surface findings

Target: implementation record (`../p0-w07-development-quality-gates-record.md`,
created in this step).

Work: inspect the tree and record, per prerequisite above, whether its surface
exists and what the bound entries' recorded spellings are (from the W03/W08
contract documents when present). This drives which dry-runs step 3 can
actually run.

**Acceptance:** the record states the availability of W02/W03/W05/W08 surfaces
and cites where each bound entry is defined.  
**Failure/blocker:** a missing declared prerequisite (W02/W05/W08) is a
recorded blocker; a missing phase-A surface (W03) defers execution evidence as
stated in §1.

### Step 2 — author the quality-gates contract document

Target: `docs/development/quality-gates.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
the normative content fixed by the [gate register](01-gate-register.md) §2–§5
and the [gate standards](02-gate-minimum-standards.md) §2–§10: the six-row
register with all seven fields, classification/promotion/mutation rules,
machine-consumability fields, per-gate minimum standards with recorded
invocation spellings per the selection rules, the development and integration
sets, and the failure-handling principles.

**Acceptance:** every register row is complete; no gate names a script, CI
file, crate, or target not owned by this design; the document contradicts no
ADR, task-book, Coding-Guidelines, or integration-workflow rule.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording.

### Step 3 — dry-run executable gates where the tree allows

Target: verification record
(`../../verification/p0-w07-development-quality-gates-verification.md`).

Work: for each gate whose prerequisite surface exists, run its recorded
invocation once on the current tree and record command, output, environment,
and timestamps. A gate whose surface is missing is recorded **not run** with
the reason and the blocking row from step 1 — that is evidence of status, not
evidence of passing. Where a gate cannot fail yet because its subject is
empty (for example formatting over a tree with no Rust sources), record that
the run proves the invocation mechanics only, not the gate's end-state
semantics.

**Acceptance:** every Required gate has an explicit run/blocked/not-run entry
with a reason; no entry claims more than it proved.  
**Failure/blocker:** a dry-run failure is recorded as failed with diagnosis;
do not widen the standard or move ownership to make it pass.

### Step 4 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing quality-gate and
check-classification work at the contract document, and add the W07 design row
to the stage implementation index with a truthful status. Change nothing else
in either file.

**Acceptance:** a newcomer starting from `docs/README.md` reaches the gates
contract in one link; all new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 5 — consumability and evidence-attribution review

Work: read the contract from the W20 perspective (can each Required row be
mapped to one check with required status and a stable name?), the W19
perspective (are the two minimum sets stated so a contributor knows what runs
where?), and the W08/W03 perspective (do the bindings reference their entries
without redefining them?). Verify every gate's evidence label appears in the
attribution rules and that the register's evidence labels are unique.

**Acceptance:** each perspective can act without inventing policy; every
attribution gap found is fixed in the contract, not in prose elsewhere.  
**Failure/blocker:** a gap that requires defining a bound entry's semantics is
a conflict against W03/W08 ownership — record it, do not resolve it here.

### Step 6 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
verify the package against its task-book requirement (P0-W07), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what was
actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W07-DV01 → P0-V06–V08 | Register review | inspect the contract against [register](01-gate-register.md) §2–§3 | six complete rows; classes and blocking flags explicit; promotion and mutation thresholds unambiguous | classification exists and is decisive; not that any check executes |
| W07-DV02 → P0-V06/P0-V07 | Standards review | inspect per-gate standards against [standards](02-gate-minimum-standards.md) §2–§7 | each gate has purpose, standard, invocation semantics, passing condition, failure semantics; bindings reference W08/W03 without redefinition | rules are defined and unambiguous; not that CI enforces them |
| W07-DV03 → P0-V06–V08 | Sets and failure-principles review | inspect [standards](02-gate-minimum-standards.md) §8–§9 against the integration workflow | both sets defined with membership; principles align with and never weaken the workflow rule | handling principles exist; not that merges have been blocked in practice |
| W07-DV04 → P0-V08 | Machine-consumability review | map each Required row to a hypothetical W20 check 1:1 | stable ID, class, blocking flag, binding, and evidence label per row; labels unique | W20 can wire without policy invention; not that GitHub enforcement exists (W20, P0-V08) |
| W07-DV05 → P0-V06/P0-V07 | Local dry-run evidence | step 3 runs per available surface | every gate has run/blocked/not-run status with reason and output where run | invocation mechanics work on the current tree; not full CI execution or `main` protection |
| W07-DV06 → P0-V09 | Discovery and link review | resolve the routing row, contract links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's taxonomy itself |
| W07-DV07 → W07 closure | Consumability review | step 5 perspectives | each consumer perspective can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. P0-V06/V07 stage
evidence ("required formatting check reports no violation", "required lint
policy passes") completes only when the required checks actually execute —
locally where dry-runs apply, and in CI through W20 (P0-V08). No W07
validation proves P0-V08, and none may be reported as doing so; none proves
EL2 or guest behavior.

## 4. Error, security, and observability model

W07 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary: a
register defect, a contradictory check, or a failed dry-run fails the
associated review and is recorded as such.

The security position is that the gate set is the automated floor of the
project's quality claims: the design's contribution is that gate semantics
have exactly one owner, weakening a gate is a traceable decision rather than a
local edit, and future checks cannot silently appear as verified. Evidence
attribution (unique labels binding results to register rows and stage
validation IDs) is the observability model; the verification record's
run/not-run entries are the only accepted proof surface.

## 5. Handoff checklist

Before handing W07 to a reviewer, provide:

- the exact changed-file list;
- the prerequisite-surface findings from step 1 (which of W02/W03/W05/W08
  surfaces existed and where each bound entry is defined);
- DV01–DV07 evidence paths and run status, including explicit blocked/not-run
  entries for gates without a prerequisite surface;
- recorded invocation spellings for the W07-fixed gates and the recorded
  bindings for the W08/W03-bound gates;
- confirmation that no script, CI workflow file, crate, target triple, Rust
  source, or `unsafe` artifact was added, and that no bound entry's semantics
  were redefined; and
- open items for W20 (register-to-check mapping and `main` protection),
  W08/W03 (binding finalization when their entries land), W09/W10 (promotion
  paths), W19 (contributor-facing set summary), and W05 (possible re-homing)
  — without resolving their contracts here.
