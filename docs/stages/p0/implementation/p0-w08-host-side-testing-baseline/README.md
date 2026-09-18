# P0-W08 Host-Side Testing Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The host-test organization rules, testable-logic boundary, CI- and
gate-callable execution entry, coverage-category matrix, and proof-boundary
statement required by
[P0-W08](../../plans/p0-w08-host-side-testing-baseline.md).  
**Owner/change context:** P0-W08 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W08. It converts the bounded
work-package plan into one normative host-test baseline contract, one
coverage-category contract, and a minimal, real host-test execution that makes
the entry trustworthy. It deliberately does **not** implement parsers,
capability logic, address types, page-table, or ABI mechanisms (plan
out-of-scope), does **not** create the Cargo workspace, host target, or any
crate ([P0-W03](../p0-w03-aarch64-build-target-baseline/README.md) owns the
workspace and target baseline), does **not** define quality gates
([P0-W07](../p0-w07-development-quality-gates/README.md) binds its host-test
gate to this package's entry), and does **not** wire CI
(P0-W20; its design is being prepared in parallel per the plan index).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task
book, and P0-W08 plan. This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W08 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W08 is: host-testable logic has a CI-executable
  baseline **independent of QEMU** (P0-V03–V04). Independence from QEMU is the
  defining property: no host test may require an emulator, and no host result
  may be reported as bare-metal evidence.
- The ADR's validation strategy (ADR-049: unit + host-side + QEMU integration
  + guest self-test + Linux regression + fuzz/property + unsafe audit) makes
  host-side testing one distinct layer; this design fixes only that layer's
  organization, not the others.
- The plan requires: the categories of future logic suited to host
  verification, a minimal host test baseline with a gate-callable execution
  path, the coverage categories future tests must address, and an executed
  baseline with its non-proof of bare-metal semantics recorded.
- W02 (declared prerequisite) supplies the pinned toolchain; the minimal host
  test executes under it. W03 supplies the workspace in which any host test
  physically runs. W03 is a phase-A package expected complete before W08 per
  the task-book phase order, although the plan index does not list it as a
  contract prerequisite; the workflow §1 failure boundary covers its absence.

Classification: the host-test baseline contract (testable-logic boundary,
organization rules, execution entry) and the coverage-category matrix are
**Required** for W08 closure. Concurrency/stress suites, fuzz/property
harnesses, QEMU-based integration tests, guest self-tests, and hardware tests
are **Reserved** for the future designs that own their subjects
([W09](../p0-w09-qemu-automation-entry-baseline/README.md), P3+ and later
stages per ADR-049). All hypervisor mechanism implementation, crate creation,
target selection, gate definition, and CI wiring are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Future-logic categories suited to host verification; goals that must not depend on QEMU | [Host-test contract](01-host-test-contract.md) §2 | W08-DV01 |
| Minimal host test baseline and gate-callable execution path | [Host-test contract](01-host-test-contract.md) §3–§5; [workflow](03-implementation-and-validation.md) steps 2–4 | P0-V03/V04 (W08-DV02–DV04) |
| Coverage categories for future tests | [Category matrix](02-test-category-matrix.md) | W08-DV05 |
| Baseline executed; non-proof boundary recorded | [workflow](03-implementation-and-validation.md) step 4; [contract](01-host-test-contract.md) §6 | P0-V04 (W08-DV03/DV04), P0-V09 (W08-DV06) |
| Document discoverability and coherence | [workflow](03-implementation-and-validation.md) step 5 | P0-V09 (W08-DV06) |
| Downstream handoff to W07/W19/W20 and later module designs | [workflow](03-implementation-and-validation.md) handoff checklist | W08 closure (W08-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p0-implementation-designs` at
`4e631ee`): no host-test document, no test taxonomy, and no test exist;
`tests/` and `crates/` contain only `.gitkeep` markers; there is no Cargo
workspace or manifest (W02's toolchain design and W03's build-target design
are proposed or in preparation — neither pin nor workspace exists in the
tracked tree); `docs/testing/README.md` is a three-line informative
placeholder. W01 is completed with recorded verification. Each ledger row
below states the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Host-verifiable logic categories and non-QEMU verification goals are defined | No test taxonomy anywhere | Testable-logic boundary section: which future logic classes belong on the host, which cannot be verified off bare metal | Without the boundary, future designs cannot tell where a behavior's evidence must come from | W08 (this design) | W08-DV01 review |
| A CI-callable host-test execution entry exists | No workspace, no entry, no invocation anywhere | One documented execution entry with fixed semantics and a recorded canonical spelling | "可执行并被 CI 使用" requires exactly one entry that W07/W20 can call without interpretation | W08 contract; spelling bound to the W03-delivered workspace | W08-DV02; CI execution completes with W20 |
| A minimal host-test baseline exists and passes | Nothing to run; no product logic exists anywhere | A minimal placeholder host test whose only subject is the entry itself, plus a failure-visibility check | P0-V04 requires baseline tests that pass; with no product logic, the honest minimum is proving the path executes and reports truthfully | W08; placement follows the W03-delivered workspace conventions | W08-DV03/DV04 |
| Coverage categories for future tests are prescribed | Absent | Category matrix (normal, boundary, invalid-input, resource failure, repeated lifecycle; concurrency when applicable) with a mapping rule for future designs | The plan requires the categories, not yet the tests | W08 | W08-DV05 |
| Non-proof of bare-metal semantics is recorded | Absent | Mandatory proof-boundary statement in the contract and in every future result report | Host passing must never be misread as EL2/hardware evidence | W08 | W08-DV06 |
| Discovery of the baseline | `docs/testing/README.md` is an unlinked placeholder | Routing row and stage-index row | P0-V09 requires coherent, reachable documentation | W08 wiring | W08-DV06 |

The dependency on W03's delivered workspace is an implementation-time
assumption with an explicit failure boundary (workflow §1); it is not a
decision blocker for this design, and W08 must not create a workspace or crate
to fill the gap.

## Resolved design decisions and their authority

1. **Contract home:** `docs/testing/host-test-baseline.md` is the sole
   normative home of host-test policy, matching the directory's declared role
   (test strategy and environments). [P0-W05](../p0-w05-documentation-baseline/README.md)
   may re-home it; semantic ownership stays with the document.
2. **Entry model:** exactly one documented host-test execution entry,
   consumable by W07's `QG-TEST-HOST` gate and W20's CI. Its semantics
   (pinned toolchain, host target, whole host-test set, truthful exit) are
   fixed by this design; its canonical spelling is recorded at implementation
   time by the selection rule in the [host-test
   contract](01-host-test-contract.md) §5, mirroring the W02
   version-selection pattern, and is stable thereafter.
3. **Baseline test set:** at P0 the minimal baseline is one placeholder host
   test that proves the entry executes and reports truthfully, placed inside
   the W03-delivered workspace per its conventions. It is infrastructure
   verification, never counted as product coverage, and is expected to be
   superseded by the first real host-tested logic; retirement is a recorded
   minor change. Truthfulness is validated by a deliberate local
   failure-mutation dry run that is reverted and never committed.
4. **Category matrix:** the five plan-named classes (normal, boundary,
   invalid input, resource failure, repeated lifecycle) are the required
   coverage categories; concurrency is the named conditional extension. Each
   future module design must map its host-testable units to the classes and
   record why any applicable class is excluded.
5. **Proof boundary:** host results prove host-side logic semantics only. The
   contract must state this and require result reports to carry it; QEMU
   evidence belongs to the [W09 runner
   entry](../p0-w09-qemu-automation-entry-baseline/README.md) and P1, hardware
   evidence to later stages.
6. **Mutation thresholds:** adding a coverage class, changing entry semantics,
   or moving the proof boundary is a design-level change; recording the entry
   spelling and baseline placement is a recorded minor change; making host
   tests optional for gate-bearing changes would contradict W07's register and
   is not available to W08.

## Work breakdown and loading order

1. Read the [host-test contract](01-host-test-contract.md) for artifact
   groups, the testable-logic boundary, organization rules, and the execution
   entry contract.
2. Read the [category matrix](02-test-category-matrix.md) before authoring the
   contract's coverage section and whenever judging a future design's test
   mapping.
3. Apply the changes in the order stated in the [implementation
   workflow](03-implementation-and-validation.md): verify prerequisite
   surfaces, author the contract, establish the minimal baseline test,
   execute it with the failure-visibility check, wire discovery, close.
4. Store actual commands, output, environment, and result in
   `../../verification/p0-w08-host-side-testing-baseline-verification.md`, and
   record decisions taken and changed artifacts in
   `../p0-w08-host-side-testing-baseline-record.md` only when implementation
   begins. Neither this design nor a written record may claim W08 complete.

## Design-level state and lifecycle

W08 adds no runtime state, registry, lock, or hypervisor code path. Its
authoritative state is one tracked contract document, one category matrix
(within the contract or as its second normative file, per the contract's
artifact table), one minimal test inside the delivered workspace, and the
discovery links. Documentary lifecycle:

```text
no host-test rules, no host test
  -> contract committed (boundary, organization, entry, categories, proof boundary)
  -> minimal placeholder test committed inside the W03-delivered workspace
  -> entry executed; pass and failure-visibility evidenced
  -> discovery links committed
  -> W07 binds QG-TEST-HOST; W19 documents the path; W20 executes it in CI
  -> first real host-tested logic supersedes the placeholder
  -> later changes mutate only through the contract's thresholds
```

The contract document is the owner of every host-test policy statement; the
placeholder test is the owner of nothing — it asserts only the entry's health.
A future design that treats a passing placeholder as product coverage is a
review failure.

## Explicitly excluded interfaces

No parser, capability mechanism, address/newtype API, page-table or ABI
mechanism, crate, target triple, gate definition, CI workflow, script, or
hypervisor source is designed or authorized by W08. The only machine-facing
surface is the execution entry contract — a documented invocation whose
semantics §5 of the [host-test contract](01-host-test-contract.md) fixes; it
is not an ABI and commits no program in this design. The placeholder test
asserts nothing beyond the entry's own health.

## Downstream handoff

- **W07** receives the execution entry as the sole binding target for
  `QG-TEST-HOST`: stable spelling, fixed scope, truthful exit semantics. W07
  owns classification and blocking; if W08's entry changes semantically, the
  change goes through W07's register thresholds, not a silent edit.
- **W19** receives the entry as the test step of the clone-to-build path and
  the proof-boundary sentence to quote in contributor onboarding.
- **W20** receives the entry as the CI-executable host check: same spelling,
  pinned toolchain, no emulator dependency; CI execution evidence completes
  P0-V03/V04.
- **P0-W03** receives the placement requirement: its delivered workspace must
  designate where host-side tests live for host-target members, or W08's
  baseline placement is blocked (workflow §1).
- **Later module designs** (P1+; e.g. hypercall parsing in P5, platform
  discovery in P2 per their plan indexes) receive the category matrix and the
  mapping rule: each design states which classes apply to which units and why
  any applicable class is excluded.
- **W09 and P1** receive the explicit statement that host passing never
  substitutes for QEMU or hardware evidence.
