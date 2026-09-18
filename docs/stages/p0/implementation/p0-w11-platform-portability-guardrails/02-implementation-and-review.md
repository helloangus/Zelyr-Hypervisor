# P0-W11 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W11 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no portability-rules document
exists yet) and a search of tracked documents for existing layering statements
(the ADR register entries, the §19 invariants, the `AGENTS.md` layering
mandate). The tree contains no code, so no violation exists to fix; W11 adds
rules, not corrections.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a delivered document (for example a sibling policy document landed in the
  meantime) contradicts the prohibitions or thresholds of this design — raise
  the conflict; do not edit the other document's authority silently;
- making the rules reviewable appears to require a platform schema, trait,
  BSP API, crate, or gate — those belong to the designs that own those
  subjects (platform-description designs in P1/P2, W07/W20 for gates), and
  reaching for them here is a scope violation;
- a maintainer decision is requested to weaken a prohibition (for example to
  allow a QEMU-name fast path in Core) — that is the ADR path per contract
  §2.9, never a local edit; or
- W05/W06 have delivered conventions that conflict with the planned header or
  escalation sections — follow the delivered rules and record the adaptation.

## 2. Ordered implementation steps

### Step 1 — verify prerequisite and conflict surfaces

Target: implementation record (created in this step).

Work: confirm which sibling prerequisite designs (W05, W06) have delivered
documents, if any, and record the convention source this implementation
follows (delivered rules if present; otherwise the existing `docs/README.md`
header mandate and `AGENTS.md` label mandate). Record any conflict surface
found.

**Acceptance:** the record names the convention source and any conflicts.  
**Failure/blocker:** an unresolved conflict with a governing document stops
the work per §1.

### Step 2 — author the portability rules document

Target: `docs/development/platform-portability-rules.md`.

Work: write the document with the status header and all sections required by
[the rules contract](01-portability-rules-contract.md) §2. Content must
reflect the resolved decisions (logical layers, identity containment,
capability-selection rule, quirk boundary, documentation constraints, both
checklists, three counterexamples, three thresholds) and must not define a
platform schema, trait, BSP API, crate layout, or gate.

**Acceptance:** every required section is present with its required content;
the document contradicts no ADR, task-book, or Coding-Guidelines rule; no
crate/module/target/dependency name appears.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording this document.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/platform/README.md`,
`docs/stages/p0/implementation/README.md`.

Work: add one routing row to `docs/README.md` pointing platform/BSP and
portability work at the rules document; add one pointer line to
`docs/platform/README.md`; add the W11 design row to the stage implementation
index with status "Proposed design; implementation not claimed" (updated
truthfully as work proceeds). Change nothing else in these files.

**Acceptance:** a contributor starting from `docs/README.md`, from the
platform directory, or from the review-checklist question "may this depend on
the board?" can reach the rules in one link; all new relative links resolve
from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — platform-difference walkthrough

Target: verification record.

Work: execute the walkthrough of W11-DV04 (below) against a clearly
hypothetical platform difference (for example: boot-time debug output uses a
different UART controller on the reference virtual platform and on the first
real-hardware SoC). Classify it strictly with the rules document: name the
owning layer, the capability/property a Core-visible difference would query,
and the quirk rules that would govern a hardware deviation. Record the
reasoning and confirm no step of the walkthrough required inventing a schema,
trait, or driver.

**Acceptance:** the walkthrough shows the rules produce one classification
without under-specified branches, and no actual event, schema, or code was
created.  
**Failure/blocker:** if the rules cannot classify the walkthrough without a
new concept, that concept is a gap in this design — stop and raise it, do not
extend the document ad hoc.

### Step 5 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement (P0-V12), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what was
actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W11-DV01 → P0-V12 | rules review | inspect the rules document against contract §2.1, §2.8 | layer table complete; directions stated; counterexamples present and labeled hypothetical | reviewable rules exist; not that any code obeys them |
| W11-DV02 → P0-V12 | prohibition review | inspect §2.2/§2.4 against ADR-043/ADR-052/§19 | no rule permits board/QEMU identity in Core or SoC/Board dependency in Arch; quirk boundary closed | the red lines match the ADR; not future compliance by later changes |
| W11-DV03 → P0-V12 | selection-rule review | apply §2.3 and checklist R1–R5 to the walkthrough record | every platform difference in the walkthrough resolves to a named capability query and owning layer | the selection rule is applicable; not that PlatformCapabilities fields exist (their designs own them) |
| W11-DV04 → P0-V12 | platform-difference walkthrough | step 4 procedure on the hypothetical UART difference | one classification, no invented schema/trait/driver, no code written | rule applicability; not hardware behavior |
| W11-DV05 → P0-V09/P0-V12 | discovery and link review | resolve routing row, platform pointer, and index row from a fresh checkout | one-link reachability from all three entry surfaces; truthful status; links resolve | documentation navigation; not W05's taxonomy |
| W11-DV06 → P0-V12/P0-V15 | consumability review | read the rules as a P1 platform-design author, a memory-design author, an IRQ-design author, and a BSP-design author | each can state the red lines their design must cite without inventing policy | handoff readiness; not that downstream designs are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. No validation here
proves P0-V01–P0-V08, P0-V10–P0-V11, or P0-V13–P0-V15, and none may be
reported as doing so.

## 4. Error, security, and observability model

W11 adds no hypervisor error model, synchronization, guest input path, or
`unsafe` code. Its failure reporting is textual: a missing rule section, a
prohibition that contradicts the ADR, an unclassifiable walkthrough, or a
duplicating policy statement fails the associated review and is recorded as
such.

The security position is preventive: platform-identity containment keeps
platform-specific behavior out of the layers that enforce guest-facing
guarantees (ADR-007's untrusted-guest posture depends on Core behavior not
being quietly specialized per platform), and the quirk boundary keeps
hardware-workaround authority declared and reviewable instead of implicit.

Observability is the evidence trail: the verification record's review output,
walkthrough reasoning, and run/not-run status are the only accepted proof
surface. Any future automated check remains W07/W20 property.

## 5. Handoff checklist

Before handing W11 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV06 evidence paths and their run status, including explicit not-run
  entries (mechanical check, support-tier content, PlatformCapabilities
  fields);
- confirmation that no platform schema, trait, BSP/quirk API, driver, crate,
  module tree, target, dependency, or CI workflow was added;
- confirmation that the rules document names no platform-specific constant or
  code location; and
- open items for W07/W20 (future check predicate), W05 (possible re-homing),
  W10 (unsafe placement consistency), and P1+ platform/memory/IRQ/BSP designs
  (red-line citations) — without resolving their contracts here.
