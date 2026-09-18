# P0-W18 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W18 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no dependency-governance policy or
register exists yet) and a search of tracked documents for existing dependency
statements. The implementing machine's locally installed crates, toolchains,
and caches are machine-local state and are never repository dependencies or
evidence.

Stop and obtain direction instead of guessing when any of the following occurs:

- a tracked document already states dependency rules (a sibling package or a
  later-delivered W05/W06 document landed first) — reconcile in one change per
  the single-source rule; do not create a second authority;
- W10's delivered unsafe governance names boundary classes or an inventory
  that conflicts with this design's unsafe-dimension wording — record the
  divergence as a cross-package conflict surface in the cross-review step and
  reconcile in the same change; do not restate W10's mechanics here;
- closure appears to require choosing, approving, or rejecting a real
  dependency, designing a wrapper API, creating a Cargo manifest, or deciding
  lockfile mechanics — those are out of scope (the plan forbids the first
  two; the build-baseline package owns the others); reaching for them here is
  a scope violation; or
- a reviewer asks to pre-populate the register with example entries — refuse:
  an invented entry falsifies the audit trail.

## 2. Ordered implementation steps

### Step 1 — record prerequisite status assumptions

Target: implementation record (`p0-w18-dependency-governance-record.md`,
created in this step).

Work: observe which prerequisite/consumer contracts exist in the tree at
implementation time: W05 documentation baseline (may re-home the policy and
register), W06 ADR governance (escalation path the policy references), W10
unsafe governance (unsafe-dimension interface), and the build-baseline package
(expected W03 per the plan index; owns the first manifest and lockfile
mechanics). Record each one's observable status and the assumption the policy
makes about it.

**Acceptance:** the record names each contract's observable status and the
assumption taken.  
**Failure/blocker:** an in-tree conflict is raised per §1, not absorbed.

### Step 2 — create the governance policy document

Target: `docs/development/dependency-governance.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
all sections required by [the governance policy](01-governance-policy.md):
purpose and fail-closed statement, the three tiers with classification and
approval rules, the eleven-dimension checklist, lifecycle/trigger/exception
rules, ADR thresholds, the three consistency rules, and mutation rules.

**Acceptance:** every required section present; no crate is named, approved,
rejected, or recommended; no wrapper API, manifest, or lockfile rule appears;
the document contradicts no ADR, task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction with a governing document is raised per
§1; fix the draft against the policy, never the policy against a convenience.

### Step 3 — create the dependency register

Target: `docs/development/dependency-register.md`.

Work: write the register with the status header, the up-front rules and the
entry/event schema per [the register schema](02-dependency-register-schema.md),
and **zero entries**. State that emptiness reflects the observed dependency
count of the repository.

**Acceptance:** schema complete; register empty; no illustrative entries; a
reader can determine exactly what a future D3 introduction must record.  
**Failure/blocker:** any invented entry fails review and is removed, not
annotated.

### Step 4 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing-table row to `docs/README.md` pointing dependency
evaluation and approval work at the policy (and register), and add the W18
design row to the stage implementation index with status "Proposed design;
implementation not claimed" (updated truthfully as work proceeds). Change
nothing else in either file.

**Acceptance:** one-link reachability from `docs/README.md`; truthful index
status; all new relative links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 5 — completeness rehearsal (work sequence 4)

Target: verification record evidence.

Work: walk a **hypothetical** future candidate through the full path and
record the walkthrough: classify it against the tier definitions (state which
tier and why, including a borderline case), run all eleven checklist
dimensions (recording what evidence a reviewer would demand for each, without
researching a real crate), obtain the tier's notional approval step, draft the
register entry and one lifecycle event per the schema, exercise one upgrade
and one exception scenario, and identify one trigger that would make the case
`ADR Required`. The candidate must remain hypothetical — a described shape
("a `no_std`-compatible parser crate with transitive dependencies"), never a
named crate. Record any rule gap the rehearsal exposes; a gap is fixed in this
change by a policy amendment, not left as a note.

**Acceptance:** the walkthrough completes end-to-end with no unanswered
"what do I do here" step, and any exposed gap is fixed in the policy in the
same change.  
**Failure/blocker:** a gap that cannot be fixed without selecting a real
dependency or making an architectural choice is recorded `blocked` with the
decision owner named.

### Step 6 — closure review and evidence

Targets: verification record and implementation record.

Work: run the validation matrix below; confirm the handoff checklist; verify
the package against its task-book requirement, prerequisite compatibility,
document links, and downstream handoff wording. Record every validation as
passed, failed, blocked, or not run with command, output, timestamp, and
reason.

**Acceptance:** all Required validations passed or explicitly blocked with a
named surface.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis; do
not weaken a tier or checklist dimension to make a check pass.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W18-DV01 → P0-V09 | checklist completeness review | inspect the policy checklist against the task-book dimensions (TCB, no_std, license, maintenance, unsafe, platform) and the plan's additions (transitive, allocation, stability, security) | all eleven dimensions present with mandatory questions; task-book dimensions unremovable; evidence-source rule present | the evaluation basis is complete and reviewable; not that any evaluation has occurred |
| W18-DV02 → P0-V09 | tier-boundary review | review §3 of the policy against the plan's three-boundary requirement | D1/D2/D3 defined by linkage; upward reclassification follows the full D3 path; D3 requires maintainer approval | TCB entry cannot pass through an ordinary review; not that the boundary is enforced by tooling |
| W18-DV03 → P0-V09 | register schema review | inspect the register against [the register schema](02-dependency-register-schema.md) | entry fields, all seven event types, maintenance rules, empty state, no illustrative entries | lifecycle records are defined and auditable; not that any dependency is recorded |
| W18-DV04 → P0-V09 | completeness rehearsal | execute Step 5's hypothetical-candidate walkthrough | end-to-end path with no unanswered step; exposed gaps fixed; candidate stays hypothetical | the rules are complete enough to govern a real future decision; not that a real candidate would pass |
| W18-DV05 → P0-V09 | locatability review | resolve the routing row, policy, register, and index links from a fresh checkout; search tracked docs for other dependency-rule statements | governance path reachable in one link from `docs/README.md`; single authority | P0-V09 locatability; not W05's taxonomy |
| W18-DV06 → P0-V09 | unsafe-interface cross-review | compare the policy's unsafe dimension and §6.2 against W10's delivered rules (or record blocked with the [W10 design](../p0-w10-unsafe-rust-governance/README.md) as the named surface) | no restatement or pre-emption of W10 mechanics; third-party unsafe assessed, not adopted; alignment recorded | the two governance documents interlock; not that W10 is implemented |
| W18-DV07 → W18 closure | consumability review | read the policy as a P1 designer (can I take a dependency through the path?), as the build-baseline package (is the manifest rule actionable?), and as W07/W20 (is a consistency check implementable?) | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the documents
without DV04 and DV07 does not satisfy W18 closure. No validation here proves
P0-V01–V08 or P0-V11–V15 and none may be reported as doing so; in particular
P0-V11 (unsafe governance) is W10's evidence, touched by W18 only at the
interface.

## 4. Error, security, and observability model

W18 adds no hypervisor error model, synchronization, guest input, hardware
access, telemetry, or `unsafe` code. Its failure reporting is documentary: an
incomplete checklist, an unclassified tier, an unregistered manifest
dependency (once manifests exist), or an expired exception fails the
associated review and is recorded as such.

The security position is supply-chain fail-closure: the register is an
allowlist (an unregistered dependency is unapproved); D3 entry requires
deliberate elevated approval; third-party unsafe is weighted as permanent,
unmodifiable TCB; advisory responses have an expedited path with explicit
risk-acceptance expiry; and exceptions always carry expiry dates so temporary
deviations cannot silently become permanent. The policy's mutation rules keep
the fail-closed default itself out of ordinary-edit reach.

Observability is the evidence trail: the verification record's rehearsal,
review outputs, statuses, and timestamps are the only accepted proof surface.
Once real dependencies exist, the register is the observable record a reviewer
or auditor uses to trace any dependency from manifest back to its evaluation
and approval.

## 5. Handoff checklist

Before handing W18 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV07 evidence paths and their run status, including explicit
  `blocked` entries (W10 cross-review pending) and the rehearsal record;
- the recorded prerequisite status assumptions from Step 1;
- confirmation that no dependency was added, approved, or rejected; no Cargo
  manifest, lockfile rule, wrapper API, vendoring mechanism, Rust source,
  `unsafe`, or CI change was made; and the register is empty; and
- open items for W10 (unsafe-interface alignment), the build-baseline package
  (first-manifest consistency rule and lockfile mechanics), W07/W20
  (candidate automated consistency/advisory checks), W05 (possible re-homing
  of policy and register), and the license-notice obligation that activates
  with the first real dependency — without resolving their contracts here.
