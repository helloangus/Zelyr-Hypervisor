# P0-W14 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W14 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no failure-classification
document exists yet) and a search of tracked documents for existing
panic/failure statements (§12, §19, the Coding Guidelines' guest-error rule,
and sibling boundary pointers). No Rust source exists, so no panic or error
path exists to classify.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a delivered sibling contract (W05 conventions; W10 policy; W12 baseline)
  conflicts with this design's identifiers, reservation, or treatments —
  raise the conflict; do not re-decide the sibling's subject here;
- implementing appears to require an error enum, a panic handler, a fault-
  injection mechanism, or any code — those are Reserved to P1+ designs, and
  reaching for them is a scope violation;
- a proposed class boundary cannot be stated without deciding a recovery
  policy (retry counts, timeouts, backoff) — the taxonomy owns classes and
  response *boundaries*; policies belong to the owning designs. A boundary
  that seems to need a policy is a design question to raise, not a policy to
  fix here; or
- a maintainer requests guest-caused faults being allowed to panic the
  hypervisor, or invariant downgrades — that is the ADR path per
  [the classification contract](01-failure-classification-contract.md) §7,
  never a local edit.

## 2. Ordered implementation steps

### Step 1 — verify prerequisite and sibling surfaces

Target: implementation record (created in this step).

Work: record which sibling designs have delivered documents (W05, W06
prerequisites; W10, W12 consumers), which convention source this
implementation follows, and any conflict surface — in particular whether
W10's SAFETY template and W12's channel sections have landed, since the
embedding section references both by subject.

**Acceptance:** the record names the convention source and every
blocked-by-prerequisite surface.  
**Failure/blocker:** an unresolved governing conflict stops the work per §1.

### Step 2 — author the classification document

Target: `docs/security/failure-classification.md`.

Work: write the document with the status header and all content required by
[the classification contract](01-failure-classification-contract.md): the
five class sections in the fixed field order, the propagation and
classification rules, the panic-worthiness rule, the diagnostic/review
embedding, and the cross-map with thresholds. No error type, handler,
policy, or code identifier may appear as an authorized surface.

**Acceptance:** every required section is present with all seven fields per
class; the document names no code symbol and fixes no recovery policy; it
contradicts no ADR, task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction is raised per §1, not absorbed by
rewording.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing row to `docs/README.md` pointing failure/panic/
containment classification work at the classification document; add the W14
design row to the stage implementation index with status "Proposed design;
implementation not claimed" (updated truthfully as work proceeds). Change
nothing else in these files.

**Acceptance:** a contributor starting from `docs/README.md` reaches the
classification document in one link; links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — representative-scenario walkthrough

Target: verification record.

Work: execute the walkthrough of W14-DV04 against five clearly hypothetical
later-stage failures: (a) a guest touches an unmapped guest-physical address
and the Stage-2 fault arrives at EL2; (b) the physical page allocator is
empty during VM creation; (c) a stage requires an SMMU capability the
platform lacks; (d) a firmware CPU_ON call fails during secondary bring-up;
(e) a page-table walker finds a mapped page whose ownership record belongs
to another VM. For each: record the class, the detection point, the
containment scope, one allowed and one prohibited response, the diagnostic
treatment, and confirm the walkthrough did not require choosing an error
type, a handler, a retry policy, or any code.

**Acceptance:** every scenario resolves to exactly one class with one
containment scope, and no step prescribed an implementation.  
**Failure/blocker:** a scenario with two defensible classes or no class
shows a taxonomy gap — stop and raise it; do not patch the document ad hoc
in the walkthrough record.

### Step 5 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement (P0-V09), prerequisite
compatibility, document links, and downstream handoff wording — including
that W10's and W12's existing designs can bind to the delivered identifiers
and reservation without contradiction. Completion is claimed only in the
verification record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W14-DV01 → P0-V09 | taxonomy review | inspect the five class sections against contract §3 field order and the plan's class list | five classes, all seven fields each, distinctions between adjacent classes explicit (FC-GUEST vs FC-INVARIANT; FC-RESOURCE vs FC-UNSUPPORTED) | architecturally distinct classes exist; not that any code classifies |
| W14-DV02 → P0-V09 | containment review | inspect FC-GUEST and §4's classification test against §19/ADR-007 | containment scope VM-local; escalation prohibition explicit; classification test stated | the guest-containment invariant is operational; not future compliance |
| W14-DV03 → P0-V09 | boundary embedding review | inspect §3 treatments, §5, and the D/C checklist items against contract §6 | per-class diagnostic treatments present; fatal channel reserved to invariant exits; checklist items checkable | boundaries reach review surfaces; not that W12's channels or W10's template exist yet |
| W14-DV04 → P0-V09 | scenario walkthrough | step 4 procedure on the five representative failures | one class, one containment scope, one diagnostic treatment per scenario; no implementation prescribed | the taxonomy classifies later-stage failures without pre-deciding them; not that those stages are designed |
| W14-DV05 → P0-V09 | discovery and link review | resolve routing row and index row from a fresh checkout | one-link reachability; truthful status; links resolve | documentation navigation; not W05's taxonomy |
| W14-DV06 → W14 closure | consumability review | read the classification as W10 (can SAFETY statements cite the classes?), W12 (is the fatal reservation bindable?), a P1 crash-diagnostics author, and a P2 allocation author | each can act without inventing policy and knows what is not W14's | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. No validation here
proves P0-V01–P0-V08, P0-V10–P0-V15, and none may be reported as doing so.

## 4. Error, security, and observability model

W14 adds no runtime error path; it defines the semantics future error paths
must carry. Security positions embedded in the taxonomy: the classification
test keeps hostile guest input from being laundered into "hypervisor bug"
framing or, worse, into fatal exits that deny the whole system for one
guest's fault; the authorization rule keeps expected denials distinguishable
from failures (an audit surface, per ADR-013); the panic-worthiness rule is
the availability boundary of the hypervisor against guest-caused and
resource conditions.

Observability: each class carries a required diagnostic treatment, so no
failure class is silent by design; the taxonomy itself is observed through
the verification record's review outputs, the scenario walkthrough, and
run/not-run statuses — the only accepted proof surface.

## 5. Handoff checklist

Before handing W14 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV06 evidence paths and their run status, including the scenario
  walkthrough record and any blocked-by-prerequisite note;
- confirmation that no error enum, panic handler, fault-injection
  mechanism, recovery policy, code surface, or CI workflow was defined or
  authorized;
- the recorded convention source from step 1 and any conflict raised; and
- open items for W10 (class citations in SAFETY templates), W12 (fatal
  reservation binding), P5 (management-input class assignment), and P1+
  error models (per-path classification) — without resolving their contracts
  here.
