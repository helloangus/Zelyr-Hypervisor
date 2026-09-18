# P0-W18 Dependency Governance — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The dependency evaluation criteria, review tiers, recording
requirements, and TCB-entry rules required by
[P0-W18](../../plans/p0-w18-dependency-governance.md).  
**Owner/change context:** P0-W18 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W18. It converts the bounded
work-package plan into small, reviewable documentation and policy changes: one
normative dependency-governance policy document, one dependency register
carrying the entry schema and starting empty, documentation-discovery wiring,
and a completeness rehearsal that walks a hypothetical future candidate through
the review path without selecting any real dependency. It deliberately does
**not** approve or reject any crate, add any dependency, design a dependency
wrapper API, create a Cargo manifest or workspace, decide lockfile mechanics,
or configure CI. Those belong to the consuming designs (P1+ dependency
decisions), the build-baseline packages, and W20 respectively.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
[the governance policy](01-governance-policy.md) for classes, criteria, and
thresholds, [the register schema](02-dependency-register-schema.md) for the
record formats, and [the implementation workflow](03-implementation-and-review.md)
for ordered steps and the validation matrix. Before editing it must also follow
the Coding Guidelines preflight, including the repository `AGENTS.md`,
documentation index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P0 task book](../../task-book-v0.1.md), and the P0-W18 plan. This document is
the proposed detailed design; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W18 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W18 is: dependencies are **evaluated for TCB,
  no_std, license, maintenance, unsafe, and platform risk** (P0-V09: the
  governance path and TCB review dimensions are locatable).
- ADR-006 permits depending on mature crates and requires reviewed `unsafe`;
  dependency use is therefore legitimate — W18's job is to prevent
  *unreviewed expansion of hypervisor-TCB dependencies*, the plan's stated
  goal, not to discourage dependency use.
- The plan's out-of-scope clause forbids approving or rejecting any specific
  future crate and designing dependency wrapper APIs; the policy and register
  therefore fix *how decisions are made and recorded*, never which crate is
  chosen.
- W10 (unsafe Rust governance) owns the project's unsafe justification and
  inventory rules; third-party unsafe cannot be modified by this project, so
  W18's unsafe dimension *assesses* a candidate's unsafe footprint and
  references W10's boundary classes without owning or restating W10's
  mechanics. W10 is a prerequisite by subject; its delivery may still be a
  parallel proposed design.
- W01 resolved the project license (Apache License 2.0, root `LICENSE`) and
  explicitly left dependency and third-party notice analysis to W18; the
  license-compatibility dimension is therefore grounded, and notice analysis
  is W18's obligation once real dependencies exist.
- The Cargo workspace and manifests do not exist yet; the register↔manifest
  consistency rule is written to bind the moment a manifest first appears
  (expected from the build-baseline package per the plan index), and lockfile
  mechanics are explicitly deferred to that owning design.

Classification: the evaluation criteria, review tiers and approval rules, TCB
entry rules, ADR thresholds, lifecycle records, and the register schema are
**Required** for W18 closure. The register's first real entries (arrive with
the first real dependency), automated register↔manifest checking (candidate
future W07/W20 gate), license-notice production (first real dependency), and
any vendoring/mirroring policy are **Reserved** with recorded triggers.
Crate selection, wrapper APIs, Cargo manifests, lockfile mechanics, CI, and
any W10 mechanic restatement are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Evaluation criteria for every candidate (work sequence 1) | [Governance policy](01-governance-policy.md) §4 | P0-V09 (W18-DV01) |
| Review tiers: development / host-only / hypervisor-TCB (work sequence 2) | [Governance policy](01-governance-policy.md) §3 | P0-V09 (W18-DV02) |
| Records for introduction, upgrade, deprecation, exception (work sequence 3) | [Register schema](02-dependency-register-schema.md) | P0-V09 (W18-DV03) |
| Rule completeness rehearsal without selecting a dependency (work sequence 4) | [workflow](03-implementation-and-review.md) step 5 | P0-V09 (W18-DV04) |
| Governance path and TCB dimensions locatable (P0-V09) | [workflow](03-implementation-and-review.md) step 4 | P0-V09 (W18-DV05) |
| Consistency with W10 unsafe governance | [Governance policy](01-governance-policy.md) §6.2 | P0-V09/P0-V11 interface (W18-DV06) |
| Downstream consumability by P1+ dependency decisions | [workflow](03-implementation-and-review.md) handoff checklist | W18 closure review (W18-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, tracked tree at `4e631ee`): no Cargo
workspace, manifest, lockfile, vendored source, or dependency of any kind
exists; the repository is a documentation scaffold. Root `LICENSE` contains
the owner-approved Apache License 2.0 text (W01). No tracked document states
dependency evaluation criteria, review tiers, or a TCB-entry rule. W05/W06/W10
have approved plans without designs (W06's ADR-governance escalation path and
W10's unsafe inventory location are therefore not yet in-tree); sibling
designs are being prepared in parallel and are referenced by slug and P0-Wxx
ID without assumed content. Each ledger row states the missing foundation the
plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Dependencies are evaluated for TCB, no_std, license, maintenance, unsafe, and platform risk | No evaluation criteria anywhere | Policy document with a fixed evaluation checklist covering the task-book dimensions plus transitive dependencies, allocation, stability, and security posture | An evaluation that is not written down cannot be applied consistently or reviewed | W18 (this design) | W18-DV01 policy review |
| TCB expansion is prevented from being unreviewed | No TCB-entry rule anywhere | Review tiers with an elevated boundary for anything entering a bare-metal image or the hypervisor TCB, plus fail-closed default | Without a tier boundary, a TCB dependency could enter through an ordinary review | W18 tiers | W18-DV02 tier review |
| Evaluation, recording, update/review responsibility defined (plan scope) | Absent | Register with entry schema and lifecycle event records; approval authority per tier | Unrecorded decisions cannot be audited, upgraded, or revoked | W18 register | W18-DV03 register review |
| Update, deprecation, and exception handling leave records | Absent | Event record templates with re-evaluation triggers, advisory path, and expiring exceptions | Drift between an approved dependency and its current version is the main slow-moving risk | W18 register | W18-DV03 |
| Rules are complete without selecting a dependency (work sequence 4) | Nothing to select | Documented hypothetical-candidate rehearsal in verification | A rule set never exercised cannot claim completeness | W18 rehearsal | W18-DV04 |
| Governance path locatable (P0-V09) | No document to locate | Policy + register + discovery routing row | P0-V09 is a locatability criterion | W18 wiring | W18-DV05 |
| License dimension grounded | Apache-2.0 in root `LICENSE` (W01) | Policy references `LICENSE` and assigns notice analysis to W18 when the first dependency lands | License compatibility is decidable today; notice production needs a real dependency | W18 dimension; W01 license | W18-DV01 |

No row above requires approving a crate, selecting a dependency, or designing
a wrapper API, so no decision blocker is outstanding for this design. Two
coordination points are recorded rather than resolved: W10's inventory
location is undefined until W10 delivers (the policy references the *rules by
plan scope*, and the cross-review step records blocked status if needed), and
lockfile mechanics belong to the owning build-baseline design (see [the
governance policy](01-governance-policy.md) §6.3).

## Resolved design decisions and their authority

1. **Policy home:** `docs/development/dependency-governance.md` is the sole
   normative home of the evaluation criteria, review tiers, approval rules,
   and ADR thresholds. Rationale: development-policy precedent under
   `docs/development/`; W05 may re-home it later without changing semantic
   ownership.
2. **Register home:** `docs/development/dependency-register.md` is the sole
   authoritative list of approved dependencies, their tiers, and their
   lifecycle records, starting empty. Rationale: a policy without a decision
   ledger cannot prove that a dependency was reviewed; placing it beside the
   policy keeps the review path one link long. W05 may re-home.
3. **Review tiers:** three tiers — D1 host-only development dependency
   (never linked into any repository-produced runtime image), D2 general
   project dependency (any reviewed dependency outside D1/D3), D3
   hypervisor-TCB candidate (linked into any repository-produced bare-metal
   image — hypervisor or validation guest — or otherwise part of the
   hypervisor TCB). Rationale: the plan requires exactly this distinction
   (general development, host-only, elevated TCB boundary); D3's
   bare-metal-image boundary is grounded in the Coding Guidelines' no_std
   rules for hypervisor and guest runtime code.
4. **Fail-closed default:** any dependency not present in the register is
   unapproved; once manifests exist, a manifest dependency absent from the
   register is a review failure. Rationale: the plan's goal is preventing
   *unreviewed* expansion; allowlisting is the only form of prevention that
   survives time and personnel changes.
5. **Approval authority:** D1 requires a register entry approved in ordinary
   PR review; D2 requires the full evaluation checklist recorded; D3 requires
   the full checklist plus an explicit maintainer approval decision recorded
   in the register. Rationale: proportionality — the elevated boundary must
   be visible in who can say yes, not only in what is assessed.
6. **ADR thresholds:** introducing a D3 dependency is within ADR-006's
   permission and needs no ADR; `ADR Required` is triggered when a dependency
   would introduce an alternative runtime or execution model into EL2, alter
   crate layering, or capture a core abstraction so that replacing it later
   is an architectural change. Rationale: ADR-006 already permits mature
   crates; the escalation is for dependency choices that *are* architecture.
7. **Unsafe dimension vs W10:** the policy assesses a candidate's unsafe
   footprint (quantity, posture, alignment with W10's boundary classes) and
   records that third-party unsafe cannot be modified or inventoried by this
   project — it is weighted, not adopted. W10's inventory remains the owner
   of project unsafe only. Rationale: keeps W18 from restating or pre-empting
   W10's mechanics.
8. **Lockfile stance deferred:** the policy requires that dependency versions
   be recorded and reproducible (the register's version pinning plus the
   future build baseline's mechanism); concrete lockfile handling is owned by
   the build-baseline design that introduces the manifest (expected W03 per
   the plan index; not yet delivered). Rationale: W18 must not make build
   mechanics decisions outside its scope, but must not leave reproducibility
   unstated either.

## Work breakdown and loading order

1. Read [the governance policy](01-governance-policy.md) (artifact groups,
   tiers, checklist, thresholds, ADR rules, consistency rules) and
   [the register schema](02-dependency-register-schema.md) (entry schema,
   event templates, maintenance rules).
2. Apply the changes in the order stated in
   [the implementation workflow](03-implementation-and-review.md): record
   prerequisite status assumptions, write the policy, create the empty
   register, wire discovery, run the completeness rehearsal, then close with
   the validation matrix.
3. Store actual rehearsal notes, review commands, and run/blocked status in
   `../../verification/p0-w18-dependency-governance-verification.md`, and
   record changed artifacts and any deviation in
   `../p0-w18-dependency-governance-record.md` only when implementation
   begins. Neither this design nor a written record may claim W18 complete.

## Design-level state and lifecycle

W18 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is two tracked documents (policy + register) plus their
discovery links. Their documentary lifecycle:

```text
no dependency rules
  -> dependency-governance policy committed (tiers, checklist, thresholds)
  -> dependency register committed, empty, schema in place
  -> discovery links (docs/README routing row, stage index row) committed
  -> completeness rehearsal recorded against a hypothetical candidate
  -> first real dependency enters only through: checklist -> tier decision
     -> register entry -> (once manifests exist) manifest reference
  -> upgrades/advisories/deprecations/exceptions append event records
  -> P1+ dependency decisions consume the path without redefining it
```

The policy owns every rule; the register owns every decision record. A
manifest (once one exists) may contain only register-approved dependencies;
the register never approves anything the policy's checklist has not recorded.
A conflict between a future consuming design and the policy is a review
failure to reconcile in the same change; an escalation that touches
architecture is labelled `ADR Required` per the policy's thresholds.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate name, Cargo manifest, workspace
layout, lockfile rule, wrapper API, vendoring mechanism, or public API is
designed or authorized by W18, and no specific dependency is approved,
rejected, or recommended. The only machine-facing surface defined is the
register's record schema as policy; the only human-facing procedure is the
evaluation and rehearsal path. Adding any excluded item is a scope conflict
requiring the applicable detailed design (at minimum the build-baseline
package for manifests and lockfile, W07/W20 for automated checks) and must be
stopped at review.

## Downstream handoff

- **P1+ dependency decisions** (every future design that reaches for a crate)
  receive the mandatory path: classify the tier, run the checklist, obtain the
  tier's approval, record the entry, then reference the dependency in the
  consuming design. P1's no_std runtime work is the first expected D3
  consumer.
- **The build-baseline package** (expected W03 per the plan index) receives
  the consistency rule: the first manifest must contain only register-approved
  dependencies, and lockfile mechanics are that design's to define with the
  reproducibility requirement stated in the policy.
- **W10** receives the boundary statement that third-party unsafe is assessed
  and weighted here, while project unsafe remains W10's inventory; the
  cross-review keeps the two documents aligned without either owning the
  other.
- **W07/W20** may later consume automated register↔manifest consistency or
  advisory monitoring as candidate checks; W18 defines no check
  implementation.
- **W01's license decision** is consumed by the license dimension; the
  first real dependency triggers W18's notice-analysis obligation (Reserved).
