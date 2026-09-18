# P6-W07 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W07 design entry](README.md).

## 1. Preconditions and failure boundary

Complete the Coding-Guidelines preflight and load
[01](01-scope-and-foundations.md), [02](02-architecture-and-state.md), and
the contract file for the step at hand. W07's upstream dependencies (W03,
P4, P5, W01) are planned but not implemented at design time, so step 1 is a
real entry review against [01](01-scope-and-foundations.md) §5. Stop and
record a blocker instead of guessing when: a P5 boundary (dispatch, handles,
rights) is missing or delivered with differences that matter — the Guest
path is blocked, the Host-mechanism path may proceed only if the entry
review says its own inputs (range declaration, vCPU liveness) are
available; the W03 lifecycle arrives merged with virtual state (boundary
violation — raise, never merge); or a step appears to require LR
programming, maintenance processing, Guest masking semantics, or wakeup
policy (scope violation — W08/W09/W10/P7 respectively).

## 2. Ordered implementation steps

### Step 1 — entry review and cross-design reconciliation

Target: implementation record (`../p6-w07-virtual-interrupt-core-record.md`,
created in this step).

Work: inspect delivered evidence for every §5 contract in
[01](01-scope-and-foundations.md): P5 dispatch/handle/rights services and
error taxonomy, P4 vCPU objects and destroy hook, W01 range/capability
declaration, and the W08/W09 consumption of the claim/return and
completion contracts (their designs are prepared in parallel — reconcile
the protocol explicitly). Record each contract as available /
with-differences / missing.

**Acceptance:** every contract has a recorded status; the claim/return
and completion protocol is acknowledged by the W08/W09 designs or the
difference is formally raised. **Failure/blocker:** missing or
contradictory contracts stop the affected W07 surface at this step with a
recorded blocker naming the owning package.

### Step 2 — place the logical modules

Target: workspace layout chosen by the workspace-owning packages (logical
modules `virq-core`, `virq-authorize`, `virq-telemetry` per
[02](02-architecture-and-state.md) §2).

Work: map logical modules onto the actual tree. W07 is Core-domain: assert
(no register access anywhere in it), and verify no dependency on
Arch/SoC/Board modules exists or is introduced.

**Acceptance:** the module graph shows W07 depends only on Core-domain
types, typed-ID conventions, and the P5 service traits.
**Failure/blocker:** any needed Arch dependency is a layering violation —
stop and record.

### Step 3 — typed identities and intake

Target: `VirqId`/`VirqPriority`/`VirqWindow`/`virq_intake` (contracts
[03](03-code-contracts-virq-lifecycle.md) §1,
[04](04-code-contracts-authorization-and-errors.md) §5).

Work: implement the newtypes and window declaration with host-side unit
tests (range edges, width edges, declaration conflicts).

**Acceptance:** no raw integers cross a W07 function boundary; intake
failure disables delivery with diagnosis. **Failure/blocker:** a missing
W01 declaration blocks bank creation, per step 1.

### Step 4 — the bank and the state machine

Target: `VirqBank` and transitions (contracts
[03](03-code-contracts-virq-lifecycle.md) §1.3, §2, §4, [02] §4).

Work: implement the bank with the single-lock discipline, inject/dedupe
outcomes, saturating occurrence counters, and the impossible-state
recovery rule.

**Acceptance:** host-side tests cover every [02] §4 transition rule,
including repeated arrival in each state and saturating counters; no
operation allocates or loops over anything but the declared window.
**Failure/blocker:** a discovered need for a new transition is a design
amendment, not a local field.

### Step 5 — presentation protocol and completion

Target: `virq_select_next_pending`, `virq_presentation_returned`,
`virq_report_guest_completion`, `virq_query_summary`, `virq_drain`
(contracts [03](03-code-contracts-virq-lifecycle.md) §3, §5–§6).

Work: implement the claim/return protocol with token validation, the D6
selection exclusion, and the deterministic drain; add protocol-misuse
tests (stale claim, double claim, claim against active).

**Acceptance:** every misuse case is a counted forced-consistent
recovery, never a panic or silent corruption; selection order is
priority-then-lowest-VirqId deterministically. **Failure/blocker:** a
protocol mismatch with the W08/W09 designs is a cross-design amendment
(they consume this protocol by citation), resolved before integration.

### Step 6 — authorization spine and Guest HVC path

Target: `authorize_and_validate`, `handle_virq_hvc_request`, denial
taxonomy (contracts [04](04-code-contracts-authorization-and-errors.md)).

Work: implement the spine over the delivered P5 services and register the
HVC handler; exercise every denial class, including the SGI-only class
rule and the default-priority rule.

**Acceptance:** every Guest-controlled integer is validated before use;
denials are structured, counted, and state-preserving; no identity/role
shortcut exists. **Failure/blocker:** a P5 taxonomy gap blocks the Guest
path (step-1-class blocker); it never blocks the Host-mechanism path.

### Step 7 — acceptance scenarios and evidence

Target: verification record
(`../../verification/p6-w07-virtual-interrupt-core-verification.md`).

Work: run the validation matrix ([06](06-validation-and-handoff.md) §1):
authorized eventual delivery, multiple pending preservation, repeated
arrival, target isolation (conditional on the multi-vCPU prerequisite),
denial coverage. Record commands, environment, outputs, timestamps, and
explicit not-run/blocked entries; complete the implementation record
(changed files, deviations — W07 adds no `unsafe` by design).

**Acceptance:** every W07-DV row has a run status with evidence or an
explicit reason; P6-V11/V12/V14/V18-related claims are made only in the
verification record and only for W07's share of those validations.

### Step 8 — closure review and handoff

Work: run the [06](06-validation-and-handoff.md) §3 handoff checklist;
verify the controller-independence review gate (no GIC/register concepts
in W07) and that handoff statements match delivered evidence.

**Acceptance:** checklist complete.

## 3. Evidence destinations

- Implementation record: `../p6-w07-virtual-interrupt-core-record.md`
  (created at step 1).
- Verification record:
  `../../verification/p6-w07-virtual-interrupt-core-verification.md`
  (created when evidence exists).
- No completion claim may appear in any design file.

The validation matrix, error/security/observability model, and handoff
checklist that close this workflow are in
[06-validation-and-handoff.md](06-validation-and-handoff.md).
