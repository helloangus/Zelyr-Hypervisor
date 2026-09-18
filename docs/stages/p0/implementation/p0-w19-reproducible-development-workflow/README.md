# P0-W19 Reproducible Development Workflow — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The documented clone-to-verified path and the branch-to-PR
integration path required by
[P0-W19](../../plans/p0-w19-reproducible-development-workflow.md).  
**Owner/change context:** P0-W19 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W19. It converts the bounded
work-package plan into small, reviewable documentation changes: one
contributor-workflow document that assembles the prerequisite packages'
contracts into a single linear clean-environment path by **reference, not
restatement**, documentation-discovery wiring, and the fresh-clone and
branch-to-PR walkthroughs that prove path discoverability. It deliberately
does **not** duplicate any prerequisite's commands or policies, implement a
GitHub workflow or required check, configure branch protection, or restate the
[branch and pull-request integration
workflow](../../../../development/integration-workflow.md) — that policy
already exists as adopted normative policy, W19 builds on it, and W20 owns
GitHub's technical enforcement.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
[the stage contracts](01-workflow-stage-contracts.md) for the required
document structure and per-stage content, [the integration path and
walkthrough design](02-integration-path-and-walkthrough.md) for the
branch→PR path and the walkthrough procedures, and
[the implementation workflow](03-implementation-and-review.md) for ordered
steps and the validation matrix. Before editing it must also follow the Coding
Guidelines preflight, including the repository `AGENTS.md`, documentation
index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P0 task book](../../task-book-v0.1.md), and the P0-W19 plan. This document is
the proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W19 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W19 is: a documented clean-environment path
  reaches **build, test, target build, QEMU entry, and the branch-to-GitHub-PR
  integration flow** (P0-V01–V05, P0-V13).
- The plan's out-of-scope clauses bind this design three ways: no reliance on
  undocumented shell history (every step must be reachable from tracked
  documents); no substitution of documentation for actual validation (the
  walkthrough proves discoverability, and says so); and no GitHub workflow,
  required-check, or branch-protection implementation in W19 (that is W20's).
- The [integration workflow](../../../../development/integration-workflow.md)
  is adopted normative policy (2026-09-17) whose boundaries section already
  assigns contributor onboarding and the documented branch-to-PR path to W19
  and GitHub enforcement to W20. W19's deliverable references and operationalizes
  that policy for contributors; it must not restate, rewrite, or amend it.
- The plan's work sequence 2 forbids expanding concrete implementation
  commands: each workflow stage cites its owning contract document for the
  "how", and carries only the assembly information the contracts do not
  (order, evidence expectation, failure attribution, next step).
- Prerequisites W01–W03, W07–W09, and W16–W17 supply the stage contracts.
  W01 is delivered; W02 has a proposed design; the others have approved plans
  with designs in preparation by parallel agents. W19's document is written
  against the **plans' contracts** and links to the contract documents; where
  a contract document does not yet exist in-tree, the stage is marked
  contract-pending and the walkthrough records it as blocked — never worked
  around by inlining the missing content.

Classification: the contributor-workflow document (stage chain, per-stage
content, host/target and placeholder boundaries, enforcement-status honesty),
discovery wiring, and both walkthroughs are **Required** for W19 closure.
Per-stage execution evidence (build, test, target build, QEMU run — owned by
the producing packages), GitHub enforcement evidence (W20), and any automated
workflow-conformance check (candidate future W07/W20 gate) are **Reserved**
with recorded triggers. GitHub workflows, required checks, branch protection,
command restatement, policy amendment, and any new validation substitution
are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Linear clean-environment workflow from W01–W03, W07–W09, W16–W17 contracts (work sequence 1) | [Stage contracts](01-workflow-stage-contracts.md) §3 | P0-V01–V05, P0-V13 (W19-DV01, DV06) |
| Per-stage inputs, expected evidence, failure attribution, next step (work sequence 2) | [Stage contracts](01-workflow-stage-contracts.md) §4 | P0-V09 (W19-DV02) |
| Host vs target difference; QEMU placeholder boundary; artifact identification (work sequence 3) | [Stage contracts](01-workflow-stage-contracts.md) §5 | P0-V05/P0-V13 (W19-DV03) |
| New branch → GitHub PR → required online checks → `main` as the mandatory contributor path; W20 owns enforcement (work sequence 4) | [Integration path](02-integration-path-and-walkthrough.md) §2 | P0-V01–V05 integration portion (W19-DV04) |
| Fresh-clone and branch-to-PR walkthrough; no false "passing" claims (work sequence 5) | [Integration path](02-integration-path-and-walkthrough.md) §3 | P0-V01–V05, P0-V13 (W19-DV06, DV07) |
| Document discoverability and coherence | [workflow](03-implementation-and-review.md) step 4 | P0-V09 (W19-DV05) |
| Downstream consumability by W20, P1 agents, and new contributors | [workflow](03-implementation-and-review.md) handoff checklist | W19 closure review (W19-DV08) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, tracked tree at `4e631ee`): the repository
navigation exists (root `README.md`, `AGENTS.md`, `docs/README.md` —
[P0-W01](../p0-w01-repository-baseline/README.md), completed) and states the
contribution flow at a high level with a link to the
integration-workflow policy; `docs/development/integration-workflow.md` is
adopted normative policy with GitHub enforcement explicitly deferred to W20;
no contributor-workflow document assembles clone → toolchain → build → test →
target build → QEMU entry → artifact identification → PR into one path; no
GitHub remote is configured (W01's record records remote publication as
pending owner inputs); no CI exists (`.github/workflows/.gitkeep` only). Of
W19's prerequisites, W01 is delivered, [W02 has a proposed
design](../p0-w02-rust-toolchain-baseline/README.md), and
W03, W07, W08, W09, W16, W17 have approved plans whose designs are being
prepared in parallel (referenced by slug and P0-Wxx ID without assumed
content). Each ledger row states the missing foundation the plan outcome
necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A documented clean-environment path reaches all P0 execution entries | No end-to-end workflow document; contracts scattered across packages | Contributor-workflow document assembling the stage chain by reference | A new contributor can currently reconstruct the path only by reading every package — exactly the undocumented shell-history risk the plan forbids | W19 (this design) | W19-DV01/DV06 walkthrough |
| Each stage has inputs, expected evidence, failure attribution, next step | Absent | Per-stage content contract in the workflow document, citing owning contracts for commands | Without failure attribution, a failed step sends contributors hunting across packages | W19 structure; contracts own content | W19-DV02 |
| Host vs target, QEMU placeholder boundary, artifact identification explained | Absent | Dedicated boundary sections citing W03/W09/W16/W17 | Confusing host test success with target build success would fake P0-V05 | W19 sections; W03/W09/W16/W17 semantics | W19-DV03 |
| Branch→PR→merge path written as the mandatory new-contributor route | Policy exists; contributor-facing assembly absent; remote unconfigured | Integration-path stage referencing the policy, with the enforcement-status statement | The policy is the authority; the workflow makes it the unavoidable path and states honestly what is and is not yet enforced | W19 path; policy owns rules; W20 owns enforcement | W19-DV04; enforcement evidence stays with W20 (P0-V08) |
| Walkthrough proves every path is repository-completable; no false passing claims (work sequence 5) | Several stage contracts not yet in-tree; remote unconfigured; no CI | Walkthrough procedures with run/blocked/not-run semantics | A walkthrough that silently assumed missing prerequisites would manufacture exactly the false evidence the plan forbids | W19 walkthrough | W19-DV06/DV07 |
| P0-V01–V05, P0-V13 discoverability coverage | Validation IDs exist in the task book with no per-stage mapping | Stage-to-validation mapping in the walkthrough matrix | Each ID must be traceable to the stage whose entry point demonstrates it | W19 matrix | W19-DV07 |

No row requires implementing GitHub enforcement, running a build, or
amending the policy, so no decision blocker is outstanding for this design.
The unconfigured remote and the not-yet-delivered prerequisite contracts are
recorded execution boundaries: they block walkthrough *evidence* for
affected stages and are reported as such, never simulated.

## Resolved design decisions and their authority

1. **Deliverable home:** `docs/development/contributor-workflow.md` is the
   single end-to-end contributor path. Rationale: the plan makes the complete
   path repository knowledge; a single document beside the other development
   policies keeps the onboarding route one link from the documentation index.
   W05 may re-home it later without changing semantic ownership.
2. **Reference-don't-restate rule:** every stage cites its owning contract
   document for all commands and policies; the workflow document carries only
   assembly facts (order, inputs, expected evidence, failure attribution,
   next step, CI correspondence). Rationale: the plan forbids expanding
   concrete commands, and duplicated commands would drift from their
   authorities.
3. **Stage chain:** seven stages — S0 clone and entry reading (W01), S1
   toolchain restoration (W02), S2 host build and tests (W08 entry, W07
   gates), S3 AArch64 target build (W03), S4 QEMU runner entry placeholder
   (W09), S5 artifact identification (W16 metadata, W17 grammar), S6
   integration path (the policy, with W20's enforcement caveat). Rationale:
   exactly the plan's chain from clone through build/test/target/QEMU to PR,
   with identity added because the plan requires artifact identification.
4. **Build-on, don't-touch the policy:** the integration-workflow document is
   linked as the authority for branch/PR/merge rules; W19's implementation
   must not edit it. Rationale: it is adopted normative policy owned by its
   adopting change; the plan's out-of-scope clause and the policy's own
   boundaries section both reserve GitHub enforcement for W20.
5. **Enforcement honesty rule:** the workflow document must state, and the
   walkthrough must obey, that until W20 lands, no GitHub required check
   exists to pass; the branch→PR path is currently a *required human process*,
   and no stage may describe an unimplemented check as passing or configured.
   Rationale: the plan's acceptance wording forbids reporting unimplemented
   online checks as passing.
6. **Contract-pending semantics:** a stage whose owning contract document is
   not yet in-tree is written with its planned contract reference and marked
   contract-pending; the walkthrough records that stage as blocked, with the
   expected document path named. Rationale: W19's document can be written
   against the plans (which exist), but discoverability evidence for a stage
   cannot be manufactured from a document that does not exist.
7. **Walkthrough boundary:** the fresh-clone walkthrough validates that a
   reader can reach every P0 execution entry point from repository documents
   alone (P0-V01–V05, P0-V13 discoverability); it does not execute the
   entries. Executing them is the producing packages' validation (and W20's CI
   evidence); W19 records any execution it actually performed separately as
   personal-environment corroboration, never as package validation.
8. **Discovery wiring:** one routing row in `docs/README.md` points
   first-time setup and contribution walkthroughs at the new document; the
   root README is not modified (W01 owns root navigation; adding a root link
   is an extension point recorded for W05/the owner). Rationale: `docs/README.md`
   is already the mandatory second read for every contributor, so one link
   there keeps the path discoverable without editing another package's
   authoritative artifact.

## Work breakdown and loading order

1. Read [the stage contracts](01-workflow-stage-contracts.md) (document
   structure, stage table, boundary sections) and [the integration path and
   walkthrough design](02-integration-path-and-walkthrough.md) (branch→PR
   path, walkthrough procedures, evidence semantics).
2. Apply the changes in the order stated in
   [the implementation workflow](03-implementation-and-review.md): audit
   prerequisite contract availability, write the workflow document, wire
   discovery, perform the walkthroughs, then close with the validation
   matrix.
3. Store actual walkthrough observations, commands, and run/blocked status in
   `../../verification/p0-w19-reproducible-development-workflow-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w19-reproducible-development-workflow-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W19 complete.

## Design-level state and lifecycle

W19 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked contributor-workflow document plus its
discovery links. Their documentary lifecycle:

```text
no end-to-end path
  -> contributor-workflow document committed (stage chain, boundaries,
     integration path, enforcement status)
  -> discovery links (docs/README routing row, stage index row) committed
  -> fresh-clone walkthrough executed; per-stage outcome recorded
     (reachable / blocked-by-pending-contract)
  -> branch-to-PR walkthrough executed; human-process steps verified against
     the policy; enforcement steps marked pending-W20
  -> as prerequisite packages deliver, their stages' contract links are
     confirmed in the same change and blocked marks are cleared
  -> W20 converts the stage-to-check mapping into configured required checks
  -> P1 onboarding consumes the same document without redefining it
```

The workflow document owns the assembly; each stage's owning contract owns
its content. A stage that contradicts its contract is a review failure
resolved by fixing the workflow document's citation (or, if the contract
itself changed, by the same-change reconciliation rule); the workflow document
must never become a second, divergent statement of any contract.

## Explicitly excluded interfaces

No Rust type, function, crate, Cargo manifest, target triple, build command
restatement, shell script, GitHub workflow file, branch-protection setting,
required-check definition, or public API is designed or authorized by W19,
and no policy text of the integration-workflow document may be amended by its
implementation. The only human-facing procedure is the contributor path and
the walkthroughs; the only machine-facing surface is none. Adding any
excluded item is a scope conflict requiring the applicable detailed design
(at minimum W20 for GitHub enforcement, the owning packages for commands) and
must be stopped at review.

## Downstream handoff

- **W20** receives the stage chain and its stage-to-validation mapping as the
  checklist of what CI must cover and classify (required vs informational vs
  future manual/hardware), plus the enforcement-honesty baseline: until W20's
  configuration exists, the document says so. W20 owns the GitHub
  implementation and the evidence that enforcement is real (P0-V08).
- **P1 Plan/Coding Agents** receive the onboarding path: the same document is
  the P1 entry route from clone to a verified environment, and the
  stage-to-validation mapping shows which P0 evidence a P1 planner may rely
  on.
- **New contributors** receive the single mandatory path, with failure
  attribution so a blocked step names its owning package instead of leaving
  them to improvise.
- **W01–W03, W07–W09, W16–W17** receive the reconciliation rule: when a
  contract changes shape, the corresponding stage citation is updated in the
  same change, keeping the assembly truthful without duplicating content.
- **W22** (P0 dependency map) receives the completed stage table as input to
  the P0-to-P1 handoff map.
