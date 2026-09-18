# P0-W04 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no governance document exists
yet and no Cargo manifest exists anywhere) and a search of tracked documents
for existing feature/profile statements (expected: ADR-047, ADR-037, ADR-046,
the task book's Reserved list, and W03's profile boundary statement only).

Stop and obtain direction instead of guessing when any of the following occurs:

- a tracked document already claims authority over feature/profile semantics
  and contradicts this design — raise the conflict; do not edit the other
  document's authority silently;
- writing the registry purposes appears to require choosing which profiles to
  implement first, or defining their Cargo mechanics — that is Reserved and
  routed by §8 of the governance contract; do not activate anything;
- a reviewer or maintainer asks W04 to classify a *real* pending switch —
  the drill of step 3 uses hypothetical switches only; a real classification
  belongs to the proposing change's review, per the document's procedure;
- making the rules "enforceable" appears to require lint configuration, CI
  checks, or manifest changes — enforcement wiring belongs to W07/W20 and is
  out of scope here.

## 2. Ordered implementation steps

### Step 1 — create the governance document

Target: `docs/development/build-profile-governance.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
exactly the required content of
[the governance contract](01-governance-contract.md) §3–§8: the three switch
classes with the precedence rule, the classification procedure, the Reserved
profile registry with the non-fork rule, the review questions, the prohibited
cases, and the three change thresholds. The document must cite ADR-047,
ADR-037, and ADR-046 as its semantic authority, must mark all examples
informative, and must not name gate commands, CI configuration, runtime
schemas, or any existing switch (there are none).

**Acceptance:** every required section is present with its required content;
the document contradicts no ADR, task-book, or Coding-Guidelines rule; no
statement implies a profile or feature exists.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1, not absorbed by rewording this document.

### Step 2 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing switch-
classification and feature/profile questions at the new governance document,
and add the W04 design row to the stage implementation index with a truthful
status. Change nothing else in either file.

**Acceptance:** a newcomer starting from `docs/README.md` can reach the
governance document in one link; the index row reflects the real status; all
new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 3 — run the representative-decision drill

Target: verification record
(`docs/stages/p0/verification/p0-w04-build-profile-feature-governance-verification.md`,
created in this step).

Work: apply the document's §4 procedure to at least four hypothetical
switches covering all three classes plus one prohibited combination — for
example: (a) "include the experimental virtio backend in the binary"
(capability); (b) "a named build for constrained deployments" (profile,
registry member); (c) "maximum number of supported VMs" (runtime policy —
must be rejected as a feature); (d) "profile that hard-codes a VM's device
set" (prohibited combination — split or reject). For each, record the
question-by-question path, the verdict, and where the rule that decided it
lives. Record what was not exercised and why.

**Acceptance:** the drill shows the procedure yields an unambiguous verdict
for each sample, including the rejections, using only the document's rules.  
**Failure/blocker:** a sample the procedure cannot classify is evidence of a
rule gap — fix the document through review and re-run the drill; do not leave
the gap or special-case the sample.

### Step 4 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement, prerequisite compatibility with
W01's delivered baseline, document links, and downstream handoff wording
(W03, W07, W16, W12, P1+). Completion is claimed only in the verification
record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W04-DV01 → P0-V09 | classification review | inspect the governance document against the contract §3–§4 | three classes defined; precedence rule explicit; procedure yields verdicts; ADR-047/037 semantics preserved | the classification contract exists and is operational; not that real switches comply yet |
| W04-DV02 → P0-V09 | profile registry review | inspect §5 against ADR-047 and the task book | all six names Reserved with purposes and boundaries; non-fork rule present; no implementation or activation implied | the set is protected and bounded; not that profiles build |
| W04-DV03 → P0-V09 | prohibited-case review | inspect §7 against the plan's five categories | all five categories prohibited for features, profiles, and any compile-time form; examples marked informative; authority cited | the red lines are locatable; not future compliance by later changes |
| W04-DV04 → P0-V09/P0-V15 | representative-decision drill | step 3 evidence in the verification record | unambiguous verdicts for all samples including rejections, decided only by the document | later agents can classify switches without re-deriving ADR semantics; not that every future case is covered |
| W04-DV05 → P0-V09 | discovery and link review | resolve the new `docs/README.md` row, governance links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's taxonomy decisions |
| W04-DV06 → P0-V15 | consumability review | read the document as W03 (is my baseline compliant? what routes through here?), W07 (gate vocabulary?), W16 (profile dimension owner?), W12 (visibility boundary?), and a P1 designer (how do I classify my proposed feature?) | each consumer can act without inventing policy | P1 need not redefine the boundary; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the document
without the DV04 drill does not satisfy the plan's fourth work item. No
validation here proves build behavior, gate behavior, or CI enforcement, and
none may be reported as doing so.

## 4. Error, security, and observability model

W04 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is textual: a
missing section, an ambiguous classification, a registry drift, or a drill
gap fails the associated review and is recorded as such.

The security relevance is indirect but real: compile-time switches are a
classic way for security-relevant surface to appear or disappear silently.
The governance model defends this by (a) requiring every capability switch to
be classified and documented, (b) prohibiting runtime policy from hiding in
build flags, and (c) keeping the `secure` profile a reviewed, registered
selection rather than an ad-hoc flag combination. The document must state
that security-posture selections follow the registered profile path, not
local flag bundles.

Observability is the evidence trail: the verification record's drill, review
outcomes, environment, and run/not-run status are the only accepted proof
surface. No logging, tracing, or test framework may be introduced for W04.

## 5. Handoff checklist

Before handing W04 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV06 evidence paths and their run status, including explicit not-run
  entries (no real-switch classification, no gate/CI enforcement, no profile
  build);
- confirmation that no Cargo manifest, feature, `[profile.*]` section,
  dependency, Rust source, `unsafe`, or CI workflow was added or modified;
- confirmation that all examples in the document are marked informative and
  that no statement implies a profile or feature exists; and
- open items for W03 (compliance is by construction; future switches classify
  here), W07 (gate vocabulary), W16 (profile metadata dimension), W12
  (visibility boundary), and P1+ (classification duty in every feature-
  bearing design) — without resolving their contracts here.
