# P0-W06 ADR Governance — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The ADR lifecycle states, ADR-threshold classification,
conflict/supersession/history/review process, ADR template, and traceability
drill required by [P0-W06](../../plans/p0-w06-adr-governance.md).  
**Owner/change context:** P0-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W06. It converts the bounded
work-package plan into an expanded normative ADR governance document at
`docs/adr/README.md`, one ADR template under `docs/templates/`, discovery
wiring, and a traceability drill executed against a clearly hypothetical
conflicting proposal. It deliberately does **not** modify the content of the
accepted ADR baseline (`adr-000`) or any decision it records, resolve any
pending register item (ADR-054 through ADR-058 and the other 待定 entries),
adjudicate any real architecture conflict, or define the general document
taxonomy (W05) or cross-layer workflow (W21).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, ADR baseline, P0 task book, and P0-W06 plan.
This document is the proposed detailed design for those changes; it is not a
completion record and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W06 plan → this design
→ Coding Guidelines. In particular:

- The ADR baseline's own change rule is the semantic anchor: accepted
  decisions are overturned only by a new ADR marked `Supersedes ADR-xxx`,
  never by silent edits; development stages may be re-split but exit criteria
  stay verifiable. W06 operationalizes this rule; it does not amend it.
- `AGENTS.md` and `docs/README.md` already mandate the labels and the
  behavior W06 makes precise: an accepted ADR is never edited to hide an
  architectural change; conflicts are recorded as `ADR Required` or
  `Architecture Change Request` issues.
- The task book outcome for W06 is: ADR lifecycle and architecture-change
  handling are **enforceable project rules** (P0-V10), and a reviewer can
  follow the lifecycle and supersession path from repository documents alone.
- The task book's P0 task list includes "freeze the v0.1 ADR register and its
  change process" and "establish the ADR template"; the template is therefore
  an explicit P0 deliverable that this package owns.
- The plan's out-of-scope list bounds the work: no existing accepted ADR is
  modified, and no new architecture choice is resolved in this package.

Classification: the lifecycle states, threshold rules, conflict procedure,
history-preservation rules, template, and drill are **Required** for W06
closure. Registration mechanics for future ADRs beyond the numbering/index
rules (for example an ADR automation or lint) are **Reserved** with triggers.
Modifying accepted ADR content, resolving pending register items, defining
W05's taxonomy or W21's workflow, and all code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Full ADR state set with transition meanings (accepted, rejected, deferred, superseded) | [ADR lifecycle contract](01-adr-lifecycle-contract.md) §3 | P0-V10 (W06-DV01) |
| Which changes require an ADR vs a local implementation choice | [ADR lifecycle contract](01-adr-lifecycle-contract.md) §4–§5 | P0-V10 (W06-DV02) |
| Conflict, supersession, history-preservation, and review-record process | [ADR lifecycle contract](01-adr-lifecycle-contract.md) §6–§7 | P0-V10 (W06-DV03, DV05) |
| Traceability drill with a hypothetical conflicting proposal | [ADR lifecycle contract](01-adr-lifecycle-contract.md) §8, [workflow](02-implementation-and-review.md) step 4 | P0-V10 (W06-DV03) |
| ADR template for recurring use | [ADR lifecycle contract](01-adr-lifecycle-contract.md) §9 | P0-V09/P0-V10 (W06-DV04) |
| Discoverability and downstream consumability | [workflow](02-implementation-and-review.md) steps 3, 5 | P0-V09/P0-V10 (W06-DV06, DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at merge 4e631ee): `docs/adr/README.md` is a
five-line note (accepted decisions are normative; add a new ADR to propose,
supersede, defer, or reject; never rewrite an accepted decision). The ADR
baseline `adr-000-architecture-baseline-v0.1.md` is one register-style
document containing decisions ADR-001 through ADR-060 with internal register
statuses (已确定/long-term reserved/待定/否决), principles, architecture,
stage tree, invariants, and its own tail change rule. `docs/templates/README.md`
requires approved templates before recurring document types and contains no
templates. `docs/README.md` routes architecture-affecting work to the ADR
baseline and mandates the `Supersedes` header field. No tracked document
defines the ADR state set, the ADR-threshold test, or the operational
supersession/history procedure. W01 is completed; W05 (this package's
prerequisite for document conventions) is a proposed design. Each ledger row
below states the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| ADR states and transitions defined (P0-V10) | Only implicit in `docs/adr/README.md`'s five lines and adr-000's register vocabulary | Lifecycle section: Proposed, Accepted, Rejected, Deferred, Superseded, with transition rules and normativity per state | "Enforceable rules" need a stated state machine a reviewer can apply | W06 within adr-000's change rule | W06-DV01 |
| ADR-threshold test defined | Thresholds exist as scattered instances (AGENTS.md, plan/coding guides, W02 contract §3.6, W04 §8) with no consolidated decision test | ADR-threshold section: trigger list for standalone ADRs plus the local-choice boundary, referencing the instances | An agent must be able to decide "does this need an ADR?" without reading the whole tree | W06 consolidation; instances stay authoritative in their homes | W06-DV02 |
| Conflict/supersession/history/review process | adr-000's tail rule and `docs/README.md` one-liner only | Operational procedure: conflict labels, draft mechanics, review record, history-preservation rule, register interaction | The supersession path must be followable from documents alone (P0-V10) | W06 | W06-DV03/DV05 |
| ADR template exists | `docs/templates/` is empty by its own rule; adr-000's header shows the field style informally | `docs/templates/adr-template.md` with the required fields | The task book names the ADR template as a P0 deliverable; the template makes the lifecycle usable | W06; header fields per W05 conventions | W06-DV04 |
| Traceability drill | Not exercised anywhere | Worked hypothetical example plus drill evidence | The plan requires a drill against a hypothetical conflicting ADR | W06 | W06-DV03 |
| Every later plan/stage can use the mechanism | Labels are used by AGENTS.md/plans but have no operational definition | Label semantics for `ADR Required` and `Architecture Change Request` | Consumers already cite the labels; the definitions must exist | W06 | W06-DV07 |

Prerequisite boundary: W05 is a proposed design, not a delivery. Its status-
header and versioning conventions are consumed by W06's new documents.
Fallback: the header field set W05 elaborates is already mandated by
`docs/README.md` in the current tree, so W06 can implement against the
existing mandate; if W05 has delivered different conventions when W06
implements, W06 follows the delivered W05 rules. A substantive conflict
between delivered W05 rules and this design is raised, not absorbed.

## Resolved design decisions and their authority

1. **Normative home.** `docs/adr/README.md` is expanded into the normative
   ADR lifecycle-and-governance document (states, thresholds, conflict
   procedure, history rules, and the ADR index table). The directory entry is
   the natural single home: it is already routed by `docs/README.md`, already
   cited by the W02 contract §3.6, and keeping process with the decisions
   avoids a third location. `adr-000` remains the sole authority for the
   decisions it records.
2. **State set.** Proposed, Accepted, Rejected, Deferred, Superseded (§3).
   Only Accepted (and Proposed, for what it may become) is forward-working;
   Rejected and Deferred records are kept to prevent relitigation without new
   information; Superseded text is preserved unmodified behind a status-line
   pointer.
3. **Post-acceptance edit rule (narrow).** After acceptance, the only
   permitted edits to an ADR file are: (a) its status/supersession header
   line, and (b) for adr-000's register, a status-line annotation that adds a
   pointer to the deciding standalone ADR without altering any decision text.
   Everything else — including errata — goes through a superseding or
   amending ADR. Rationale: this mirrors the narrowly-allowed metadata edit
   class the baseline itself implies, keeps the register usable as an index,
   and still makes every semantic change traceable through a new document.
   This interpretation is flagged to the owner for confirmation (open
   question in the workflow file); if the owner rejects (b), register rows
   stay untouched and standalone ADRs alone carry the decisions.
4. **Label semantics.** `ADR Required` = work is blocked because a required
   choice conflicts with, or falls outside, an accepted decision or a
   declared ADR threshold; the blocking issue must be recorded and an ADR
   drafted before the work proceeds. `Architecture Change Request` = a
   deliberate proposal to change accepted architecture, raised even when
   nothing is yet blocked. Both route into the same draft-and-review
   mechanism (§6); the labels differ in who raises them and when, not in the
   path.
5. **Register interaction.** adr-000's internal register statuses
   (已确定/长期预留/待定/否决) are adr-000's own vocabulary; W06 does not
   redefine them. The governance document aligns them informatively
   (approximately: accepted / long-term reserved / decision pending /
   rejected) and states the operational rule: deciding a 待定 register item
   is done by a new standalone ADR; the register row then receives the
   pointer annotation per decision 3. Register items are never decided by
   editing their decision text.
6. **Numbering and index.** Standalone ADRs take the next unused number after
   the highest registered ID (currently ADR-060 → next standalone is ADR-061),
   with file names following the `adr-NNN-<slug>-v<major>.<minor>.md` pattern
   of the baseline. `docs/adr/README.md` carries the index table (ID, title,
   state, supersedes) that every new ADR must be added to in the same change.
7. **ADR template.** `docs/templates/adr-template.md` fixes the required
   fields (ID, title, date, state, scope, supersedes/superseded-by, context,
   decision, consequences, alternatives considered, change history) consistent
   with the baseline's header style and the W05 header conventions. The
   template is the enforcement artifact of the lifecycle; an ADR PR missing
   required sections fails review.
8. **Drill discipline.** The traceability drill uses a hypothetical proposal
   (Core code branching on board name, conflicting with the accepted
   board-independence decisions), presented in the governance document as a
   clearly labeled informative worked example and executed for evidence in
   the verification record. The drill creates no real ADR, resolves nothing,
   and must never be mistaken for a pending architectural proposal.

## Work breakdown and loading order

1. Read [the ADR lifecycle contract](01-adr-lifecycle-contract.md) for the
   artifact groups, the governance document's required sections, the template
   contract, and the drill specification.
2. Apply the changes in the order stated in the
   [implementation workflow](02-implementation-and-review.md): expand
   `docs/adr/README.md`, create the template, wire discovery, run the
   traceability drill, then close.
3. Store actual commands, output, environment, and result in
   `../../verification/p0-w06-adr-governance-verification.md`, and record
   changed artifacts, the owner-confirmation status of the narrow-edit rule,
   and any deviation in `../p0-w06-adr-governance-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W06 complete.

## Design-level state and lifecycle

W06 adds no runtime state, registry, lock, allocation, or code path. Its
subject is itself a lifecycle — the ADR document state machine, which the
governance document fixes and every later change follows:

```text
 Proposed --review--> Accepted --superseding ADR--> Superseded (text preserved)
    |                     |
    |--review--> Rejected  |--owner decision--> Deferred (revisit trigger recorded)
    |                     |
    +--withdrawn----------+ (author withdraws before review: state Proposed,
                            file kept or removed per author's choice, logged)
```

The governance document owns the state definitions and transitions; each ADR
file owns its own state field; the index in `docs/adr/README.md` owns the
registry view. A conflict between an ADR's stated state and the index is a
review failure. The documentary lifecycle of W06's own artifacts:

```text
five-line adr/README note + no template
  -> adr/README.md expanded (states, thresholds, procedure, index)
  -> docs/templates/adr-template.md committed
  -> docs/README.md routing row added
  -> traceability drill executed and evidenced
  -> every later plan/stage raises conflicts through this mechanism
```

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, or build
surface is designed or authorized by W06. Additionally excluded: any edit to
adr-000's decision text or principles (beyond the narrow rule of decision 3),
any resolution of a 待定 register item, any real architecture proposal, W05's
taxonomy, W21's workflow, and code or CI enforcement of the lifecycle
(W20 consumes the rules mechanically later if a check is designed). Enforcing
the lifecycle by tooling under W06 is a scope conflict to be raised at review.

## Downstream handoff

- **Every later plan and stage** receives the operational mechanism: the
  definitions of `ADR Required` and `Architecture Change Request`, the state
  machine, the template, and the supersession path. A P1 planner who
  discovers a conflict with an accepted decision uses this path instead of
  local redesign.
- **W21** receives the fixed escalation path its workflow layer references
  (when a plan-to-code transition needs an approved design versus an
  architecture change); W21 defines the admission rules, W06 defined the
  escalation mechanism they cite.
- **W05** receives the ADR lifecycle as the class-specific detail its
  taxonomy reserved; the W05 baseline points here for ADR supersession
  mechanics.
- **W10, W11, W14, W15, W18** receive the pattern for their own
  policy-threshold sections (routine / policy decision / ADR required), which
  reference `docs/adr/README.md` by pointer exactly as the W02 contract
  already does.
- **W20** receives the reviewable rules its future documentation checks could
  verify mechanically (index completeness, status-line consistency); any such
  check is W20's to design.
- **adr-000 pending items (ADR-054 through ADR-058, and the §18 open
  questions)** receive their resolution path: a standalone ADR per decision,
  never a register edit. W06 resolves none of them.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
