# P0-W10 Unsafe Rust Governance — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The unsafe justification rules, inventory location and schema,
allowed boundary categories, review rules, and safe-Rust-first principle
required by
[P0-W10](../../plans/p0-w10-unsafe-rust-governance.md).  
**Owner/change context:** P0-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W10. It converts the bounded
work-package plan into two normative documents — the unsafe Rust policy and
the unsafe inventory — plus minimal discovery wiring. It deliberately does
**not** decide any concrete wrapper, register interface, pointer API, or
assembly implementation (plan out-of-scope), does **not** write any Rust code
or introduce any `unsafe` (none can exist: the tree has no sources), does
**not** define CI checks ([P0-W07](../p0-w07-development-quality-gates/README.md)
owns the future gate that will consult this policy), and does **not** judge
third-party dependency unsafe ([P0-W18](../p0-w18-dependency-governance/README.md)
owns dependency intake and covers transitive unsafe there).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), whose
unsafe rules this policy operationalizes for this repository. It then loads
only the linked supporting file needed for its assigned step. Before editing
it must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task
book, and P0-W10 plan. This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W10 plan → this
design → Coding Guidelines. In particular:

- The task-book outcome for W10 is: unsafe justification, inventory, review,
  and boundary rules exist **before EL2 code** (P0-V11: policy and inventory
  location exist and are usable for a first unsafe change). The deliverable
  is therefore governance, reviewable without any code.
- ADR-006 permits necessary assembly and controlled `unsafe`, requires it
  concentrated at arch/HAL/low-level-data-structure boundaries, and requires
  an audit list. ADR-049 names unsafe audit as a standing validation layer.
  The policy must implement these constraints, not re-decide them.
- The [Coding Guidelines](../../../../development/coding-guidelines.md)
  unsafe rules (nearby `SAFETY` explanations, small audited boundaries, no
  `static mut`, no casual `transmute`, no borrow-checker evasion) are binding
  inputs; the policy fixes how this repository applies them.
- P0-W14, panic/failure classification (a prerequisite
  by its consumer list) supplies the failure-class semantics the policy's
  failure-guarantee statements reference; P0-W11, platform portability
  guardrails, supplies the layering constraints unsafe placement must respect;
  P0-W06, ADR governance, supplies the escalation path.
  These sibling designs are being prepared per the plan index; the policy
  references their subjects, not their content, so authorship is not blocked
  (workflow §1 covers the boundary cases).

Classification: the policy document (safe-Rust-first principle, justification
and SAFETY-comment requirements, boundary categories, forbidden patterns,
review rules, escalation thresholds) and the inventory (location, schema,
lifecycle) are **Required** for W10 closure. Any concrete unsafe segment,
wrapper API, register interface, ASM sequence, audit-gate implementation, and
third-party-unsafe judgment are **Reserved** to the designs that introduce
their subjects (P1+ and W18). All code, crate boundaries, module trees, and
CI wiring are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Per-unsafe justification: safety preconditions, responsible owner, review record | [Policy contract](01-unsafe-policy-contract.md) §3–§4 | W10-DV01 |
| Inventory location, update timing, auditable fields | [Inventory contract](02-inventory-contract.md) §2–§4 | P0-V11 (W10-DV02/DV03) |
| Allowed boundary categories; anti-expansion rules | [Policy contract](01-unsafe-policy-contract.md) §5–§6 | W10-DV01 |
| Review rules and safe-Rust-first principle | [Policy contract](01-unsafe-policy-contract.md) §4, §7 | W10-DV01 |
| Institution usable for a first future unsafe change | [workflow](03-implementation-and-validation.md) step 4 walkthrough | P0-V11 (W10-DV04) |
| Document discoverability and coherence | [workflow](03-implementation-and-validation.md) step 5 | P0-V09 (W10-DV06) |
| Downstream handoff to P1+ low-level work and W18 | [workflow](03-implementation-and-validation.md) handoff checklist | W10 closure (W10-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p0-implementation-designs` at
`4e631ee`): no unsafe policy or inventory document exists; `docs/security/README.md`
is a two-line informative placeholder naming unsafe inventory/audits as its
subject; there is no Rust source anywhere, so the tree contains zero `unsafe`
trivially; no CI exists that could enforce a gate. W01 is completed with
recorded verification; W05/W06 sibling designs (prerequisites) are being
prepared per the plan index, so the documentation taxonomy and ADR-change
mechanics they will deliver are references by subject, not by content. Each
ledger row below states the missing foundation the plan outcome necessarily
requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every non-trivial unsafe segment has defined justification content | No unsafe rules beyond the Coding Guidelines' general text | Policy sections fixing the justification statement and SAFETY-comment template | "制度可用" requires the exact record content, not principles alone | W10 (this design) | W10-DV01 review |
| Inventory location and update timing exist | No inventory; only an informative directory placeholder | Inventory document at the declared location with schema, lifecycle, and update rules | P0-V11 requires a usable inventory location before the first unsafe | W10 | W10-DV02/DV03 |
| Auditable inventory fields | Absent | Field-by-field schema contract | Auditability is a property of the fields, not the file | W10 | W10-DV03 |
| Allowed boundary categories | Absent | Category list tied to ADR-006's concentration constraint, with placement rules | Categories make review decisions repeatable and anti-expansion enforceable | W10 | W10-DV01 |
| Anti-expansion and safe-Rust-first rules | Absent | Necessity-statement, minimality, encapsulation, and forbidden-pattern rules | The plan forbids convenience-driven unsafe growth | W10 | W10-DV01 |
| Usable for a first unsafe change without re-deciding basics | Unverifiable until exercised | Paper walkthrough of a hypothetical first unsafe change through design → comment → inventory → review | P0-V11's "usable" is a rehearsal property | W10 | W10-DV04 |
| Downstream consumers can bind | No gate, no dependency policy yet | Handoff statements to W07 (future gate), W18 (third-party split), P1+ designs | Consumers must know what this policy does and does not govern | W10 | W10-DV07 |

No row requires inventing code, APIs, or crate boundaries, so no decision
blocker is outstanding for the design itself. The W05/W06/W11/W14 references
carry the standard sibling-design assumption and failure boundary (workflow §1).

## Resolved design decisions and their authority

1. **Policy home:** `docs/security/unsafe-rust-policy.md` is the sole
   normative home of unsafe governance, matching the directory's declared
   role (unsafe inventory/audits, security reviews). [P0-W05](../p0-w05-documentation-baseline/README.md)
   may re-home it; semantic ownership stays with the document.
2. **Inventory as data, policy as schema:** `docs/security/unsafe-inventory.md`
   is created empty (zero entries — none can exist), carrying entries only;
   the entry schema, lifecycle, and update rules live in the policy document
   and are summarized at the inventory's head by pointer. This mirrors the
   W02 manifest/contract split: one owner per statement.
3. **SAFETY comment template:** the policy fixes a required-content template
   (precondition, establishment argument, failure-class consequence per
   W14's taxonomy, inventory back-link). This is documentation governance
   within the plan's scope (每段非平凡 unsafe 必须记录), implementing the
   Coding Guidelines' SAFETY-comment rule, not a code decision.
4. **Boundary categories:** six allowed categories derived from ADR-006's
   concentration constraint (arch registers, MMIO/volatile, memory-management
   primitives, low-level data structures, ASM glue, boot-state
   establishment). Each category carries placement constraints that defer to
   W11's layering rules; anything outside the categories requires a policy
   decision, and anything contradicting ADR-006 requires an ADR.
5. **First-party scope:** the inventory registers unsafe written in this
   repository. Unsafe inside third-party dependencies is governed by
   [W18](../p0-w18-dependency-governance/README.md) evaluation criteria at
   dependency intake; the policy states this split so neither document is
   read as covering both.
6. **Gate relationship:** the policy defines what a future
   inventory-consistency check must verify (every `unsafe` item has a current
   inventory entry); the check's classification and CI realization belong to
   W07's future-class promotion and W20. The policy must not define the gate
   itself.
7. **Escalation thresholds:** adding a justified unsafe inside an approved
   design and category is ordinary reviewed change; adding a category or
   reclassifying a forbidden pattern is a policy decision; making
   convenience-driven or Core-policy unsafe possible contradicts ADR-006 and
   goes to the ADR path (P0-W06 subject).

## Work breakdown and loading order

1. Read the [policy contract](01-unsafe-policy-contract.md) for the artifact
   groups, safe-Rust-first principle, justification and template
   requirements, boundary categories, forbidden patterns, review rules, and
   escalation thresholds.
2. Read the [inventory contract](02-inventory-contract.md) before creating
   the inventory or judging any future entry: schema fields, entry lifecycle,
   update timing, and audit triggers.
3. Apply the changes in the order stated in the [implementation
   workflow](03-implementation-and-validation.md): verify prerequisite
   surfaces, author the policy, create the empty inventory, run the
   first-unsafe walkthrough, wire discovery, close.
4. Store actual review commands, output, environment, and result in
   `../../verification/p0-w10-unsafe-rust-governance-verification.md`, and
   record decisions taken and changed artifacts in
   `../p0-w10-unsafe-rust-governance-record.md` only when implementation
   begins. Neither this design nor a written record may claim W10 complete.

## Design-level state and lifecycle

W10 adds no runtime state and no code. The authoritative state is two tracked
documents plus their discovery links. Documentary lifecycle:

```text
no unsafe governance; zero unsafe in tree
  -> policy committed (principle, justification, categories, review, escalation)
  -> empty inventory committed (schema pointer, zero entries)
  -> first-unsafe walkthrough evidenced (hypothetical change traced end to end)
  -> discovery links committed
  -> P1+ low-level designs cite the policy; first real unsafe enters the inventory
     in the same change as its code
  -> future inventory-consistency gate promoted through W07's future class
  -> policy changes only through its escalation thresholds
```

The policy document is the owner of every governance statement; the inventory
is the owner of every entry. An entry contradicting the policy, or unsafe code
without a current entry, is a review failure — and, once promoted, a gate
failure.

## Explicitly excluded interfaces

No Rust type, function, trait, module, crate, wrapper API, register
interface, pointer abstraction, ASM sequence, gate, or CI workflow is
designed or authorized by W10. The SAFETY-comment template and inventory
schema are documentary contracts about *future* code; they fix no code
artifact. If implementing W10 appears to require writing code or choosing an
API, that is a scope boundary: stop and record.

## Downstream handoff

- **P1+ low-level designs** (arch, MMIO, page-table, ASM, FFI-adjacent work
  per their plan indexes) receive the review baseline: every proposed unsafe
  segment must arrive with its category, justification, SAFETY-comment
  draft, and inventory entry, and designs must name their unsafe segments
  explicitly for review.
- **W07** receives the consistency predicate its future gate must check
  (policy §8) and the trigger (first unsafe); classification and wiring stay
  with W07/W20.
- **W18** receives the first-party/third-party split: dependency-internal
  unsafe is evaluated under W18's intake criteria, and a dependency's unsafe
  surface may be a recorded evaluation input without entering this
  inventory.
- **W11** receives the placement constraint hook: category placement must
  remain consistent with its layering rules once delivered; conflicts go to
  the escalation thresholds, not to local exceptions.
- **W14** receives the requirement that SAFETY comments name the failure
  class from its taxonomy once delivered; until then the policy's placeholder
  failure-class wording applies.
- **W05** may re-home the documents under its documentation taxonomy.
