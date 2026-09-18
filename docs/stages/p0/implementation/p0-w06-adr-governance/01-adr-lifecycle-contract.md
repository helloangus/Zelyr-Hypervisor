# P0-W06 ADR Lifecycle Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W06 detailed design](README.md).

## 1. Logical artifact groups and ownership

W06 is governance-documentation work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| ADR governance document | `docs/adr/README.md` (expanded) | adr-000's change rule, `AGENTS.md`/`docs/README.md` mandates, this design | the sole normative home of ADR states, transitions, thresholds, conflict procedure, history rules, label semantics, numbering/index rules, and the worked example; it does not restate W05's taxonomy, W21's workflow, or any accepted decision |
| ADR template | `docs/templates/adr-template.md` | governance document, baseline header style | the required field set for every future ADR; it is not itself an ADR and asserts no decision |
| Documentation routing | one row in `docs/README.md` routing table | governance document location | discoverability of ADR-process guidance; it does not restate process |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w06-adr-governance-record.md` (created when work starts) | actual decisions taken | changed artifacts, owner-confirmation status of the narrow-edit rule, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w06-adr-governance-verification.md` (created when evidence exists) | actual drill and review evidence | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it. `adr-000` remains the sole authority for the decisions it
records; the governance document governs the *process*, never the *content*.

## 2. Prerequisite contract consumed from W05

The governance document and template must carry the status-header and
versioning conventions that W05's documentation baseline defines. Assumption:
W05 delivers (or `docs/README.md`'s existing mandate already supplies) the
fields Status, Scope, Version, Owner/change context, Supersedes. Failure
boundary: `docs/README.md` in the current tree already mandates exactly these
fields, so W06 can implement against it directly; if delivered W05 rules
differ, W06 follows the delivered rules and raises any substantive conflict —
it does not fork a private convention.

## 3. ADR states and transitions (required governance-document content)

The governance document must define exactly these states, their meaning, and
their normativity:

| State | Meaning | Normativity |
|---|---|---|
| Proposed | an open architecture proposal under review | not binding; may be changed freely by its authors |
| Accepted | a decision the project follows | normative; binds designs, code, and reviews |
| Rejected | a considered-and-declined proposal, with rationale | not binding on behavior; binding as precedent — revisiting requires a new Proposed ADR citing the new information |
| Deferred | a real need acknowledged but postponed, with a recorded revisit trigger | not binding; the trigger names what reopens it |
| Superseded | replaced by a named successor; historical text preserved unmodified | no longer binding; the successor is |

Transition rules the document must state:

- Proposed → Accepted or Rejected by the review recorded with the ADR
  (decision, date, reviewer/owner identity, and the PR that carried it per the
  integration workflow).
- Accepted → Superseded only by a later ADR that names this one in its
  Supersedes field; the superseded file's status line gains the successor
  pointer. No other edit.
- Accepted → Deferred is not a transition: a deferred item was never accepted.
  An accepted decision whose enforcement must pause is a defect to record, not
  a state change.
- Deferred → Proposed or Rejected when its revisit trigger fires or the owner
  closes it.
- Withdrawal before review is logged in the Proposed file's change history
  (or the file is removed if it was never merged); withdrawal after merge of
  a Proposed state follows the same route as Rejected.

## 4. What requires an ADR (required governance-document content)

The governance document must provide a decision test, consolidated by
reference to the scattered instances (each instance stays authoritative in its
own home). An ADR is required when the change would:

1. conflict with, override, or narrow any Accepted decision or Implementation
   Invariant of the ADR baseline (rare exceptions codified in the baseline's
   architecture-change rule);
2. decide a register item marked 待定 (ADR-054 through ADR-058 and the §18
   open questions) with architectural effect;
3. cross a threshold another baseline marks as ADR-required — by reference,
   not restatement: the
   [W02 toolchain contract](../p0-w02-rust-toolchain-baseline/01-toolchain-contract.md)
   §3.6 (unstable features load-bearing for the TCB, channel/pinning-mechanism
   changes), the
   [W03 target thresholds](../p0-w03-aarch64-build-target-baseline/01-target-baseline-contract.md)
   §8, and the
   [W04 build-choice thresholds](../p0-w04-build-profile-feature-governance/01-governance-contract.md)
   §8 as approved (all currently proposed designs);
4. introduce a new architectural principle, layer boundary, object model
   element, or security/trust-boundary decision not derivable from the
   baseline;
5. change the architecture-change process itself.

And is *not* required — a local implementation choice — when the change stays
inside an approved detailed design and frozen contracts: ordinary module or
function design within stated boundaries, routine version bumps, target or
dependency additions following their governance, documentation maintenance
under the W05 rules, and coding-level decisions owned by the Coding
Guidelines. The document must state the tie-breaker: when unsure, raise the
question as `ADR Required` and let the owner decide; an unnecessary ADR costs
a document, a missed ADR costs the architecture.

## 5. Label semantics (required governance-document content)

- **`ADR Required`** — recorded when work is *blocked*: a required choice
  conflicts with an accepted decision, falls into §4.1–§4.5, or breaches a
  referenced threshold. The blocking issue is recorded where the work tracks
  it (implementation record, PR, or issue), the affected decision is cited by
  ID, and the affected work stops until the ADR path completes.
- **`Architecture Change Request`** — recorded when a contributor *proposes*
  changing accepted architecture, whether or not anything is yet blocked.
  Same destination: a Proposed ADR carrying the request.
- Both labels produce the same artifact — a Proposed ADR and its review —
  and the governance document must say so, so the two names never fork into
  two processes.

## 6. Conflict, supersession, and review-record process (required content)

The governance document must fix the operational path:

```text
conflict found (or change proposed)
  -> record the label and the cited decision ID(s) in the affected work's
     tracking surface; stop the affected work if blocked
  -> draft a Proposed ADR from docs/templates/adr-template.md
       - number: next unused after the highest registered ID (ADR-061 next)
       - Supersedes: named decision(s), if any
       - context must cite the accepted text it interacts with
  -> review: decision + rationale + reviewer/owner + carrying PR recorded
     in the ADR's change history; merge only per the integration workflow
  -> on acceptance: update the index row and each superseded file's status
     line (the only permitted edit to a superseded file)
  -> on rejection: state the rationale; the record stands as precedent
```

History preservation rules: accepted and superseded ADR text is never
rewritten; the only post-acceptance edits are the status/supersession header
line and — for adr-000's register only — a status-line pointer annotation to
the deciding standalone ADR, without altering decision text. Everything else,
including errata, goes through a superseding or amending ADR. (This narrow
edit class is raised to the owner for confirmation; see the workflow file's
open-questions entry.)

## 7. Register interaction (required governance-document content)

The governance document must state, informatively: adr-000's internal register
statuses are that document's own vocabulary and are not redefined here
(approximate alignment: 已确定 ≈ Accepted; 长期预留 ≈ long-term reserved
intent; 待定 ≈ decision pending; 否决 ≈ Rejected). Operationally: a 待定
register item is decided by a standalone ADR via §6; the register row then
receives the pointer annotation per §6's narrow-edit rule. Register items are
never decided, reworded, or "cleaned up" by direct edits. The baseline ADR
itself is superseded as a whole, if ever, by a successor baseline ADR — not by
distributed edits.

## 8. Traceability drill specification (required content)

The governance document must contain, as a clearly labeled informative
appendix, one worked hypothetical example:

- *Hypothetical proposal:* allow hypervisor Core code to branch on board
  names to shorten an interrupt path on one specific board.
- *Conflict:* directly contradicts the accepted board-independence decisions
  (the register's board-name prohibitions, ADR-052/ADR-043) and the Core
  invariants.
- *Correct path:* record `Architecture Change Request` (nothing is blocked —
  it is a proposal; `ADR Required` would apply if a task needed the change to
  proceed), draft a Proposed ADR superseding ADR-052/ADR-043 with the
  performance rationale and alternatives, route through review; until
  acceptance, Core stays board-name-free.
- *Lesson:* the reviewer's test is §4's decision test; the path is §6's.

The appendix must state in bold that the example is hypothetical, resolves
nothing, and creates no ADR. The drill itself is executed for evidence per
the workflow file's step 4.

## 9. ADR template contract

Create `docs/templates/adr-template.md` with the required field set:

```text
# ADR-NNN — <title>
Date / State (one of §3's states, Proposed initially)
Supersedes: <ADR-IDs or none>    Superseded-by: <filled only via status line>
Scope: <what the decision governs and what it deliberately does not>
Context: <forces at play, constraints, cited accepted decisions affected>
Decision: <the choice, stated normatively>
Consequences: <binding effects, new constraints, follow-up work>
Alternatives considered: <options rejected and why>
Change history: <append-only; review decision, date, owner, carrying PR>
```

Field rules: every field is required for review (an ADR PR missing one fails
review); the change-history section is append-only and is where the §6 review
record lives; the template carries the W05/W05-fallback header conventions
and no project-specific decision. `docs/templates/README.md` needs no edit:
it already asks for templates to be added here before the recurring type is
created, and this template satisfies that rule for ADRs.

## 10. Explicitly excluded interfaces

There are no Rust types, functions, traits, modules, crates, APIs, build
artifacts, or tooling behaviors in this design, and no CI enforcement of the
lifecycle. Also excluded: any change to adr-000's decision text or principles
beyond §6's narrow-edit rule, any resolution of a 待定 register item, any
real architecture proposal (the drill is hypothetical), W05's taxonomy, and
W21's workflow. Enforcing the lifecycle with lint tooling under W06 is a
scope conflict to be raised at review.
