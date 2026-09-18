# P8-W14 Implementation and Review Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W14 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any step, the implementer verifies it has loaded the documents named in
the parent README and inspects the actual state of the sources: the W02
governance record, the W04/W05–W09 approved records, the W15 fixture manifest,
and the W16 harness design. Per the plans index, only evidenced/approved facts
may be cited; planned documents are not sources.

Stop and obtain direction instead of improvising when any of the following
occurs:

- a source record is unapproved — the dependent matrix rows stay `blocked`
  with the value cell empty; do not fill them from plans, QEMU observations,
  or plausibility;
- authoring the drift test appears to require harness mechanics, a new
  inspection path, or a fixture change — that belongs to W16/W15; record the
  need in the test plan as a consumer requirement and stop;
- a proposed change alters an existing entry's value or semantics — classify
  C3/C4 per [01 §4](01-compatibility-matrix-and-policy.md) and route through
  the governance/ADR process; W14 never approves its own version change;
- a matrix row cannot name an owner or authoritative source — record it as an
  open item against the owning package; do not adopt the fact silently;
- any work seems to require migration/snapshot compatibility decisions — those
  are Reserved for later stages; record the need, do not design them;
- an authority requests a v1 value inside W14 artifacts — refuse and route to
  the W02 governance route; selecting values is out of scope for this package.

## 2. Ordered implementation steps

### Step 1 — inventory the source facts

Target: implementation record (`../p8-w14-machine-abi-compatibility-record.md`,
created in this step).

Work: for each row of [01 §1](01-compatibility-matrix-and-policy.md), record
the source record's status (approved / planned) and the sections that will
supply the matrix cells. Confirm every plan-enumerated Guest-visible area
(map, device location, interrupt assignment, DT compatible, CPU topology,
PSCI, timer, console, machine identity) has an owner.

**Acceptance:** the inventory covers all areas with named owners; unapproved
sources are marked, not skipped.
**Failure/blocker:** an unowned fact area is an open item for the governance
route; the matrix entry stays blocked.

### Step 2 — author the compatibility matrix and policy document

Target: the artifact per [01 §1](01-compatibility-matrix-and-policy.md) (under
`docs/abi/`, name recorded in the implementation record).

Work: write the document with the status header required by the documentation
index (status, scope, version, owner/change context, supersedes) and the
required content of [01 §2–§7](01-compatibility-matrix-and-policy.md): entry
schema, full enumeration with `value` cells empty and status `blocked`,
change classes, escalation conditions, test plan, and QEMU firewall. Cite
sources; transcribe nothing.

**Acceptance:** every required section present; every entry has fact_id,
category, source, owner, comparison method, and a blocked/empty value cell;
no address, count, register model, feature value, or PSCI subset appears
anywhere; the firewall section is present.
**Failure/blocker:** a value that has appeared is removed and its source
routed to governance; the document never absorbs one.

### Step 3 — review the policy and matrix

Target: review evidence in the verification record.

Work: run the W14-DV02/DV03/DV05 reviews (§3) with the source inventory:
completeness of the enumeration, mechanical applicability of the change
classes, and firewall coverage. Verify against the plan's scope wording.

**Acceptance:** reviews pass; open items are recorded with owners.
**Failure/blocker:** a gap in the enumeration or policy fails the review; fix
the document, not the review bar.

### Step 4 — record the drift test plan and its blocked state

Target: test-plan section of the artifact; verification record.

Work: finalize the test plan per
[01 §6](01-compatibility-matrix-and-policy.md), name its consumer requirements
for W16 (evidence paths it must collect) and W15 (fixture/configuration
pinning), and record explicitly that execution is **blocked** until v1 facts
are approved — with the blocking condition quoted from the plan.

**Acceptance:** the plan is complete enough for W16 to design mechanics
without re-deciding policy; the blocked state and its condition are recorded.
**Failure/blocker:** a missing consumer requirement is an open item for that
consumer; the plan is not declared runnable.

### Step 5 — closure review

Work: run the validation matrix (§3), confirm the handoff checklist (§5), and
verify the package against the plan's acceptance wording and the task-book
P8-V19 row — including its explicit condition that P8-V19 cannot pass before
v1 facts are approved, so its planned execution evidence is recorded as
blocked/not run, never as passed. Completion of W14 itself (policy artifact +
review) is claimed only in the verification record, with evidence, and only
for what was actually reviewed.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W14-DV01 → P8-V19 | source-fact inventory review | inspect W02/W04/W06–W09 record status against 01 §1 | every matrix area has a named owner and source status; none assumed | the inventory is real; not that values exist |
| W14-DV02 → P8-V19 | matrix completeness review | review the artifact against 01 §2–§3 | all enumerated categories present with schema-conforming entries; all values blocked/empty; non-comparable list present | the comparison set is complete and owned; not that it matches a running system |
| W14-DV03 → P8-V19 | change-class/escalation review | review 01 §4–§5 against ADR-040 and the plan scope | classes mechanically classifiable; C3/C4 routes end in governance/ADR, not W14; drift never resolved by editing the matrix | the policy prevents silent version/ADR-level change; not future compliance |
| W14-DV04 → P8-V19 | drift-test plan review (and blocked execution record) | review 01 §6 for consumer completeness; record execution as blocked with the plan's condition | plan is policy-complete for W16; blocked state explicit; no run claimed | the route is planned and honestly blocked; once unblocked and executed: same-configuration drift detection — not guest-software compatibility, performance, or migration |
| W14-DV05 → P8-V19/P8-V03 | QEMU firewall review | review 01 §7; search the artifact for environment-derived values | firewall rules present; zero QEMU-derived values or behavior-keyed assertions in the artifact | environment observations cannot become ABI facts; not that all future artifacts comply |
| W14-DV06 → closure | consumer consumability review | read the artifact as W16 (mechanics), W20 (closeout), P9 (extension rule), W02 (classification aid) | each consumer can act without inventing policy or values | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Authoring the matrix
without the reviews does not satisfy the plan, and P8-V19 must never be
recorded as passed while its execution remains blocked. Nothing here
contributes to P8-V12–V18 or P8-V20–V26.

## 4. Error, security, and observability model

**Errors and failure guarantee.** W14 adds no runtime behavior. Its failure
model is procedural: an unapproved source, an incomplete enumeration, an
unclassifiable change, or a firewall violation fails the associated review and
is recorded as such. The preserved guarantee: an approved machine version
cannot drift silently — every Guest-visible delta is either classifiable as C1,
routed as C2, version-bearing as C3, or ADR-level as C4, and drift is never
resolved by editing the matrix to match the implementation.

**Security.** The matrix is part of the machine-ABI governance surface
(ADR §19 `MUST` on versioned machine ABI): the policy is what prevents a
review-visible but underconsidered weakening of Guest-visible guarantees (for
example, a reserved window quietly becoming mapped, or a PSCI function quietly
changing semantics). The firewall is also a security rule: environment-derived
"facts" are exactly the channel through which host-specific assumptions leak
into a host-independent contract (P8-V03's boundary). No code, no Guest input,
and no runtime surface is touched by this package.

**Observability.** The evidence surface is the verification record: the
source inventory with statuses, the review outcomes, the explicit blocked
state of the drift test with its condition, and the consumer open items. The
artifact itself is the durable observable: any later reviewer can see which
entries are approved, blocked, or changed, and under which class.

## 5. Handoff checklist

Before handing W14 to a reviewer, provide:

- the exact changed-file list (expected: the compatibility matrix/policy
  document under `docs/abi/`; the implementation record; verification entries;
  no code and no contract edits);
- the source inventory with per-area approval status (W14-DV01);
- confirmation that the artifact contains no v1 value, register model,
  address, count, feature value, or PSCI subset, and that all value cells are
  empty/blocked (W14-DV02/DV05);
- the change-class and escalation review outcome (W14-DV03);
- the drift-test plan with named W15/W16 consumer requirements and the explicit
  blocked-execution record (W14-DV04);
- open items: unowned fact areas for the governance route, source-record
  format gaps for W04/W06–W09 owners, Reserved migration/snapshot needs for
  later stages — without resolving them here.
