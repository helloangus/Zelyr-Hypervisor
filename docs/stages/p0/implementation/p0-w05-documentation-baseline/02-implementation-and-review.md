# P0-W05 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked documentation
tree. Useful read-only discovery: `git ls-files docs` (inventory every entry
document), a per-directory README existence check, and a Markdown-link
resolution pass over entry documents. The exact tooling is machine-local
convenience; the results are the contract.

Stop and obtain direction instead of guessing when any of the following occurs:

- an existing entry document claims authority over taxonomy or metadata rules
  in a way that conflicts with this design — raise the conflict; do not edit
  the other document's authority silently;
- a coherence finding would require moving, re-homing, or re-versioning an
  accepted contract (for example the toolchain baseline) — record it as a
  Reserved follow-up with the owning package per
  [the taxonomy contract](01-taxonomy-and-baseline-contract.md) §6; do not
  execute it in W05;
- "completing" the inventory appears to require writing W06's ADR lifecycle
  or W21's workflow rules — those are their packages' outputs; write the
  pointer, not the content;
- a class seems needed that the inventory lacks for a not-yet-designed
  output — add it only through the §6 thresholds; do not pre-create classes
  speculatively.

## 2. Ordered implementation steps

### Step 1 — create the documentation baseline document

Target: `docs/development/documentation-baseline.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
exactly the required content of
[the taxonomy contract](01-taxonomy-and-baseline-contract.md) §3–§8: the
class inventory, the normative/informative discipline, the metadata rules,
the version and change thresholds, the stage-document separation table, and
the entry/link/referencability rules. The document must cite the existing
`docs/README.md` mandate and the task book's delivery hierarchy as its basis
and must not contradict either; it must contain no ADR lifecycle content and
no workflow admission rules.

**Acceptance:** every required section is present with its required content;
every W06–W22 output named in the parent README's handoff has a home in the
inventory; the document itself satisfies its own header rules.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording this document.

### Step 2 — run the coherence sweep

Targets: existing directory READMEs and entry documents under `docs/`, and
the stage trees' entry documents.

Work: check every entry document against the new baseline: role statement
present and correctly classified; no contradiction with the inventory,
metadata rules, or separation table; required headers present on normative
documents (expected already true for the concise guides and baselines);
single-home rule respected. Correct only what the baseline justifies:
minimal, factual edits. Record every finding and every edit (including "no
edit needed") in the implementation record. Findings needing re-homing or
owner consent become Reserved follow-ups, not edits.

**Acceptance:** the sweep report is complete for all entry documents; no
contradiction remains; every edit maps to a specific baseline rule; no style
rewrite occurred.  
**Failure/blocker:** a finding that cannot be fixed within the bounded-edit
policy is recorded as a blocker or Reserved follow-up with its owner; the
sweep is not "passed" by ignoring findings.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing documentation
creation/versioning/classification questions at the new baseline document,
and add the W05 design row to the stage implementation index with a truthful
status. Change nothing else in either file beyond sweep-justified edits from
step 2.

**Acceptance:** a newcomer starting from `docs/README.md` can reach the
baseline document in one link; the index row reflects the real status; all
new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — run the referencability walkthrough

Target: verification record
(`docs/stages/p0/verification/p0-w05-documentation-baseline-verification.md`,
created in this step).

Work: role-play three authors using only repository documents: (a) a W06
author creating an ADR governance document and template — where do they go,
which header rules apply, which class slot receives the output; (b) a P1
Plan Agent placing an EL2 design and later its verification evidence — which
stage sublocations and rules apply; (c) a contributor adding a new policy
baseline — which class, which header, which thresholds. Record the paths
followed, the rules applied, and any point where the inventory failed to
answer. Record what was not exercised and why.

**Acceptance:** each walkthrough reaches a complete answer without inventing
a location or rule; any gap found is fixed in the baseline through review
and the walkthrough re-run.  
**Failure/blocker:** an unfixable gap (for example a class W05 may not add)
is a recorded blocker or a §6 policy decision, never a silent workaround.

### Step 5 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement, prerequisite compatibility
with W01's delivered baseline, document links, and downstream handoff
wording (W06, W10–W22, P1+). Completion is claimed only in the verification
record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W05-DV01 → P0-V09 | class-inventory review | inspect the baseline's inventory against the actual `docs/` tree | every existing directory maps to exactly one class with a status; no existing document is unclassifiable; no phantom class exists | the taxonomy is reviewable and complete for P0; not that future classes will not be needed |
| W05-DV02 → P0-V09 | metadata-rules review | apply §5 rules to a sample of normative documents (concise guides, governance baselines) | all sample documents carry the required fields; no rule contradicts the `docs/README.md` mandate | the rules are mechanical and honored; not that every future document will comply |
| W05-DV03 → P0-V09 | stage-separation review | inspect the §7 table against `docs/stages/p0/` and `docs/stages/p1/` actual content | current placements conform; no mixing found; W06/W21 pointers present | the separation rule is written and currently true; not that future violations cannot occur |
| W05-DV04 → P0-V09 | entry/link review | resolve all links in entry documents and the new routing row from a fresh checkout; check per-directory README existence | all links resolve; every `docs/` directory has a conforming role statement | navigation coherence; not content correctness of the linked documents |
| W05-DV05 → P0-V09 | bounded-edit scope review | diff the sweep edits against the recorded findings | every edit maps to a baseline rule; no style rewrites; Reserved follow-ups recorded, not executed | sweep discipline; not that all documentation is now complete |
| W05-DV06 → P0-V09/P0-V15 | referencability walkthrough | step 4 evidence in the verification record | all three author roles reach complete answers from repository documents alone | later packages can locate homes and rules; not that their documents exist |
| W05-DV07 → W05 closure | consumability review | read the baseline as W06, W07, W21, and a P1 planner per the parent README's handoff list | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the baseline
document without the sweep and walkthrough does not satisfy the plan's fourth
work item. No validation here proves ADR lifecycle correctness (W06),
workflow rules (W21), or CI documentation checks (W20), and none may be
reported as doing so.

## 4. Error, security, and observability model

W05 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is textual: an
unclassifiable document, a missing header, an unresolved link, a separation
violation, or an unanswered walkthrough step fails the associated review and
is recorded as such.

The security relevance is documentation integrity: normative/informative
mixing and unowned supersession are how constraints silently disappear. The
baseline's metadata and labeling rules make that detectable by review, and
later mechanically by W07/W20 checks; W05 itself adds no enforcement tool.

Observability is the evidence trail: the sweep report, walkthrough evidence,
environment, and run/not-run status in the verification record are the only
accepted proof surface. No logging, tracing, or test framework may be
introduced for W05.

## 5. Handoff checklist

Before handing W05 to a reviewer, provide:

- the exact changed-file list, including every sweep edit with its finding;
- DV01–DV07 evidence paths and their run status, including explicit not-run
  entries (no ADR lifecycle authored, no workflow rules, no templates, no CI
  checks);
- the Reserved follow-up list (re-homing requests and policy decisions) with
  owners;
- confirmation that no ADR content, task-book/plan content, template, code,
  build artifact, or CI configuration was added; and
- open items for W06 (ADR class slot and header conventions ready), W21
  (stage locations fixed, flow undefined), W07/W20 (mechanical coherence
  rules available to gate on), and P1 (standing rules for new documents) —
  without resolving their contracts here.
