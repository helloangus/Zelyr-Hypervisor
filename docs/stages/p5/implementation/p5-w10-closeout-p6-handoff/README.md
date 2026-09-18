# P5-W10 P5 Factual Closeout and P6 Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The evidence-bound closeout, factual documentation publication,
and P6 handoff contract required by
[P5-W10](../../plans/p5-w10-closeout-p6-handoff.md).  
**Owner/change context:** P5-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W10. It defines which
artifacts the P5 closeout may publish, what factual content each requires,
which evidence must back every published statement, and precisely which P6
packages consume which P5 deliverables. It is a completion-review design:
it cannot manufacture runtime evidence, and every rule here is written so
that absence of work is recorded as absence — never as a claim. It
deliberately does **not** author the ABI or security content (that belongs
to the implementing packages), design P6 interrupt semantics, freeze a
management or machine ABI, or change any accepted ADR.

A coding agent (here: the closeout agent) starts with this file and the
mandatory [Coding Guidelines](../../../../development/coding-guidelines.md)
preflight, then loads:

| Assigned step | Load |
|---|---|
| Artifact groups, factual-content requirements, P6 consumer map, publication rules | [01 — closeout contract](01-closeout-contract.md) |
| Ordered closeout workflow, validation matrix, handoff checklist | [02 — closeout workflow and validation](02-closeout-workflow-and-validation.md) |

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → frozen contracts →
P5-W10 plan → this design → Coding Guidelines. In particular:

- The plan covers P5-D01–D09, EC-P5-01–EC-P5-12, and P5-V17: factual HVC
  ABI, handle/capability/Guest-data/security documentation links, invariant
  and compatibility review, Validation Guest and host/QEMU evidence, unsafe
  delta, performance baseline, limitations, regression inventory, and the
  P6 handoff.
- The task book's closure rule is absolute: P5 may close only with real
  evidence for P5-V01 through P5-V17 and all twelve exit criteria; the P6
  handoff may rely only on evidenced facts; every absent criterion or
  later-stage item stays explicitly unimplemented.
- The documentation route (which governed locations receive ABI, security,
  handle, capability, and input-safety artifacts) is W01's deliverable;
  W10 consumes that routing and publishes only into it.
- [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) is the
  structural precedent: a closeout records facts with linked evidence,
  preserves limitations, and constrains the downstream stage to proven
  behavior.

Classification:

- **Required:** the artifact-group contract of
  [01 §1](01-closeout-contract.md); the evidence-backing rule (nothing
  published without a linked record and verification evidence); the
  factual-content minimums; the invariant-inventory reconciliation; the
  unsafe/dependency/limitation records; the P6 consumer map by package ID;
  the closure evaluation without manufactured evidence.
- **Reserved:** publishing artifacts for choices implemented but not yet
  compatibility-reviewed (publish at the moment review completes, per
  [01 §3](01-closeout-contract.md)); extending the handoff with later
  evidence during P6 planning (as recorded amendments, not silent edits).
- **Out of Scope:** writing specifications, security claims, test reports,
  or completion assertions without supporting evidence; freezing a
  management or machine ABI; creating P6 interrupt/timer semantics; editing
  an accepted ADR; re-running or re-scoping W01–W09 work; performance KPI
  language.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Factual HVC ABI, handle/capability/Guest-data/security documentation links (P5-D01–D05 family; plan scope) | [closeout contract](01-closeout-contract.md) §1–§2 | P5-V17 (W10-DV02) |
| Invariant and compatibility review reconciled with outcomes | [closeout contract](01-closeout-contract.md) §4 | P5-V17 (W10-DV03) |
| Validation Guest and host/QEMU evidence indexed; environment, durations, performance method/results recorded | [closeout contract](01-closeout-contract.md) §1, §5 | P5-V17 (W10-DV04) |
| Unsafe delta, dependencies, known limitations recorded | [closeout contract](01-closeout-contract.md) §5 | P5-V17 (W10-DV05) |
| Regression inventory and limitations explicit | [closeout contract](01-closeout-contract.md) §5 | P5-V17 (W10-DV04–DV05) |
| P6 handoff constrained to proven behavior, by package ID | [closeout contract](01-closeout-contract.md) §6 | P5-V17 (W10-DV06) |
| Closure evaluated without manufacturing evidence | [workflow](02-closeout-workflow-and-validation.md) steps 5–6 | P5-V17 (W10-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p5-implementation-designs`,
`git ls-files`): no P5 implementation, evidence, or record exists; P1–P4
likewise have no implementation or verification records; `docs/abi/`,
`docs/security/`, and `docs/architecture/` contain only README indexes; the
W01–W09 detailed designs are parallel work on this branch and none are
approved-and-implemented yet. Every closeout input is therefore a future
artifact whose absence today is the expected state, not a defect of this
package.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Factual closeout records exist (P5-V17) | No W01–W09 records exist | Implemented W01–W09 packages with records and verification evidence | A closeout can only summarize existing facts | W01–W09 (assigned prerequisites) | W10-DV01 prerequisite review |
| ABI/security/input-safety/handle/capability documents published | `docs/abi/`, `docs/security/` hold index READMEs only | The artifacts, authored by implementing packages and published through [01 §3](01-closeout-contract.md) rules | Publication without implemented, reviewed behavior is prohibited by the task book | W02–W05 (content); W10 (publication gate) | W10-DV02 |
| Invariant inventory reconciled (P5-V17) | INV-P5-01..10 exist as task-book wording only | The reconciliation record of [01 §4](01-closeout-contract.md) | Closure requires stating each invariant's evidence or absence | W10 (compilation); W06–W09 (evidence) | W10-DV03 |
| Unsafe delta, dependencies, limitations recorded | Nothing tracked | The factual records of [01 §5](01-closeout-contract.md) | The task book and AGENTS.md reporting rules require them | implementing packages (data); W10 (compilation) | W10-DV05 |
| P6 handoff usable and constrained | P6 task book entry table references P5 outputs | The consumer map of [01 §6](01-closeout-contract.md) and the handoff record | P6 must consume only evidenced facts, by named deliverable | W10 | W10-DV06 |

No row requires inventing evidence; each is a compilation, gate, or
publication rule over artifacts the prerequisite packages must already have
produced.

## Resolved design decisions and their authority

1. **Everything published is evidence-backed.** A closeout statement may be
   recorded only when it links (a) the owning package's implementation
   record and (b) verification evidence with run status. Unimplemented or
   unevidenced items are listed as such in the same record. Authority:
   plan work-sequence items 1 and 5; task book §7.
2. **The closeout compiles; it does not author.** ABI, security,
   handle/capability, and input-safety content is written by the packages
   that implemented and reviewed the behavior; W10's deliverable is the
   publication gate, the link index, and the factual summary records. This
   prevents a closeout-stage rewrite of technical decisions that escaped
   implementation review. Authority: plan out-of-scope; task book §3
   delivery hierarchy.
3. **Publication gate for ABI artifacts.** A factual HVC ABI artifact
   enters `docs/abi/` only when the W02-designed boundary is implemented,
   the compatibility analysis exists, and verification evidence distinguishes
   the experimental internal commitment from any later public contract
   (task book §7). Authority: task book §2 and §7; W02 plan item 4.
4. **The invariant inventory is reconciled item by item.** Each of
   INV-P5-01..10 gets: its enforcing packages, its evidence rows, and its
   status (evidenced / partially evidenced / not implemented), with
   deferrals and any architecture-change records referenced. Authority:
   task book §6 invariant set; plan work-sequence item 4.
5. **The P6 consumer map is by package ID and deliverable, not by summary.**
   Each P6 entry condition and package that may consume P5 facts is mapped
   to the specific evidenced deliverable
   ([01 §6](01-closeout-contract.md)), so P6 planning reads the owning
   record rather than a closeout paraphrase. Authority: plan handoff
   section; [P6 task book](../../../p6/task-book-v0.1.md) entry conditions
   (named consumer only).
6. **Limitations and absences are first-class deliverables.** The known-
   limitations record must state, at minimum, the reserved and out-of-scope
   boundaries from the task book plus every implementation limitation the
   packages recorded — including the single-minimal-operation dispatch,
   test-convention (non-ABI) status of markers and constants, and the
   QEMU-only scope of all timing and behavior evidence. Authority: plan
   scope; task book §7 non-handoffs.

## Work breakdown and loading order

1. Load [01](01-closeout-contract.md) for the artifact groups, publication
   rules, invariant reconciliation format, and the P6 consumer map.
2. Execute the ordered steps in
   [02](02-closeout-workflow-and-validation.md); record the closeout facts
   in `../p5-w10-closeout-p6-handoff-record.md` (created when work starts)
   and the closure review in
   `../../verification/p5-w10-closeout-p6-handoff-verification.md`. Neither
   this design nor any record may claim P5 complete; closure is claimed
   only in verification material, only with the evidence P5-V17 requires.

## Explicitly excluded interfaces

No runtime code, no ABI values or wire formats authored here, no security
claims beyond compiled evidence, no P6 semantics, no scheduler/timer/IRQ
content, no edits to accepted ADRs or upstream records (corrections route
back to the owning package), and no new test execution owned by this
package (re-runs belong to the owning packages; W10 may require a fresh
run as a closure condition, per
[02](02-closeout-workflow-and-validation.md) step 4). The only artifacts
this package creates are the closeout records and the publication-gated
links described in [01](01-closeout-contract.md).

## Downstream handoff

- **P6 planning** ([task book entry conditions](../../../p6/task-book-v0.1.md),
  packages W01/W07/W08/W12/W13) receives the consumer map of
  [01 §6](01-closeout-contract.md): the evidenced HVC/authority/object
  foundation, containment facts, telemetry and regression boundary, and the
  explicit non-deliverables. P6-W01's entry review consumes the handoff
  record; P6 must separately design interrupt-object semantics.
- **Stage review** receives the closure evaluation and the evidence index
  for P5-V01–V17.
- **Later documentation consumers** receive the link index into `docs/abi/`
  and `docs/security/` artifacts, which remain owned by their authors'
  compatibility rules.
