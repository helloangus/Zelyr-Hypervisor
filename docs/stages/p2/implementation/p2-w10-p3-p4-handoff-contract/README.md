# P2-W10 P3/P4 Handoff Contract — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewable P2 consumer contract, evidence map, known
limitations, and stage-gate checklist that P3, P4, and the P2 completion
review consume, required by
[P2-W10](../../plans/p2-w10-p3-p4-handoff-contract.md).  
**Owner/change context:** P2-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W10. W10 is a
documentation-bearing package: its work product is one published artifact —
the P2 consumer-contract record — whose content this design fixes section
by section. The design deliberately does **not** declare P2 complete or
pre-fill any evidence status, does not write verification evidence, does
not resolve [P2-ACR-01](../../task-book-v0.1.md) (it records it as
unresolved), does not design any P3/P4 mechanism (P3/P4 task books and
plans are named consumers only, never designed content), and does not
implement any downstream runtime mechanism.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) (which
apply only insofar as W10 adds no code — it adds none; the guide's
preflight discipline still governs). Before writing, it must also follow
the repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), the
[P2-W10 plan](../../plans/p2-w10-p3-p4-handoff-contract.md), the
[P2 plan index](../../plans/README.md), and the nine prior P2 designs and
their records/evidence as they then exist. This document is a proposed
design; it contains no implementation or validation claim.

| Supporting file | Load it for |
|---|---|
| [01-consumer-contract.md](01-consumer-contract.md) | The deliverable catalog D-01–D-09, the P3 handoff statement, the P4 handoff statement, non-authorization boundaries, known limitations, and P2-ACR-01 handling — i.e. the required content of the published record. |
| [02-evidence-map-and-gates.md](02-evidence-map-and-gates.md) | The P2-V01–V13 evidence-map table, the stage-gate checklist, the completion-review questions, and the record-authoring workflow with acceptance. |

The design is intentionally compact (two supporting files, following the
P0-W02 two-file pattern): W10's substance is curation of already-designed
contracts, not new mechanism.

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W10 plan](../../plans/p2-w10-p3-p4-handoff-contract.md) → this design
→ Coding Guidelines. Binding constraints:

- Task book §8 fixes the handoff semantics: the P3 handoff exposes CPU
  inventory, boot-CPU relation, PSCI/capability facts, and allocation
  availability **only** — it does not authorize AP startup. The P4 handoff
  exposes host RAM topology, protected-range exclusion, allocation/free,
  and the ownership-accounting extension foundation **only** — it does not
  authorize Stage-2, VM, or Guest work. The published record must state
  both non-authorizations verbatim in substance.
- Task book §8 also fixes the mandatory known-limitations list: PCI
  discovery, SMMU/IOMMU, GIC initialization, AP bring-up, and Orange Pi 3B
  runtime support, plus unresolved P2-ACR-01.
- Exit criteria: P2 is complete only when evidence exists for P2-V01
  through P2-V13 and all six §8 conditions hold; the completion reviewer
  must require all evidence before any `DONE` status and preserve
  P2-ACR-01 until authorized resolution (plan handoff wording).
- P3/P4 are consumers: their task books and plans are referenced by path
  as *named consumers* of specific deliverables; nothing in the record
  designs, previews, or constrains their internal content beyond what they
  already state as their prerequisites.

Classification. **Required:** the deliverable catalog with per-deliverable
producer/validation-ID/consumer mapping; the two handoff statements with
non-authorization clauses; the limitations list (mandatory five +
recorded design residuals + P2-ACR-01); the P2-V01–V13 evidence map with
locations and gate conditions; the authoring workflow that fills statuses
truthfully from actual records. **Reserved:** extending the contract into
a versioned cross-stage interface document (if P3/P4 designs later want a
frozen machine-readable contract, that is a new design); recording
downstream acknowledgements. **Out of Scope:** completion claims,
verification evidence authorship, P2-ACR-01 resolution, P3/P4 internals,
any code, and any change outside the W10 record file.

## Requirement-to-design mapping

The tracked sources define P2-L01–L05 at group granularity only; the rows
below are this design's reviewable enumeration from the plan's scope
wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-L01 | Platform-discovery contract: what W02's normalized result and capability model mean for consumers, and how they may be consumed | [01 §2, D-01/D-02](01-consumer-contract.md) | P2-V12 review |
| P2-L02 | Boot-memory ownership rules: protected-range rules, sealed-map authority, allocator/heap contracts and extension surface | [01 §2, D-03–D-05](01-consumer-contract.md) | P2-V12 review |
| P2-L03 | P3 and P4 handoff statements with supported inputs and explicit non-authorizations | [01 §3, §4](01-consumer-contract.md) | P2-V12 review |
| P2-L04 | Known limitations: mandatory five, design residuals, unresolved P2-ACR-01 | [01 §5](01-consumer-contract.md) | P2-V12/P2-V13 review |
| P2-L05 | Stage-gate evidence checklist: P2-V01–V13 map, gate conditions, links, package coverage, acyclicity | [02](02-evidence-map-and-gates.md) | P2-V13 review |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
all ten P2 plans exist; the five W01–W05 implementation designs exist;
the W06–W10 designs (including this one) are being added; no P2
implementation record or verification evidence exists yet, and no P0/P1
implementation beyond P0-W01 exists. W10's record can therefore be
authored only as a *structure with statuses to be filled*, and its
evidence map points at record paths that do not exist yet — by design.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reviewable P3/P4 consumer contract and evidence locations (P2-V12) | No handoff record exists | The W10 record artifact per [01](01-consumer-contract.md)/[02](02-evidence-map-and-gates.md) structure | Consumers need one locatable document, not nine designs to reverse-engineer | W10 (this design) | P2-V12 review of the authored record |
| One plan per package; validation IDs with conditions; links resolve; dependencies acyclic; P2-ACR-01 visible (P2-V13) | Plans exist and satisfy structure today; statuses unevidenced | The gate checklist + package/validation mapping in the record | The completion reviewer needs the map in one place, cross-checked, not recomputed | W10; task book §4/§6 as source | P2-V13 review |
| Known limitations recorded (plan work sequence 4) | Scattered across designs | Consolidated limitations section | Limitations must be findable by P3/P4 without reading every design | W10 | P2-V12 review |
| W01–W09 outcomes, evidence locations available (plan step 1) | Designs exist; records/evidence do not yet | Record references record paths truthfully; statuses filled only from real records | A handoff that pre-claims evidence is a completion claim in disguise | W01–W09 owners produce records; W10 curates | Each record when created |
| P3/P4 named as consumers without designing their content (plan scope) | P3/P4 plans exist | Consumer mapping by plan IDs only | The record routes, it does not design | W10 | P2-V12 review |

No ledger row resolves P2-ACR-01, claims stage completion, or designs
downstream mechanisms.

## Resolved design decisions and their authority

1. **The published contract is the W10 record file:**
   `docs/stages/p2/implementation/p2-w10-p3-p4-handoff-contract-record.md`.
   W10's deliverable is a document, so its implementation record and its
   published artifact coincide; verification reviews (P2-V12/V13) still
   land in the separate verification record. Rationale: respects the
   stage layout (implementation notes and traceability live under
   `implementation/`) and the existing record-path convention; P3/P4 get
   one stable path.
2. **Deliverables are catalogued, not restated.** Each catalog entry names
   the owning package/design, the semantic contract in one paragraph,
   pointers to the authoritative design files, its validation IDs, and its
   named consumers. Rationale: duplication would create a second authority
   that drifts; the task book forbids inferring implementation from
   summaries.
3. **Consumer mapping goes to plan IDs, not designs-of-consumers.** P3
   entries name p3-w01, p3-w02, p3-w04, p3-w06, p3-w11, p3-w13; P4 entries
   name p4-w01, p4-w02, p4-w03, p4-w04, p4-w05, p4-w08 — exactly where
   those plans list P2 prerequisites ([README authority](../../plans/README.md)).
   Rationale: the record must help consumers find their inputs without
   previewing their designs.
4. **Statuses are evidence-derived, never aspirational.** Each evidence-map
   row carries a status field filled only from the referenced record's
   actual content (passed / failed / blocked / not run / record-absent).
   Rationale: the P2-V13 review and the task book's completion review both
   fail if the map claims anything the records do not show.
5. **P2-ACR-01 is quoted, kept visible, and marked unresolved** with its
   task-book §3 authority. Rationale: plan out-of-scope explicitly bars
   resolving it; exit criterion 6 requires consumers can find it.
6. **The record is stage-crossing but stage-local in ownership.** P3/P4
   read it; edits after P2 closure require a P2-record revision (documented
   change note), not a silent edit. Rationale: handoff documents that
   mutate silently lose their audit value for stage reviews.

## Work breakdown and loading order

1. Read [01-consumer-contract.md](01-consumer-contract.md) and author the
   record's contract sections from it (deliverables, handoffs,
   limitations, ACR).
2. Read [02-evidence-map-and-gates.md](02-evidence-map-and-gates.md) and
   author the evidence map, gate checklist, and completion-review
   sections, filling every status strictly from existing records.
3. Validate per [02 §4](02-evidence-map-and-gates.md) and hand off per its
   §5. Decisions/deviations go in the record's change note; review
   evidence goes to
   `../../verification/p2-w10-p3-p4-handoff-contract-verification.md`
   (created when review evidence exists). Nothing claims W10 or P2
   complete.

## Explicitly excluded interfaces

No code, no API, no ABI, no command surface, no CI artifact; no new
normative contract beyond what the record restates by pointer (the record
is informative-traceability for stage crossing; normative authority stays
with the cited designs and the task book); no edits to other packages'
designs, plans, task books, or verification files; no P3/P4 design
content; no completion language anywhere in the record ("P2 status:
in progress — see evidence map" is the strongest permitted wording until
the authorized completion review says otherwise).

## Downstream handoff

- **P3** ([../../../p3/README.md](../../../p3/README.md), named consumers:
  p3-w01 CPU topology inputs, p3-w02 secondary bring-up, p3-w04 per-CPU
  runtime, p3-w06 concurrency, p3-w11 observability, p3-w13 QEMU
  regression, p3-w14 handoff): consumes the P3 handoff statement, D-01–D-05,
  D-09, and the limitations — without AP-startup authorization.
- **P4** ([../../../p4/README.md](../../../p4/README.md), named consumers:
  p4-w01 entry reconciliation, p4-w02 Stage-2 address space, p4-w03 guest
  memory/image, p4-w04 vCPU entry/exit, p4-w05 validation guest, p4-w08
  QEMU regression): consumes the P4 handoff statement, D-01–D-05, D-09,
  and the limitations — without Stage-2/VM/Guest authorization.
- **P2 completion review** (task book §8/§9): consumes the gate checklist
  and evidence map as the review instrument.
