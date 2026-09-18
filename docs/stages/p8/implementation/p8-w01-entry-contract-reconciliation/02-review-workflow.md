# P8-W01 Review Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before writing the record, the implementer verifies it has loaded the
documents named in the parent README and inspects the tracked tree. Useful
read-only discovery: `git ls-files docs/stages` to enumerate the P0–P7
implementation and verification content actually present, and
`git ls-files docs/machine-types docs/abi` to confirm both contract
directories are still stubs. The tree inspected is the branch the work targets;
if the merge-base state differs materially from the findings stated in the
parent README (for example, a P1–P7 verification record has appeared), the
record reflects the *observed* state with its inspection date, not the
README's snapshot.

Stop and obtain direction instead of guessing when any of the following
occurs:

- an upstream record exists but contradicts the task book §2 boundary for its
  stage — register a **Conflict** row and an `Architecture Change Request`; do
  not reinterpret the boundary locally;
- a required input has neither a handoff plan nor a record — register
  **Blocked** with the owning stage; do not draft the missing input inside P8;
- resolving an entry appears to require choosing a machine value, a PSCI
  subset, a CPU feature baseline, or any other Guest ABI detail — route it to
  the [W02](../p8-w02-machine-contract-governance/README.md) decision route;
  choosing it here is a scope violation;
- two inputs contradict each other — register the **Conflict**; do not pick a
  winner.

## 2. Ordered implementation steps

### Step 1 — load and verify the authorities

Target: working context (no artifact change).

Work: read the ADR baseline, P8 task book, P8 plan index, the P8-W01 plan,
this design, and skim every P0–P7 handoff plan named in
[the input inventory](01-reconciliation-contract.md) §2. Confirm the task-book
§2 boundary wording for each stage is understood as the review standard.

**Acceptance:** the implementer can state, per stage, the required input and
the boundary without re-reading.  
**Failure/blocker:** a missing or unreadable authority document is a recorded
blocker; W01 cannot be reviewed against a document it could not load.

### Step 2 — inspect the evidence state

Target: [the reconciliation record](01-reconciliation-contract.md) §5 §2
(per-stage table), created in this step.

Work: for each §2 row, inspect the named implementation and verification
locations in the tracked tree and record the observed state with the
inspection date. Copy evidence-implied limits verbatim or by exact reference;
do not paraphrase limits into weaker ones.

Suggested observation: `git ls-files docs/stages/pX/implementation
docs/stages/pX/verification` per stage.

**Acceptance:** every row has an observed state grounded in the tracked tree;
no row says "assumed".  
**Failure/blocker:** an unreadable or ambiguous location is recorded as
observed state ("location empty/placeholder") — never skipped.

### Step 3 — classify each input

Target: record §2 disposition column.

Work: apply the class decision rules from
[the reconciliation contract](01-reconciliation-contract.md) §3.1 to each row.
Assign the design-time and implementation-time reliance fields per §3.1.

**Acceptance:** every row has exactly one class; **Planned-only** rows state
the assumed-contract rule for designs and the implementation block; no
**Planned-only** row is marked implementation-usable.  
**Failure/blocker:** a row that seems to fit two classes is a schema defect —
raise it; do not invent a fifth class.

### Step 4 — write the constraint register

Target: record §5 §3.

Work: instantiate [the constraint register](01-reconciliation-contract.md) §4
with citations, adjusting only wording, not substance. Add a row only with a
normative citation.

**Acceptance:** all six required constraint families are present, each cited;
no uncited constraint exists.  
**Failure/blocker:** a constraint that cannot be cited to a governing document
is raised as a potential new-policy question, not silently included.

### Step 5 — route conflicts and blocks

Target: record §5 §4 (conflict and block register).

Work: for every **Conflict** and **Blocked** row, name the owner and the
decision route: W02's machine decision route for machine-model inputs;
`Architecture Change Request` / `ADR Required` records for architecture or
security contradictions; the owning upstream stage for missing inputs. Verify
each task-book §8 unresolved topic appears either as routed or as explicitly
out of P8's need.

**Acceptance:** no conflict row lacks an owner and route; no machine value is
resolved anywhere in the record.  
**Failure/blocker:** a conflict whose owner is genuinely unknown is itself
recorded as an open item for the stage coordinator; it must not be resolved
locally to finish the review.

### Step 6 — consumer cross-check and closure review

Target: record §5 §5–§6, verification record.

Work: cross-check each record row against the consumers the [P8 plan index]
(../../plans/README.md) lists for W02–W20: for every consumer plan, confirm
its needed inputs are covered by rows whose limits are stated. Confirm the
parent README's downstream-handoff list matches the record. Run the validation
matrix below; record results with explicit not-run entries in the verification
record.

**Acceptance:** P8-V01 condition met (every row linked or blocked; no
undocumented-behavior reliance) and every consumer's dependency visible.  
**Failure/blocker:** a failing review item is recorded as failed with
diagnosis; it is not fixed by weakening a row's limit wording to pass.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W01-DV01 → P8-V01 | authority and inventory review | inspect record §1–§2 against this design §2 | every P0–P7 row present with its handoff-plan citation and inspected locations; status header and section set per contract §5 | the review covers the required inputs; not that any upstream mechanism works |
| W01-DV02 → P8-V01 | disposition completeness review | per row, re-derive the class from the §3.1 rules and the cited location | every row has an evidence link or an explicit recorded absence/block; classes follow the rules; inspection dates present | the link-or-block property P8-V01 requires; not that *Evidenced* rows prove future behavior |
| W01-DV03 → P8-V01 | consumer coverage review | map each W02–W20 consumer plan's prerequisite column to record rows | every consumer input is covered by a row with stated limits; no consumer relies on an undocumented behavior | the plan's "later packages receive only reconciled factual inputs and recorded limits" handoff; not that consumers are correct |
| W01-DV04 → P8-V01 | constraint-register review | check register rows against ADR/task-book citations | all six required families present and cited; no uncited entry | constraints are traceable to authority; not that later packages comply |
| W01-DV05 → P8-V01 | conflict-routing review | inspect §4 register against task book §8 and ADR §18 | every conflict has owner + route; no machine value resolved; task-book §8 topics all routed or explicitly out of need | unresolved inputs stay unresolved with a route; not that they are resolved |
| W01-DV06 → P8-V01 | link-integrity and claim review | resolve every link from a fresh checkout; search the record for completion or freeze language | all links resolve; the record asserts no implementation, validation, stage completion, or machine freeze | the review is a review; not runtime evidence of anything |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with method, input, date, and reason. P8-V01 is satisfied only when DV01–DV06
are recorded, with failures resolved or explicitly routed. No validation here
proves P8-V02 through P8-V26, and none may be reported as doing so.

## 4. Error, security, and observability model

W01 adds no runtime error path, guest input handling, or telemetry. Its
failure model is documentary: an incorrect disposition, a missing limit, or an
uncited constraint fails the associated review and is corrected in the record
or routed as a conflict. The security-relevant property is informational
integrity: the record is what later packages consult to decide what may be
assumed, so an overstated row is a security-relevant defect (it could cause a
consumer to rely on an unproven isolation or capability property). The
observability surface is the verification record: per-validation status,
method, and date, with planned, run, blocked, and failed evidence kept
distinct.

## 5. Handoff checklist

Before handing W01 to a reviewer, provide:

- the exact changed-file list (expected: the record, the verification record,
  and any stage-index status row added per that index's conventions);
- the inspection date and branch/commit the review reflects;
- DV01–DV06 evidence paths and run status, including explicit not-run entries;
- confirmation that no machine value, PSCI subset, CPU feature list, or Guest
  ABI detail was selected anywhere in the record;
- confirmation that no upstream stage document was edited and no completion
  claim was made for any stage; and
- the open items for the routed conflicts and blocks, each with owner and
  decision route, for consumption by W02 and the stage coordinator.
