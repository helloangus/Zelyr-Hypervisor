# P0-W21 Admission and Traceability Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W21 detailed design](README.md).

## 1. Authority position of this contract

Every rule below consolidates instances that already exist in governing
documents — the task book §3 delivery hierarchy, the Coding Guide's preflight
and authority order, the Plan Agent guide's authority order, `AGENTS.md`
scope rules, `docs/README.md` same-change rule, and the
[integration workflow](../../../../development/integration-workflow.md) — and
fixes them into one decision surface. Where a consolidated rule and its
instance ever diverge, the instance's owning document governs and this
workflow document is corrected. The contract creates no new authority; its
value is that an agent can answer "may I proceed, and what must I hold?" from
one table.

## 2. Admission decision table (normative content of the workflow document)

The workflow document must contain this table as its plan-to-code admission
rule. "Holds" means the named artifact exists in its authoritative location
with a status that permits the activity, and the actor has read it per the
routing rules.

| Activity | Must hold before starting | Escalation when blocked |
|---|---|---|
| Code-bearing change in a work package | the approved plan; an approved detailed design for the change; the Coding Guide preflight completed; the plan index's prerequisite contracts delivered or their absence recorded as a blocker | a plan that defers a needed decision to "the owning design" with no design in that location is a blocker to record, never a license to design while coding |
| Governance/policy change inside a P0-style package (documents, contracts, configuration) | the approved plan; the design-level artifact contract the plan defers decisions to (the established P0 pattern), or an explicit statement in the plan that the plan itself is the contract | a normative rule change with no owning document is a policy decision per [W05](../p0-w05-documentation-baseline/README.md) thresholds, not a local edit |
| Task-book or plan authoring/change | the Plan Agent guide; the ADR baseline; the applicable task book; the stage-work-package-planning skill | task books and plans are never written from implementation notes; the hierarchy is one-way downward for content and upward for escalation |
| Architecture proposal or accepted-decision change | a draft ADR per the [W06](../p0-w06-adr-governance/README.md) lifecycle and template | never executed as a side effect of another package; a superseding ADR is its own reviewed change |
| Conflict with an accepted decision or declared threshold discovered during work | label the issue `ADR Required` (work is blocked) or `Architecture Change Request` (deliberate proposal) with the W06 definitions and record it | the affected work stops at the boundary; no local redesign, no silent reinterpretation |
| Merge and integration | the [integration workflow](../../../../development/integration-workflow.md) policy; from W20's delivery onward, the configured required checks | merge is never granted by a layer below L7's stage completion; policy-bound bypasses are recorded, never routine (per the W20 design) |
| Stage completion claim | every package's verification evidence per the task-book validation matrix; the exit criteria reviewed | a missing evidence item fails completion; it is never waived by process text (the exact failure this package exists to prevent) |

Two corollaries the workflow document must state with the table:

- **A plan alone never authorizes code.** The plans/README reading order
  ("Coding Guide plus an approved detailed design for code changes") and the
  task book §3 rule (no code from a task-book bullet alone) are restated here
  by reference as the first row's basis.
- **Process text never substitutes for design.** Adding admission prose to a
  process document in order to start coding without a design is a review
  failure, not a shortcut.

## 3. Traceability and evidence-placement rules (normative content)

The workflow document must fix these cross-reference rules. The
P\<stage\>-Wxx work-package ID and the P\<stage\>-Vxx validation ID are the
traceability keys.

1. **Design ↔ plan:** every detailed design's header names its parent plan;
   every requirement the design maps carries the plan wording it implements
   (the requirement → design-location → acceptance table is the established
   P0 pattern and is required for code-bearing and governance designs alike).
2. **Implementation record ↔ design ↔ plan:** the implementation record names
   the design directory it implements, the plan it satisfies, the changed
   artifacts, decisions taken, and deviations; it links to its verification
   record. It carries no command logs and no completion claims.
3. **Verification record ↔ validation matrix:** the verification record maps
   each of its entries to the design's validation-matrix IDs and the
   task-book P\<stage\>-Vxx IDs, states run/failed/blocked/not-run with
   command, environment, timestamp, and reason, and states proves /
   does-not-prove per entry. Completion claims exist only here.
4. **Same-change rule:** documentation that a change invalidates is updated
   in the same change (`docs/README.md` mandate, cited); a contract lagging
   its implementation is a defect, not a TODO.
5. **Placement rule:** decisions live in implementation records; evidence
   lives in verification records; neither migrates into designs, task books,
   plans, or process documents (the W05 §7 separation, cited not restated).
6. **Handoff rule:** a stage's handoff package (L7 output) is assembled by
   linking layer outputs — records, evidence, and governing contracts — never
   by copying their content; [W22](../p0-w22-stage-dependency-map/README.md)
   applies this rule to P0's package specifically.
7. **Failure attribution rule:** every blocked or failed activity names its
   owning layer and package, so a failure never strands an agent without an
   owner (the W19 per-stage pattern, generalized).

## 4. Explicitly excluded content

No lifecycle states or label definitions (W06 owns them; the table's
escalation rows cite them), no directory rules (W05), no merge-policy text
(the policy's owning change), no per-stage reading orders beyond citing the
plan indexes, and no template bodies (owning packages deliver templates per
the `docs/templates/` gating rule). If a table row seems to require new
policy, the row cites the existing authority instead — or the gap is raised
as a policy decision, never absorbed by silent rewording.
