# P8-W20 Closure Workflow and P9 Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W20 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any closure work, the implementer verifies it has loaded the documents
named in the parent README and inspects the tracked tree. The following are
**assumed contracts**; a failure in any of them produces a gate status of
`missing` or `blocked` — never a local repair, never a rewrite of another
package's scope:

- **W14 and W16–W19 verification records**
  (`p8-w14-machine-abi-compatibility` through
  `p8-w19-validation-guest-dual-track`): the evidence this package indexes
  and the requirement statements it audits. *Failure boundary:* absent
  records ⇒ those gates stay `missing`; W20 does not summarize plans as
  evidence.
- **W01–W13 verification records**: required for the full gate matrix;
  handled identically.
- **The W02-governed machine-value approval**
  (`p8-w02-machine-contract-governance`; task book §8 `ADR Required`):
  precondition for publishing the factual machine specification. *Failure
  boundary:* unapproved values ⇒ the specification is not published and its
  gate stays `blocked`; closure is recorded as not reachable, factually.
- **P0 unsafe and dependency governance** (P0-W10 unsafe governance,
  P0-W18 dependency governance, per the P0 plan index): the format and
  review rules the delta record assembles from.
- **The [integration workflow](../../../../development/integration-workflow.md)**:
  closure decisions and publication go through branch/PR review like any
  change; W20 claims no bypass.

Stop and obtain direction when: evidence contradicts a factual document
candidate; a gate cannot be classified under the closed enum; an owning
package's record and the task book conflict; or closing would require an
unapproved decision.

## 2. Ordered closure workflow

### Step 1 — collect requirement and record inventory

Target: closure review working notes (verification record, created in this
step).

Work: inventory every P8 package's verification record, requirement
statements, recorded limitations, escalations, and unsafe/dependency deltas.
Verify each against [01](01-closure-artifact-contract.md) §4's gate groups.

**Acceptance:** a complete inventory with per-package links; gaps listed,
not guessed.  
**Failure/blocker:** an absent record is recorded `missing` with the owner
named.

### Step 2 — assemble and publish factual documents

Target: factual machine specification, boot specification, compatibility
policy ([01](01-closure-artifact-contract.md) §1 locations).

Work: assemble each factual document strictly from approved decisions and
linked evidence, per the §2 publication rule. Each document carries a status
header (status, scope, version, owner/change context, supersedes) per
`docs/README.md`, and cites its evidence.

**Acceptance:** every published factual statement traceable to an approved
decision and a linked record; nothing published ahead of its approval.  
**Failure/blocker:** a missing approval blocks publication; the gate stays
`blocked`. Under no condition is a candidate value published to unblock a
gate.

### Step 3 — evaluate the gate matrix

Target: validation report and evidence index
(`docs/stages/p8/verification/`).

Work: evaluate every gate P8-V01–V26 to its closed-enum status with linked
evidence; audit each `evidenced` claim against the actual verification
content (does the record show a run, a review, and an environment, or only
intent?). Record the limitations/unresolved-decision register and the
unsafe/dependency delta record per [01](01-closure-artifact-contract.md) §5.

**Acceptance:** all 26 gates classified; register and delta records complete
with source links; any `ADR Required` / `Architecture Change Request` item
visible.  
**Failure/blocker:** an unclassifiable gate is escalated to the project
owners as an open question; the enum is never extended ad hoc.

### Step 4 — author the P9 consumer statement

Target: P9 consumer statement ([01](01-closure-artifact-contract.md) §1).

Work: state only evidenced facts, bounded by task book §7: the approved
machine identity and Guest-visible contract facts, the bounded reservation
policy, Guest-only DTB and Linux boot inputs, console behavior, Linux SMP
integration and its limits, the compatibility-test route, and the repeatable
regression fixtures — each with its evidence location and limit. State the
exclusions explicitly: no Virtio semantics, no multi-VM containment, no
hardware-transferable performance or isolation claims.

**Acceptance:** every sentence has an evidence link or is an explicit
exclusion; a P9 planner can decide from it what to rely on.  
**Failure/blocker:** a desired statement without evidence becomes a
register row, not a softened sentence.

### Step 5 — closure review and publication

Work: run the validation matrix (§5); present the closure review record —
gate statuses, registers, deltas, and the handoff statement — to the project
owners through the integration workflow. The completion decision is theirs;
this package's output is the decision *input*. Completion is claimed only in
the verification record, with evidence, for what was actually reviewed.

**Acceptance:** review record complete and reviewable; no closure claim
beyond what evidence supports.  
**Failure/blocker:** open security or architecture blocks keep the affected
gates non-`evidenced`; P8 is recorded as not closed, factually and without
embellishment.

## 3. Conflict and escalation handling

- **Evidence vs document conflict:** the evidence is authoritative; the
  document is corrected or its publication is withdrawn. Never the reverse.
- **Plan/design conflict found at closure:** recorded as a finding routed to
  the owning package; W20 does not reinterpret a plan to make a gate pass.
- **ADR conflict:** any closure finding that contradicts an accepted ADR is
  recorded `Architecture Change Request` or `ADR Required` with the
  superseding-ADR route cited (`docs/adr/README.md`); the affected gate stays
  non-`evidenced`.
- **Scope creep temptation:** requests to fold Virtio, management, or
  hardware claims into P8 closure documents are refused and recorded;
  they belong to their own stages.

## 4. P9 handoff contract

The handoff is a one-way, evidenced-facts contract:

```text
P9 may rely on            (each with evidence link and stated limit)
  - approved machine identity rusthv-arm-virt-v1 and its Guest-visible
    contract facts
  - the bounded, documented reservation policy
  - Guest-only DTB and Linux boot inputs (fixture references included)
  - console behavior and its containment limits
  - Linux SMP integration facts and their stated limits
  - the machine-compatibility test route
  - the repeatable regression fixtures and their evidence

P9 must design separately (receives nothing from P8 for these)
  - virtio protocol, queues, transports, device behavior
  - any management, multi-VM, DMA/IOMMU, or hardware-platform capability

P9 may not do
  - cite planned-but-unevidenced P8 work as a foundation
  - quote QEMU/TCG baselines as hardware expectations
  - treat single-VM containment evidence as multi-VM isolation
```

Future maintenance stages receive the same statement plus the factual
specification set and compatibility policy as the reference for what v1 is;
the limitations register travels with it.

## 5. Validation matrix

All rows are planned evidence; none asserts a review has occurred. Record
each as **passed**, **failed**, **blocked**, or **not run** with input,
environment, timestamp, and reason.

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W20-DV01 → prerequisites | inventory review | inspect §1 assumed contracts and the Step 1 inventory | every package's record status known; gaps listed with owners | inventory completeness; not that evidence exists |
| W20-DV02 → P8-V26 | document-kind and publication review | audit §2–§3 of [01](01-closure-artifact-contract.md) against the published set | no factual document published ahead of approval+evidence; no kind mixing; headers present | publication discipline; not document correctness |
| W20-DV03 → P8-V26 | gate-matrix audit | evaluate all P8-V01–V26 rows; sample-verify `evidenced` links against actual verification content | all gates classified; every `evidenced` link resolves to real evidence; no planned work counted as evidence | the exit-criterion map is real; not that every gate passed |
| W20-DV04 → P8-V26 | register and delta review | review the limitations register and unsafe/dependency deltas against package records | complete, sourced, with classes and effects; inherited `ADR Required`/`Architecture Change Request` items visible | factual limits/deltas; not their resolution |
| W20-DV05 → P8-V26 | handoff-statement review | read the P9 statement against task book §7 and the evidence links | every statement evidenced or an explicit exclusion; a P9 planner can act on it | handoff readiness; not P9's design |
| W20-DV06 → W20 closure | closure-review integrity review | re-run a sample of gate evaluations; check no completion claim exceeds evidence | review outputs reproducible; claims bounded to evidence | review discipline; never a claim that P8 closed |

Only W20-DV03/DV04/DV05 with real records can satisfy P8-V26, and P8-V26
itself is the *review* that all other evidence is present — the last gate to
evaluate, never the first.

## 6. Error, security, and observability model

- **Failure model:** W20's named failure modes are: premature publication,
  kind-mixing, evidence-free `evidenced` statuses, softened limitation
  statements, silently dropped register rows, and closure claims exceeding
  evidence. Each is a review failure with a recorded remedy (withdraw,
  reclassify, restore the row, or record the block).
- **Security position:** W20 introduces no trust boundary, but it is where
  security claims could be overstated; the containment-limit statement
  ([W18](../p8-w18-security-isolation-regression/README.md)) and the
  limitations register are the controls. Closure documents must carry
  QEMU-scope and single-VM-scope limits next to any containment sentence.
- **Observability:** the closure review record, gate matrix, registers, and
  handoff statement are the proof surface. Every downstream reader must be
  able to trace any closure statement back to a package record and its
  evidence.

## 7. Handoff checklist

Before handing W20 to a reviewer, provide:

- the exact artifact list (factual documents, validation report, evidence
  index, registers, handoff statement, review record) with publication
  status per the §2 rule;
- W20-DV01–DV06 evidence paths and run status, including explicit
  not-run/blocked entries;
- the gate matrix with all 26 statuses and the list of non-`evidenced`
  gates with their owners;
- the limitations/unresolved-decision register, including every
  `ADR Required` / `Architecture Change Request` item and the inherited
  task book §8 machine-values item;
- the unsafe/dependency delta record with linked approvals;
- confirmation that no Virtio, management, multi-VM, or hardware claim
  entered any closure artifact, and that no accepted ADR was edited;
- confirmation that no closure claim appears anywhere except as bounded
  review output — without resolving any open item here.
