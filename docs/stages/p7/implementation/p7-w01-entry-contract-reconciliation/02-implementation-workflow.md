# P7-W01 Implementation Workflow, Validation, and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm which upstream implementation and
verification records actually exist — today only the P0 repository-baseline
set does) and a search of tracked documents for any existing P7 input-boundary
statement.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a cited upstream plan path does not resolve, or a plan's handoff wording
  contradicts the task book §2 predecessor table — raise it per the §5
  procedure of [the register contract](01-input-boundary-register.md); do not
  rewrite the row to match the deviation;
- producing the register appears to require editing `docs/README.md`, the
  stage index, a task book, a plan, or any upstream document — that is out of
  W01 scope; stop and record the blocker;
- a reviewer or maintainer requests adding an input class outside the task
  book §2 table — that is a task-book change, not a register edit; or
- two upstream contracts conflict with each other — record the conflict as a
  blocked row; W01 does not arbitrate between predecessors.

## 2. Ordered implementation steps

### Step 1 — establish the register skeleton

Target: `../p7-w01-entry-contract-reconciliation-record.md` (created in this
step).

Work: create the W01 implementation record with the seven-column register
schema of [the register contract](01-input-boundary-register.md) §2 and empty
`P7-IN-01…09` rows. Record the audit date, branch, and the `git ls-files`
observation set.

Suggested observation: `git -C <checkout> ls-files docs/stages | sort` to
confirm which upstream implementation/verification records exist.

**Acceptance:** the record exists with the exact schema and nine empty rows;
no upstream document was modified.  
**Failure/blocker:** schema deviation fails review; the schema changes only
through a new design decision, not at implementation time.

### Step 2 — populate the input rows

Target: the register rows P7-IN-01…09.

Work: for each row in [the inventory](01-input-boundary-register.md) §3,
verify the cited source paths resolve in the checkout, write the row's
`Source authority`, `P7 consumption`, `Failure boundary`, and
`Primary P7 consumers` columns, and set the initial evidence status to
`contract-mapped (evidence pending)` unless a real implementation/verification
record exists and matches — in which case link it and mark
`available (evidence linked)`.

**Acceptance:** every row complete; every cited path resolvable from a fresh
checkout; every status justified by a linked record or the default.  
**Failure/blocker:** an unresolvable path or an unverifiable `available`
status is a recorded blocker; statuses are never set optimistically.

### Step 3 — reconcile and mark conflicts

Target: the register rows; the issue record (only if a conflict exists).

Work: for each row, check the stated P7 consumption against the task book §2
predecessor table and the ADR constraints (layering, no board-name policy,
capability authority, untrusted-guest rules). Where a row cannot be stated
without contradicting an authority, or where the plan-level wording of two
sources conflicts, stop that row and run the §5 procedure: mark
`blocked (issue linked)` and write the issue with the correct `Architecture
Change Request` / `ADR Required` label or the blocked-prerequisite
classification.

**Acceptance:** every row is either consistent with the authorities or
explicitly blocked with a linked issue; no row is silently reworded to hide a
conflict.  
**Failure/blocker:** discovering a true conflict does not fail W01 — hiding
one does; W01's outcome is the explicit record.

### Step 4 — write the entry-boundary section

Target: register §E (inside the W01 implementation record).

Work: transfer the boundary statement and the E1–E3 bypass classes from
[the register contract](01-input-boundary-register.md) §6 verbatim in
substance, including the P7-IN-05 compatibility finding for P4's direct-entry
path.

**Acceptance:** the section states the boundary, the three classes, the
enumerability rule, and the W01/W02 ownership split.  
**Failure/blocker:** an instance-level enumeration here would be W02 scope
creep; stop and defer it to
[P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md).

### Step 5 — boundary review against ADR constraints

Target: review evidence in
`../../verification/p7-w01-entry-contract-reconciliation-verification.md`
(created when the review runs).

Work: a reviewer (not the author) reads the register end to end and checks:
locatability of every citation; consistency of every consumption statement
with the task book §2 boundary column; conformance with ADR-002/004/015/016
assumptions; and that every blocked row names its affected packages.

**Acceptance:** the review record states, per P7-V01, that each input can be
located with its P7 use, and that absent or contradictory contracts are
explicitly blocked.  
**Failure/blocker:** a failed finding reopens the relevant step; the review
record keeps the failed attempt and its resolution.

### Step 6 — closure review and handoff

Work: run the validation matrix below, confirm the handoff checklist, and
verify W01 against its task-book requirement and consumer map. Completion is
claimed only in the verification record, with evidence, and only for what was
actually reviewed.

## 3. Validation matrix

All rows are planned evidence; none is claimed to have run.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W01-DV01 → P7-V01 | register schema review | inspect the record's register section against [the register contract](01-input-boundary-register.md) §2 | seven columns exactly; nine rows; ids stable; no prose rows | a reviewable artifact exists; not that the inputs are implemented or true |
| W01-DV02 → P7-V01 | locatability review | resolve every cited path from a fresh checkout | every `Source authority` path resolves; every `available` status links a dated record | citations are locatable; not that cited contracts are correct |
| W01-DV03 → P7-V01 | consistency review | compare each row's consumption statement with the task book §2 boundary column and the ADR baseline | no row contradicts an authority; each row's failure boundary names affected packages | the reconciled boundary is coherent; not that predecessors will honor it |
| W01-DV04 → P7-V01 | entry-boundary review | read register §E against [the register contract](01-input-boundary-register.md) §6 | boundary statement, E1–E3 classes, enumerability rule, and W01/W02 split present; no instance list (W02 scope) | the boundary is reviewable; not that any gate enforces it yet |
| W01-DV05 → P7-V01 | blocked-handling procedure review | walk the §5 procedure against at least one real row status (or, if none is blocked, against the P1–P6 evidence-pending default) | procedure steps are executable as written; labels used correctly | the procedure is actionable; not that upstream issues are resolved |
| W01-DV06 → W01 closure | consumer consumability review | read the register as W02 (can I enumerate bypass instances?), W03/W04/W05 (is my input row sufficient?), W14 (can I extract limitations?) | each consumer can act citing `P7-IN-xx` ids without restating contracts | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with input, reviewer, environment, timestamp, and reason. A written register
without the W01-DV02/DV03 review does not satisfy P7-V01. No validation here
proves P7-V02 through P7-V30 and none may be reported as doing so.

## 4. Error, security, and observability model

W01 adds no hypervisor error model, synchronization, guest-input handling, or
`unsafe`. Its failure reporting is textual: a schema deviation, an
unresolvable citation, an inconsistent row, or a mishandled conflict fails
the associated review and is recorded as such.

The security surface is authority integrity: the register is the choke point
that prevents P7 packages from silently widening or reinterpreting what
predecessors promised. The §5 procedure and the label discipline
(`Architecture Change Request` / `ADR Required`) are the controls; a
conflict-absorbing rewording is the failure mode the review hunts for.

Observability is the evidence trail: the register's status column plus the
verification record's run/not-run entries are the only accepted proof
surface. The stage implementation index row (coordinator-owned) and W14's
handoff consume these; W01 itself claims nothing.

## 5. Handoff checklist

Before handing W01 to a reviewer, provide:

- the exact changed-file list (expected: only the W01 implementation record
  and the verification record, once created);
- the register's final row-status summary, including every `blocked` row with
  its issue link and affected packages;
- W01-DV01…DV06 evidence paths and run status, including explicit not-run
  entries;
- confirmation that no upstream document, task book, plan, stage index, code
  file, or CI configuration was modified;
- confirmation that the entry-boundary section contains classes only, with
  instance enumeration explicitly deferred to P7-W02; and
- open items for consumers: W02 (gate design must enumerate instances against
  E1–E3), W03–W05 (consume their rows' failure boundaries), W09/W12/W13
  (respect evidence-status gates), W14 (carry limitation summary into the P8
  handoff) — without resolving their contracts here.
