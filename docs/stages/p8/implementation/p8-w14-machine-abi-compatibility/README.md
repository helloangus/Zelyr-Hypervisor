# P8-W14 Machine ABI Compatibility — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The compatibility-test route that prevents silent drift of an
approved machine version, as required by
[P8-W14](../../plans/p8-w14-machine-abi-compatibility.md).  
**Owner-change context:** P8-W14 implementation handoff; compatibility policy
over the approved `rusthv-arm-virt-v1` machine-contract facts. It owns no
machine value and freezes nothing.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W14. The plan makes W14 the
package that ensures an approved machine version cannot drift silently: the
Guest-visible facts that must be compared, the distinction between a compatible
internal change and a version/ADR change, the escalation conditions, and the
planned drift-detection test. This design converts that into (a) a
compatibility fact matrix whose entries name categories and owner packages
while every value comes from the approved v1 record, (b) a change-class policy
with explicit escalation, (c) a drift-detection test plan at policy level, and
(d) the QEMU firewall that keeps environment observations from becoming ABI
facts. It deliberately does **not** select v1 values, implement test mechanics,
or address migration/snapshot compatibility, and it never changes an approved
contract silently.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) — in
particular its ABI rule that external ABI requires explicit representation,
width, endianness, padding, versioning, and compatibility behavior. It then
loads only the linked supporting file needed for its assigned step:

- [Compatibility matrix and policy](01-compatibility-matrix-and-policy.md) —
  read before authoring or reviewing the matrix, policy, or test plan.
- [Implementation and review](02-implementation-and-review.md) — read before
  executing; ordered steps, validation matrix, failure/security/observability
  model, and handoff checklist.

Before editing, the agent must also satisfy the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index, ADR baseline, P8 task book, and
the P8-W14 plan. Nothing here claims that v1 facts are approved or that a
drift test has run; per the plan, P8-V19 cannot pass before v1 facts are
approved.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → approved machine-contract
facts → P8-W14 plan → this design → Coding Guidelines. In particular:

- ADR-024 (versioned Generic ARM64 VM machine), ADR-040 (independent
  `schema_version` / `machine_version` / `management_abi_version`), and ADR §19
  (`MUST`: machine ABI carries versions) are the architectural basis. W14
  operationalizes compatibility for `rusthv-arm-virt-v1`; it does not create
  the versioning scheme, which ADR-040 and the
  [P8-W02](../p8-w02-machine-contract-governance/README.md) governance route
  own.
- The task book routes every concrete machine value through the W02
  decision route; the plan forbids selecting values, implementing test
  mechanics, and silently changing an approved contract. This design therefore
  authorizes only the comparison framework, policy, and test plan.
- The plan's acceptance is explicitly conditional: "P8-V19 requires a policy
  and planned drift-detection test for the same approved configuration. It
  cannot pass before v1 facts are approved." The design treats an unapproved
  matrix value as a blocked entry, never a placeholder that silently becomes a
  fact.

Classification:

- **Required:** the Guest-visible fact matrix (categories, entry schema, owner
  citations) with values deferred to the approved v1 record; the change-class
  policy (compatible internal / version-bearing / ADR-required) and escalation
  conditions; the drift-detection test plan over the same approved
  configuration; the QEMU firewall rules; the consumer handoff to W16, W20,
  and P9+.
- **Reserved:** migration-compatibility and snapshot-format policy (P17+
  lane; named Reserved so nothing here blocks them); compatibility automation
  beyond the planned test (W16's mechanics); multi-version coexistence rules.
- **Out of Scope:** selecting v1 values (addresses, slot counts, register
  models, feature values, PSCI subset, Linux configuration); test-harness
  implementation mechanics; migration stream/snapshot format; workload marker
  and validation-parameter facts (explicitly not ABI); any edit to an approved
  contract.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02/W04/W06–W09 and all approved machine-contract facts | [ledger](#current-state-findings-and-goal-to-baseline-ledger); [matrix](01-compatibility-matrix-and-policy.md) §1–§2 | W14-DV01 source review |
| Enumerate Guest-visible facts that must enter compatibility comparison | [matrix](01-compatibility-matrix-and-policy.md) §3 | W14-DV02 matrix review |
| Define version and incompatibility escalation conditions | [matrix](01-compatibility-matrix-and-policy.md) §4–§5 | W14-DV03 policy review |
| Relate fixtures and DTB assertions to the approved machine contract | [matrix](01-compatibility-matrix-and-policy.md) §6 | W14-DV04 test-plan review |
| Review that QEMU observations cannot become ABI facts | [matrix](01-compatibility-matrix-and-policy.md) §7 | W14-DV05 firewall review |
| P8-V19 policy + planned drift-detection test | [workflow](02-implementation-and-review.md) §2, §3 | W14-DV04 (execution blocked until v1 approved) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation scaffold only. No machine contract record, no approved v1
facts, no fixture, no regression harness, and no compatibility artifact exist.
The machine-identity decision itself is ADR-deferred (ADR section 18 via the
task book). Each ledger row states the missing foundation the plan outcome
requires and who owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| No silent drift of an approved machine version (P8-V19) | No machine version exists; v1 values ADR-deferred | Approved `rusthv-arm-virt-v1` facts via the W02 governance route (categories now, values later) | Comparing drift requires an approved baseline to compare against | W02 route; ADR-024/040 | Approved v1 machine-contract record |
| Guest-visible fact enumeration | No enumeration exists | The matrix (01 §3): category, entry schema, owner citations | The comparison set must be complete and owned before values exist | W14 (this design); facts per owner packages | W14-DV02 |
| Compatible vs version/ADR change distinction | No policy exists | Change-class policy with escalation (01 §4–§5) | Without it, review cannot classify a machine-affecting change | W14 (this design), within ADR-040 versioning | W14-DV03 |
| Drift-detection test plan | No test, no fixture, no harness | Test plan over the same approved configuration (01 §6), realized by W16 mechanics with the W15 fixture | "Same approved configuration" must be a controlled comparison, not an ad-hoc boot | W14 plan; W15 fixture; W16 execution | W14-DV04; execution blocked until v1 approved |
| QEMU observations never become ABI facts | QEMU is the only runnable reference (ADR-003) | Firewall rules (01 §7) | Environment behavior must not leak into the contract (P8-V03 boundary) | W14 (this design); W02 governance enforces | W14-DV05 |
| DTB/device/IRQ/timer/console/PSCI facts comparable | W04/W06–W09 are plans | Each owner package's approved design/record as the fact source per matrix row | A comparison entry is only as good as its owned, evidenced source | W04, W06, W07, W08, W09 designs | W14-DV01 |

No row authorizes W14 to fix values; where an upstream approval delivers
differently than assumed, [the workflow](02-implementation-and-review.md) §1
failure boundary applies.

## Resolved design decisions and their authority

1. **The matrix enumerates categories and entry schema now; values only from
   the approved v1 record.** Each matrix row names a Guest-visible fact
   category, its owner package, and its authoritative record; the value column
   stays empty until the v1 record exists and is then filled by citation, not
   transcription. Rationale: the plan forbids selecting values, yet the
   enumeration itself must exist for the policy and test plan to be complete.
2. **Change classes: C1 compatible internal change, C2 additive
   contract-governed change, C3 Guest-visible change (version-bearing), C4
   ADR-level change.** The plan assigns W14 the distinction; the thresholds are
   stated in [01 §4](01-compatibility-matrix-and-policy.md) so review can
   classify mechanically. Any C3/C4 routes through the machine-contract
   governance and ADR process rather than W14's own discretion.
3. **Machine version identity is a matrix fact like any other.** The version
   string/number itself is enumerated and drift-checked; W14 defines no new
   versioning scheme (ADR-040's). Rationale: prevents "the version changed but
   nothing else did" from escaping comparison.
4. **The drift test compares declared facts, not behavior samples.** The test
   plan compares each matrix entry's declared value/semantic against the
   approved record, using Guest-observable and hypervisor-query evidence from a
   boot of the pinned fixture ([W15](../p8-w15-reproducible-linux-fixture/README.md))
   in the same approved configuration. Timing, performance, and QEMU-version
   behavior are excluded (01 §7). Rationale: P8-V19's "same approved v1
   configuration detects drift in … facts" wording.
5. **Validation parameters are explicitly non-ABI.** RAM-class capacities
   (W12), workload markers (W11/W12), scenario bounds, and fixture build
   metadata are enumerated as non-comparable so they cannot drift into the
   matrix. Rationale: keeps test-harness facts from becoming Guest ABI by
   accretion.
6. **Blocked-until-approved is a first-class state.** Until v1 facts are
   approved, the drift test is `blocked`, P8-V19 cannot pass, and no entry may
   be provisionally filled. Rationale: the plan's explicit condition.

## Work breakdown and loading order

1. Read [the compatibility matrix and policy](01-compatibility-matrix-and-policy.md)
   to understand the fact enumeration, entry schema, change classes,
   escalation, test plan, and firewall rules.
2. Read [the implementation and review workflow](02-implementation-and-review.md)
   to author the matrix/policy artifact, review it, and record evidence.
3. Author and review in the order given there: source inventory, matrix, policy,
   test plan, firewall, then the blocked-state closure with explicit consumer
   handoff.
4. Store review evidence in
   `../../verification/p8-w14-machine-abi-compatibility-verification.md`, and
   record the authored artifact location and any deviation in
   `../p8-w14-machine-abi-compatibility-record.md` only when implementation
   begins. Neither this design nor a written record may claim W14 complete, and
   neither may claim P8-V19 passed while v1 facts are unapproved.

## Explicitly excluded interfaces

W14 designs no machine value, register layout, address map, PSCI subset, CPU
feature set, DTB property value, or console/timer/GIC behavior; no code
interface of any kind — no type, function, crate, module, script, or harness
internal is authorized (the drift test's mechanics are W16's); no migration or
snapshot format; no edit to any approved contract. The only artifacts fixed
here are the compatibility matrix/policy document and the test plan, both
defined in
[01-compatibility-matrix-and-policy.md](01-compatibility-matrix-and-policy.md).
If any work seems to require an excluded item, stop and record it per
[the workflow](02-implementation-and-review.md) §1.

## Downstream handoff

- **W16** ([design](../p8-w16-automated-linux-regression/README.md)) receives
  the matrix, change classes, test plan, and firewall rules as normative
  content for its drift-detection regression rows (P8-V21 references; P8-V19
  execution). W16 owns mechanics and evidence.
- **W20** ([design](../p8-w20-documentation-closure-handoff/README.md)) receives
  the compatibility route, the matrix's owner citations, and the explicit
  blocked state of P8-V19 for closeout and the factual-documentation route.
- **P9 and later** receive the binding rule that virtio windows, machine
  reservations, and any new Guest-visible surface enter the matrix through the
  C2/C3/C4 routes before implementation — no silent accretion of Guest-visible
  facts. Migration/snapshot compatibility policy (P17) starts from this matrix
  but supersedes its own scope.
- **W02 governance** receives the change classes as the review's mechanical
  classification aid; authority over machine decisions stays with the W02
  route and the ADR process.
- **W04/W06–W09** receive the requirement that their approved records name
  their Guest-visible facts in matrix-citable form; gaps route back to them.
