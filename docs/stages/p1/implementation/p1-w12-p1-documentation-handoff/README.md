# P1-W12 P1 Documentation and P2 Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewable P1 contract set, limitation record, P2 handoff, and
stage-gate evidence map required by
[P1-W12](../../plans/p1-w12-p1-documentation-handoff.md).  
**Owner/change context:** P1-W12 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W12. It defines one compact,
documentation-only package: eight authoritative contract/limitation/handoff
documents assembled from the accepted boundaries of W01–W11, plus the
evidence map that tells every reviewer where P1-V01–P1-V21 evidence belongs.
It deliberately does **not** produce stage completion evidence itself,
implementation reports, new ADR decisions, or any P2 design content, and it
performs no retroactive acceptance.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) (for the
documentation-relevant rules: status headers, no completion claims, truthful
reporting). It then loads [01-contracts-and-review.md](01-contracts-and-review.md),
which contains the per-document content contracts, the review workflow, and
the validation matrix — W12 is small enough that no further split is useful.

Before authoring, follow the preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), the [P1-W12
plan](../../plans/p1-w12-p1-documentation-handoff.md), and — for the named
consumers only, not their design content — the [P2 task
book](../../../p2/task-book-v0.1.md) and [P2 plan
index](../../../p2/plans/README.md). This design contains no implementation
or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W12 plan → this design
→ Coding Guidelines. In particular:

- P1-V20 requires the boot, initialization, address-space, diagnostics,
  reference-environment, and limitations contracts to be consistent;
  P1-V21 requires governance: one plan per package, objective conditions,
  resolving links, acyclic dependencies, and **no completion claims**.
- The boundaries W12 documents are the ones **already accepted** by W01–W11
  designs, records, and verification evidence. W12 is an assembler: where
  sources contradict, the contradiction is a finding returned to the owning
  package, never silently reconciled here ([Resolved decision
  3](#resolved-design-decisions-and-their-authority)).
- Documentation placement is bounded by AGENTS.md's stage-separation rule;
  the repository-wide documentation taxonomy has its own authority (the P0
  documentation-baseline package), so W12 fixes a stage-local location and
  records re-homing as an open coordination item, not a decision.

Classification: the eight documents of
[01-contracts-and-review.md](01-contracts-and-review.md) §1 and the evidence
map are **Required**. Re-homing under a future repository-wide taxonomy, and
versioning bumps when consumers demand interface-level stability guarantees,
are **Reserved** with recorded triggers. Completion evidence, implementation
reports, P2 designs, ADR decisions, Guest/SMP/GIC/platform/memory mechanism
documentation, and retroactive acceptance are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Collect accepted boundaries and open issues from W01–W11 (work seq 1) | [01-contracts-and-review.md](01-contracts-and-review.md) §2 step 1 | P1-V20 (W12-DV02) |
| Produce the required P1 contract and limitation documents (work seq 2) | [01-contracts-and-review.md](01-contracts-and-review.md) §1 | P1-V20 (W12-DV01) |
| Map each validation and exit criterion to its future evidence location (work seq 3) | [01-contracts-and-review.md](01-contracts-and-review.md) §1 row 8 | P1-V21 (W12-DV04) |
| Review links, one-to-one coverage, acyclicity, scope (work seq 4) | [01-contracts-and-review.md](01-contracts-and-review.md) §2 steps 3–4 | P1-V21 (W12-DV03) |
| Define completion-review questions and unresolved-decision treatment (work seq 5) | [01-contracts-and-review.md](01-contracts-and-review.md) §1 row 8; §3 | P1-V21 (W12-DV03, DV06) |
| Hand off stable assumptions and explicit non-goals to P2 (work seq 6) | [01-contracts-and-review.md](01-contracts-and-review.md) §1 row 7; downstream handoff below | P1-V20 (W12-DV05) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs` at
`4e631ee`): no P1 contract, limitation, or handoff document exists anywhere in
the tracked tree; `docs/stages/p1/` contains the task book, plans, and two
`.gitkeep`-only `implementation/` and `verification/` directories; no
`contracts/` area exists. No P1 verification evidence exists, so every
evidence-map row will initially point at future locations. The P2 task book
and plan index already name their P1 inputs; nothing on the P1 side answers
them yet.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reviewable P1 contract set (P1-V20) | No contract documents exist | The eight documents of [01-contracts-and-review.md](01-contracts-and-review.md) §1, assembled from accepted W01–W11 sources | A contract set cannot exist unless each required contract has one authoritative home | W12 (documents); content per W01–W11 | W12-DV01/DV02 |
| Known limitations recorded | Limitations are scattered across plans and designs | One limitations document consolidating them with pointers | Reviewability requires a single list with traceable sources | W12 | W12-DV01 |
| P2 handoff (work seq 6) | P2 task book names its P1 inputs; P1 has no answering document | One handoff document mapping stable assumptions and explicit non-goals to the named P2 consumers | The P2 plans' prerequisite lines must resolve to a P1 artifact | W12; consumers named by P2 task book/plans | W12-DV05 |
| Validation/exit criteria mapped to evidence locations (P1-V21) | `docs/stages/p1/verification/` is empty | The stage-gate evidence map covering P1-V01–P1-V21 with future locations | Every criterion needs a designated evidence home before evidence exists | W12 | W12-DV04 |
| No completion claims anywhere (P1-V21) | N/A until authoring | Status-header and wording rules enforced in every W12 document | Governance review must find zero claim violations | W12 | W12-DV06 |
| Consistency of the assembled set (P1-V20) | Sources not yet all accepted | Contradiction-handling rule: findings return to owners | Assembling contradictory sources would fabricate consistency | W12 rule; findings to W01–W11 owners | W12-DV01/DV02 |

No row requires new architecture, mechanism, or ADR content; no decision
blocker is outstanding for the design itself.

## Resolved design decisions and their authority

1. **Location and filenames of the contract set.** The eight documents live
   under `docs/stages/p1/contracts/` with the exact filenames fixed in
   [01-contracts-and-review.md](01-contracts-and-review.md) §1. Rationale:
   AGENTS.md keeps stage artifacts under `docs/stages/<stage>/`; a `contracts/`
   sibling of `plans/`, `implementation/`, and `verification/` preserves that
   separation without touching the repository-wide taxonomy. The
   documentation-taxonomy authority may re-home them later; semantic ownership
   follows the documents, not their path (Reserved trigger recorded).
2. **Eight documents, not more.** The evidence map and the stage completion
   checklist are one review surface (P1-V21 governance) and share one
   document; splitting them would duplicate the same table. Every other task
   book W12 scope item maps one-to-one to a document.
3. **Assembly, not authoring.** W12 documents state accepted boundaries with
   pointers to their sources; they introduce no new technical decision. A
   contradiction between sources is recorded as a finding and returned to the
   owning package; W12 does not pick a winner. Rationale: the plan forbids
   retroactive acceptance and new decisions; P1-V20 demands consistency that
   must come from the sources, not from editorial smoothing. Authority:
   P1-W12 plan (scope, out of scope, work seq 1).
4. **Every document carries the normative status header** (status, scope,
   version `v0.1`, owner/change context, supersedes) and a truthful status
   line ("Contract as of <source set>; validation evidence per the evidence
   map"). No document asserts that implementation or validation happened;
   evidence language is "located at", never "demonstrated by" — unless the
   referenced evidence record itself states the result.
5. **The handoff speaks to named P2 packages only.** W12 maps P1 deliverables
   to the P2 packages that the P2 task book and plan index already name, and
   states for each what is consumed and what stays out; it freezes no P2 API,
   module structure, or design content. Authority: P1-W12 plan handoff;
   P2 task book §2 and plan index.

## Work breakdown and loading order

1. Load [01-contracts-and-review.md](01-contracts-and-review.md) §1 for the
   authoritative document groups and each document's required content.
2. Execute the review workflow in
   [01-contracts-and-review.md](01-contracts-and-review.md) §2: collect
   accepted sources, author each document by its content contract, build the
   evidence map, then run the link/governance/no-claim reviews.
3. W12 creates documentation only. Implementation-side traceability stays in
   the W01–W11 records; commands and results stay in
   `../../verification/` records. W12's own review outcomes are recorded in
   `../p1-w12-p1-documentation-handoff-record.md` and, where the plan's
   acceptance demands a run/not-run statement, in
   `../../verification/p1-w12-p1-documentation-handoff-verification.md`.
   Neither file may exist yet; neither this design nor a record may claim
   W12 complete.

## Explicitly excluded interfaces

No code interface, Rust type, ABI, wire format, script, or CI artifact is
designed or authorized. No new normative technical decision (architecture,
ownership, ordering, naming of mechanisms) is made; where a document must
state a decision, it cites the source that owns it. No P2 design content —
parser, memory map, allocator, inspection layout — is sketched, named, or
constrained beyond restating P1's non-goals. No completion, validation, or
performance claim appears in any W12 document.

## Downstream handoff (P2 stage entry)

P2 enters P1 through the following named consumers (P2 task book work-package
map and plan index; P2 design content remains P2's):

| P1 deliverable | Document home | P2 consumer (named) | Consumed as |
|---|---|---|---|
| Stable QEMU `virt` AArch64 EL2 Rust entry; boot assumptions and rejection boundary | aarch64-boot-contract.md | P2-W01 (boot platform description intake); P2-W09 (QEMU integration regression) | the stable boot environment its plans list as prerequisite |
| Supplied-DTB treatment at P1 and hypervisor-image physical range | aarch64-boot-contract.md; host-address-space.md | P2-W01 (DTB handoff); P2-W03 (boot memory map: image range) | reserved-range and intake inputs |
| Stable Host Stage-1 execution environment and temporary mapping assumptions | host-address-space.md | P2-W03, P2-W04, P2-W05 (memory foundation chain) | the host runtime its mechanisms extend |
| Capability knowledge (required/optional/future classification) | aarch64-boot-contract.md; el2-initialization-contract.md | P2-W02 (platform discovery normalization) | context only — not a discovery implementation |
| Early console and exception diagnostic conventions | exception-diagnostics-contract.md | P2-W06 (platform/memory inspection); P2-W09 | output conventions its inspection and regression reports follow |
| Negative/fault boundary statement (terminal, phase-attributed, marker-observable) | exception-diagnostics-contract.md; known-limitations.md | P2-W08 (host robustness regression) | diagnostic and boundary contract only — explicitly not a fault framework |
| Regression conventions: single runner entry, content-class verdicts, retained evidence | reference-qemu-environment.md | P2-W09 (QEMU integration regression) | precedent; P2 defines its own scenarios and tokens |
| Evidence locations, known limitations, stage-gate map | stage-gate-evidence-map.md; known-limitations.md | P2-W10 (P3/P4 handoff contract) | upstream references its handoff must cite |
| Declared stable environment at lifecycle handoff | el2-initialization-contract.md | all P2 boot-path work | the lifecycle point after which P2 mechanisms attach |

Explicit non-goals handed to P2 (restated, not redesigned): P2 owns DTB
validation and discovery, physical-memory map, and allocation; P1 provides no
allocator, no `PlatformInfo`, no GIC/IRQ service, no secondary-CPU support,
and no guarantee beyond the documented temporary assumptions.
