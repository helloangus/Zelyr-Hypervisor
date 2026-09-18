# P7-W01 Entry Contract Reconciliation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewed P0–P6 input boundary, the blocked-prerequisite handling
procedure, and the normal scheduler-controlled Guest-entry boundary required by
[P7-W01](../../plans/p7-w01-entry-contract-reconciliation.md).  
**Owner/change context:** P7-W01 implementation handoff; stage-local design
freedom is recorded in the numbered decisions below.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W01. It converts the bounded
work-package plan into one authoritative documentation artifact — the **P7
input boundary register** — plus the review workflow that produces and closes
it. It deliberately does not design scheduler mechanics, lifecycle semantics,
placement, preemption, or M:N operation; those belong to P7-W02 through
P7-W05 and their sibling designs. It also does not repair, re-plan, or
re-verify any predecessor stage.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). W01 is
documentation and review work: no code interface is authorized, so after this
entry file the agent loads only the supporting file named for its assigned
step:

- To produce the register artifact itself (its schema, input inventory,
  evidence-status model, blocked-handling procedure, and the entry-boundary
  statement), load [the input boundary register](01-input-boundary-register.md).
- To execute the reconciliation and closure work, load
  [the implementation workflow](02-implementation-workflow.md), which also
  carries the validation matrix and handoff checklist.

Before editing anything, the agent must follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P7 task book, and
the P7-W01 plan). This document is a proposed design; it contains no
implementation or validation claim, and no record or verification file is
created by this design.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → frozen P0–P6 handoff
contracts → P7-W01 plan → this design. In particular:

- The task book §2 predecessor table fixes *which* inputs P7 requires; this
  design may organize and annotate them but may not add, drop, or narrow an
  input class.
- The task book §2 rule "Absent, contradictory, or unevidenced input blocks
  the affected package … do not repair upstream scope in P7" is binding: W01
  records blocks; it never repairs predecessors.
- ADR-002/004/015/016 constrain what the reconciled boundary may assume
  (AArch64-first, modular policy-independent core, early SMP, static-to-M:N
  evolution); the register's consistency review checks each input row against
  these constraints.
- The task book requires "Scheduler-controlled normal Guest entry and return
  after exit, with only bounded early-boot/bring-up/emergency-debug
  exceptions." W01 fixes the exception *classes*; the admission-gate mechanics
  that enforce them are W02 scope
  ([P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md)).

Classification:

- **Required:** the register artifact and its fixed schema; the input
  inventory P7-IN-01 through P7-IN-09 with per-input semantic consumption and
  failure boundary; the per-input evidence-status model; the
  blocked-prerequisite handling procedure; the entry-boundary statement and
  bounded exception classes; the closure review.
- **Reserved:** re-running the evidence-status reconciliation as P0–P6
  implementation and verification records land during P7 (the register is a
  living gate artifact; its update rules are fixed in
  [the register contract](01-input-boundary-register.md) §4). Carrying the
  register's limitation summary into the P8 handoff belongs to
  [P7-W14](../p7-w14-documentation-p8-handoff/README.md).
- **Out of Scope:** repairing or re-planning P0–P6; scheduler algorithms,
  queues, locking, APIs, or any scheduler mechanic; lifecycle, placement,
  preemption, or M:N semantics (W02–W05); editing any upstream plan, the task
  book, `docs/README.md`, or the stage implementation index; verification
  evidence for P7-V01 (produced only in the verification record when the
  review actually runs); any Rust type, function, module, crate, or public
  API.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect the required P0–P6 handoffs and their records | [Workflow](02-implementation-workflow.md) steps 1–2 | P7-V01 (W01-DV02 locatability) |
| Reconcile timer, event, vCPU, SMP, capability, fault, diagnostics inputs | [Register contract](01-input-boundary-register.md) §3 (P7-IN-01…09) | P7-V01 (W01-DV03 consistency) |
| Record compatible assumptions, missing evidence, ACR/ADR issues | [Register contract](01-input-boundary-register.md) §4–§5 | P7-V01 (W01-DV05 procedure) |
| Normal scheduler-controlled Guest-entry boundary | [Register contract](01-input-boundary-register.md) §6 | P7-V01 (W01-DV04 boundary review) |
| Review boundary against ADR constraints; hand to all P7 packages | [Workflow](02-implementation-workflow.md) steps 5–6; [handoff](02-implementation-workflow.md) §5 | P7-V01 (W01-DV06 consumability) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs`,
audit via `git ls-files`): the repository is a P0 documentation scaffold. P0–P6
task books and plan sets are tracked; there is no Cargo workspace, no Rust
source, and no CI workflow. The only implementation-stage artifacts that exist
are P0's repository-baseline design/record/verification set; P1–P6
`implementation/` and `verification/` directories contain only `.gitkeep`
markers. P7 sibling detailed designs for W06–W14 are being prepared in
parallel; this design references them by slug only and assumes nothing about
their content. Consequently every P0–P6 input below is an **assumed
contract with a recorded failure boundary**, not an evidenced fact; the
register's evidence-status column exists precisely to keep that distinction
explicit through P7 execution.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Establish one reviewed P0–P6 input set and P7 scheduler boundary" (plan Goal) | Input contracts exist scattered across upstream plans; no document maps them to P7 use; P1–P6 evidence is absent | One authoritative input boundary register with fixed schema and per-input evidence status | A reviewed single input set cannot exist without exactly one artifact that locates each input and states its P7 consumption | P7-W01 (this design) | W01-DV01/DV02 review (P7-V01) |
| "Reconcile the timer, event, vCPU, SMP, capability, fault, and diagnostics inputs P7 consumes" (work sequence 2) | Not reconciled anywhere; upstream handoff packages (P2-W10, P3-W14, P4-W09, P5-W10, P6-W13) name what P7 receives but P7 has no consolidated view | Register rows P7-IN-01…09, each citing the source plan path, the semantic contract consumed, and the failure boundary if delivered differently | Reconciliation means each named input is located, its P7 use is stated, and a mismatch path exists | P7-W01 | W01-DV03 consistency review |
| "Record compatible assumptions, missing evidence, and any Architecture Change Request or ADR Required issue" (work sequence 3) | No blocked-prerequisite record or status model exists for P7 | Evidence-status state machine per input plus a blocked-handling procedure that names the stop/record/escalation steps | Explicit recording requires a defined status vocabulary and procedure before the first row is written | P7-W01 | W01-DV05 procedure review |
| "the normal scheduler-controlled Guest-entry boundary" (plan Scope) | The task book states the requirement; P4 delivers a non-scheduler direct first-entry path; no P7 classification of permitted bypasses exists | Entry-boundary statement plus the bounded exception-class list (early-boot, bring-up, emergency-debug) handed to W02 | W02 cannot design an admission gate without knowing which paths are authorized to bypass it | W01 owns the classes; W02 owns the gate mechanics | W01-DV04 boundary review |
| "a reviewer can locate each input and its P7 use; an absent or contradictory contract remains an explicit block" (P7-V01) | No artifact, no review | Register plus closure review producing review evidence in the verification record | P7-V01 is a reviewable property only if the reviewed artifact exists with locatable citations | P7-W01 produces the artifact; verification record carries the review | P7-V01 review evidence |

No row above requires inventing a crate, target, remote, license, or runtime
policy, so no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Register location.** The input boundary register lives as the section
   "Input boundary register" of the W01 implementation record at
   `docs/stages/p7/implementation/p7-w01-entry-contract-reconciliation-record.md`
   (created when W01 work starts; not created by this design). Authority
   basis: the documentation index assigns factual traceability to the stage
   `implementation/` area; creating a new normative document outside that
   area would require routing changes that W01 is not authorized to make.
   The register section is the single authoritative home of P7 input
   statements; other P7 documents cite it by `P7-IN-xx` id.
2. **Register schema and stable input ids.** Seven fixed columns and input
   ids P7-IN-01 through P7-IN-09, defined in
   [the register contract](01-input-boundary-register.md) §2–§3. Adding a row
   is a reviewed register change; removing or renumbering a row requires a
   task-book change, because the rows mirror the task book §2 predecessor
   table.
3. **Evidence-status model.** Each row carries exactly one of
   `contract-mapped (evidence pending)`, `available (evidence linked)`, or
   `blocked (issue linked)`, with declared transition rules
   ([register contract](01-input-boundary-register.md) §4). Today every row
   defaults to the first state: this records the audit truth that P1–P6
   evidence is absent, without either claiming predecessor completion or
   declaring a block that the task book does not yet justify.
4. **Entry-boundary classification owned here; mechanics owned by W02.** The
   three authorized bypass classes — early-boot, bring-up, emergency-debug —
   and the rule that every bypass instance must be enumerable and reviewed are
   fixed in [the register contract](01-input-boundary-register.md) §6. The
   admission gate that enforces the boundary in code is
   [P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md) scope. This
   split keeps W01's review surface stable while W02 designs mechanics.
5. **Consumption by id, not by restatement.** W02–W14 designs and records
   reference inputs as `P7-IN-xx` and must not restate or reinterpret the
   upstream contracts; a perceived contract change is handled by the §4
   procedure, never by silent rewording in a consumer.
6. **No Architecture Change Request or ADR Required issue is raised by this
   design today.** The audited plan sets are mutually consistent at contract
   level with the task book §2 boundary table and the ADR baseline. The
   absence of P1–P6 evidence is a status to track, not a conflict; should any
   row's contract arrive contradicting its register statement, the §4
   procedure applies and the issue is labeled at that point.

## Work breakdown and loading order

1. Load this README, then the Coding Guidelines preflight documents.
2. For register structure and content rules, load
   [the input boundary register](01-input-boundary-register.md). It is the
   contract the produced artifact must satisfy.
3. For the ordered reconciliation steps, evidence rules, and closure, load
   [the implementation workflow](02-implementation-workflow.md). Its final
   sections carry the validation matrix (including the P7-V01 mapping), the
   error/security/observability model, and the handoff checklist.
4. Record actual findings in
   `../p7-w01-entry-contract-reconciliation-record.md` (created when work
   starts) and review evidence in
   `../../verification/p7-w01-entry-contract-reconciliation-verification.md`
   (created when evidence exists). Neither this design nor the record may
   claim W01 complete; completion is claimed only in the verification record,
   for what was actually reviewed.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
configuration schema, CI workflow, or script is designed or authorized by W01.
W01 also authorizes no edits to `docs/README.md`, the stage implementation
index, any task book, any plan, or any upstream document. The only
machine-consumed surface is the register's fixed schema inside the W01
implementation record. Adding any excluded artifact is a scope conflict and
must be stopped at review.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md) consumer map:

- **W02** consumes the whole register plus the entry-boundary classes (§6 of
  the register contract); the admission-gate design
  ([P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md)) must enumerate
  every bypass instance it relies on against those classes.
- **W03–W05** consume the rows their scope touches (placement: P7-IN-03/04;
  preemption: P7-IN-07/08; M:N: P7-IN-04) and inherit the failure boundaries
  recorded there.
- **W06–W11** consume P7-IN-02/05/06/07/08 as applicable to wakeup, pause,
  SMP, accounting, and guest-suite work.
- **W12–W13** consume the evidence-status column: automated QEMU and
  performance evidence may only rely on inputs whose status is
  `available (evidence linked)` at their run time.
- **W14** carries the register's limitation summary and any open blocked rows
  into the factual P8 handoff, so that P8 inherits P7's declared input limits
  rather than assumed completeness.

A downstream package that finds the register wrong or stale raises the §4
procedure against W01's record; it does not patch its own copy.
