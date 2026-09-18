# P3-W14 P4 SMP Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The explicit, bounded Host SMP contract that P4 receives,
required by [P3-W14](../../plans/p3-w14-p4-smp-handoff.md).  
**Owner/change context:** P3-W14 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W14. It defines the
handoff package's authoritative artifact — a P4-facing contract document
consolidating exactly what P4 can rely on from P3, by P3-Wxx ID, with
limits, blocked items, and the items P4 must design independently — plus
the P4-consumer mapping and the review that decides the handoff is
sufficient. It deliberately does **not** design any P4 mechanism (no
Stage-2 API, TLBI semantics, VM/vCPU lifecycle, guest execution, or
scheduling — the plan's out-of-scope list), does not create evidence
(evidence belongs to the producing packages' verification records), and
does not declare P3 or any package complete: the contract consolidates
*evidenced* guarantees and honestly lists everything still missing.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) (this is
documentation work; the guide governs the no-new-mechanism boundary). It
then loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, prerequisite boundaries, resolved decisions |
| [02-p4-consumer-map.md](02-p4-consumer-map.md) | the deliverable contract structure and the per-P4-package consumption map |
| [03-workflow-and-review.md](03-workflow-and-review.md) | ordered workflow, P4-consumer review, validation matrix, handoff checklist |

This design follows the compact pattern (per the plan's bounded scope):
the contract artifact's structure and the consumer map are the substance;
there is no code contract, and the checklist template's interface section
is satisfied by the explicit statement below.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W14 plan → this
design → Coding Guidelines. Binding constraints:

- The task book (§6) fixes the handoff contents: reviewed P3 plans,
  implementation and verification records, CPU topology/identity
  contract, physical-CPU lifecycle and per-CPU-state contract, boot
  synchronization and shared-state rules, cross-CPU notification and
  TLB-transport contract, SMP audit/telemetry contracts, QEMU regression
  evidence, and unresolved issues. P3-V14 requires a P4 planner to locate
  and rely on these without redesigning P3.
- The P4 task book (§2) names what P4 consumes from P3: "SMP-safe
  allocator and synchronization, logical pCPU identity, cross-CPU
  notification, TLB-shootdown transport, CPU-local state — no permanent
  single-pCPU assumption and later invalidation integration", and §8
  classifies a missing or contradictory P3 handoff as an Architecture
  Change Request / ADR Required issue that P4 must not repair. This
  design's contract is written to be checkable against exactly that
  consumer list.
- The handoff consolidates; it does not supersede. The authority order
  (ADR → task book → plans → established contracts) is untouched: where
  the contract summarizes a package's design, the design remains
  normative and the contract links to it. A summary conflict is a
  contract bug, never a silent re-decision.
- Host/guest separation (task book; ADR object model): everything handed
  over concerns host physical CPUs. The contract must be unreadable as a
  vCPU or VM contract — P4 designs those against its own task book.

Classification:

- **Required** for W14 closure: the handoff contract artifact with the
  reliance statements, limits, blocked items, and P4-independent-design
  list; the per-P4-package consumer map; the sufficiency review
  (P4-consumer review); and traceability from every contract statement
  to its P3-Wxx source.
- **Reserved** with recorded triggers: contract updates after P3 closure
  (trigger: an approved P3 change — the contract is re-issued, not
  edited silently); P5+ consumption statements (trigger: the P4-W09
  closeout's own handoff; P5 consumes P3 through P4's records, not
  directly); a machine-readable contract format (trigger: a tooling
  decision — v0 is a document).
- **Out of Scope:** any P4 detailed design or mechanism; Stage-2/TLBI/VM/
  vCPU/scheduling semantics; new P3 mechanisms; completion claims for
  P3 or P4; editing upstream plans or designs to make the handoff look
  complete; the stage closure review (W15's package).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| State what P4 can rely on (identity, CPU-local state, shared-state protection, notification/completion, TLB transport, CPU-attributed faults, audit/telemetry rules, regression evidence, unresolved limits) | [contract structure](02-p4-consumer-map.md) §2 (R1–R9 reliance statements) | P3-V14 (W14-DV01) |
| Consolidate only evidenced guarantees; blocking issues listed | [workflow](03-workflow-and-review.md) step 2; honesty rules in [foundations](01-scope-and-foundations.md) §1.2 | P3-V14 (W14-DV02) |
| Integrate with the P4 planning reading order and stage-document boundaries | [consumer map](02-p4-consumer-map.md) §3 | P3-V14 (W14-DV03) |
| Review against host/guest separation and ADR constraints | [workflow](03-workflow-and-review.md) step 4 | W14 closure review (W14-DV04) |
| P4-consumer review for sufficiency | [consumer review](03-workflow-and-review.md) step 5 | P3-V14 (W14-DV05) |
| Record the handoff package and P4-independent items | [consumer map](02-p4-consumer-map.md) §4 | W14 closure review (W14-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P3 has plans for W01–W15 and proposed implementation
designs for W01–W05 (this worktree); no implementation records, no
verification records, and therefore no evidenced P3 guarantees exist
today. The P4 planning set exists ([P4 task book](../../../p4/task-book-v0.1.md),
[P4 plan index](../../../p4/plans/README.md)) and is the named consumer;
no P4 design or implementation exists. The handoff contract artifact this
design defines does not exist yet and cannot be truthfully filled today —
that is expected and recorded as the deferred phase, not as a gap in the
design.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P4 receives an explicit, bounded Host SMP contract | No handoff artifact exists | The contract document (structure fixed in [02](02-p4-consumer-map.md) §2) at `docs/stages/p3/implementation/p3-w14-p4-smp-handoff-contract.md` | P3-V14 requires a locatable, relyable contract; without a fixed artifact there is nothing for a P4 planner to read | W14 (this design) | W14-DV01 structure review |
| Only evidenced guarantees are consolidated | No P3 evidence exists; upstream designs are proposals, not records | The honesty rules: each reliance statement cites its evidence location and status; unevidenced items appear as blocked/limits, never as guarantees | A handoff that claims unaudited guarantees transfers risk invisibly — the exact failure the plan's "consolidate only evidenced P3 guarantees" forbids | W14 (rules); producing packages (evidence) | W14-DV02 evidence-link audit |
| A P4 planner can locate and rely on it | P4 reading order exists but names no P3 contract artifact | The consumer map keyed to P4 package IDs with the P4 reading-order integration | P3-V14's sufficiency is a P4-planner test; the map must be organized by *their* consumption, not by P3's internal layout | W14 (map); P4 plans (named consumers) | W14-DV03/DV05 |
| Host/guest separation and ADR compliance | Contracts are host-CPU only today | The separation review: no statement readable as a vCPU/VM/guest contract; ADR constraints restated only by reference | The ADR object model and P3 task book make this a boundary the handoff must visibly keep | W14 (review) | W14-DV04 |
| Items P4 must design independently | Not enumerated anywhere | The independence list (Stage-2 semantics, TLBI operation, VMID, VM/vCPU objects, guest execution, additional synchronization) | The plan's scope: the handoff bounds P3 by also bounding what it does not give | W14 (list); P4 task book (their side) | W14-DV01/DV06 |
| Unresolved limits and blocking issues recorded | P2-ACR-01 and parallel-design seams unresolved | The unresolved-items register inside the contract, seeded from the plans' recorded open items | Task book §6 requires unresolved issues in the handoff; hiding them would fake sufficiency | W14 (register); owners (resolution) | W14-DV02 |

No ledger row requires designing any P4 mechanism; the contract is
documentation with review evidence.

## Resolved design decisions and their authority

1. **One authoritative artifact, fixed location.** The handoff contract
   lives at `docs/stages/p3/implementation/p3-w14-p4-smp-handoff-contract.md`
   (created only when W14 executes), with the structure fixed in
   [02](02-p4-consumer-map.md) §2. Rationale: P3-V14's "a P4 planner can
   locate" requires one canonical location; summaries elsewhere would
   fork it. It is a W14-owned document, distinct from the W14 design
   (this file set) and from W15's closure package.
2. **Reliance statements cite evidence, or they are not reliance
   statements.** Each R1–R9 statement carries: the P3-Wxx source, the
   design/plan path, the evidence path, and the evidence status
   (evidenced / planned / blocked). Statements without evidence are
   rendered as explicit limitations. Rationale: the plan's "consolidate
   only evidenced P3 guarantees"; a contract that outlives its evidence
   is the classic stage-handoff failure.
3. **The contract is keyed to the P4 consumer list, not to P3's internal
   structure.** The map in [02](02-p4-consumer-map.md) §3 addresses each
   P4 package (P4-W01…P4-W09) by the P3 deliverables it consumes, per
   the P4 task book §2 and plan index. Rationale: P3-V14's sufficiency
   test is a P4 planner's; organizing by consumer makes the test
   mechanical.
4. **Bounds are part of the contract.** The independence list (what P4
   must design itself) and the non-guarantees (no timing proof, no
   hardware proof, no cross-CPU coherence, reserved capacities are
   opaque) are contract sections with equal standing to the reliance
   statements. Rationale: "bounded" in the plan goal; a handoff of
   only-guarantees invites boundary creep into P4 designs.
5. **The contract never edits upstream authority.** Where consolidation
   surfaces a conflict (a design seam unresolved, a plan wording
   mismatch), the contract records the conflict and its label
   (`ADR Required` / `Architecture Change Request`) instead of resolving
   it. Rationale: the skill's conflict rules; silent choice is forbidden
   at every layer.
6. **Sufficiency is decided by a structured P4-consumer review, and its
   result is recorded, not claimed.** The review walks each P4 package's
   consumption needs (from the P4 task book/plan index) against the
   contract and records sufficient/insufficient per item; insufficient
   items become blocking issues in the contract. Rationale: plan step 5
   ("perform a P4-consumer review for sufficiency without claiming P4
   implementation") — the review is evidence, and P4-V14 is satisfied by
   the review's success condition, not by P4 starting work.
7. **Contract lifecycle: re-issue, don't mutate.** After P3 closure, a
   changed P3 guarantee re-issues the contract (versioned header,
   supersedes line) through the owning package's change path. Rationale:
   the checklist's status/owner-change discipline; P4 must be able to
   cite a stable version.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for
   the ledger, boundaries, and decisions.
2. Read [02-p4-consumer-map.md](02-p4-consumer-map.md) for the contract
   structure (R1–R9), consumer map, and independence list.
3. Execute per [03-workflow-and-review.md](03-workflow-and-review.md):
   evidence inventory first, then drafting, then reviews. The contract
   artifact is only written when the evidence inventory makes an honest
   draft possible; until then W14's records state the deferral.
4. Record implementation decisions in
   `../p3-w14-p4-smp-handoff-record.md` and evidence in
   `../../verification/p3-w14-p4-smp-handoff-verification.md` only when
   produced. P3-V14 is *planned* until the contract artifact and its
   review evidence exist.

## Explicitly excluded interfaces

No code interface, runtime state, or mechanism is designed or authorized
by W14. The contract document defines no new API, type, or ABI: its
"interfaces" are citations of established P3 contracts, and any statement
that would need a new interface to become true is recorded as a
limitation or blocking issue instead. The contract must not redefine any
upstream term (identity, eligibility, readiness, telemetry semantics);
it links. A handoff that starts specifying P4 structures (Stage-2 calls,
vCPU fields, VMID rules) has crossed the stage boundary and is stopped at
review. This design also explicitly states it authorizes no guest-visible
document, ABI, or wire format.

## Downstream handoff

- **P4 (all packages)** consume the contract artifact through the P4
  reading order; [P4-W01](../../../p4/plans/p4-w01-entry-contract-reconciliation.md)
  is the designated reconciliation point that checks P3 handoff evidence
  per the P4 task book §2.
- **W15** consumes the handoff package for P3 closure traceability: the
  contract is the last artifact P3's documentation package links as the
  P4-facing summary.
- **P15** inherits the hardware-validation gap as recorded in the
  contract's limits section.
- **Later stages (P5+)** consume P3 only through P4's records (Reserved
  trigger in the scope classification).
