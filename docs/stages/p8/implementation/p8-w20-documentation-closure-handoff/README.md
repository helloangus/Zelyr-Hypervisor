# P8-W20 Documentation, Closure, and P9 Handoff — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The factual P8 publication route, stage-gate evidence review, and
bounded P9 handoff required by
[P8-W20](../../plans/p8-w20-documentation-closure-handoff.md).  
**Owner/change context:** P8-W20 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W20. It defines the *route*
to factual P8 closure: the authoritative artifact groups that must exist
([01](01-closure-artifact-contract.md)), the gate matrix that cross-checks
every P8 exit condition against real validation IDs and evidence locations,
and the ordered closure workflow and handoff contract
([02](02-closure-workflow-and-handoff.md)). It is a closure-design package:
it defines what closure review will require and where its outputs live; it
performs no closure, writes no factual specification in advance of approvals
or evidence, and contains no completion claim. The plan says it directly:
*this plan itself does not close P8.*

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
[01](01-closure-artifact-contract.md) when authoring or reviewing the
factual document set and gate matrix, and
[02](02-closure-workflow-and-handoff.md) when executing the closure workflow
or preparing the P9 handoff. Before editing, the agent must also follow the
Coding Guidelines preflight (repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P8
task book](../../task-book-v0.1.md), and the P8-W20 plan). This document
claims nothing about P8's actual state.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W20 plan → this
design → Coding Guidelines. In particular:

- The ADR change rules (`docs/adr/README.md`; baseline §15 v0.1 rule) govern
  architecture changes found during closure: an accepted-ADR conflict is
  never absorbed by rewording a P8 document — it is recorded as a
  superseding-ADR proposal or `Architecture Change Request`. ADR-049/§12
  validation strategy and ADR-040 versioning discipline shape what the
  factual documents must state.
- The task book binds this package to P8-V26: factual implementation
  documents, evidence, limits, unsafe/dependency status, and the P9 handoff
  are present, *and planned evidence is not treated as evidence*. P8 closes
  only with real evidence for P8-V01 through P8-V26 (task book §7); W20's
  review verifies that, and nothing here asserts it.
- The plan's exclusions are binding: **no writing factual specifications
  before their governing approvals and evidence exist, no claiming P8
  closure, no defining Virtio (P9 scope), and no changing accepted ADRs.**

Classification. **Required** for W20 closure: the factual document set, the
gate matrix over P8-V01–V26 with evidence locations, the
limitations/unresolved-decision register, the unsafe/dependency delta
record, and the P9 consumer statement. **Reserved** with recorded triggers:
long-term maintenance documentation re-organization (trigger: an approved
documentation-taxonomy change), CI-visible stage dashboards (trigger: the
P0 CI baseline package), and any post-P8 evidence curation beyond linking
(trigger: a later governance decision). **Out of Scope:** Virtio or any P9
content; machine-ABI values (W02 route); re-running or re-judging any
package's validation; editing plans, task book, or accepted ADRs; and any
speculative "closure report" written before evidence exists.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W14 and W16–W19 evidence requirements and implementation records (work seq 1) | README ledger; [workflow](02-closure-workflow-and-handoff.md) §1 | prerequisite review (W20-DV01) |
| Define the factual document set and the plan/design/implementation/verification distinction (work seq 2) | [artifact contract](01-closure-artifact-contract.md) §2, §3 | P8-V26 (W20-DV02) |
| Cross-check every exit gate against real validation IDs and evidence locations (work seq 3) | [artifact contract](01-closure-artifact-contract.md) §4 gate matrix | P8-V26 (W20-DV03) |
| Record limitations, unresolved decisions, unsafe/API/dependency changes, and architecture conflicts (work seq 4) | [artifact contract](01-closure-artifact-contract.md) §5; [workflow](02-closure-workflow-and-handoff.md) §3 | P8-V26 (W20-DV04) |
| Define the P9 handoff as evidenced facts only (work seq 5) | [workflow](02-closure-workflow-and-handoff.md) §4 | P8-V26 (W20-DV05) |
| Closure review without claiming closure here | [workflow](02-closure-workflow-and-handoff.md) §5, §7 | W20-DV06 |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation-only P0 scaffold. `docs/stages/p8/implementation/` contains the
stage index and the detailed designs now being authored; the machine
specification, Linux boot specification, compatibility policy, validation
report, evidence index, and handoff statement do not exist; `verification/`
contains only `.gitkeep`. P8-W14 and W16–W19 are plans (and, where authored,
proposed designs) with no evidence. Every closure input is therefore an
assumed contract of its owning package.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P8-V26: factual machine specification exists | Absent; concrete v1 values are `ADR Required` (task book §8) via [W02](../p8-w02-machine-contract-governance/README.md) | Factual machine specification, published only after W02-governed approval and implementation evidence | The plan forbids writing factual specs before approvals/evidence; the spec's existence is the exit criterion | Spec content: the approved W02 route and owning implementation designs; publication route: W20 (this design) | gate-matrix row; **failure boundary:** unapproved values ⇒ spec stays unwritten and P8 stays open |
| P8-V26: Linux boot specification and compatibility policy | Absent | Factual boot specification ([W03](../p8-w03-linux-boot-contract/README.md) route) and compatibility policy ([W14](../p8-w14-machine-abi-compatibility/README.md) route), published only with evidence | Same prohibition; both are named task-book §7 closure outputs | W03/W14 fact sources; W20 publication route | gate-matrix rows |
| P8-V26: evidence index over P8-V01–V26 | No index exists | Gate matrix with per-gate evidence location and status enum ([01](01-closure-artifact-contract.md) §4) | "Closes only with real evidence for P8-V01–V26" is checkable only via an explicit per-gate map | W20 (this design); evidence: each owning package | W20-DV03 audit |
| P8-V26: known limits, unsafe/dependency deltas | No register exists | Limitations register and delta record as factual artifacts ([01](01-closure-artifact-contract.md) §5) | ADR-006 unsafe governance and P0 dependency governance require deltas to be reported, and task book §7 requires limits | W20 (register); content: owning packages' records | W20-DV04 review |
| P8-V26: P9 consumer statement | Absent | Evidenced-facts-only handoff statement ([02](02-closure-workflow-and-handoff.md) §4) | Task book §7 bounds what P9 may rely on; the bound must be written as a consumable statement | W20; facts: evidenced packages | W20-DV05 review |
| Document-kind separation enforced | Layering exists by convention (`docs/README.md`) | Per-artifact status discipline: plan vs design vs implementation vs verification never merge ([01](01-closure-artifact-contract.md) §3) | Mixing kinds is how planned evidence becomes "evidence" — the exact P8-V26 failure | W20 | W20-DV02 review |

No row above requires inventing content: W20 owns routes, structures, and
reviews; every factual sentence it will later publish must be traceable to
an owning package's record.

## Resolved design decisions and their authority

1. **Closure is a review act, not an authoring act.** W20 authors only:
   the gate matrix, the register/record *structures*, and the handoff
   statement skeleton. Every factual specification, validation report, and
   evidence index entry is assembled from owning-package records. Rationale:
   the plan forbids manufacturing specifications or evidence; authoring
   factual content from primary records is assembly, inventing it is
   fabrication. Stage-local design freedom owned here.
2. **Factual documents live in the implementation area; evidence stays in
   `verification/`.** The factual machine specification, boot specification,
   and compatibility policy are published under
   `docs/stages/p8/implementation/` only after their governing decisions and
   evidence exist (per the stage implementation index's own rule); the
   validation report, evidence index, and all raw evidence stay under
   `docs/stages/p8/verification/`. Rationale: `docs/README.md` layering;
   keeps the W02-preserved plan/design/implementation/verification
   separation mechanical rather than judgment-based.
3. **Gate statuses are a closed enum.** `evidenced / missing / blocked /
   failed / superseded`, with `evidenced` requiring a link to real
   verification material. No fifth state ("basically done") may be
   introduced. Rationale: P8-V26's core sentence is the planned-vs-real
   evidence distinction; a closed enum makes drift reviewable.
4. **Open security/architecture blocks block closure.** A register entry
   labeled `ADR Required` or `Architecture Change Request` (e.g., from
   [W18's escalation](../p8-w18-security-isolation-regression/02-workflow-oracles-and-handoff.md)
   or the inherited task book §8 machine-values item) leaves its gate
   non-`evidenced`. Rationale: task book §8 requires blocked choices to
   stop the affected work; closure review must inherit, not launder, them.
5. **The handoff statement is bounded to task book §7.** P9 receives exactly
   the evidenced machine identity/contract facts, reservation policy, DTB
   and boot inputs, console, Linux SMP integration and its limits,
   compatibility route, and repeatable regression fixtures — and explicitly
   not Virtio semantics, which remain P9 design work. Rationale: the task
   book's P9 handoff paragraph is the authority for the boundary.
6. **Two-file compact pattern.** Per the W02 model, W20 splits into one
   artifact-contract file and one workflow file; no third file is justified
   at this scope. Stage-local structure owned here.

## Work breakdown and loading order

1. Read [01](01-closure-artifact-contract.md) for the authoritative artifact
   groups, the document-kind discipline, the gate matrix, and the
   register/record structures.
2. Read [02](02-closure-workflow-and-handoff.md) for preconditions, the
   ordered closure workflow, the P9 handoff contract, validation matrix,
   error model, and handoff checklist.
3. Execute the workflow of [02](02-closure-workflow-and-handoff.md) §2–§5
   when implementation is authorized. Review outputs go to
   `../../verification/p8-w20-documentation-closure-handoff-verification.md`;
   factual decisions go to
   `../p8-w20-documentation-closure-handoff-record.md` — both created only
   when that work begins. Nothing here claims P8 closed or will close.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, wire format,
or persistent layout is designed or authorized by W20. The only W20-owned
structures are documentary: the gate matrix, the status enums, the register
and record schemas, and the handoff-statement skeleton. W20 designs no code,
no tooling, no CI, and no document content beyond these structures. Editing
the stage task book, any plan, or an accepted ADR is outside W20 authority;
apparent needs route to the ADR change process. Adding anything else under
W20 authority is a scope conflict to stop at review.

## Downstream handoff

Per the [plan index](../../plans/README.md) consumer map, W20's consumers
are **P9, the project owners, and future maintenance stages**:

- **P9** receives the P9 consumer statement ([02](02-closure-workflow-and-handoff.md)
  §4): only evidenced machine, boot, DTB, console, Linux SMP, compatibility,
  fixture, and regression facts, with their locations and limits. P9 designs
  virtio protocol, queues, transports, and device behavior separately and
  may not cite planned-but-unevidenced P8 work.
- **Project owners** receive the closure review record: gate matrix
  statuses, the limitations and unresolved-decision register, the
  unsafe/dependency deltas, and any `ADR Required` / `Architecture Change
  Request` items — the inputs for the stage-completion decision, which is
  theirs, not this package's.
- **Future maintenance stages** (P10+ management domains, P14 IOMMU, P15
  hardware port, and later machine versions) receive the factual
  specification set and the compatibility policy as the stable reference for
  what v1 actually is, plus the explicit limits (QEMU-scoped evidence,
  single-VM containment scope, non-KPI baselines) that later work must not
  silently generalize.
