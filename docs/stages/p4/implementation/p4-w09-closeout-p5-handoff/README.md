# P4-W09 P4 Factual Closeout and P5 Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The completion-review package required by
[P4-W09](../../plans/p4-w09-closeout-p5-handoff.md) (P4-L01–L03, P4-V16):
the factual boot-contract record, Stage-2 capability matrix, known-limitation
record, unsafe-inventory and ADR-deviation review, evidence index,
exit-criterion review, and the limited P5 handoff.  
**Owner/change context:** P4-W09 implementation handoff; this design owns
the closeout artifact set and the review procedure that decides what P5 may
rely on. It owns no runtime mechanism and creates no evidence.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W09. It converts the bounded
work-package plan into the authoritative closeout artifact groups (what is
recorded, from which sources, under which truthfulness rules), the
P5-consumer mapping (which P5 package consumes which P4 deliverable, by ID),
and the review workflow that evaluates P4-V16 — recording missing evidence,
failed criteria, and deferred scenarios explicitly instead of making
completion claims. It deliberately does **not** create evidence before
implementation, declare P4 complete without all matrix evidence, freeze the
P8 machine ABI or any temporary P4 layout value as an ABI, define P5 HVC/
capability/handle contracts, or change accepted ADR decisions. This is a
completion-review package: it cannot manufacture successful runtime
evidence, and a review that finds gaps records them as gaps.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads the one supporting file for its assigned step:

- [01-closeout-contracts.md](01-closeout-contracts.md) — the authoritative
  artifact groups (closeout record sections with required content and
  sources), the P5 consumer-by-ID mapping, and the truthfulness rules that
  keep temporary P4 facts from becoming later contracts. Load for recording
  work (plan steps 2, 3, 6).
- [02-review-workflow.md](02-review-workflow.md) — the ordered review
  workflow, the P4-V16 evaluation rules, the validation matrix, the
  error/observability model, and the handoff checklist. Load for review work
  (plan steps 1, 4, 5).

Before performing the review the agent must also follow the reading order in
the [plan index](../../plans/README.md): ADR, P4 task book, this plan, and
the W01–W08 implementation and verification records as they actually exist.
This document is a proposed design; it contains no implementation, review,
or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W09 plan](../../plans/p4-w09-closeout-p5-handoff.md) → this design →
Coding Guidelines. Binding constraints include:

- Task book §3: "record implementation facts, limitations, evidence, and the
  P5 handoff only after the required work has actually been performed and
  verified"; implementation and detailed-design records belong in
  `implementation/`, completion evidence in `verification/`.
- Task book §7 exit criteria (1–6) and §6 matrix P4-V01–V16: P4 can close
  only with real evidence for every row; the closeout evaluates, it never
  waives.
- Task book §1 Reserved/§8: temporary Validation Guest IPA values, scenario
  conventions, P4 exit classes, and layout values are implemented facts, not
  the `rusthv-arm-virt-v1` machine ABI (P8) and not a frozen guest boot ABI;
  no record may silently freeze them (W01 assumption A4).
- Task book §7: P5 may rely only on proven P4 facts — a usable single-Guest/
  vCPU execution boundary, Stage-2 isolation and fault classification,
  bounded Guest memory and image path, the maintained Validation Guest, and
  QEMU regression entry points. P5 still owns formal HVC ABI, capabilities,
  object handles, and management semantics.
- ADR change rules: an accepted ADR is never edited to absorb a deviation;
  deviations are recorded and routed as `ADR Required` / `Architecture
  Change Request` items.
- W01 gap rules: `Blocked prerequisite`, `Architecture Change Request`, and
  `ADR Required` labels are carried into closeout unresolved unless the
  authoritative owners resolved them; P2-ACR-01 stays visible.

Classification: the closeout artifact groups, the P5 consumer mapping, and
the review workflow ([01](01-closeout-contracts.md),
[02](02-review-workflow.md)) are **Required** for P4-L01–L03 and P4-V16.
Stage-gate coordination with the P0–P3 owners (when closeout finds upstream
blocks) is **Required as recording duty only** — W09 never repairs upstream
scope. Re-running P4 mechanisms, new validation creation (beyond what W08's
regression already defines), P5 contract design, machine-ABI statements, and
completion claims are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-L01 implemented Validation Guest boot contract recorded | [closeout contracts](01-closeout-contracts.md) §2 (BC record) | factual, evidence-linked, non-ABI (P4-V16 review) |
| P4-L02 Stage-2 capability matrix recorded | [closeout contracts](01-closeout-contracts.md) §2 (CM record) | every matrix row cites evidence or records absence |
| P4-L03 known limitations recorded | [closeout contracts](01-closeout-contracts.md) §2 (KL record) | limitations explicit with downstream owners |
| Unsafe-inventory and ADR-deviation review | [closeout contracts](01-closeout-contracts.md) §2 (US/ADR records) | delta stated; deviations labeled, none absorbed |
| Evidence index and exit-criterion review | [review workflow](02-review-workflow.md) steps 1–3; [closeout contracts](01-closeout-contracts.md) §2 (EI record) | every P4-V01–V16 row has a location or an explicit absence |
| P5 handoff by consumer and deliverable | [closeout contracts](01-closeout-contracts.md) §3 | each named P5 package can locate its input or its recorded absence |
| Missing evidence / failed criterion / deferred scenario handled explicitly | [review workflow](02-review-workflow.md) step 5 | recorded as such; no completion claim substitutes |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs`):
documentation-only repository. `docs/stages/p4/implementation/` contains the
W01–W05 design sets plus the parallel W06–W09 design effort;
`docs/stages/p4/verification/` contains only `.gitkeep`. No P4 package has
an implementation record or verification evidence; P0–P3 are planned and
undelivered except P0-W01. The P5 stage exists as task book plus plans
(`docs/stages/p5/`) and is the named consumer. The fine-grained P4-L
requirement wording of the superseded root source task book is not tracked
(W01 item A9 provenance pattern); requirement meaning here comes from the
tracked task book §5/§6/§7 rows and the W09 plan scope sentences.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Factual records only for work actually performed and verified | No P4 implementation or verification record exists | The truthfulness rules and per-record source maps of [01 §2](01-closeout-contracts.md) | a closeout that cannot distinguish "performed" from "planned" would falsify the stage | W09 rules; W01–W08 records as sources | review-run evidence in the W09 verification record |
| Implemented boot contract recorded (P4-L01) | Nothing recorded; W03/W05 designs propose the route and conventions | The boot-contract record section ([01 §2](01-closeout-contracts.md)) citing delivered facts | P5-V01's "no undocumented P4 behavior assumed" needs one citable boot-fact record | W09 records; W03/W05/W04 own the underlying facts | P4-V16 row |
| Stage-2 capability matrix recorded (P4-L02) | Nothing recorded; W02 design proposes capabilities | The capability matrix section: capability → evidence-or-absence rows | P5 must know exactly what Stage-2 can enforce without re-deriving it | W09 records; W02/W06 own the facts | P4-V16 row |
| Known limitations recorded (P4-L03) | Nothing recorded; limitations are scattered across designs | The limitations record with downstream owners ([01 §2](01-closeout-contracts.md)) | unrecorded limitations get rediscovered as P5 defects | W09 records; each W0x names its own | P4-V16 row |
| Unsafe-inventory and ADR-deviation review | No P4 unsafe delta exists yet; P0-W10 governance planned | The delta/deviation sections ([01 §2](01-closeout-contracts.md)) | the audited-unsafe property must be checkable at stage boundary | W09 records; W0x records as sources | P4-V16 row |
| Evidence index and exit-criterion review (P4-V16) | `verification/` is empty | The evidence index + criterion review procedure ([02](02-review-workflow.md) steps 1–3) | P4 closure is defined as P4-V01–V16 evidence; the index is how that is checkable | W09 | the review itself is the evidence |
| Limited P5 handoff by ID | No mapping exists anywhere | The consumer-by-ID mapping ([01 §3](01-closeout-contracts.md)) | the task book names P5 the primary downstream; P5-W01 consumes P4 facts per its entry conditions | W09 mapping; [P5 task book](../../../p5/task-book-v0.1.md) as named consumer | mapping review row |

No row above requires a decision outside this design's authority; the
record section structure, truthfulness rules, and mapping are stage-local
design freedom in the plan's declared scope.

## Resolved design decisions and their authority

Summarized here; full rationale and authority citations in
[01 §4](01-closeout-contracts.md):

1. **One closeout record, fixed section set:** all factual closeout content
   lives in `../p4-w09-closeout-p5-handoff-record.md` (created when the
   review runs) with the sections of [01 §2](01-closeout-contracts.md);
   verification evidence for the review itself lives in
   `../../verification/p4-w09-closeout-p5-handoff-verification.md`.
2. **Facts vs plans split is mechanical:** every statement cites a W01–W08
   implementation/verification record path; a statement without such a
   citation is a plan reference and is recorded in the not-delivered
   section, never in the factual sections.
3. **Capability matrix rows are evidence-or-absence:** no row may say
   "supported" without a verification citation; absence rows name the
   blocking prerequisite per the W01 gap taxonomy.
4. **Temporary facts are labeled at every mention:** layout values, scenario
   conventions, exit classes, run-record grammar, and probe channels carry
   the non-ABI label (W01 A4) in the record itself, so downstream readers
   cannot mistake them for machine ABI.
5. **The P5 handoff is a mapping, not a grant:** it names, per P5 package,
   the P4 deliverable IDs it may consume and their evidence status; it
   confers no new authority and defines no P5 semantics (P5 task book
   §2/§3 own those).
6. **Completion language is confined:** only the P4 stage verification
   record, per row, may state pass/fail/blocked/not-run with evidence; no
   closeout statement, design document, or implementation record claims
   stage completion.
7. **W09 adds no code and no `unsafe`**; it is documentation and review
   work; deviations discovered during review are routed, not patched.

## Work breakdown and loading order

1. Load [01-closeout-contracts.md](01-closeout-contracts.md) to understand
   the record sections, their required content, sources, and truthfulness
   rules, plus the P5 consumer mapping.
2. Execute the review in the order given by
   [02-review-workflow.md](02-review-workflow.md): inspect, build the
   evidence index, evaluate each exit criterion, review the
   temporary-facts/containment boundaries, evaluate P4-V16, and write the
   handoff.
3. Record review evidence in
   `../../verification/p4-w09-closeout-p5-handoff-verification.md` only when
   the review is actually performed; neither this design nor any record may
   claim P4-W09 or stage completion.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, ABI, wire format, script, or
CI configuration is designed or authorized by W09; see
[01 §5](01-closeout-contracts.md). W09 also does not: create or rerun
validation, repair P0–P3 or W01–W08 scope, resolve P2-ACR-01 or any labeled
conflict, define P5 HVC/capability/handle/management semantics, freeze the
P8 machine ABI or any P4 test convention as an ABI, or state stage
completion anywhere.

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md), W09's output
serves:

- **P5 stage planning and P5-W01** (design consumer path per the
  [P5 task book](../../../p5/task-book-v0.1.md) §2 entry-conditions table):
  the closeout record's evidence index, capability matrix, boot contract,
  limitations, and consumer mapping are the P4 side of the P5 entry
  reconciliation; per-package detail in
  [01 §3](01-closeout-contracts.md).
- **P4 stage review:** the exit-criterion review and the explicit
  missing/failed/deferred record are the stage review's input.
- **P8 guard:** the closeout's non-ABI labels are what a later P8 machine
  design reads to know which P4 facts must be re-established rather than
  inherited.
