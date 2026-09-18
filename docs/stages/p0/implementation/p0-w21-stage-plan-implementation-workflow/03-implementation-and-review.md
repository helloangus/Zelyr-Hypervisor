# P0-W21 Implementation Workflow and Validation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W21 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tree. Useful read-only
discovery: `git ls-files` (confirm no `docs/development/stage-workflow.md`
exists and inventory which prerequisite documents are in-tree), and a search
of tracked documents for existing layer-flow statements (task book §3,
`docs/README.md`, the two agent guides, `AGENTS.md`) so the workflow document
cites rather than duplicates them.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a delivered W05 or W06 rule contradicts this design's matrix or admission
  rows — follow the delivered rule, reconcile the workflow document in the
  same change, and raise a substantive conflict; never restate the rule with
  different words to avoid the collision;
- authoring the workflow document appears to require defining ADR lifecycle
  detail, new document classes, or merge-policy text — that is the W06/W05/
  policy boundary; cite instead or stop;
- the drill appears to require executing a build, test, or QEMU run to
  "prove" the chain — it does not; the drill proves discoverability only,
  and execution evidence belongs to the owning packages' verification
  records.

## 2. Ordered implementation steps

### Step 1 — audit governing statements and prerequisite surfaces

Target: implementation record (created in this step).

Work: inventory where the layer hierarchy is currently stated (task book §3,
`docs/README.md` reading rules and normative-source list, Plan Agent guide
authority section, Coding Guide authority section, `AGENTS.md`, the
integration policy) and record the exact statements the workflow document
will cite. Record the delivery status of the W05 taxonomy document and the
W06 ADR governance document (expected paths `docs/development/documentation-baseline.md`
and expanded `docs/adr/README.md`).

**Acceptance:** the record names each cited statement with its file and
section, and each prerequisite with delivered/absent status.  
**Failure/blocker:** absent prerequisites are consumed via the fallback rule
(parent README "Current-state findings"); the workflow document cites the
existing mandates and marks the routed references as pending where their
documents do not exist.

### Step 2 — author the stage-workflow document

Target: `docs/development/stage-workflow.md`.

Work: write the normative document with the status header required by
`docs/README.md` (status, scope, version `v0.1`, owner/change context,
supersedes: none) and the required content of [the layer
contract](01-layer-contract.md) §2–§4 and [the admission and traceability
contract](02-admission-and-traceability.md) §2–§3: the seven-layer matrix
with its six fields per layer, the non-mixing rules, the admission decision
table with its two corollaries, and the seven traceability rules. Cite every
owning document; restate nothing.

**Acceptance:** every required section present; no statement duplicates or
contradicts a cited authority; no layer's internal content rules are
redefined.  
**Failure/blocker:** a contradiction is raised per §1, not absorbed.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row pointing workflow/responsibility/admission
questions at the new document, and add the W21 stage-index row with truthful
status. Change nothing else.

**Acceptance:** a newcomer starting from `docs/README.md` reaches the
workflow document in one link; the index row reflects the real status; all
new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — run the discoverability drill

Target: verification record.

Work: execute the drill of work sequence 4 with the two packages fixed in
the parent README (P0-W02 as the P0 chain, P1-W10 as the future P1 chain):

1. For the P0 package, walk the layer chain as each role: from the workflow
   document, confirm a planner can reach the plan, a designer the design
   entry, an implementer the record location, a reviewer the verification
   location, and an escalator the W06 mechanism — naming each document and
   the link path taken.
2. For the P1 package, walk the reading order a P1 planner would follow
   (P1 plan index entry → the P1-W10 plan → the P0 contracts it consumes via
   the plan-index prerequisite map and, when delivered, the W22 handoff map)
   and confirm each consumed P0 input is findable with a distinguishable
   status (delivered / design-proposed / plan-only).
3. Record each step as `reachable`, `blocked` (with the missing document and
   owning package named), or `failed` (an inconsistency, broken link, or
   contradiction). A `blocked` outcome from a not-yet-delivered sibling is
   truthful and expected; a `failed` outcome is a defect to fix in this
   package's document.

**Acceptance:** every chain step resolves or carries a named, actionable
blocked mark; no failed outcome remains; the drill records no execution
evidence and claims none.  
**Failure/blocker:** a failure is fixed in the workflow document (assembly
defect) or routed to the owning package (content defect); it is never fixed
by improvising content.

### Step 5 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
verify the package against its task-book requirements (P0-V09, P0-V15),
prerequisite compatibility, document links, and downstream handoff wording.
Completion is claimed only in the verification record, with evidence, and
only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W21-DV01 → P0-V09 | layer-matrix review | inspect `docs/development/stage-workflow.md` against the layer contract §2–§4 | seven layers, six fields each, non-mixing rules present, no restated authority | the responsibility flow is explicit; not that any agent has followed it yet |
| W21-DV02 → P0-V09/P0-V15 | admission-table review | apply each table row to a hypothetical case (a plan with no design; a conflict with adr-000; a stage completion attempt without evidence) | every case resolves to a required-hold set and an escalation path with an owner | the admission rule is decidable; not future compliance by later changes |
| W21-DV03 → P0-V09 | traceability review | trace one real package (P0-W01, delivered) through the rules: design ↔ plan, record ↔ design, verification ↔ validation IDs | every link the rules require exists and resolves for the traced package | the rules are applicable to a real chain; not that all packages comply before they run |
| W21-DV04 → P0-V09 | discovery and link review | resolve the routing row, workflow-document links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's taxonomy ownership |
| W21-DV05 → P0-V15 | P0 drill | step 4 procedure, part 1 | every layer step `reachable` (or `blocked` with named owner); no `failed` | a P0 agent can find what to read and produce; not that the P0 package's work is done |
| W21-DV06 → P0-V15 | P1 drill | step 4 procedure, part 2 | every P1-W10 consumed P0 input findable with distinguishable status; no `failed` | a P1 planner can start without reconstructing P0 policy; not that P1 is planned or executable |
| W21-DV07 → W21 closure | consumability review | read the workflow document as W22 (map shape), W06 (citation not restatement), and a hypothetical P1 planner | each consumer can act without inventing rules | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command or observation, input, environment, timestamp, and reason. The
drill is discoverability evidence only: it proves nothing about builds,
tests, QEMU, hardware, or any package's completion, and no entry may be
reported as doing so.

## 4. Error, security, and observability model

W21 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary: a
missing layer input, a restated authority, an undecidable admission case, or
a failed drill step fails the associated review and is recorded as such. The
security-relevant property is procedural integrity — the admission table is
the control that keeps unapproved implementation and silent architecture
changes out of the repository — and its failure mode (bypassing the table)
is made visible by the traceability rules: any change whose record cannot be
traced to a plan and design through the table is a review failure.

Observability is the drill and review evidence trail in the verification
record: observations, link paths, outcomes, and run/not-run status are the
only accepted proof surface.

## 5. Handoff checklist

Before handing W21 to a reviewer, provide:

- the exact changed-file list;
- the cited-statement inventory from step 1 with prerequisite statuses;
- DV01–DV07 evidence paths and run status, including explicit blocked entries
  for not-yet-delivered sibling documents;
- confirmation that no ADR lifecycle rule, document class, merge-policy
  text, or contributor-path statement was restated or amended;
- confirmation that the drill created no execution evidence and claimed none;
- open items for W22 (the map must fit the completion-layer contract),
  W05/W06 (any routed reference still pending delivery), and P1 planning
  (the workflow document as the planner entry point) — without resolving
  their contracts here.
