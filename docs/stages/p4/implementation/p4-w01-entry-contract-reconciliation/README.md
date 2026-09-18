# P4-W01 Entry Contract Reconciliation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewable P4 entry boundary: reconciliation of the P0–P3
handoff contracts, the P4 assumption ledger, and the blocked/conflict
classification required by [P4-W01](../../plans/p4-w01-entry-contract-reconciliation.md).  
**Owner/change context:** P4-W01 implementation handoff; this design is the
reconciliation authority for the P4 stage entry review.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W01. P4-W01 is a reconciliation
contract, not a mechanism package: it establishes what P4 may assume when
P4-W02 through P4-W09 start, records where each assumed input's evidence lives
or that it is absent, and classifies every gap without redesigning P0–P3 or
inventing P4 mechanisms. It deliberately does **not** implement Stage-2,
Guest memory, vCPU, or Guest behavior, does not repair any P0–P3 package, and
does not define module trees, APIs, or algorithms for later P4 packages; those
belong to the W02–W09 detailed designs.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- [01-reconciliation-contract.md](01-reconciliation-contract.md) — the
  authoritative artifact groups, the prerequisite-to-consumer reconciliation
  matrix, the P4 entry-assumption ledger that W01 re-fixes for P4, and the
  gap classification rules. Load this file for workflow steps 1–5.
- [02-review-workflow.md](02-review-workflow.md) — the ordered review
  workflow, validation matrix, evidence conventions, and handoff checklist.
  Load this file for workflow step 6 and any closure review.

Before performing the review the agent must also follow the reading order in
the [plan index](../../plans/README.md): ADR, P4 task book, this plan, and the
available P0–P3 handoff records. This document is a proposed design; it
contains no implementation, review, or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W01 plan](../../plans/p4-w01-entry-contract-reconciliation.md) → this
design. In particular:

- The task book makes W01 an entry condition for all P4 implementation:
  "P4-V01 requires a linked review showing every P0–P3 prerequisite, its
  evidence location or absence, its P4 consumer, and any block." W01 produces
  a review, never runtime evidence, and never repairs upstream scope.
- The task book entry-conditions table (§2) names the four upstream handoff
  surfaces (P0, P1, P2, P3) and states: if a handoff is absent or contradicts
  needed P4 behavior, record the blocked prerequisite or an ADR-required
  issue. This design turns that instruction into an explicit ledger and
  classification procedure.
- The ADR invariants (§19) bind every P4 package; W01 records them as entry
  constraints but owns none of them.
- W01 owns no runtime behavior. It has **no code interfaces** (see
  [the reconciliation contract](01-reconciliation-contract.md) §5).

Classification: the reconciliation matrix, the P4 entry-assumption ledger, and
the gap classification rules in [01-reconciliation-contract.md](01-reconciliation-contract.md)
are **Required** for P4-V01. Unsafe-inventory alignment and validation-input
alignment are **Required** as review inputs but the underlying governance
belongs to P0 packages. Multi-pCPU Stage-2 shootdown, upstream-stage repair,
P4 mechanism design, and any completion claim are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect ADR, task book, plan index, P0–P3 handoff/implementation/verification records | [review workflow](02-review-workflow.md) step 1 | recorded inspection list, one row per source |
| Reconcile EL2 exception, Host memory ownership, allocation, SMP, TLB-transport assumptions | [reconciliation matrix](01-reconciliation-contract.md) §2 | every P4 consumer package has an identified, linked input |
| Record authority, security, layering, telemetry, test-environment constraints | [entry-assumption ledger](01-reconciliation-contract.md) §3 | P4-binding constraints enumerated with authority basis |
| Map each P4 prerequisite to evidence or unresolved dependency | [reconciliation matrix](01-reconciliation-contract.md) §2 + [workflow](02-review-workflow.md) step 3 | each row names its evidence location or an explicit absence |
| Review missing/incompatible/ambiguous inputs and classify | [gap classification rules](01-reconciliation-contract.md) §4 + [workflow](02-review-workflow.md) step 4 | every gap labeled Blocked, Architecture Change Request, or ADR Required; none silently absorbed |
| Record the entry review and hand the accepted contract set to W02–W09 | [workflow](02-review-workflow.md) steps 5–6 and handoff checklist | consumer packages can name their accepted inputs and recorded gaps |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs` at
`4e631ee`): the repository is a documentation-only P0 scaffold. There is no
Cargo workspace, no Rust source, no built or verified P1–P3 mechanism.
`docs/stages/p0/implementation/` contains detailed designs for P0-W01/P0-W02
and a verification record for P0-W01 only; `docs/stages/p1/implementation/`,
`p2/implementation/`, `p3/implementation/` and the corresponding
`verification/` directories contain only `.gitkeep`. P1–P3 work is planned
(task books plus plans) but not delivered. The P4 stage has task book, plan
index, nine plans, and an empty `implementation/` and `verification/` area.

Consequence for this design: at the time W01 executes, most upstream evidence
rows will truthfully resolve to "planned, not delivered." The reconciliation
review must therefore record **assumed-contract status** per row: the P4
consumer may proceed only by treating the upstream plan as an assumed contract
with an explicit failure boundary, and any P4 implementation acceptance that
depends on the missing evidence stays blocked until the upstream evidence
exists. The task book already states this dependency rule ("This task book
records a dependency; it does not assert any upstream stage is complete").

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every P0–P3 prerequisite identified with evidence location or absence | No P4-side ledger exists; P1–P3 verification directories contain only `.gitkeep` | The reconciliation matrix of [01 §2](01-reconciliation-contract.md) executed against the real tree | P4-V01 requires the linked review to exist before implementation | W01 (this design) | executed matrix in the W01 implementation record |
| P4 assumptions actually supported by upstream contracts are recorded | No P4 assumption ledger exists | The entry-assumption ledger of [01 §3](01-reconciliation-contract.md) | "no P4 package assumes an undocumented upstream behavior" is the P4-V01 passing condition | W01 | assumption rows with authority links |
| Gaps classified, not silently absorbed | No classification procedure exists for P4 | Gap classification rules of [01 §4](01-reconciliation-contract.md) | Plan work sequence item 5 requires classification against the ADR | W01; conflicting decisions escalate per ADR/task book | classified gap list with labels |
| W02–W09 receive the accepted contract set | No P4 design consumes a reconciliation result yet | Handoff checklist of [02 §4](02-review-workflow.md) | Plans index makes W01 a prerequisite of W02–W09 | W01 produces; W02–W09 consume | handoff section in the implementation record |
| Validation and unsafe-review inputs aligned | P0 governance (unsafe inventory, quality gates) is planned, not delivered | Alignment rows in [01 §2](01-reconciliation-contract.md) (unsafe, telemetry, QEMU, CI) | Plan scope names "align planned validation and unsafe-review inputs" | W01 records; P0 owners deliver | alignment status per row |

No row above requires inventing a crate, API, target, or runtime mechanism, so
no decision blocker is created by this design. The absence of P0–P3 *evidence*
is a recorded finding, not a blocker this design may resolve.

## Resolved design decisions and their authority

1. **W01 output is a review record, not a new normative contract document.**
   The executed reconciliation matrix, assumption ledger, and gap list live in
   the implementation record (`../p4-w01-entry-contract-reconciliation-record.md`,
   created when the review runs); verification evidence lives in
   `../../verification/p4-w01-entry-contract-reconciliation-verification.md`.
   Rationale: the task book separates implementation notes and verification
   evidence; W01 has no runtime artifact to normatively freeze. Authority:
   P4 task book §3 delivery hierarchy.
2. **Reconciliation is by contract surface, not by stage summary.** Each row
   names the owning plan or handoff package (for example
   [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) for the P2
   consumer contract, [P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md) for
   the Host SMP contract, [P1-W12](../../../p1/plans/p1-w12-p1-documentation-handoff.md)
   for the P1 documentation handoff) rather than a whole stage. Rationale:
   P4-V01 requires per-prerequisite traceability; stage-level summaries hide
   the row a consumer actually depends on. Authority: P4-W01 plan work
   sequence items 1–2.
3. **Planned-but-undelivered upstreams are recorded as assumed contracts.**
   Where a row's contract exists only as an approved plan and no verification
   record exists, the reviewer records `assumed: planned` plus the failure
   boundary from [01 §4](01-reconciliation-contract.md), and P4 packages that
   consume the row remain blocked at their implementation acceptance until
   real evidence exists. Rationale: the task book forbids asserting upstream
   completion and forbids absorbing missing inputs into P4. Authority: P4
   task book §2.
4. **W01 re-fixes P4-local interpretation items explicitly, as stage-local
   design freedom.** Items upstream sources do not state but P4 needs (Guest
   address terminology, guest-fault containment, temporary-layout discipline)
   are enumerated in [01 §3](01-reconciliation-contract.md) with their ADR
   basis, and each later P4 detailed design either consumes them or re-owns
   them explicitly. W01 does not design mechanisms. Rationale: the plan's work
   sequence item 3 asks for recorded constraints bounding P4; without a fixed
   interpretation each W02–W09 design would re-derive them inconsistently.
   Authority: Plan Agent guidelines (stage-local design freedom tier) plus the
   cited ADR invariants.
5. **Conflict labels follow the task book taxonomy.** Missing prerequisite →
   `Blocked prerequisite`; contradiction with a P0–P3 contract →
   `Architecture Change Request`; contradiction with or gap under the ADR →
   `ADR Required`. W01 never resolves a labeled item. Rationale: task book §8
   and the Plan Agent authority order.
6. **The review is repeatable.** The matrix and ledger are re-runnable after
   upstream stages deliver evidence; the implementation record records the run
   date, tree revision, and per-row outcome of each execution. Rationale: W01
   executes once before P4 implementation starts, but its output must stay
   truthful as upstream evidence lands during P4.

## Work breakdown and loading order

1. Read [01-reconciliation-contract.md](01-reconciliation-contract.md) §1–§2
   to understand the artifact groups and the reconciliation matrix structure,
   then execute the inspection pass (workflow step 1) against the current
   tree.
2. For each P4 consumer package, complete its matrix rows and the entry
   assumption rows it depends on ([01 §2–§3](01-reconciliation-contract.md));
   classify every gap with [01 §4](01-reconciliation-contract.md).
3. Apply the recording, validation, and handoff procedure in
   [02-review-workflow.md](02-review-workflow.md): write the implementation
   record, run the P4-V01 review checks, and state the handoff for W02–W09.
4. Record review evidence in
   `../../verification/p4-w01-entry-contract-reconciliation-verification.md`
   only when the review is actually performed; neither this design nor the
   records may claim P4-W01 complete.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
persistent data layout, command script, or CI configuration is designed or
authorized by W01; see [01 §5](01-reconciliation-contract.md). W01 also does
not repair P0–P3 scope, does not select Stage-2 or image mechanisms for W02–W05,
and does not freeze the P8 machine ABI or any temporary P4 layout values.

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W02** (design: `../p4-w02-stage2-address-space/README.md`) consumes the
  reconciled P2 memory-ownership, P3 TLB-transport, and P1 EL2-environment
  rows; it must not assume a behavior absent from the accepted set.
- **P4-W03** (design: `../p4-w03-guest-memory-image/README.md`) consumes the
  P2 allocation/protected-range and ownership-accounting rows.
- **P4-W04** (design: `../p4-w04-vcpu-entry-exit/README.md`) consumes the P1
  exception/EL2-baseline, P3 CPU-local, and cross-CPU transport rows.
- **P4-W05** (design: `../p4-w05-validation-guest/README.md`) consumes the
  toolchain/build-target, QEMU-environment, and image-route governance rows.
- **P4-W06–P4-W09** (designs `../p4-w06-fault-isolation-diagnostics/README.md`,
  `../p4-w07-repeatability-telemetry/README.md`,
  `../p4-w08-qemu-integration-regression/README.md`,
  `../p4-w09-closeout-p5-handoff/README.md`) consume the validation-input,
  telemetry, and evidence-location rows relevant to their scopes.
- Every W02–W09 design must state, in its own current-state findings, which
  W01 rows it consumes and how it fails if a row was recorded as blocked.
- **P5** is not a W01 consumer; P5 receives only the evidenced facts recorded
  later by P4-W09.
