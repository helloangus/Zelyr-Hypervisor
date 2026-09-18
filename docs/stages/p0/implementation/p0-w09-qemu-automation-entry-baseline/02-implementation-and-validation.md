# P0-W09 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no runner, script, or QEMU
document exists) and a search of tracked documents for any existing QEMU
automation statement.

Prerequisite-surface expectations at implementation time:

- **W02** (declared prerequisite): the toolchain pin and contract exist. The
  runner contract records the pinned-toolchain constraint for future
  implementers but executes nothing, so a missing W02 surface is a recorded
  inconsistency to raise, not a blocker to authorship.
- **W03** (phase-A package, expected complete; not a listed contract
  prerequisite): the AArch64 build surface exists. The contract references it
  only as the future boot subject; if absent, the reference is marked
  "awaiting W03 entry" and the contract is still authorable. If W03's plan
  scope turns out not to deliver a bootable artifact surface, that is an
  Architecture Change Request against the plan-index dependency map — the
  P1 boot-regression consumer could not exist without one.
- **W05** (declared prerequisite): documentation location and status-header
  conventions exist for the contract document.
- **W17** (not a prerequisite): the artifact-naming baseline may not exist.
  Assumption and boundary: the evidence section's placeholder naming rule
  applies until W17 delivers; if W17 delivers incompatible naming, the entry
  contract gets a version bump — never a silent rewrite of past-run records'
  meaning.

Stop and obtain direction instead of guessing when: a tracked document
already describes QEMU automation (single-source conflict); defining the
grammar appears to require choosing the runner's language, location, or a QEMU
version pin (implementing-design or W19 decisions); or any step appears to
require executing QEMU in P0 (plan out-of-scope — stop, do not run).

## 2. Ordered implementation steps

### Step 1 — record prerequisite-surface findings

Target: implementation record
(`../p0-w09-qemu-automation-entry-baseline-record.md`, created in this step).

Work: inspect the tree; record the availability of the W02 pin, W03 build
surface, W05 conventions, and W17 naming baseline, with pointers. Note any
existing QEMU mention that the single-entry rule would constrain.

**Acceptance:** the record states each surface's availability and lists every
tracked QEMU mention found.  
**Failure/blocker:** conflicting existing automation statements are raised
per §1, not absorbed.

### Step 2 — author the runner entry contract

Target: `docs/testing/qemu-runner-entry.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1` — the entry contract's own version —
owner/change context, supersedes: none) and the normative content fixed by
the [runner entry contract](01-runner-entry-contract.md) §2–§9: single-entry
rule, responsibility boundary, grammar sketch, parameter classes, runtime
behavior requirements, exit taxonomy, evidence content set, and the
placeholder marking.

**Acceptance:** every section present; the taxonomy is exactly the six
classes; no QEMU flag recipe, runner language, image, or timeout default is
fixed; the placeholder marking is present.  
**Failure/blocker:** a contradiction with the ADR, task book, plan, or a
consumer's plan (W19/W20/P1-W10) is raised per §1, not reworded away.

### Step 3 — grammar walkthrough (documentary validation)

Target: verification record
(`../../verification/p0-w09-qemu-automation-entry-baseline-verification.md`).

Work: on paper, trace one hypothetical future invocation — a P1-style boot
smoke with a repeat-count regression parameter — through the contract:
grammar acceptance, profile resolution, parameter carrying, launch
requirements, serial capture, a success path, a timeout path, a launch-failure
path, and the evidence set each produces. For each path, confirm the contract
already fixes the outcome, status class, and evidence content without needing
an undocumented decision. Record the trace and every gap found; fix gaps in
the contract, then re-trace.

**Acceptance:** every traced path resolves to a fixed status class and a
complete evidence content set with no undocumented decision.  
**Failure/blocker:** a path that requires inventing policy (for example a
default timeout) is fixed by adding the *mechanism*, never a concrete value
owned by a future design.

### Step 4 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md` (and,
optionally, a pointer line in `docs/testing/README.md` that does not restate
policy).

Work: add one routing-table row to `docs/README.md` pointing QEMU automation
and runner work at the contract, and add the W09 design row to the stage
implementation index with a truthful status. Change nothing else.

**Acceptance:** one-link reachability from `docs/README.md`; all new relative
links resolve from a fresh checkout; the row marks the entry as a P0
placeholder.  
**Failure/blocker:** broken or duplicating links fail review.

### Step 5 — single-source and consumability review

Work: re-scan tracked files for competing automation entries (scripts,
workflow files, documents embedding QEMU command lines for automated use) —
there must be none, and the contract must be their designated home. Then read
the contract from the W19 perspective (can the placeholder boundary be
documented from it?), the W20 perspective (is the future/non-P0 classification
stated so absence is never misread?), and the P1-W10 perspective (could its
reference invocation, verdict logic, and evidence set be implemented against
this contract without renegotiating semantics?).

**Acceptance:** no competing entry exists; each perspective can act without
inventing policy.  
**Failure/blocker:** a consumer need the contract cannot express is recorded
as an open item against the contract's next version — not solved by
prose elsewhere.

### Step 6 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
verify the package against its task-book requirement (P0-W09), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what was
actually reviewed. The record must state explicitly that no QEMU execution
occurred and none was required.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W09-DV01 → P0-V13 | Contract completeness review | inspect the contract against [runner entry contract](01-runner-entry-contract.md) §2–§8 | single-entry rule, responsibility boundary, grammar, runtime behavior, taxonomy, evidence set, and placeholder mark all present and unambiguous | one documented interface exists; not that a runner exists or runs |
| W09-DV02 → P0-V13 | Placeholder marking review | inspect §8 wording and the stage-index row | P0 status is interface-only and no artifact implies a run or EL2 evidence | no false EL2/guest claim; not future compliance |
| W09-DV03 → P0-V13 | Grammar walkthrough | step 3 trace over success, timeout, launch-failure, and usage-error paths | every path ends in a fixed status class and complete evidence set with no undocumented decision | the interface is complete and extensible; not that QEMU behaves as assumed |
| W09-DV04 → P0-V09 | Discovery and link review | resolve routing row, contract links, and index row from a fresh checkout | one-link reachability; truthful placeholder status; all links resolve | documentation navigation; not W05's taxonomy |
| W09-DV05 → P0-V13 | Single-source review | search tracked files for QEMU automation statements outside the contract | no competing automated entry; human-recipe mentions remain clearly non-automation | uniqueness of the interface; not future enforcement (W20) |
| W09-DV06 → P0-V09 | Prerequisite-reference review | check W02/W03/W17 references and their failure-boundary markings | each reference states its assumption and boundary truthfully | honest dependency record; not prerequisite delivery |
| W09-DV07 → W09 closure | Consumability review | step 5 perspectives | each consumer perspective can act without inventing policy | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with method, input, timestamp, and reason. No W09 validation involves a QEMU
execution; any output suggesting an EL2 or guest run occurred under this
package is a false claim and fails review. P0-V13 is satisfied by the
documentary evidence above; the runner's real behavior is proven only by its
implementing package (expected P1-W10).

## 4. Error, security, and observability model

W09 adds no hypervisor error model, synchronization, guest input handling, or
`unsafe` code; it defines the error *vocabulary* future QEMU tests share. Its
design points:

- **Runner health vs target outcome:** the exit taxonomy (§6) exists so a
  broken environment or a broken runner can never be reported as a failing —
  or passing — hypervisor test. Class `2`/`5` outcomes are never evidence
  about the target.
- **Fail-closed evidence:** timeout and crash paths preserve partial
  evidence and a status class; silence is never success. This is the
  observability contract every future QEMU test inherits.
- **Untrusted input stance:** the grammar treats everything outside the
  reserved parameter classes as a usage error rather than a heuristic match —
  the runner interface itself applies the project's validate-before-use rule.
- **Observability surface:** the evidence content set (§7) is the minimum
  record every future run leaves behind; the verification record for W09
  itself contains only documentary review evidence.

## 5. Handoff checklist

Before handing W09 to a reviewer, provide:

- the exact changed-file list;
- the prerequisite-surface findings from step 1 (pin, build surface, W05
  conventions, W17 naming baseline — or their recorded absence);
- DV01–DV07 evidence paths and status, including the full grammar-walkthrough
  trace and an explicit statement that no QEMU execution occurred;
- the recorded entry contract version and any open items logged against its
  next version;
- confirmation that no script, program, CI workflow, crate, target, QEMU
  flag recipe, image, or timeout default was committed; and
- open items for W19 (placeholder boundary and prerequisites documentation),
  W20 (future/non-P0 classification), P1-W10 (first implementation), W17
  (naming baseline supersession), and W07 (promotion path) — without
  resolving their contracts here.
