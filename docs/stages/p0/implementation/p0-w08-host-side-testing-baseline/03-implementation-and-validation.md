# P0-W08 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm whether the workspace and pin
exist yet), inspection of the delivered workspace structure if present, and a
search for any existing host-test statement.

Prerequisite-surface expectations at implementation time:

- **W02** (declared prerequisite): the toolchain pin and contract exist; the
  host target of the pinned toolchain is what the entry runs on. If absent,
  record a blocker; do not define the entry against an unpinned toolchain.
- **W03** (phase-A package, expected complete per the task-book phase order,
  though not listed among W08's contract prerequisites): the Cargo workspace
  exists with at least one host-testable member and a designated location for
  host-side tests. This is an **assumption with a failure boundary**: if the
  workspace or the designated test location is absent, W08 delivers the
  contract documents and records execution evidence (P0-V03/V04 local
  portion) as blocked, naming W03's contract as the missing surface; it must
  not create a workspace, member, or crate to fill the gap. If W03's plan
  scope turns out not to deliver a host-testable location at all, that is
  recorded as an Architecture Change Request against the plan-index
  dependency map, because the W08 outcome cannot be executed without it.
- **W07/W20** (consumers): their gates and wiring do not exist yet or are
  being prepared in parallel; W08 must deliver the entry so they can bind
  without renegotiating semantics.

Stop and obtain direction instead of guessing when: a tracked document
already claims host-test policy (single-source conflict); making the baseline
executable appears to require a crate, target, or workspace decision (W03
scope); or a test appears to require QEMU, a network, or machine-local state
(that contradicts the plan's independence requirement — redesign the test or
reclassify it out of the host baseline).

## 2. Ordered implementation steps

### Step 1 — record prerequisite-surface findings

Target: implementation record
(`../p0-w08-host-side-testing-baseline-record.md`, created in this step).

Work: inspect the tree; record whether the W02 pin and the W03 workspace and
test location exist, and what the workspace conventions designate for
host-side tests. This decides whether steps 3–4 can execute or are blocked.

**Acceptance:** the record states each surface's availability with pointers.  
**Failure/blocker:** a missing W02 surface is a blocker; a missing W03
surface defers execution evidence per §1.

### Step 2 — author the host-test baseline contract

Target: `docs/testing/host-test-baseline.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
the normative content fixed by the [host-test contract](01-host-test-contract.md)
§1–§7 and the [category matrix](02-test-category-matrix.md) §2–§7 (either as
a section or as a clearly linked second normative file owned by the same
contract): testable-logic boundary and rules, organization rules, entry
semantics, the proof boundary, and the coverage categories with the mapping
rule. Leave the canonical entry spelling recorded per step 3's rule.

**Acceptance:** all required sections present; the contract names no crate,
target, gate, or CI mechanism owned elsewhere; it contradicts no ADR,
task-book, or Coding-Guidelines rule.  
**Failure/blocker:** contradictions are raised per §1, not reworded away.

### Step 3 — establish the minimal baseline test

Target: one placeholder host test inside the W03-delivered workspace, at the
designated host-test location.

Work: create the minimal placeholder test — its only subject is the entry's
health, asserting only that the harness executes and reports. Record the
placement decision and the canonical entry spelling, selected per
[host-test contract](01-host-test-contract.md) §5, in the contract document
and the implementation record. If the prerequisite surface is missing, this
step is recorded blocked with the §1 boundary — no substitute artifact is
created.

**Acceptance:** the test is committed, placed per the designated conventions,
asserts nothing beyond entry health, and the recorded spelling is the one
step 4 executes.  
**Failure/blocker:** any temptation to give the placeholder product meaning,
or to create a member for it, is a W03/scope boundary — stop and record.

### Step 4 — execute the baseline and the failure-visibility check

Target: verification record
(`../../verification/p0-w08-host-side-testing-baseline-verification.md`).

Work: (a) run the recorded entry spelling; record command, full output,
environment (OS, toolchain, date), and result. (b) Failure-visibility check:
locally mutate the placeholder to fail, re-run the entry, confirm a non-zero
exit and a visible failure report, then revert the mutation; record both runs
and the revert. The mutated variant is never committed. Record what was not
run and why (CI execution waits for W20; QEMU-class checks are out of scope).

**Acceptance:** the record shows a passing run, a failing run with non-zero
exit and visible failure output, and explicit not-run entries for deferred
proofs.  
**Failure/blocker:** an entry that passes a broken test, hides a skip, or
reports success on compilation failure is a semantics defect — fix the entry
contract's realization, not the expectation; a genuine blocker is recorded.

### Step 5 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md` (and,
optionally, a pointer line in `docs/testing/README.md` that does not restate
policy).

Work: add one routing-table row to `docs/README.md` pointing host-testing and
test-organization work at the contract, and add the W08 design row to the
stage implementation index with a truthful status. Change nothing else.

**Acceptance:** one-link reachability from `docs/README.md`; all new relative
links resolve from a fresh checkout.  
**Failure/blocker:** broken or duplicating links fail review.

### Step 6 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
verify the package against its task-book requirement (P0-W08), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what was
actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W08-DV01 → P0-V09 | Boundary review | inspect the contract's §2 classes against ADR-049 and the [category matrix](02-test-category-matrix.md) §4 | both boundaries stated with the three rules; no future mechanism implemented | the boundary exists and is decisive; not that any logic is tested |
| W08-DV02 → P0-V03 | Entry executability | run the recorded spelling; entry builds every host-target member | one invocation compiles the host-test set and exits truthfully on the pinned toolchain | the documented entry works locally; not CI execution (W20) |
| W08-DV03 → P0-V04 | Baseline pass | step 4(a) evidence | the minimal baseline passes with a quotable summary | the baseline executes and passes; not product correctness |
| W08-DV04 → P0-V04 | Failure visibility | step 4(b) mutated run and revert | failing test yields non-zero exit and visible report; revert leaves tree clean | the entry's success signal is truthful; not flakiness-freedom over time |
| W08-DV05 → P0-V09 | Category-matrix review | inspect the matrix against the plan's five classes plus the conditional extension | five categories defined with gap meanings; mapping rule present; applicability table consistent | future designs have binding categories; not that future tests exist |
| W08-DV06 → P0-V09 | Proof-boundary and discovery review | inspect contract §6 verbatim-in-meaning presence; resolve routing and index links from a fresh checkout | boundary stated and referenced in result-report rules; one-link reachability; truthful status | non-proof is contractual; not W09/QEMU or hardware evidence |
| W08-DV07 → W08 closure | Consumability review | read the entry as W07 (bindable?), W19 (quotable in onboarding?), W20 (CI-executable?), and a future module design (mappable?) | each perspective can act without inventing policy | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. P0-V03/V04 stage
evidence completes when W20 executes the entry in CI; W08's local evidence
establishes the entry and baseline but does not report the CI portion as
done. No W08 validation proves QEMU, guest, EL2, or hardware behavior, and
none may be reported as doing so.

## 4. Error, security, and observability model

W08 adds no hypervisor error model, synchronization, guest input handling,
hardware access, telemetry, or `unsafe` code. Its failure reporting is the
entry's own truthfulness: failures must be visible, skips must be reported,
and success must mean success — validated by DV04.

The security position: the invalid-input category and the testable-logic
boundary operationalize the ADR's untrusted-input principle at the test layer
— externally influenced logic must have hostile-input coverage before its
design is accepted, and a passing host suite must never be quoted as
isolation or hardware evidence. The proof boundary is therefore a security
statement, not only a documentation rule.

Observability is the evidence trail: the entry's summary (run/passed/failed/
skipped), the verification record's commands and outputs, and the run/not-run
status per validation are the only accepted proof surfaces.

## 5. Handoff checklist

Before handing W08 to a reviewer, provide:

- the exact changed-file list, including the placeholder test's location;
- the prerequisite-surface findings from step 1 (pin, workspace, designated
  test location — or the recorded blocked status);
- the recorded canonical entry spelling, toolchain, and selection date;
- DV01–DV07 evidence paths and run status, including the failure-visibility
  pair and explicit not-run entries (CI execution, QEMU-class checks);
- confirmation that no crate, workspace, target triple, parser, hypervisor
  mechanism, gate definition, CI workflow, or `unsafe` artifact was added;
- confirmation that the contract names no gate semantics beyond the entry
  binding W07 consumes; and
- open items for W07 (QG-TEST-HOST binding), W19 (workflow step), W20 (CI
  execution evidence), W03 (designated test location if it was missing), and
  the future module designs that consume the category matrix — without
  resolving their contracts here.
