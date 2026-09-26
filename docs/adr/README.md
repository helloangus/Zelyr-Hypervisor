# ADRs — architecture decision records and lifecycle

**Status:** Normative ADR governance (states, transitions, thresholds,
conflict procedure, history rules, numbering/index).  
**Version:** v0.1  
**Owner/change context:** P0-W06 ADR governance; process changes go through
review, and changes to the architecture-change process itself require an ADR
(§2.5).  
**Supersedes:** The absence of an explicit written ADR process (the previous
five-line index note, whose statements this document preserves).

Accepted decisions are normative.  Add a new ADR to propose, supersede, defer,
or reject an architectural change; never rewrite an accepted decision to make
an implementation appear compliant.

## 1. ADR states and normativity

| State | Meaning | Normativity |
|---|---|---|
| Proposed | an open architecture proposal under review | not binding; may be changed freely by its authors |
| Accepted | a decision the project follows | normative; binds designs, code, and reviews |
| Rejected | a considered-and-declined proposal, with rationale | not binding on behavior; binding as precedent — revisiting requires a new Proposed ADR citing the new information |
| Deferred | a real need acknowledged but postponed, with a recorded revisit trigger | not binding; the trigger names what reopens it |
| Superseded | replaced by a named successor; historical text preserved unmodified | no longer binding; the successor is |

Transitions:

- **Proposed → Accepted or Rejected** by the review recorded with the ADR
  (decision, date, reviewer/owner identity, and the PR that carried it, per
  the [integration workflow](../development/integration-workflow.md)).
- **Accepted → Superseded** only by a later ADR that names this one in its
  Supersedes field; the superseded file's status line gains the successor
  pointer. No other edit.
- **Accepted → Deferred is not a transition**: a deferred item was never
  accepted. An accepted decision whose enforcement must pause is a defect to
  record, not a state change.
- **Deferred → Proposed or Rejected** when its revisit trigger fires or the
  owner closes it.
- **Withdrawal** before review is logged in the Proposed file's change
  history (or the file is removed if it was never merged); withdrawal after
  merge of a Proposed state follows the same route as Rejected.

## 2. What requires an ADR

An ADR is **required** when the change would:

1. conflict with, override, or narrow any Accepted decision or Implementation
   Invariant of the [ADR baseline](adr-000-architecture-baseline-v0.1.md)
   (rare exceptions follow the baseline's architecture-change rule);
2. decide a register item marked 待定 (ADR-054 through ADR-058 and the §18
   open questions of the baseline) with architectural effect;
3. cross a threshold another baseline marks as ADR-required — by reference,
   not restatement: the [toolchain
   baseline](../development/toolchain-baseline.md) (unstable features
   load-bearing for the TCB, channel/pinning-mechanism changes), the
   [build-target baseline](../development/build-target-baseline.md)
   thresholds, and the [build-choice
   governance](../development/build-profile-governance.md) thresholds;
4. introduce a new architectural principle, layer boundary, object-model
   element, or security/trust-boundary decision not derivable from the
   baseline;
5. change the architecture-change process itself (this document's rules).

An ADR is **not required** — a local implementation choice — when the change
stays inside an approved detailed design and frozen contracts: ordinary
module or function design within stated boundaries, routine version bumps,
target or dependency additions following their governance, documentation
maintenance under the [documentation
baseline](../development/documentation-baseline.md), and coding-level
decisions owned by the Coding Guidelines.

**Tie-breaker:** when unsure, raise the question as `ADR Required` and let
the owner decide. An unnecessary ADR costs a document; a missed ADR costs the
architecture.

## 3. Label semantics

- **`ADR Required`** — recorded when work is *blocked*: a required choice
  conflicts with an accepted decision, falls into §2.1–§2.5, or breaches a
  referenced threshold. Record the blocking issue where the work tracks it
  (implementation record, PR, or issue), cite the affected decision by ID,
  and stop the affected work until the ADR path completes.
- **`Architecture Change Request`** — recorded when a contributor *proposes*
  changing accepted architecture, whether or not anything is yet blocked.

Both labels produce the same artifact — a Proposed ADR and its review — and
never fork into two processes.

## 4. Conflict, supersession, and review process

```text
conflict found (or change proposed)
  -> record the label and the cited decision ID(s) in the affected work's
     tracking surface; stop the affected work if blocked
  -> draft a Proposed ADR from docs/templates/adr-template.md
       - number: next unused after the highest registered ID
       - Supersedes: named decision(s), if any
       - context must cite the accepted text it interacts with
  -> review: decision + rationale + reviewer/owner + carrying PR recorded
     in the ADR's change history; merge only per the integration workflow
  -> on acceptance: update the index row (§6) and each superseded file's
     status line (the only permitted edit to a superseded file)
  -> on rejection: state the rationale; the record stands as precedent
```

History preservation rules:

- Accepted and superseded ADR text is never rewritten. The only
  post-acceptance edits are (a) the status/supersession header line and
  (b) — for adr-000's internal register only — a status-line pointer
  annotation to the deciding standalone ADR, without altering decision text.
  Everything else, including errata, goes through a superseding or amending
  ADR.
- The baseline ADR itself is superseded as a whole, if ever, by a successor
  baseline ADR — not by distributed edits.

## 5. Register interaction (informative)

adr-000's internal register statuses are that document's own vocabulary and
are not redefined here (approximate alignment: 已确定 ≈ Accepted; 长期预留 ≈
long-term reserved intent; 待定 ≈ decision pending; 否决 ≈ Rejected).
Operationally: a 待定 register item is decided by a standalone ADR via §4;
the register row then receives the pointer annotation per §4's narrow-edit
rule. Register items are never decided, reworded, or "cleaned up" by direct
edits.

## 6. Index

| ADR | Title | State |
|---|---|---|
| [ADR-000](adr-000-architecture-baseline-v0.1.md) | Architecture baseline v0.1 | Accepted |
| [ADR-061](adr-061-defer-p1-asynchronous-vector-validation-to-p6.md) | Defer executed EL2 asynchronous-vector validation from P1 to P6 | Accepted |

New standalone ADRs append a row here when they reach Accepted; a Proposed
ADR may carry an index row marked Proposed.

## Appendix: worked example (informative)

**This example is hypothetical. It resolves nothing and creates no ADR.**

- *Hypothetical proposal:* allow hypervisor Core code to branch on board
  names to shorten an interrupt path on one specific board.
- *Conflict:* directly contradicts the accepted board-independence decisions
  (register items ADR-052 and ADR-043: no board-name-driven Core logic) and
  the Core invariants of the baseline.
- *Decision test (§2):* lands in §2.1 — conflicts with Accepted decisions —
  so an ADR is required.
- *Label:* `Architecture Change Request` applies (a proposal; nothing is
  blocked). `ADR Required` would apply instead if, for example, a P1 task
  needed the change to proceed — then the task stops and records the blocking
  issue.
- *Correct path (§4):* draft a Proposed ADR with the next unused number superseding
  ADR-052/ADR-043, with the performance rationale, alternatives (platform
  capability query, BSP/Quirk placement), and context citing the accepted
  text; route through review; on acceptance, update the index row and the two
  superseded status lines — and nothing else in those files. Until
  acceptance, Core stays board-name-free.
- *Lesson:* the reviewer's test is §2's decision test; the path is §4's.
