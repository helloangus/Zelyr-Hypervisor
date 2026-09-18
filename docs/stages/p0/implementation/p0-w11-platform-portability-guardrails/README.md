# P0-W11 Platform Portability Guardrails — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Core/Arch/SoC/Board-BSP/Quirk isolation rules, capability-driven
selection rules, review checklists, and documentation constraints required by
[P0-W11](../../plans/p0-w11-platform-portability-guardrails.md).  
**Owner/change context:** P0-W11 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W11. It converts the bounded
work-package plan into one normative portability-rules document, minimal
discovery wiring, and a platform-difference walkthrough. It deliberately does
**not** design a Platform trait, a DTB parser, a BSP API, or any QEMU/RK3566
driver (plan out-of-scope), does **not** define the PlatformInfo schema or any
PlatformCapabilities field (owned by the designs that introduce them, first in
P1/P2), does **not** create or gate any code (no code exists in the tree, and
gate machinery belongs to [W07](../p0-w07-development-quality-gates/README.md)
and [W20](../p0-w20-ci-baseline/README.md) by their plan scopes), and does
**not** re-decide any ADR layering principle.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task
book, and P0-W11 plan. This document is the proposed detailed design for those
changes; it is not a completion record and contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W11 plan → this design
→ Coding Guidelines. In particular:

- The ADR fixes the semantics these guardrails operationalize: principle 7
  (portability over board special-casing), ADR-041 (Arch / SoC / Board /
  Firmware / Discovery / Driver / Quirk layering), ADR-042 (discovery sources
  convert to PlatformInfo; Core never parses raw platform descriptions),
  ADR-043 (no board-conditional branches in Core), ADR-044 (selection by
  PlatformCapabilities, never by platform name), ADR-052 (rejected:
  board-name-driven core logic), and the implementation invariants (Core MUST
  NOT depend on board names; Arch MUST NOT depend on SoC/Board). The rules
  document implements these constraints; it does not amend them.
- The task-book outcome for W11 is: Core/Arch/SoC/Board separation and
  capability-driven platform rules are **reviewable** (P0-V12: no rule permits
  board/QEMU dependencies in Core or Board dependencies in Arch). Reviewability
  is the deliverable, so the rules must be applicable by a reviewer without
  reading the ADR's whole stage tree.
- Prerequisite status: [W05](../p0-w05-documentation-baseline/README.md)
  (document conventions) and [W06](../p0-w06-adr-governance/README.md) (label
  and escalation semantics) are proposed designs, not deliveries. The current
  tree already mandates status headers via `docs/README.md` and conflict labels
  via `AGENTS.md`, so W11 can implement against that existing mandate; when
  delivered W05/W06 rules differ, they take precedence, and a substantive
  conflict is raised rather than absorbed (workflow §1).
- W11 delivers rules, not enforcement. Any mechanical check derived from these
  rules (for example a platform-name search over Core positions) is a future
  gate owned by W07's future-class promotion and W20's wiring; this design
  defines the predicate a future check would verify, never the gate.

Classification: the portability-rules document (layer dependency rules,
platform-identity prohibition, capability-selection rule, quirk boundary,
documentation constraints, review checklists, counterexamples, change
thresholds) and its discovery wiring are **Required** for W11 closure. A
mechanical portability check, support-tier classification content (ADR-045's
subject), PlatformCapabilities field definitions, and an `AGENTS.md` routing
amendment are **Reserved** with recorded owners and triggers. Platform traits,
DTB parsing, BSP APIs, QEMU/RK3566 drivers, PlatformInfo schema, CI workflows,
crates, module trees, and all code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| ADR layering converted into reviewable engineering rules and counterexamples (work item 1) | [Rules contract](01-portability-rules-contract.md) §2.1, §2.8 | P0-V12 (W11-DV01) |
| Core/board-QEMU and Arch/SoC-Board prohibitions; quirk allowed boundary (work item 2) | [Rules contract](01-portability-rules-contract.md) §2.2, §2.4 | P0-V12 (W11-DV02) |
| Capability-based selection review requirements (work item 3) | [Rules contract](01-portability-rules-contract.md) §2.3, §2.6 | P0-V12 (W11-DV03) |
| Discoverability from documentation, review checklist, and agent entry points (work item 4) | [workflow](02-implementation-and-review.md) step 3 | P0-V09/P0-V12 (W11-DV05) |
| Downstream consumability by P1+ platform, memory, IRQ, and BSP work | [workflow](02-implementation-and-review.md) handoff checklist | P0-V12/P0-V15 (W11-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at `4e631ee`): the layering rules exist only
as ADR principles, register entries, and one-line agent mandates — no tracked
document turns them into per-layer engineering rules a reviewer can apply.
`docs/platform/README.md` is a short informative placeholder naming platform
contracts as its subject; it contains no rules. There is no Rust source
anywhere, so no violation can exist yet, and no review checklist for
platform-dependency questions exists in any tracked file. W01 is completed;
W05/W06 sibling designs (prerequisites) are proposed; several other sibling
designs exist as untracked parallel work and are referenced by slug only.
Each ledger row below states the missing foundation the plan outcome
necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Layering isolation rules are reviewable before platform code exists | ADR principles and invariants only; no per-layer engineering rules | Portability-rules document fixing the layer model, dependency directions, and prohibitions | "固化" (solidified) requires one authoritative rule set, applicable without re-deriving the ADR | W11 (this design) within ADR-041/ADR-043/§19 semantics | W11-DV01 rule review |
| Core must not depend on board/QEMU; Arch must not depend on SoC/Board | Stated as ADR invariant text only; no definition of what "depend" prohibits (identity references, constants, cfg, strings) | Prohibition sections enumerating the forbidden mechanisms per layer | A red line that does not name its forbidden mechanisms cannot be checked at review | W11 elaboration of ADR-043/§19 | W11-DV02 |
| Quirk allowed boundary | ADR-041 names the Quirk layer; no boundary rules exist | Quirk-boundary section: what a quirk may declare, what it may never change | Without a boundary the quirk layer becomes a backdoor for platform branching | W11 | W11-DV02 |
| Capability-based selection is a review requirement | ADR-044 states the principle; no review requirement operationalizes it | Selection rule plus design-review questions that reject platform-name selection | Reviewers must be able to reject a misclassified behavior difference mechanically | W11 within ADR-044 | W11-DV03 |
| Rules and counterexamples exist together (work item 1) | No examples anywhere | Informative, clearly-labeled violation examples with compliant alternatives | Boundary illustrations make abstract prohibitions applicable | W11 | W11-DV01/DV04 |
| Docs, review checklist, and agent entry points can discover the rules (work item 4) | No routing row; `docs/platform/README.md` placeholder; checklists absent | `docs/README.md` routing row, platform-directory pointer, stage implementation index row | Work item 4 requires discovery from all three surfaces | W11 | W11-DV05 |
| P1+ platform/memory/IRQ/BSP work can consume the red lines | Nothing to consume | Handoff statements naming how consumer designs must cite the rules | Handoff is the plan's stated purpose (向 P1+ 提供分层红线) | W11 delivers; consumers cite | W11-DV06 |

No row requires inventing a trait, parser, driver, crate, or platform
constant, so no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Normative home:** `docs/development/platform-portability-rules.md` is the
   sole normative home of the layering guardrails, following the established
   P0 pattern (policy documents under `docs/development/`: W02 toolchain,
   W04 build governance, W05 documentation baseline, W07 quality gates).
   [W05](../p0-w05-documentation-baseline/README.md) may later re-home it;
   semantic ownership stays with the document. One pointer line is added to
   `docs/platform/README.md` so platform-contract work finds the rules.
2. **Layers are architecture concepts, not crates or module names.** The rules
   fix dependency directions between the logical layers Core, Arch, SoC,
   Board/BSP, Quirk, Driver, and Discovery/Firmware inputs. They name no crate,
   module, or file, because workspace member boundaries are owned by
   [W03](../p0-w03-aarch64-build-target-baseline/README.md) and later designs;
   any future crate layout must satisfy the logical rules.
3. **Prohibition formulated as platform-identity containment.** Platform
   identity (board, SoC, vendor, machine, and QEMU/virt names) may be
   referenced only inside the BSP/Quirk layer and inside discovery inputs;
   Core and Arch must not reference it through any mechanism — branching,
   conditional compilation, string matching, constants, or features. Core must
   additionally never consume raw platform descriptions (ADR-042): it sees
   only the normalized platform description, whose schema is owned by its
   future design.
4. **Capability-driven selection as a review rule.** Any behavior difference
   between platforms must be expressed as a query against a declared
   capability or property of the platform description. The design-review
   checklist requires every such difference to name the capability it queries;
   introducing a new capability field is a design-level decision owned by the
   introducing design, never a local review-time choice.
5. **Quirk boundary.** A quirk is a declared, localized, documented correction
   for an observed hardware/firmware deviation. It must live in the BSP/Quirk
   layer, name the deviation and its activation condition, and may never alter
   architectural semantics, Core invariants, or platform capabilities. A
   "quirk" used to route Core behavior by platform name is a violation of the
   platform-identity prohibition, not a quirk.
6. **Checklists live inside the rules document.** One design-review checklist
   and one code-review checklist, each item individually checkable, are
   sections of the rules document (§2.6–§2.7 of the contract) so reviewers
   have exactly one surface. Mechanical techniques (searching for platform
   names) are suggested observations, not gates.
7. **Counterexamples are informative and hypothetical.** Each example shows a
   violation and its compliant alternative and is labeled as hypothetical so
   it is never read as existing code (§2.8 of the contract).
8. **Change thresholds.** Declaring a quirk or BSP artifact inside the rules is
   routine reviewed change; adding an inter-layer boundary case or
   reclassifying a layer's allowed dependencies is a policy decision;
   permitting platform-name branching in Core, an Arch→SoC/Board dependency,
   or Core parsing of raw platform descriptions contradicts accepted ADR
   decisions and goes to the ADR path. Escalation uses W06's mechanism once
   delivered; until then the `AGENTS.md`/`docs/README.md` label mandate
   applies.

## Work breakdown and loading order

1. Read [the rules contract](01-portability-rules-contract.md) for the
   artifact groups, the required sections of the rules document, the
   checklists, and the counterexample format.
2. Apply the changes in the order stated in
   [the implementation workflow](02-implementation-and-review.md): verify
   prerequisite surfaces, author the rules document, wire discovery, run the
   platform-difference walkthrough, then close.
3. Store actual review commands, output, environment, and result in
   `../../verification/p0-w11-platform-portability-guardrails-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w11-platform-portability-guardrails-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W11 complete.

## Design-level state and lifecycle

W11 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked rules document plus its discovery links.
Their documentary lifecycle:

```text
layering stated only as ADR principles and agent one-liners
  -> platform-portability-rules.md committed (layers, prohibitions,
     capability rule, quirk boundary, checklists, counterexamples, thresholds)
  -> docs/README.md routing row + docs/platform/README.md pointer committed
  -> platform-difference walkthrough evidenced (hypothetical difference
     classified by the rules, no code written)
  -> P1+ platform/memory/IRQ/BSP designs cite the red lines at design entry
  -> future mechanical check derived from the rules' predicate is promoted
     through W07's future class if ever designed
  -> rules change only through the document's thresholds
```

The rules document owns every guardrail statement; `docs/platform/README.md`
and the routing row point at it and never restate it. A platform-conditional
design or code fragment that contradicts the rules is a review failure, not a
local choice.

## Explicitly excluded interfaces

No Platform trait, PlatformInfo or PlatformCapabilities schema or field, DTB
parser, BSP or quirk API, driver, crate, module tree, Rust type, function,
public API, ABI, target, or CI workflow is designed or authorized by W11. The
layers in the rules document are logical architecture concepts; mapping them
onto crates or modules belongs to the designs that create those artifacts. If
implementing W11 appears to require naming any of these, that is a scope
boundary: stop and record.

## Downstream handoff

- **P1+ platform, memory, IRQ, and BSP designs** (per their stage plan
  indexes) receive the red lines: each such design must show that
  platform-conditional behavior is expressed as capability queries and that
  platform knowledge sits in the allowed layers; a design that cannot is
  defective at design review, not at code review.
- **W10** receives the placement constraint hook it reserved: unsafe boundary
  categories must remain consistent with these layering rules; conflicts go to
  the escalation thresholds, not to local exceptions.
- **W07 and W20** receive the predicate a future mechanical portability check
  could verify (platform-identity containment per §2.2 of the contract);
  classification and wiring stay with them.
- **Future platform-contract work under `docs/platform/`** receives the
  documentation constraints: platform documents state which layer owns each
  behavior; no document may grant an informal exemption from the prohibitions.
- **W05** may re-home the rules document under its documentation taxonomy.
- **W22** maps this deliverable into the P1 handoff package by its own plan
  scope; W11 does not write the map.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
