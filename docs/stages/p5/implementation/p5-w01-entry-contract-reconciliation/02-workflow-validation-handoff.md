# P5-W01 Workflow, Validation, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before producing any review output, the implementer verifies it has read the
documents named in the [parent README](README.md) §Purpose and use, and
inspects the tracked tree. Useful read-only discovery: `git ls-files`
(confirm which upstream implementation and verification records actually
exist — expected in the current scaffold: none beyond the P0-W01
repository-baseline record) and a search of `docs/stages/p5/` for any
pre-existing entry-review material.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a tracked upstream document contradicts the [ledger](01-reconciliation-ledger.md)
  row derived from it — re-derive the row from the source; if the conflict is
  between two governing documents, apply label `Contract Conflict`;
- reconciliation appears to require designing an HVC value, object API, or
  any P5 mechanism to "complete" a row — that is Out of Scope; route the need
  to the owning package instead;
- a maintainer directs W01 to absorb or resolve P2-ACR-01 or any inherited
  open item — refusal is correct per the design; the item moves only through
  its owning stage's process; or
- the entry-review record is asked to state that an upstream stage closed —
  W01 cannot make that claim at any point.

## 2. Ordered implementation steps

### Step 1 — inventory the sources

Target: entry-review record, inventory section.

Work: enumerate the exact tracked sources the review uses — the ADR, the P5
task book and plan index, every P0–P4 plan and handoff-contract path cited in
[ledger](01-reconciliation-ledger.md) §2, and every upstream implementation
or verification record that exists at review time. Record the review date and
the observed absence of the expected upstream records.

Suggested observation: `git ls-files docs/stages docs/adr` and a directory
listing of each stage's `implementation/` and `verification/`.

**Acceptance:** the record lists every source with its path and its
present/absent status; no source is cited that was not inspected.  
**Failure/blocker:** a cited-but-unlocatable path is corrected before
proceeding; a missing expected plan is itself a `Blocked Prerequisite` row.

### Step 2 — reconcile the fact domains

Target: entry-review record, FD-1 through FD-5 sections.

Work: for each row of [ledger](01-reconciliation-ledger.md) §2, verify the
row against its current source text (contract-level mode), record the
evidence status, and note any wording drift between the ledger row and the
source. Where P4 or later records exist (evidence-level mode), verify the row
against the record and update the Evidence column with the record path.

**Acceptance:** every row carries an evidence status or record path; every
deviation between row and source is either corrected or labeled per §5 of the
ledger.  
**Failure/blocker:** a contradiction between two sources becomes a
`Contract Conflict` row; the review never rewrites either source.

### Step 3 — route the future artifacts

Target: entry-review record, routing section.

Work: instantiate [ledger](01-reconciliation-ledger.md) §4 against the
current tree: confirm `docs/abi/` and `docs/security/` exist as governed
locations, that no P5 design or plan has placed factual ABI/security content
outside them, and that each consumer package's design (as it comes to exist)
carries the routing pointer.

**Acceptance:** the routing table is confirmed or amended with reasons; any
misplaced factual content is recorded as a finding for the owning package.  
**Failure/blocker:** a governance-location conflict (for example a plan
demanding a different home) is a `Contract Conflict` against that plan —
recorded, not absorbed.

### Step 4 — classify gaps and inherited items

Target: entry-review record, findings section.

Work: apply the §5.1 labels to every absent, ambiguous, or contradictory
input found in steps 2–3; instantiate the §5.2 register with current status.
Each classification names the affected consumer package and the stop/record
action.

**Acceptance:** no finding is unlabeled; no label implies P5-side resolution
of an upstream decision; P2-ACR-01 and ADR-056 remain visible and unowned by
P5.  
**Failure/blocker:** a finding that seems to need a new label indicates the
vocabulary is insufficient — raise it in the record for design review instead
of inventing a label locally.

### Step 5 — record the entry review

Target: `../p5-w01-entry-contract-reconciliation-record.md` (created in this
step).

Work: write the record with: the source inventory, the reconciled domains,
the routing confirmation, the labeled findings, the per-consumer input
assignment (ledger §6), and any deviation from this design. Record actual
review commands and citations in
`../../verification/p5-w01-entry-contract-reconciliation-verification.md`.
The record states what W01 reviewed and handed off; it does not claim any
P5 mechanism exists.

**Acceptance:** the record is complete against steps 1–4, links resolve from
a fresh checkout, and its status wording makes no completion or evidence
claim beyond the documentation review itself.  
**Failure/blocker:** a link that does not resolve fails the step; fix the
link, not the target's location.

### Step 6 — hand off and closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
each W02–W10 consumer can locate its inputs from the record without
re-deriving the boundary. W01's acceptance (P5-V01) is judged on this
documentation review; real runtime evidence is not sought or claimed here.

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W01-DV01 → P5-V01 | Sources inventoried | record review | compare the record's source list against `git ls-files` and the ledger citations | every source present/absent status is true at review date | inventory accuracy at review time; not later drift |
| W01-DV02 → P5-V01 | P4 facts reconciled (FD-1, FD-3) | row-by-row review | read each row against its cited P1/P4 source | each row's fact, evidence status, and failure boundary match the source; no strengthening | contract-level reconciliation; not that P4 behavior exists |
| W01-DV03 → P5-V01 | P2/P3 constraints reconciled (FD-2, FD-5) | row-by-row review | as DV02 for P2/P3 sources | as DV02; P2-ACR-01 visible and unresolved | as DV02 |
| W01-DV04 → P5-V01 | Artifact routing | governance review | walk ledger §4 paths from a fresh checkout | all routing targets exist or are defined governed locations; no factual ABI/security content outside them | routing correctness; not that the artifacts exist |
| W01-DV05 → P5-V01 | Gap classification | findings review | re-derive each label from the §5.1 definitions | every finding labeled; no silent redesign; blocked/conflict items stop their consumers | classification discipline; not resolution of any item |
| W01-DV06 → P5-V01 | Handoff consumability | consumer walkthrough | read the record as W02–W10 in turn; locate each package's inputs | each consumer finds its domain inputs, sibling-contract rules, and routing without inventing policy | handoff readiness; not that consumer packages are designed or done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with citations, date, and reason. No validation here proves any P5 runtime
property (P5-V02 through P5-V17) and none may be reported as doing so.

## 4. Error, security, and observability model

W01 adds no runtime error model, guest input handling, synchronization, or
`unsafe` code. Its failure reporting is the review record itself: a
mis-derived row, an unlabeled finding, an unresolvable citation, or a routing
gap fails the associated DV and is recorded as failed or blocked with
diagnosis.

Security posture: W01's security contribution is *negative space* — it fixes
that no P5 package assumes undocumented upstream behavior, that inherited
open items stay visible, and that factual security content can appear only in
the governed locations. A review that silently normalizes a contradiction
would itself be the security event; the labels exist to make that impossible
to do quietly.

Observability is the evidence trail: the verification record's citations,
dates, and run/not-run status are the only accepted proof surface for
P5-V01.

## 5. Handoff checklist

Before handing W01 to a reviewer, provide:

- the exact changed-file list (expected: the three design files of this
  directory, the entry-review record, and the verification record);
- the source inventory with per-source present/absent status and review date;
- DV01–DV06 evidence paths and run status, including explicit not-run entries
  for anything evidence-level reconciliation could not yet reach;
- confirmation that no P5 mechanism, ABI value, or upstream repair was
  introduced, and that P2-ACR-01 and ADR-056 remain visible and unresolved;
- the per-consumer input assignment for W02–W10 (ledger §6), with any
  `Blocked Prerequisite` / `Contract Conflict` / `ADR Required` /
  `Documentation Gap` findings routed to their owners; and
- open items for later packages: W02–W05 must repeat the assumed-contract
  form of [ledger](01-reconciliation-ledger.md) §3; W10 must consume the
  routing and open-item registers at closeout — without resolving their
  contracts here.
