# P0-W17 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W17 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no naming document exists yet) and
a search of tracked documents for existing artifact-name conventions or
extension statements. The implementing machine's local build outputs, if any,
are machine-local state and never evidence.

Stop and obtain direction instead of guessing when any of the following occurs:

- the W16 metadata contract has landed with a different field set than this
  design's mapping assumes — record the divergence as a cross-package conflict
  surface in the cross-review step; do not silently reword either side;
- a sibling design has already introduced naming rules (for example inside a
  build-baseline design) — reconcile in one change per the single-source rule;
  two naming authorities is a review failure;
- closure appears to require choosing a target triple, a profile value, a file
  format, an output directory, or a generation tool — those belong to W03,
  W04, and the owning consumer designs; reaching for them here is a scope
  violation; or
- a reviewer asks to fix the official project name or declare a format —
  refuse: the first is pending under ADR-054, the second violates the plan's
  out-of-scope clause.

## 2. Ordered implementation steps

### Step 1 — record prerequisite status assumptions

Target: implementation record (`p0-w17-artifact-naming-baseline-record.md`,
created in this step).

Work: observe which prerequisite/consumer contracts exist in the tree at
implementation time (W16 metadata contract, W03 target boundary, W04 profile
governance; several may still be parallel proposed designs). Record each one's
observable status and the assumption the grammar makes about it: W16's field
schema as mapping authority, W03's platform boundary as platform-vocabulary
populator, W04's governance as profile-vocabulary populator.

**Acceptance:** the record names each contract's observable status and the
assumption taken.  
**Failure/blocker:** an in-tree conflict is raised per §1, not absorbed.

### Step 2 — create the naming contract document

Target: `docs/development/artifact-naming.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and all
sections required by [the naming contract](01-naming-contract.md): governed
scope, dimensions, grammar and character-set rules, revision/dirty rules, the
five vocabularies with their governance and initial entries, stability rules,
the W16 mapping table, the category inventory per
[the inventory file](02-artifact-category-inventory.md), format
non-commitment, and evolution rules.

**Acceptance:** every required section present; no target triple, profile
value, file format decision, output directory, or generation process appears;
no official project name is decided; the document contradicts no ADR,
task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1; fix the draft against the grammar, never the grammar against a draft
convenience.

### Step 3 — parseability rehearsal (design validation, pre-discovery)

Target: verification record evidence.

Work: exercise the grammar on paper before discovery wiring: for the
`hypervisor` class, derive the canonical field sequence from its applicability
row, substitute representative values for every field **except** those whose
vocabularies are deliberately empty (`profile`) or deferred (`platform` until
W03 names the boundary), and show that the remaining fields still parse
positionally; show one intentionally malformed name (hyphen inside a value,
uppercase character, extra dimension) and the rule that rejects it. The
rehearsal is a design-consistency check of the grammar text, not an artifact
claim.

**Acceptance:** the rehearsal shows clean parses, the two deferred-value
cases, and the rejection cases, with the governing rule cited for each.  
**Failure/blocker:** any ambiguity found is a grammar defect — fix the grammar
text in this change; ambiguity discovered after first production would be far
more expensive.

### Step 4 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing artifact-naming
work at the new contract document, and add the W17 design row to the stage
implementation index with status "Proposed design; implementation not claimed"
(updated truthfully as work proceeds). Change nothing else in either file.

**Acceptance:** one-link reachability from `docs/README.md`; truthful index
status; all new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 5 — run the cross-reviews

Targets: verification record (evidence); W16's contract as reviewee when
in-tree.

Work: perform W17-DV05: check the mapping table against the W16 contract's
field schema (if in-tree; otherwise record `blocked` with the
[W16 design](../p0-w16-version-build-metadata-baseline/README.md) as the named
review surface), verify
every name field traces to a schema field, and verify no schema field outside
the declared encodable subset appears in names. Note W19/W20 consumability
without assuming their content.

**Acceptance:** the review passed, or blocked with the conflict surface and
owners named.  
**Failure/blocker:** a semantic disagreement is recorded, not reworded away.

### Step 6 — closure review and evidence

Targets: verification record and implementation record.

Work: run the validation matrix below; confirm the handoff checklist; verify
the package against its task-book requirement, prerequisite compatibility,
document links, and downstream handoff wording. Record every validation as
passed, failed, blocked, or not run with command, output, timestamp, and
reason.

**Acceptance:** all Required validations passed or explicitly blocked with a
named surface; the first-instance check (W17-DV04) is `not run` unless a
governed artifact already exists.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis; do
not weaken the grammar to make a check pass.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W17-DV01 → P0-V14 | grammar and parseability review | inspect the contract against [the naming contract](01-naming-contract.md) §§3–4 | fixed dimension order; single separator; character-set rules; per-class applicability; parse-by-split rule stated | names are machine-processable by rule; not that any artifact name exists yet |
| W17-DV02 → P0-V14 | dimension/stability review | review §3, §5, §6 against the plan's dimension and no-human-memory requirements | every dimension has a vocabulary authority; vocabularies are the only token sources; determinism and no-timestamp rules present | names distinguish dimensions deterministically; not future compliance by later changes |
| W17-DV03 → P0-V14 | inventory completeness review | compare the inventory with the plan's category list (hypervisor, validation guest, Linux guest, boot package, Control Domain, DTB, firmware, snapshot, migration, symbols, reports) | every plan category has a row, token, applicability, and non-committed format owner; extension rule present | category coverage; not that categories are activated |
| W17-DV04 → P0-V14 (instance half) | first governed name instance | when W03's baseline artifact exists, verify its name against the grammar and applicability row | the instance parses and its fields match the producing build's identity | the full P0-V14 naming outcome; expected `not run` at W17 closure — recorded as such, never implied |
| W17-DV05 → P0-V09/P0-V14 | cross-review with W16 | compare the mapping table against the W16 field schema (or record blocked with surface) | every name field traces to a schema field; reserved positions and capability_summary are not name-encodable; no second identity authority | naming and metadata are associable; not that W16 is implemented |
| W17-DV06 → P0-V09 | non-commitment and boundary review | review §2, §8, and the inventory's non-category list against the plan's out-of-scope clause | no format, generation, output-layout, or internal-layout rule present; governed scope stated verbatim | the grammar does not overreach; not that consumers obey it yet |
| W17-DV07 → W17 closure | consumability review | read the contract as W19 (can I add an identify-the-artifact step?), W20 (is a conformance check implementable from this?), W03 (is the hypervisor row actionable?), W04 (is the profile slot clear?) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the document
without DV05 and DV07 does not satisfy W17 closure. No validation here proves
P0-V01–V08 or P0-V13 and none may be reported as doing so.

## 4. Error, security, and observability model

W17 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary: a
grammar ambiguity, a missing vocabulary authority, an untraceable name field,
or a failed cross-review fails the associated validation and is recorded as
such.

The security position is that names are identity assertions and must fail
closed: a name is derived from the build's identity inputs, never asserted
independently; `+dirty` may not be omitted for a dirty build; a value outside
the vocabularies is a violation, not an extension point; reserved tokens
prevent class confusion between, say, a report file and a hypervisor image.
Because generated artifacts are untracked under the existing ignore policy,
the name plus the W16 metadata record is what later evidence chains (W19/W20)
rely on — a spoofed or mutated name would poison that chain, which is why
derivation and vocabulary membership are review obligations.

Observability is the evidence trail: the verification record's rehearsal
outputs, cross-review results, statuses, and timestamps are the only accepted
proof surface. Once artifacts exist, the name itself is the human-observable
handle that ties an artifact back to its metadata record.

## 5. Handoff checklist

Before handing W17 to a reviewer, provide:

- the exact changed-file list;
- the parseability rehearsal output and the malformed-name rejection cases;
- DV01–DV07 evidence paths and their run status, including explicit
  `blocked`/`not run` entries (W16 cross-review pending, first-instance
  check);
- the recorded prerequisite status assumptions from Step 1;
- confirmation that no format decision, generation process, output directory,
  target triple, profile value, official-name decision, Rust source,
  `unsafe`, or CI change was added; and
- open items for W03 (first `hypervisor` name instance; platform vocabulary
  population), W04 (profile vocabulary), W16 (mapping-table reconciliation),
  W19 (identify-the-artifact step), W20 (candidate conformance check), and
  ADR-054 (official name migration) — without resolving their contracts here.
