# P0-W19 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W19 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. The decisive
precondition is the **prerequisite contract availability audit**: for each of
W01, W02, W03, W07, W08, W09, W16, W17, determine whether the package's
contract document exists in-tree (delivered), exists only as a proposed design
or plan, or is absent. This audit is recorded in the implementation record and
drives which stages can be written as fully `reachable` versus
`contract-pending`.

Useful read-only discovery: `git ls-files`; link resolution over
`docs/development/` and `docs/stages/p0/`; a check of whether a GitHub remote
is configured (read-only `git remote -v`; W01's record states remote
publication is pending owner inputs).

Stop and obtain direction instead of guessing when any of the following occurs:

- a prerequisite contract has landed with a different shape than the stage
  table assumes (for example W03 splitting target-build documentation
  differently, or W09 renaming the runner entry) — cite the delivered shape
  and record the reconciliation in the same change; do not describe the
  assumed shape;
- closure appears to require writing a GitHub workflow, defining a required
  check, configuring branch protection, or amending the integration policy —
  those are W20's and the policy owner's; reaching for them here is a scope
  violation and the plan forbids it explicitly;
- closure appears to require restating another package's commands because its
  contract document is missing — refuse: mark the stage contract-pending
  instead; or
- a reviewer asks to record the walkthrough as proving CI enforcement —
  refuse: the plan's acceptance wording explicitly limits the walkthrough to
  discoverability.

## 2. Ordered implementation steps

### Step 1 — prerequisite availability audit

Target: implementation record (`p0-w19-reproducible-development-workflow-record.md`,
created in this step).

Work: perform the audit described in §1. For each prerequisite package record:
contract document present (path) / only design or plan present / absent; and
the resulting stage mode (full citation vs contract-pending). Record the
remote-configuration observation for the branch-to-PR walkthrough boundary.

**Acceptance:** the audit covers all eight prerequisite packages and the
remote state, and determines each stage's mode up front.  
**Failure/blocker:** none expected; a surprising conflict is raised per §1.

### Step 2 — create the contributor-workflow document

Target: `docs/development/contributor-workflow.md`.

Work: write the document with the status header required by `docs/README.md`
and all parts required by [the stage contracts](01-workflow-stage-contracts.md)
§2: audience and promise; the seven-stage chain with the five per-stage
content elements each; the four boundary sections; the S6 integration path per
[the integration path design](02-integration-path-and-walkthrough.md) §2; and
the blocked-stage legend. Link the integration-workflow policy without
restating it; mark pending stages per the audit.

**Acceptance:** every stage carries purpose, authoritative references,
declared inputs, expected evidence, and failure attribution; the boundary
sections are present; no command, policy paraphrase, GitHub workflow, or
check definition appears; no stage claims an unimplemented check is
configured or passing.  
**Failure/blocker:** any temptation to inline a missing contract's content is
refused per §1 — the stage stays contract-pending.

### Step 3 — link-integrity pass

Target: the workflow document's link set.

Work: resolve every relative link in the document from the document's
directory (simulating a fresh checkout), including links into
`docs/development/`, `docs/stages/p0/plans/`, and stage directories. Fix or
flag every broken link; for links to documents not yet in-tree, the stage must
already be marked contract-pending (a broken link inside a *reachable* stage
is a defect, not a pending mark).

**Acceptance:** zero broken links among reachable-stage citations; every
pending citation names its expected path.  
**Failure/blocker:** a broken link inside a reachable stage is fixed before
discovery wiring.

### Step 4 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing first-time
environment setup and contribution walkthroughs at the new document (leaving
the existing integration-policy row untouched), and add the W19 design row to
the stage implementation index with status "Proposed design; implementation
not claimed" (updated truthfully as work proceeds). Change nothing else in
either file; do not modify the root README.

**Acceptance:** one-link reachability from `docs/README.md`; the policy row
still points at the policy; truthful index status; all new links resolve.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 5 — execute the walkthroughs

Targets: verification record evidence; per
[the integration path and walkthrough design](02-integration-path-and-walkthrough.md)
§3.

Work: execute the fresh-clone walkthrough and the branch-to-PR walkthrough
exactly as designed, recording per-stage outcomes, the P0-V01–V05/P0-V13
mapping, lifecycle-step classifications, and any personal-environment
corroboration with full commands and output. Where the audit predicted a
pending stage, confirm the outcome matches the prediction (if a "pending"
stage turns out reachable, update both the audit and the document — the two
must agree).

**Acceptance:** both walkthroughs complete with no `failed` outcome; every
`blocked` outcome names its missing contract and owning package; the evidence
semantics of walkthrough §4 are respected.  
**Failure/blocker:** a `failed` outcome is an assembly defect — fix the
workflow document and re-run the affected walkthrough; do not weaken the
passing condition.

### Step 6 — closure review and evidence

Targets: verification record and implementation record.

Work: run the validation matrix below; confirm the handoff checklist; verify
the package against its task-book requirement, prerequisite compatibility,
document links, and downstream handoff wording. Record every validation as
passed, failed, blocked, or not run with command, output, timestamp, and
reason.

**Acceptance:** all Required validations passed or explicitly blocked with a
named surface; enforcement evidence is explicitly recorded as out of W19's
scope (W20, P0-V08).  
**Failure/blocker:** a failed review is recorded as failed with diagnosis; do
not mark unimplemented enforcement as present to close the package.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W19-DV01 → P0-V01–V05, P0-V13 | stage-chain assembly review | inspect the workflow document against [the stage contracts](01-workflow-stage-contracts.md) §3 and the plan's work sequence 1 | all seven stages present in order, each citing the correct owning contract; stage modes match the audit | the linear path exists and cites its authorities; not that any stage executes |
| W19-DV02 → P0-V09 | per-stage content review | check every stage for the five required content elements and the no-duplication rule | no stage restates a command or policy; every failure has an owner; pending stages are marked | assembly completeness; not contract correctness (owned by the contracts) |
| W19-DV03 → P0-V05/P0-V13 | boundary-section review | review §5 of the stage-contract requirements against the document | host/target non-interchangeability, QEMU placeholder boundary, artifact identification, and enforcement status all present and truthful | a reader cannot mistake host success for target success or a placeholder for EL2 evidence; not actual build/run behavior |
| W19-DV04 → P0-V01–V05 (integration portion) | integration-path review | compare S6 against the policy and [the integration path design](02-integration-path-and-walkthrough.md) §2 | the policy is referenced without restatement; contributor responsibilities table present; enforcement honesty holds | the branch→PR path is repository knowledge; not that GitHub enforces anything (W20, P0-V08) |
| W19-DV05 → P0-V09 | discovery and link review | resolve the routing row, document links, and index row from a fresh checkout | one-link reachability; policy row untouched and intact; zero broken links in reachable stages | documentation navigation; not W05's taxonomy |
| W19-DV06 → P0-V01–V05, P0-V13 | fresh-clone walkthrough | execute walkthrough §3.1 | every validation ID maps to a reachable stage or a blocked stage with named missing contract; no failed outcome | all P0 execution entries are discoverable from repository documents; not that the entries succeed when executed |
| W19-DV07 → P0-V01–V05 | branch-to-PR walkthrough | execute walkthrough §3.2 | every lifecycle step classified with owner; no unimplemented check described as configured or passing; remote-pending recorded if applicable | the mandatory integration route is completable as documented human process; not that online checks exist |
| W19-DV08 → W19 closure | consumability review | read the document as W20 (is the stage-to-check mapping usable?), as a P1 planner (can I onboard from this?), as a new contributor (can I follow S0–S6?) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the document
without DV06 and DV07 does not satisfy W19 closure — the walkthroughs are the
package's primary evidence. No validation here proves P0-V08 and none may be
reported as doing so.

## 4. Error, security, and observability model

W19 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary:
every stage's failure-attribution element is itself an error model for
contributors — a blocked step names the owning package and the record to
update, so failures are routed instead of absorbed.

The security position is honesty about enforcement: the design's central
failure mode is a contributor (or agent) believing a change is protected
because a document implied checks exist. The enforcement-status section, the
walkthrough's `pending-W20` classification, and the validation matrix's
explicit P0-V08 exclusion are the controls. A second exposure is the reverse:
documentation that silently depends on machine-local state; the walkthrough's
clone-only rule is the control.

Observability is the evidence trail: the verification record's walkthrough
observations, outcomes, timestamps, and environment are the only accepted
proof surface. The workflow document itself is the durable observability
artifact for every future contributor: it is where the project can see,
at any time, which entries are reachable and which are pending.

## 5. Handoff checklist

Before handing W19 to a reviewer, provide:

- the exact changed-file list;
- the prerequisite availability audit and the resulting per-stage modes;
- DV01–DV08 evidence paths and their run status, including explicit
  `blocked` entries (pending contracts, unconfigured remote) and any
  personal-environment corroboration, clearly labelled;
- DV06's stage-to-validation mapping for P0-V01, P0-V02, P0-V03, P0-V04,
  P0-V05, and P0-V13;
- confirmation that no GitHub workflow, required check, branch-protection
  setting, policy amendment, command restatement, Rust source, `unsafe`, or
  root-README change was made; and
- open items for W20 (stage-to-check mapping awaiting configuration;
  enforcement-status retirement), the pending prerequisite packages (their
  deliveries clear blocked marks in the same change), W05 (possible re-homing
  of the document; root-README link extension point), and W22 (stage table as
  handoff-map input) — without resolving their contracts here.
