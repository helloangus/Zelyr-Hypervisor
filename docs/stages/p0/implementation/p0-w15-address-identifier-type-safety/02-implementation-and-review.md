# P0-W15 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W15 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no type-safety requirements
document exists yet) and a search of tracked documents for existing
address/identity statements (ADR §13, ADR-018, the Coding Guidelines' newtype
rule, sibling checklist citations of this package). No Rust source exists, so
no address or identifier appears anywhere in code.

Stop and obtain direction instead of guessing when any of the following
occurs:

- [W05](../p0-w05-documentation-baseline/README.md) has delivered document
  conventions that conflict with the planned header or placement — follow
  the delivered rules and record the adaptation; a substantive conflict is
  raised, not absorbed;
- writing an inventory entry appears to require choosing a width, a trait
  shape, a conversion function, or a crate placement — that is the owning
  P1+ design's decision, and reaching for it is the exact scope violation
  work item 4 guards against;
- an inventory entry cannot be written without deciding a translation
  mechanism's implementation (for example how Stage-2 translation works) —
  the inventory states that the translation exists and is named; its
  mechanics belong to the owning design. Stop and raise if the semantics
  themselves seem wrong; or
- a maintainer requests that P0 create the concrete address/ID types — that
  is the interpretation question recorded in the parent README's authority
  section; resolve it through the recorded mechanism, not by adding types
  under W15.

## 2. Ordered implementation steps

### Step 1 — verify prerequisite and citation surfaces

Target: implementation record (created in this step).

Work: record whether W05 has delivered its conventions and which convention
source this implementation follows; record the sibling citations of this
package ([W11](../p0-w11-platform-portability-guardrails/README.md)'s R6,
[W14](../p0-w14-panic-failure-classification/README.md)'s subject reference)
and confirm this design's document satisfies what those citations expect (a
findable semantic inventory) without depending on their content.

**Acceptance:** the record names the convention source and the citation
surfaces checked.  
**Failure/blocker:** an unresolved governing conflict stops the work per §1.

### Step 2 — author the type-safety requirements document

Target: `docs/development/address-identifier-type-safety.md`.

Work: write the document with the status header and all content required by
[the semantics contract](01-semantics-and-review-contract.md): the
complement-relationship statement, both inventories in the fixed entry form,
the six red-line rules, both checklists, the reserved semantics with
triggers, and the thresholds. No type name, trait shape, width, conversion
signature, crate, or module may appear as an authorized surface.

**Acceptance:** every required section is present; both inventories carry
the required entries in the entry form; the document names no Rust artifact;
it contradicts no ADR, task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction is raised per §1, not absorbed by
rewording.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing row to `docs/README.md` pointing address/identifier
semantic and typing questions at the requirements document; add the W15
design row to the stage implementation index with status "Proposed design;
implementation not claimed" (updated truthfully as work proceeds). Change
nothing else in these files.

**Acceptance:** a P1 planner starting from `docs/README.md` reaches the
requirements document in one link; links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — non-prescription walkthrough

Target: verification record.

Work: execute the walkthrough of W15-DV04 against clearly hypothetical
future design questions: (a) a P1 interface reporting a fault address from
guest state — which semantics appear and what the design must state; (b) a
Stage-2 translation producing a host physical address from a guest physical
address — how rule §5.2/§5.3 applies without any type being named; (c) a
VM-allocation design tempted to reuse the Stage-2 VMID as the VM identifier
— which inventory entry forbids it; (d) a discovery design carrying memory
ranges from firmware data — which rule forces validation at entry. For each,
record the inventory entries and rules that decide the answer, then confirm
no step required naming a type, width, trait, or module.

**Acceptance:** every walkthrough item resolves by the written semantics,
and no step implied a Rust artifact.  
**Failure/blocker:** an item resolvable only by choosing a representation
shows a non-prescription leak in this design — stop and raise it; do not
patch the document ad hoc in the walkthrough record.

### Step 5 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement (P0-V09), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what
was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W15-DV01 → P0-V09 | address-inventory review | inspect contract §3's entries in the document | four required semantics present in entry form; GPA/IPA equivalence recorded; DMA-visible semantics reserved with trigger | the authoritative address distinction list exists; not that any type exists |
| W15-DV02 → P0-V09 | identity-inventory review | inspect contract §4's entries in the document | five required semantics present; VM-identifier vs Stage-2 VMID distinction explicit; reservations carry triggers | identity distinctions are authoritative; not that any type exists |
| W15-DV03 → P0-V09 | rule and checklist review | inspect contract §5–§6 in the document | six red-line rules present; both checklists checkable; Coding Guidelines linked, not restated | conversion explicitness is reviewable; not future compliance by later designs |
| W15-DV04 → P0-V09 | non-prescription walkthrough | step 4 procedure on the four hypothetical design questions | each item resolves by semantics alone; no type, width, trait, or module implied | the requirement constrains without reverse-specifying; not that P1+ designs are done |
| W15-DV05 → P0-V09 | discovery and link review | resolve routing row and index row from a fresh checkout | one-link reachability from the design entry; truthful status; links resolve | design-entry discoverability; not W05's taxonomy |
| W15-DV06 → W15 closure | consumability review | read the requirements document as a P1 address-space author, a P2 allocator author, a P5 handle author, and a P6 interrupt author | each can state the red lines for their subject without inventing types or policy | handoff readiness; not that downstream designs exist |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. No validation here
proves P0-V01–P0-V08, P0-V10–P0-V15, and none may be reported as doing so.

## 4. Error, security, and observability model

W15 adds no runtime error path, synchronization, or `unsafe` code. Its
security position is preventive and load-bearing: the untrusted-entry rule
(§5.5) and the translation rules (§5.2–§5.3) are the type-level expression
of ADR-007's guest-input posture — address confusions and identifier
re-use are exactly the defect classes that become cross-VM memory exposure
or authority confusion. The document states this consequence so reviewers
treat checklist failures as security-relevant, not stylistic.

Observability is the evidence trail: the verification record's review
outputs, the non-prescription walkthrough, and run/not-run statuses are the
only accepted proof surface. Any future mechanical check is W07/W20
property.

## 5. Open questions and handoff checklist

**Open question (owner confirmation):** whether the task-book P0 task-list
bullet naming "新类型 ID、地址类型" requires concrete types during P0. This
design's recorded reading is no — the requirement baseline satisfies the
W15 outcome, and P0's own out-of-scope clause (§1) and exit criterion 6
forbid freezing API artifacts. An owner decision to the contrary is a scope
change through the ADR-governance mechanism, not a W15 edit.

Before handing W15 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV06 evidence paths and their run status, including the walkthrough
  record and any blocked-by-prerequisite note;
- confirmation that no type, trait, width, conversion API, crate, module,
  or CI workflow was defined or authorized;
- the recorded convention source from step 1 and any conflict raised; and
- open items for the owning P1+ designs (first types for HVA/HPA/GPA/GVA
  and the identity semantics), the reserved-semantics owners (P2, P5, P6,
  SMMU stage), and W05 (possible re-homing) — without resolving their
  contracts here.
