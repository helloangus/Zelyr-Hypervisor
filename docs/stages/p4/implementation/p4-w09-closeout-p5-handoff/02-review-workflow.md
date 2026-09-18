# P4-W09 Review Workflow, Validation, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

Before reviewing, the implementer verifies it has loaded the documents named
in the parent README and inventories what actually exists: `git ls-files`,
then a per-package check of `docs/stages/p4/implementation/*-record.md` and
`docs/stages/p4/verification/*.md` as they exist on the review date. The
review evaluates reality; a missing record is data, not an obstacle to work
around.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a package's implementation record and its verification evidence
  disagree — record the conflict in the register with both citations; do
  not pick one;
- a criterion can only pass by reinterpreting its wording — record it
  failed/blocked; criterion text is governed by the task book, not by the
  review;
- a P5-side expectation seems to require a P4 fact P4 never recorded — the
  mapping records the absence; defining the fact is P4-owning-package work
  or a P5 decision, never a closeout invention;
- resolving an item would require changing an accepted ADR, an upstream
  contract, or another stage's document — that is a register entry with
  routing, never a local edit;
- the review is pressured to state stage completion while any matrix row
  lacks evidence — the completion language rule (D6 of
  [01 §4](01-closeout-contracts.md)) forbids it; record the state as it is.

## 2. Ordered review steps

### Step 1 — source inventory

Target: the implementation record's source inventory section.

Work: list every W01–W08 implementation and verification record that exists
on the review date, with revision/date; list every one that does not. This
is the factual basis for every later section.

**Acceptance:** one row per package record with existence status; no
inference from names.  
**Failure/blocker:** none — absence is a valid, recorded outcome.  
**Evidence:** `../../verification/p4-w09-closeout-p5-handoff-verification.md`
(DV01).

### Step 2 — evidence index construction

Target: the evidence-index section ([01 §1](01-closeout-contracts.md)).

Work: for each P4-V01–V16 row, record the evidence location(s) or the
explicit absence, with run status as stated by the verification record
itself (passed / failed / blocked / not run).

**Acceptance:** all sixteen rows present; no row says "supported" without a
verification citation.  
**Failure/blocker:** a verification record whose wording is ambiguous is a
recorded review finding routed to that package.  
**Evidence:** DV02.

### Step 3 — exit-criterion and scope review

Target: the review notes in the implementation record.

Work: evaluate the task book §7 exit criteria 1–6 against the evidence
index; check the temporary-layout boundary (every non-ABI label present per
[01 §2](01-closeout-contracts.md) rule 3), the Guest-fault/Hypervisor-
invariant boundary statements against W06's actual evidence, the not-
delivered scope statement, and the deferral records for planned scenarios.

**Acceptance:** each criterion has a stated evaluation with citations;
labels verified; deferrals complete per task book §6.  
**Failure/blocker:** a failed criterion is recorded as failed with the
missing evidence named — the review never waives a criterion.  
**Evidence:** DV03/DV04.

### Step 4 — factual record drafting

Target: the boot-contract, capability-matrix, limitations, unsafe-delta,
and conflict-register sections ([01 §1](01-closeout-contracts.md)).

Work: draft each section strictly from Step 1 sources under the truthfulness
rules; fill the capability matrix with the two-state rule; populate the
conflict register (P2-ACR-01, A9 provenance, Specification Investigation
items, any new findings with routing).

**Acceptance:** every statement carries a citation; the two-state rule
holds; the register is complete against the W01 gap list plus new findings.  
**Failure/blocker:** a drafting need without a source is moved to the
not-delivered section — never filled from design intent.  
**Evidence:** DV05.

### Step 5 — P4-V16 evaluation

Target: the verification record.

Work: evaluate P4-V16 per its task-book wording — implementation facts, boot
contract, capability matrix, limitations, unsafe inventory updates, evidence
locations, and P5 inputs recorded; unimplemented scope explicit. The
evaluation states exactly which parts are satisfied and which are blocked,
with the review evidence linked. If any matrix row lacks evidence, the
closeout says so and stage closure does not occur; that is a valid review
outcome, not a review failure.

**Acceptance:** a P4-V16 row in the verification record with per-part
status and citations; no completion claim beyond what the rows support.  
**Failure/blocker:** none — honest incompleteness is the designed outcome
for an unfinished stage.  
**Evidence:** DV06.

### Step 6 — P5 handoff finalization

Target: the P5 handoff statement ([01 §3](01-closeout-contracts.md)).

Work: fill the per-deliverable evidence status in the consumer mapping;
state for each P5 package what it may consume now, what is blocked and on
which row, and what remains P4-unimplemented scope. Hand the closeout record
and verification record paths to P5 planning as the P4 side of the
[P5 task book](../../../p5/task-book-v0.1.md) entry conditions.

**Acceptance:** every mapping row has a status; the statement contains no
P5 semantic definitions and no machine-ABI statements.  
**Evidence:** DV07.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W09-DV01 | Source inventory complete | review against `git ls-files` and the tree | every W01–W08 record's existence status listed with date/revision | inventory accuracy; not package correctness |
| W09-DV02 | Evidence index complete and truthful | cross-check index rows against cited verification records | 16 rows, each cited or explicitly absent; statuses match the cited records verbatim | index fidelity; not that evidence exists |
| W09-DV03 | Exit-criteria evaluation sound | review notes vs task book §7 | all six criteria evaluated with citations; failures recorded as failures | evaluation fidelity; not stage completion |
| W09-DV04 | Non-ABI labeling and containment-boundary review | text review of the factual sections | every temporary convention labeled; containment claims scoped to W06's exercised evidence | facts stay test contracts; not P8 machine-ABI fitness |
| W09-DV05 | Factual-record citation integrity | citation audit | every factual statement resolves to an existing record; uncited content absent or moved to not-delivered | truthfulness rules enforced; not underlying evidence quality |
| W09-DV06 → P4-V16 | Closeout evaluation itself | the review run of step 5 | P4-V16 row states per-part status with citations; missing/failed/deferred items explicit | the closeout deliverable exists and is honest; it does not manufacture the missing evidence |
| W09-DV07 | P5 mapping consumability | read the mapping as each §3 consumer | each P5 package can locate its input or its blocking row without invention | handoff readiness; not P5 work |

Record each row as **passed / failed / blocked / not run** with input,
date, and reason. This matrix validates the review deliverables only; it
never substitutes for P4-V01–V15 evidence, and none of its rows may be
reported as stage completion.

## 4. Error and observability model

- **Error model:** the review's failure modes are truthfulness failures —
  an uncited claim, a softened capability state, a missing label, a waived
  criterion, a silent conflict. Each is a review failure fixed by correcting
  the record, not by argument. Conflicts between sources are recorded with
  both citations and routed; the review never adjudicates architecture.
- **Observability model:** the review's outputs are the closeout record and
  its verification record; both are written so a P5 reader can distinguish,
  for every claim, whether it is evidenced, blocked (and on which W01 row),
  or absent. Review evidence (inventory, index, evaluations) lives only in
  `../../verification/p4-w09-closeout-p5-handoff-verification.md`.

## 5. Handoff checklist

Before handing the W09 review to the stage owner:

- the closeout record path with all §1 sections present, or the explicit
  statement of which sections are empty and why;
- the verification record path with DV01–DV07 statuses and dates;
- the conflict register contents: P2-ACR-01 (unchanged, `ADR Required`), the
  A9 provenance finding (open question), every Specification Investigation
  item, and any new register entries with routing;
- the not-delivered scope statement and, if any, the planned-scenario
  deferral records meeting the task book §6 requirements;
- the P5 handoff statement with per-deliverable evidence status and the
  named [P5 task book](../../../p5/task-book-v0.1.md) consumption points;
- confirmation: no completion claim outside verification records, no
  machine-ABI or guest-ABI statement, no upstream document modified, no
  evidence created by the review itself;
- open items carried to the stage owner: every blocked matrix row with its
  blocking dependency, so the stage review can sequence the remaining work.
