# P8-W02 Review Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before editing, the implementer verifies it has loaded the documents named in
the parent README and confirms the observable state: `docs/machine-types/`
contains only its stub README, and no tracked document defines machine
categories or a `rusthv-arm-virt-v1` process. It also confirms the
[P8-W01](../p8-w01-entry-contract-reconciliation/README.md) record exists or,
if W01 has not yet run, proceeds per §2 step 1's fallback.

Stop and obtain direction instead of guessing when any of the following
occurs:

- drafting a category appears to require naming an address, interrupt number,
  slot count, register model, PSCI function, CPU feature, or console device —
  that value is a routed open item; record the route, do not pick a value;
- the ADR, task book, and W01 record disagree on a route or a constraint —
  raise the conflict per the task book §8 handling; do not reconcile it by
  rewording the governance document;
- a consumer package asks the governance document to bless a value — that is
  the §6 route's job through the ADR/review process, not a governance edit;
- publishing the v1 specification is requested — the §9 gate has not been
  passed by construction at this stage; refuse and record the request.

## 2. Ordered implementation steps

### Step 1 — verify the W01 input

Target: working context; governance document §2 (Normative inputs).

Work: confirm the W01 reconciliation record exists and read its constraint
register and conflict routes. If it does not yet exist, proceed using the W01
design's [constraint register](../p8-w01-entry-contract-reconciliation/01-reconciliation-contract.md)
§4 and §3.3 as the interim source, and record in the implementation record
that the citation must be refreshed to the record once it exists.

**Acceptance:** the governance document's normative-inputs section cites the
W01 register by its final path, or the interim rule is recorded.  
**Failure/blocker:** a W01 conflict that touches machine-model routing must be
resolved through its recorded route before W02 re-states it; W02 never
overrides a W01 conflict row.

### Step 2 — draft the governance document

Target: `docs/machine-types/machine-contract-governance-v0.1.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
the sections fixed in [the governance contract](01-governance-contract.md):
§2 normative inputs; §3 identity rules; §4 the ten categories with required
content; §5 fact classes; §6 the decision-route table; §7 the
host-independence rule; §8 compatibility and version rules; §9 the freeze
gate; §10 consumer rules. Content reflects the resolved decisions; it names
no Guest-visible value anywhere.

**Acceptance:** all sections present with the required content; every open
task-book §8 item appears in §6; no section contains an IPA number, interrupt
number, slot count, register model, PSCI function list, CPU feature value, or
DTB node value.  
**Failure/blocker:** a category that cannot be defined without a value is
defined by its required content plus its route — never by the value.

### Step 3 — host-independence and self-consistency review

Target: the draft document.

Work: review every Guest-visible-shaped statement against the §7 rule (can it
be derived from architecture sources, Linux/PSCI/GIC consumption
requirements, or a routed Zelyr decision — not from host observation?).
Review the document against §4–§9 internal consistency: classes, routes, and
the gate must not contradict each other or the task book's Reserved-item
prohibition.

**Acceptance:** every statement classifiable under §7; no internal
contradiction; the reservation prohibition is checkable as written.  
**Failure/blocker:** a statement that fails §7 is rewritten as a category or
route, or removed — not softened.

### Step 4 — wire discovery

Target: `docs/README.md` routing table;
`docs/stages/p8/implementation/README.md` status row (per that index's
conventions, coordinated with parallel W06–W20 work).

Work: add one routing row pointing machine-contract governance and
machine-model work to the governance document; add the W02 design row with
truthful status. Change nothing else in either file.

**Acceptance:** a reader starting from `docs/README.md` reaches the governance
document in one link; the status row reflects reality; all new relative links
resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review; a status row
that overstates progress is corrected, not defended.

### Step 5 — consumer consumability review

Target: verification record; implementation record.

Work: read the governance document as each named consumer, per
[W02-DV06](#3-validation-matrix), and record the answers. Record any consumer
need that the categories cannot express — that is a governance gap to fix
before W03–W09 depend on it.

**Acceptance:** each consumer question is answerable from the document alone
without inventing policy; gaps are either fixed in the document or routed as
open items.  
**Failure/blocker:** a gap requiring a value decision is routed per §6, not
filled locally.

### Step 6 — closure review

Work: run the validation matrix, confirm the handoff checklist, and verify
the package against the task-book requirement and the plan's acceptance
wording ("Passing does not mean v1 is frozen"). Completion is claimed only in
the verification record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W02-DV01 → P8-V02 | normative-input review | inspect governance §2 against ADR-024/025/040, ADR §18, task book §1/§2/§8 | every listed input carried with citation; ADR §18 items recorded as open with `ADR Required` status | the governance rests on stated authority; not that the ADR decisions exist |
| W02-DV02 → P8-V02 | category completeness review | map task book §2 Required enumeration to governance §4 categories | all ten categories present with required content and routes; identity rules per §3 | a conforming specification is checkable; not that any specification exists |
| W02-DV03 → P8-V02 | fact-class review | test the §5 rules against borderline examples (reservation text, DTB node names, test-environment notes) | each example assigns to exactly one class; default rule stated; reservation prohibition checkable | classification discipline; not future compliance by later documents |
| W02-DV04 → P8-V02 | route-coverage review | enumerate task book §8 topics and ADR §18 machine items; locate each in governance §6 | every open item has exactly one row with owner and status handling; no item resolved locally | escalation is preserved; not that any item is decided |
| W02-DV05 → P8-V02/P8-V03 | host-independence and compatibility review | scan the document for host names, host addresses, and host-fact statements; test §8 rules against sample changes | no Host fact in any Guest-visible statement; incompatible changes unambiguously force a new identity; family-contract binding rule present | the P8-V03 property holds for the proposed contract boundary; not that later documents obey it |
| W02-DV06 → P8-V02/P8-V03 | consumer consumability review | read as W03 (which categories bound the boot contract?), W04 (DTB consistency rule?), W05 (CPU category posture?), W06 (PSCI route?), W14 (compatibility rule?), W20 (freeze gate?) | each consumer can act from categories and gates without inventing values | handoff readiness; not that consumers are designed or v1 frozen |
| W02-DV07 → W02 closure | claim hygiene review | search all W02 artifacts for freeze/publication/completion language | none present; the gate's "not passed" state is stated | scope discipline; nothing else |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with method, input, date, and reason. P8-V02 and P8-V03 are satisfied only
when DV01–DV07 are recorded. No validation here freezes any machine value,
proves P8-V04 through P8-V26, or substitutes for the §9 gate.

## 4. Error, security, and observability model

W02 adds no runtime error path. Its failure model is procedural: a category
answered by an unrouted value, a Host fact in contract position, or an
unowned open item fails the associated review and is corrected or routed. The
security property is boundary integrity: the host-independence rule keeps
host topology and addresses out of Guest-visible statements, which is a
Guest-isolation precondition for every later P8 package; the reservation
prohibition keeps unimplemented mechanisms out of the contract so a Guest
cannot assume them. Observability is the verification record: per-validation
status with planned, run, blocked, and failed evidence kept distinct.

## 5. Handoff checklist

Before handing W02 to a reviewer, provide:

- the exact changed-file list (expected: the governance document, routing
  row, status row, implementation record, verification record);
- DV01–DV07 evidence paths and run status, including explicit not-run entries;
- confirmation that no Guest-visible value (address, number, slot count,
  register model, PSCI subset, CPU feature, DTB value, console model) appears
  in any W02 artifact;
- confirmation that the v1 specification was not created and the §9 gate was
  not claimed as passed;
- the open items handed onward: every §6 route still unresolved, with owner,
  for W03–W20 and the ADR process; and
- the recorded interim-citation status if the W01 record did not yet exist at
  review time.
