# P0-W22 Implementation Workflow and Validation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W22 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tree. Useful
read-only discovery: `git ls-files` (confirm no stage-root handoff map
exists and inventory the delivered records under `docs/stages/p0/`), a
reading of the [stage implementation index](../README.md) for design/record
statuses, and a scan of the verification area for recorded evidence. W22 is
last in the P0 dependency chain; much of its input is the delivery state of
the other twenty-one packages, which is whatever it is at implementation
time — the map registers that truthfully rather than waiting for an ideal
state.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the delivered [W05](../p0-w05-documentation-baseline/README.md)
  stage-separation rules do not admit a stage-root governance document —
  raise the placement question under W05's policy-decision threshold and
  record it (decision 1 of the parent README); do not relocate the map
  silently;
- a register row's deliverable cannot be located from the plan, the stage
  index, or a delivered design — fill the row with the owning package and
  `plan-only` (or `blocked`) status; never invent a location;
- filling the map appears to require judging whether a delivered package's
  evidence is *good* — that is the completion review's job; the map points
  to evidence and reports its existence, it does not re-adjudicate it;
- the drill appears to require executing builds or QEMU — it does not; the
  drill is a reading-order exercise.

## 2. Ordered implementation steps

### Step 1 — audit the delivery state of W01–W21

Target: implementation record (created in this step).

Work: for each of W01–W21, determine and record: the deliverable's
authoritative location (from the plan, the delivered design, or the stage
implementation index — in that order of authority), the current status per
the four-state vocabulary, and the evidence pointer where `delivered`.
Sources: the stage implementation index, `git ls-files` for delivered
artifacts, and the verification area for evidence. W11–W15 are referenced by
slug and P0-Wxx ID with whatever status the index shows.

**Acceptance:** twenty-one rows of status data with pointers, no invented
location, no unevidenced `delivered`.  
**Failure/blocker:** an unlocatable deliverable is recorded as `plan-only`
or `blocked` with the owner named; the gap is data for the map, not a reason
to stall the whole register.

### Step 2 — author the handoff-map document

Target: `docs/stages/p0/p0-handoff-map.md`.

Work: write the document with the status header and exactly the parts
required by [the handoff-map contract](01-handoff-map-contract.md) §2: the
reading rule, the status vocabulary, the register filled from step 1, the
three consumption tables, the completion-report contract, and the update
rules. Every row points; nothing restates contract content.

**Acceptance:** all six parts present; every row complete per the register
rules; no contract prose; the [P1 task
book](../../../p1/task-book-v0.1.md) §1 supply list is fully resolved by
§4.1's rows.  
**Failure/blocker:** a contradiction with a governing document or the plan
index is raised per §1, not absorbed.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row pointing handoff/consumption/P1-onboarding
questions at the map, and add the W22 stage-index row with truthful status.
Change nothing else in either file.

**Acceptance:** a P1 planner starting from `docs/README.md` or the P1 plan
index reaches the map in one link; the index row reflects the real status;
all new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — run the P1 reading-order drill

Target: verification record.

Work: execute the drill of plan work sequence 4 as a P1 planner would:

1. Start from the [P1 plan index](../../../p1/plans/README.md) reading order
   and the P1 task book §1 supply list. For each supply-list item, follow
   the map row to the authoritative artifact and record: found / not found,
   the link path taken, and the status shown.
2. For the two anchor packages — P1-W01 (reference boot contract, whose
   prerequisites name the P0 target/build/QEMU entry) and P1-W12 (P1
   documentation handoff, which consumes the whole P0 baseline) — confirm
   the plan-index prerequisite map plus the register lets a planner name
   every P0 input and its status without reading more than the named
   pointers.
3. Confirm the negative case: for at least one input that is not yet
   `delivered` at drill time, verify the row's blocking implication names
   the upstream-defect rule and owner rather than inviting redesign.
4. Record each step as `reachable`, `blocked` (missing pointer, with the
   owning package named), or `failed` (an inconsistency, contradiction, or
   status that misrepresents the evidence). `blocked` outcomes from
   not-yet-delivered packages are truthful and expected; `failed` outcomes
   are defects to fix in the map.

**Acceptance:** every supply-list item resolves or carries an actionable
blocked mark; no failed outcome remains; the drill records no execution
evidence and claims none.  
**Failure/blocker:** an assembly defect is fixed in the map; a content
defect is routed to the owning package; neither is fixed by improvising map
content.

### Step 5 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
verify the package against its task-book requirement (P0-V15), prerequisite
compatibility (the map's inputs are the delivered state, recorded
truthfully), document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what
was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W22-DV01 → P0-V15 | register review | inspect the map's register against the [plan index](../../plans/README.md), the stage index, and step 1's audit | twenty-one complete rows; statuses truthful; pointers resolve; no invented location; no contract prose | the register is a complete, truthful view; not that any registered package is complete |
| W22-DV02 → P0-V15 | consumption-mapping review | apply §4.1–§4.3 rows against the P1/P2/P3 task books' stated needs | every stated need resolves to deliverables with reuse condition and blocking implication; no internal sequencing implied | consumers know what they may rely on and what a gap means; not that the consumers have started |
| W22-DV03 → P0-V15 | completion-report contract review | inspect §5 against task book §7's handoff-package list and the [W21 completion-layer contract](../p0-w21-stage-plan-implementation-workflow/01-layer-contract.md) | all required content elements present; location and author fixed; nothing pre-written | the report can be authored at completion without new policy; not that the report exists |
| W22-DV04 → P0-V09 | discovery and link review | resolve the routing row, map links, and index row from a fresh checkout | one-link reachability from `docs/README.md` and the P1 plan index; truthful status; all links resolve | documentation navigation; not W05's taxonomy ownership |
| W22-DV05 → P0-V15 | P1 drill, supply-list walk | step 4 procedure, part 1 | every supply-list item found with distinguishable status; no `failed` | a P1 planner finds complete inputs; not that P1 is planned or executable |
| W22-DV06 → P0-V15 | P1 drill, anchors and negative case | step 4 procedure, parts 2–3 | both anchor walks complete without reading beyond named pointers; the not-yet-delivered case routes to the upstream-defect rule | P0 need not be recreated by P1; not that every P0 input is already delivered |
| W22-DV07 → W22 closure | consumability review | read the map as the P0 completion review (can I author the report from it?) and as a P1 planner (can I start from it?) | each consumer can act without inventing policy or re-deriving P0 choices | handoff readiness; not stage completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command or observation, input, environment, timestamp, and
reason. The drill is discoverability evidence only: it proves nothing about
any package's implementation or validation, and no entry may be reported as
doing so. Nothing in W22 constitutes evidence toward P0-V01–V14; those
belong to their owning packages.

## 4. Error, security, and observability model

W22 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary: a
row with an invented location, a status contradicting the pointed-to
evidence, a supply-list item without a resolution path, or a failed drill
step fails the associated review and is recorded as such. The
security-relevant property is honesty of the handoff: the map's value is
that delivered, unverified, and not-started work cannot be confused, so a
status that flatters the delivery state is treated as the defect class this
package exists to prevent.

Observability is the drill and review evidence trail in the verification
record: link paths, statuses observed, and run/not-run status are the only
accepted proof surface.

## 5. Handoff checklist

Before handing W22 to a reviewer, provide:

- the exact changed-file list;
- the step 1 audit summary with per-package statuses and evidence pointers;
- DV01–DV07 evidence paths and run status, including explicit blocked entries
  for not-yet-delivered packages encountered during the drill;
- confirmation that no contract content was restated, no P1+ sequencing was
  planned, no completion report was pre-written, and no completion claim was
  made;
- confirmation that the placement question (stage-root document vs. the
  delivered W05 separation rules) is either cleanly admitted by the delivered
  W05 rules or raised and recorded — not silently resolved;
- open items for the P0 completion review (author the report per §5 when
  closure evidence exists) and for P1 planning (the map as the entry point) —
  without resolving their work here.
