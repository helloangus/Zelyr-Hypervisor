# P0-W04 Build Profile / Feature Governance — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The non-overlapping governance of build capabilities (Cargo
features), build profiles, and runtime resource/policy choices required by
[P0-W04](../../plans/p0-w04-build-profile-feature-governance.md).  
**Owner/change context:** P0-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W04. It converts the bounded
work-package plan into one normative governance document, its discovery
wiring, and a representative-decision review drill. It deliberately does
**not** implement any profile, define any Cargo feature, create or modify a
Cargo manifest (none exists in the tracked tree, and W03's baseline member has
none of these), define the future runtime configuration schema, or design the
code organization of features; those belong to W03, the future configuration
designs, and P1+ respectively.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, ADR baseline, P0 task book, and P0-W04 plan.
This document is the proposed detailed design for those changes; it is not a
completion record and contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W04 plan → this design
→ Coding Guidelines. In particular:

- ADR-047 fixes the semantics this governance operationalizes: profiles such
  as minimal/research/embedded/general/secure/full are supported; a feature
  expresses "what the binary is capable of"; and runtime strategy such as VM
  counts and RAM must not be abused as Cargo features.
- ADR-037 fixes the configuration layering: build configuration controls
  binary capability only; boot and VM/runtime configuration are separate
  objects owned by later stages. ADR-046 fixes the workspace rule the
  governance must not fight.
- The task book outcome for W04 is: build capabilities, runtime configuration,
  features, and profiles have **non-overlapping governance** (P0-V09, P0-V15),
  and the Reserved profile set must not be made unavailable by P0 decisions.
- The plan's out-of-scope list bounds the document: it must not implement all
  profiles, must not decide the future runtime configuration schema, and must
  not design feature code organization.

Classification: everything in
[the governance contract](01-governance-contract.md) is **Required** for W04
closure. The six named profiles are **Reserved** (purpose recorded, mechanism
unimplemented, activation only through the thresholds). Cargo `[profile.*]`
customization, feature declarations, the runtime configuration model, and all
code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Switch-classification criteria for binary capability, build profile, runtime resource/policy | [Governance contract](01-governance-contract.md) §3–§4 | P0-V09 (W04-DV01) |
| Long-term profile purposes, applicability, and reserved status without architecture forks | [Governance contract](01-governance-contract.md) §5 | P0-V09 (W04-DV02) |
| Review questions and prohibited cases for new switches | [Governance contract](01-governance-contract.md) §6–§7 | P0-V09 (W04-DV03) |
| Representative-decision review confirming later agents can classify switches | [Implementation workflow](02-implementation-and-review.md) step 3 | P0-V09/P0-V15 (W04-DV04) |
| Document discoverability and P1 consumability | [Implementation workflow](02-implementation-and-review.md) steps 2, 4 | P0-V09/P0-V15 (W04-DV05, DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at merge 4e631ee): the only governance-
relevant tracked statements are the ADR register entries (ADR-047, ADR-037,
ADR-046), the task book's Reserved profile list, and the scaffold directories.
There is no Cargo manifest, no feature, no profile section, and no tracked
document that tells an agent how to classify a proposed switch. W01 is
completed; W02 and W03 are proposed designs. Each ledger row below states the
missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Non-overlapping governance exists (P0-V09) | No repository document defines switch classes or boundaries | Normative governance document with class definitions, a precedence rule, and a classification procedure | "Non-overlapping" must be a stated contract, not reviewer folklore | W04 (this design) | W04-DV01 policy review |
| Switch classes are identifiable (binary capability vs profile vs runtime policy) | ADR-047 states the principle only; no operational criteria | Classification procedure with ordered questions an agent can apply | An agent must reach a verdict without re-deriving ADR semantics | W04 within ADR-047's semantics | W04-DV01/DV04 |
| Reserved profile set with purposes and boundaries | Names appear in ADR-047 and the task book; no purpose, applicability, or non-fork statement | Profile registry section: one-line purpose and applicability per profile, Reserved status, non-fork rule | The task book requires the set to stay available; purposes prevent drift into architecture forks | ADR-047 set; W04 elaboration | W04-DV02 |
| Review questions and prohibited cases | Absent | Review-question checklist plus the plan's five prohibited categories (VM count, memory, vCPU, affinity, device selection) with informative examples | The plan requires agents to be able to reject misclassified switches at review | W04 | W04-DV03 |
| Later agents can classify a switch (P0-V15) | Not exercised anywhere | Representative-decision drill across all three classes plus one prohibited case | Acceptance requires demonstrated classifiability, not just rules | W04 | W04-DV04 drill evidence |
| Downstream consumers (W03, W07, W16, W12, P1+) can rely on it | No contract to rely on | Handoff statements per consumer | Consumers select builds, gates, metadata dimensions, and visibility via this vocabulary | W04 delivers; consumers own their contracts | W04-DV06 |

No row requires creating a manifest, feature, profile implementation, or
runtime schema, so no decision blocker is outstanding for this design. W01 is
the only implemented prerequisite and is delivered; no other prerequisite
blocks this package.

## Resolved design decisions and their authority

1. **Normative home.** `docs/development/build-profile-governance.md` is the
   sole normative home of switch classification, the profile registry, review
   questions, prohibited cases, and change thresholds; discovery is one
   `docs/README.md` routing row. This mirrors the approved W02/W03 pattern
   (policy in `docs/development/`, index routing, no policy duplication).
2. **Three-class taxonomy with a precedence rule.** Binary capability (Cargo
   feature), build profile, and runtime resource/policy are the only switch
   classes (§3). When a proposal mixes classes, the runtime-policy reading
   wins: a switch that cannot be cleanly assigned is split or rejected, never
   approximated as a feature.
3. **Profiles are a project concept, not Cargo sections.** The ADR-047
   profiles are deployment/research build selections over features; Cargo's
   built-in `dev`/`release` are build configurations, not members of the
   reserved set. No `[profile.*]` customization exists in P0; W03's baseline is
   compliant by construction.
4. **Reserved registry elaboration.** All six profile names are recorded
   Reserved with one-line P0-level purpose and applicability statements
   (§5.1). These statements are elaborations of ADR-047, explicitly marked as
   refinable by the design that first implements a profile; they do not amend
   the ADR.
5. **Non-fork rule.** Profile differences enter code only through features a
   profile may select; hypervisor Core must never branch on profile identity,
   and a profile must never imply architectural divergence (§5.2). This is the
   plan's "避免把 profile 变成架构分叉" requirement.
6. **Prohibited cases.** The five plan-named categories — VM count, memory
   sizing, vCPU count, CPU affinity, and device selection — are prohibited as
   features and as profile-intrinsic constants; they belong to the future
   runtime configuration model (ADR-037). Illustrative hypothetical examples
   are marked informative so they are never read as existing switches (§7).
7. **Change thresholds.** Adding a classified feature is routine; introducing
   custom `[profile.*]` sections, activating a reserved profile, or
   reclassifying a switch is a reviewed policy decision; making runtime policy
   compile-time or allowing profile-specific architecture forks is ADR
   required (§8).
8. **Boundary pointers, not duplication.** Targets (W03), toolchain (W02),
   dependencies (W18), build identity (W16), and diagnostic visibility (W12)
   are referenced by pointer; this document does not restate their rules.

## Work breakdown and loading order

1. Read [the governance contract](01-governance-contract.md) for the artifact
   groups, the required document sections, and the mutation rules.
2. Apply the changes in the order stated in the
   [implementation workflow](02-implementation-and-review.md): write the
   governance document, wire discovery, run the representative-decision drill,
   then close.
3. Store actual commands, output, environment, and result in
   `../../verification/p0-w04-build-profile-feature-governance-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w04-build-profile-feature-governance-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W04 complete.

## Design-level state and lifecycle

W04 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked policy document plus its discovery links.
Its documentary lifecycle:

```text
no switch governance
  -> build-profile-governance.md committed (classes, registry, thresholds)
  -> docs/README.md routes switch-classification work to it
  -> representative-decision drill evidenced
  -> later packages mutate only through the document's thresholds
     (W03/W08/P1 declare features only after classifying them; W07 may gate
      profile selections; W16 records the profile dimension; W12 aligns
      visibility rules with profile semantics)
```

The governance document owns every classification statement. A conflict
between it and any manifest-level switch introduced later is a review failure,
not a local choice.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, public API, ABI, Cargo feature,
`[profile.*]` section, runtime configuration schema, or code-organization
convention is designed or authorized by W04; see
[the governance contract](01-governance-contract.md) §9. The only machine-
facing surface this package touches is none: it adds no manifest and no code.

## Downstream handoff

- **W03** receives the confirmation that its feature-free, custom-profile-free
  baseline is compliant, and the rule that any future profile or feature on
  the hypervisor member is classified here first. W04 imposes no additional
  constraint on W03's target choice.
- **W07** receives the class vocabulary for gate matrices: a gate that builds
  a specific capability/profile selection must name the class it exercises;
  gate failure semantics remain W07's.
- **W16** receives the profile identifier as a metadata dimension with the
  vocabulary owned here; W16 owns how it is recorded and versioned.
- **W12** receives the boundary that release visibility and trimming
  decisions reference profile semantics defined here, not ad-hoc cfg names.
- **P1 and later stages** receive the classification procedure: every proposed
  Cargo feature, profile activation, or resource-policy switch in a P1+ design
  must carry its classification and review per this document; a runtime
  quantity that cannot pass §6 is a design defect to fix in the design, not a
  feature to add.
- **W21** may later reference this document as an instance of the
  plan-to-design-to-code admission rules it defines; W04 does not define the
  workflow itself.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
