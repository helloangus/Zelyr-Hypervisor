# P0-W06 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files docs/adr docs/templates` (confirm the
five-line `docs/adr/README.md`, the baseline ADR, and the empty-by-rule
`docs/templates/`), plus a search of tracked documents for existing ADR-threshold
statements (expected instances: `AGENTS.md`, both concise guides, and the
W02–W04 governance baselines if delivered).

Stop and obtain direction instead of guessing when any of the following occurs:

- an existing tracked document claims authority over ADR process in a way
  that conflicts with this design — raise the conflict; do not edit the other
  document's authority silently;
- completing the governance document appears to require resolving a 待定
  register item, changing an accepted decision, or adjudicating a real
  pending conflict — that is exactly this package's out-of-scope list; record
  the item as an open question for the owner;
- the owner declines the narrow register-annotation edit rule — drop rule
  (b), keep register rows untouched, and record the decision in the
  implementation record; do not implement the annotation anyway;
- the drill appears to require creating an actual ADR file to be
  "realistic" — it must not; the drill is evidence in the verification
  record plus the labeled informative appendix;
- "enforcing" the lifecycle appears to require CI checks or lint tooling —
  that is W20/W07 scope and must be designed there, not improvised here.

## 2. Ordered implementation steps

### Step 1 — expand the ADR governance document

Target: `docs/adr/README.md`.

Work: rewrite the five-line note into the normative governance document with
the status header required by `docs/README.md` (status, scope, version
`v0.1`, owner/change context, supersedes: the absent explicit process) and
exactly the required content of
[the ADR lifecycle contract](01-adr-lifecycle-contract.md) §3–§9: the state
table and transitions, the ADR decision test with the local-choice boundary,
the label semantics, the conflict/supersession/review process, the history
preservation rules, the register interaction, the drill appendix, and the
initial index table (one row: ADR-000, the architecture baseline, Accepted).
The document must preserve every true statement of the existing five lines
(they are consistent with it) and must not restate W05's taxonomy or W21's
workflow, nor alter any decision text of adr-000.

**Acceptance:** every required section is present with its required content;
the document contradicts no accepted decision, no `AGENTS.md`/`docs/README.md`
mandate, and no delivered governance baseline; the index row for adr-000 is
present.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording this document.

### Step 2 — create the ADR template

Target: `docs/templates/adr-template.md`.

Work: write the template exactly per
[the template contract](01-adr-lifecycle-contract.md) §9, with instructional
comments for each field and no project-specific decision content.

**Acceptance:** all required fields present; the template validates against
the governance document's states and numbering rules; `docs/templates/README.md`'s
gating rule is satisfied without editing that file.  
**Failure/blocker:** a mismatch between template fields and the governance
document's process is a review failure; fix whichever is wrong through
review, not by silent divergence.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add or extend one routing-table row in `docs/README.md` so that
ADR-creation, lifecycle, and architecture-change questions route to
`docs/adr/README.md` (the existing "architecture-affecting work" row already
points at the baseline; add the process pointer without duplicating rows),
and add the W06 design row to the stage implementation index with a truthful
status. Change nothing else in either file.

**Acceptance:** a newcomer starting from `docs/README.md` can reach both the
ADR baseline and the ADR process in one link each; the index row reflects the
real status; all new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — run the traceability drill

Target: verification record
(`docs/stages/p0/verification/p0-w06-adr-governance-verification.md`,
created in this step).

Work: execute the drill of
[the lifecycle contract](01-adr-lifecycle-contract.md) §8 as a reviewer
would: take the hypothetical board-name proposal, apply the §4 decision test
(showing it lands in "requires an ADR"), record the label that applies
(`Architecture Change Request` here; state when `ADR Required` would instead),
walk the §6 path to its hypothetical acceptance (index row, superseded
status lines), and demonstrate the history-preservation rule by listing
exactly which bytes of the affected existing files would have changed
(status lines only) and which would not. Record every step and its basis.
Then confirm the negative case: the drill created no file, changed nothing,
and resolved nothing. Record what was not exercised and why.

**Acceptance:** the drill evidence shows a reviewer can determine the
conflicting ADRs, the correct label, and the correct path using only
repository documents; no repository file changed during the drill.  
**Failure/blocker:** a step the documents cannot answer is a rule gap — fix
the governance document through review and re-run the drill; do not leave
the gap or resolve the hypothetical for real.

### Step 5 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement, prerequisite compatibility
with delivered W05 conventions, document links, and downstream handoff
wording (all later plans and stages, W21, W05, W10/W11/W14/W15/W18, W20).
Completion is claimed only in the verification record, with evidence, and
only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W06-DV01 → P0-V10 | lifecycle review | inspect the governance document against the lifecycle contract §3 | all five states defined with transitions and normativity; no transition contradicts adr-000's change rule or `AGENTS.md` | the state machine is stated and followable; not that all future states of real ADRs are anticipated |
| W06-DV02 → P0-V10 | threshold review | apply the §4 decision test to sample changes (a routine pin bump, a profile activation, a board-branch proposal, a documentation fix) | each sample classifies unambiguously; instances referenced, not restated | the ADR-vs-local boundary is decidable; not that every future case is covered |
| W06-DV03 → P0-V10 | traceability drill | step 4 evidence in the verification record | reviewer reaches conflicting ADRs, label, and path from documents alone; no file changed; negative case confirmed | the supersession path is followable; not that a real conflict has been resolved |
| W06-DV04 → P0-V09/P0-V10 | template review | inspect the template against the lifecycle contract §9 and a trial fill-in | all required fields present and mutually consistent with the process; trial ADR draft carries a complete review record | the template makes the lifecycle usable; not that any ADR has been proposed |
| W06-DV05 → P0-V10 | history-preservation review | inspect §6's narrow-edit rule against `AGENTS.md`/`docs/README.md` and the adr-000 tail rule; git-history spot check that adr-000 decision text is untouched by this package | rule set is consistent and strictly narrower than the mandates; adr-000 untouched except permitted lines | accepted decisions are structurally protected; not that no future PR will attempt an improper edit |
| W06-DV06 → P0-V09 | discovery and link review | resolve the routing rows, governance links, template links, and index row from a fresh checkout | one-link reachability for both baseline and process; truthful index status; all links resolve | documentation navigation; not W05's taxonomy decisions |
| W06-DV07 → W06 closure | consumability review | read the governance document as a P1 planner with a conflict, as W21 defining admission rules, as W05 pointing at ADR mechanics, and as an owner of a threshold section (W02-style) | each consumer can act without inventing policy; the two labels are defined and routed identically | the mechanism is usable by every later plan/stage; not that any has used it yet |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the
governance document and template without the DV03 drill does not satisfy the
plan's fourth work item. No validation here proves that GitHub enforcement,
CI checks, or any future ADR decision exists, and none may be reported as
doing so.

## 4. Error, security, and observability model

W06 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is textual: an
ambiguous state, an unclassifiable change, a drill gap, a template/process
mismatch, or an inconsistent index fails the associated review and is
recorded as such.

The security relevance is architectural integrity: the accepted decisions
protect the trust boundaries (guest untrusted, capability-based
authorization, layer separation, no silent ADR edits). A bypassed or ambiguous
lifecycle is how those boundaries erode. This design's defenses are textual
and procedural — the narrow-edit rule, the append-only change history, the
index reconciliation — and are auditable by review; W06 adds no tooling.
Observability of the lifecycle is the index table plus each ADR's change
history; observability of this package is the verification record's drill
and run/not-run evidence, the only accepted proof surface.

## 5. Handoff checklist

Before handing W06 to a reviewer, provide:

- the exact changed-file list;
- the owner-confirmation status of the narrow register-annotation rule
  (decision 3 of the parent README), or its recorded rejection;
- DV01–DV07 evidence paths and their run status, including explicit not-run
  entries (no real ADR proposed, no real conflict resolved, no CI or lint
  enforcement);
- confirmation that adr-000's decision text, principles, and register rows
  are byte-identical except for any permitted status-line annotations (and
  none without owner confirmation);
- confirmation that the drill created no ADR file and resolved nothing; and
- open items for W21 (escalation path ready for its admission rules), W05
  (ADR-class detail now defined here), the threshold-section owners
  (W02-style references now resolvable), and every later stage (label
  semantics usable) — without resolving their contracts here.

## 6. Open questions and ADR-required items

- **Open question (owner confirmation):** the narrow-edit rule's clause (b) —
  permitting status-line pointer annotations on adr-000 register rows after a
  standalone ADR decides a 待定 item. This design applies it because it
  operationalizes adr-000's own supersession rule without touching decision
  text; if the owner declines, register rows remain untouched and standalone
  ADRs alone carry such decisions. Not an `ADR Required` item: no accepted
  decision is changed by either outcome.
- **`ADR Required` items raised by this design:** none. Every choice above is
  stage-local governance within the plan's scope; nothing here alters an
  accepted decision or an architectural constraint.
