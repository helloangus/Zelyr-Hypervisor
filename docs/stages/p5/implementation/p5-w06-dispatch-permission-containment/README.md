# P5-W06 Dispatch, Permission, and Fault Containment — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The integrated P5 service path that combines the W02 ABI, W03
Guest-data, W04 object-reference, and W05 authority boundaries into one minimal
authorized hypercall with contained Guest-facing outcomes, per
[P5-W06](../../plans/p5-w06-dispatch-permission-containment.md).  
**Owner/change context:** P5-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W06, the first integrated P5
service boundary. It defines the ordered validation pipeline that every
dispatched call must pass, the containment rule that separates Guest-caused
rejections from Hypervisor invariant failures, and the single minimal permitted
operation that exercises the complete chain. It deliberately does **not**
design a general VM-management interface, future hypercall families, delegation
or attenuation semantics, scheduler or timer/IRQ mechanisms, or any P6+
behavior; those remain with their owning packages and stages.

A coding agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| Pipeline order, dispatch context, dispatch and outcome contracts | [01 — dispatch pipeline contract](01-dispatch-pipeline-contract.md) |
| Failure classification, containment rules, result-category taxonomy | [02 — containment and error classification](02-containment-and-error-classification.md) |
| Ordered implementation, validation matrix, records, handoff | [03 — implementation workflow and validation](03-implementation-workflow-and-validation.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P5
task book, and the P5-W06 plan. This document is a proposed design; it makes
no completion, implementation, or validation claim, and its pseudocode is not
runnable production code.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → frozen contracts →
P5-W06 plan → this design → Coding Guidelines. In particular:

- ADR-007 makes every Guest-supplied register value, buffer, and object
  reference untrusted; ADR-013 requires capability plus rights plus generation
  and forbids identity-derived authority; the ADR implementation invariants
  require capability-check failure never to degrade into a role/VM-ID bypass
  and a Guest-caused fault never to become a global panic by default.
- The task book defines the minimal closed loop this package must realize:
  controlled HVC → request validation → guest-data boundary → opaque object
  reference → caller-associated capability/right check → allowed operation or
  structured denial. It requires malformed requests to remain Guest-facing
  controlled outcomes and forbids reclassifying a genuine invariant violation
  as a Guest error.
- The plan's prerequisites are the W03, W04, and W05 boundaries and the P4
  exception boundary. W02's request/result/error model reaches this package
  through W05's error/authority routing and is cited directly only for the
  outcome vocabulary. All four are sibling detailed designs delivered as
  parallel work on this branch and are treated here as **assumed contracts**
  with explicit failure boundaries (see
  [01 §2](01-dispatch-pipeline-contract.md)); none may be reinvented here.
- The P4 handoff ([P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md))
  supplies the evidenced Guest-EL1 entry/exit, HVC/exception classification,
  Stage-2 fault isolation, Validation Guest, and QEMU regression facts. The
  P2 handoff ([P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md))
  and P3 handoff ([P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md)) supply
  the protected-memory, checked-address, pCPU-identity, and synchronization
  constraints the pipeline must preserve.

Classification:

- **Required:** the ordered validation pipeline; the dispatch context and
  outcome contracts; the containment and classification rules; the one minimal
  integrated operation exercising ABI, caller, reference, type, rights, and
  argument checks; the valid/malformed/unsupported/invalid-address/
  invalid-reference/wrong-type/no-right/revoked/bad-state acceptance classes;
  the Guest-versus-invariant separation; observability responsibilities
  (result categories, handed to W09).
- **Reserved:** generalization to further hypercall families and a final
  dispatcher/module/API shape; side-effecting operations with full rollback
  semantics; VM-facing abuse/rate policy; a frozen error-number assignment and
  ABI encoding (owned by the W02 design and the factual ABI artifact);
  telemetry transport and counters (integrated by W09 under the P0-W12/W13
  contracts).
- **Out of Scope:** broad VM management, identity policy, scheduler, timer/IRQ
  or any P6+ mechanism; the final telemetry API/backend; a management or
  machine ABI; real-hardware claims; reclassifying invariant violations as
  Guest errors.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Integrated validation order before operation executes (P5-T02, T11) | [pipeline contract](01-dispatch-pipeline-contract.md) §3–§5 | P5-V09 (W06-DV01–DV03) |
| Caller-associated authority check with no identity shortcut (P5-T11) | [pipeline contract](01-dispatch-pipeline-contract.md) §3, §5 | P5-V09 (W06-DV02) |
| Guest-caused outcomes contained; Guest fault vs invariant separation (P5-T13, T14) | [containment](02-containment-and-error-classification.md) §1–§3 | P5-V10 (W06-DV04–DV06) |
| Minimal permitted operation integrated with all checks (plan step 3) | [pipeline contract](01-dispatch-pipeline-contract.md) §6 | P5-V09 (W06-DV03) |
| Defined acceptance classes: valid, malformed, unsupported, invalid-address, invalid-reference, wrong-type, no-right, revoked, bad-state (plan step 4) | [containment](02-containment-and-error-classification.md) §2, [workflow](03-implementation-workflow-and-validation.md) validation matrix | P5-V09/V10 (W06-DV04, DV07) |
| Review: Guest input cannot panic the Hypervisor; invariant handling separately diagnosable (plan step 5) | [workflow](03-implementation-workflow-and-validation.md) W06-DV05/DV06 | P5-V10 (W06-DV05, DV06) |
| Observability responsibilities handed to W09 | [containment](02-containment-and-error-classification.md) §4 | W09 consumes (P5-V15 groundwork) |
| Handoff of stable scenario outcomes to W07–W10 | [workflow](03-implementation-workflow-and-validation.md) handoff checklist | W06 closure review (W06-DV08) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p5-implementation-designs`,
`git ls-files`): the repository is a documentation-only P0 scaffold. There is
no Cargo workspace, no `rust-toolchain.toml`, and no Rust source in any
tracked directory (`crates/`, `hypervisor/src/`, `guests/validation-aarch64/`,
`tests/` hold only `.gitkeep` placeholders). P1–P4 stages contain plans and
task books but no implementation or verification records; P0 contains the W01
records and the W02 detailed-design exemplar only. The W01–W05 detailed
designs are parallel work on this branch and may not be present yet; links to
them are by slug and are forward references. No tracked artifact of the P5
dispatch path exists. Each ledger row states the missing foundation the plan
outcome necessarily requires and its owner.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| One minimal service path validates ABI, caller, reference, type, rights, arguments before the operation (P5-V09) | No code, no dispatch path, no W03–W05 implementations | Implemented W03/W04/W05 boundaries plus the W06 pipeline integrating them | The chain cannot be exercised unless each boundary exists and is composed in one path | W03/W04/W05 designs and records; W06 composes | W06-DV01–DV03 in `../../verification/p5-w06-dispatch-permission-containment-verification.md` |
| Guest-caused failures receive a defined contained result (P5-V10) | No outcome model exists in any tracked file | The outcome taxonomy and containment rules of [02](02-containment-and-error-classification.md), implemented in the dispatch path | Containment is a property of the dispatch result mapping, not of individual boundaries | W06 (this design) | W06-DV04–DV06 |
| Request/result/error vocabulary is distinguishable (plan scope; W02 boundary) | W02 design and the factual ABI artifact do not exist yet | W02's delivered request/result model consumed as an assumed contract | Dispatch maps outcomes into W02's categories; inventing a second vocabulary would fork the ABI | W02 design + factual ABI artifact | W06-DV01 prerequisite review |
| Guest-data, reference, and authority checks behave as designed | No W03/W04/W05 implementations | Each boundary's contract interface consumed as an assumed contract | W06 owns composition and ordering only; duplicating boundary logic would create two authorities | W03/W04/W05 designs | W06-DV02/DV03 and their review records |
| The trap actually reaches the dispatch path | P1/P4 exception designs exist as plans only | P4-evidenced HVC/exception classification feeding a standard exit reason | Dispatch is downstream of the Arch-layer trap classification; it must not re-derive it | P1/P4 designs; P4-W09 evidence | W06-DV01 prerequisite review |
| The pipeline has a physical home in the code tree | No crate tree exists | Logical modules fixed here; physical placement follows the crate/module contracts established by P0–P4 designs | Checklist §2 requires logical modules without assuming a file tree | P0–P4 designs own the tree; W06 owns logical boundaries | W06-DV01 prerequisite review; implementation record notes placement |
| Multi-pCPU dispatch does not assume a single caller pCPU | P3 handoff is a plan, not an implementation | Concurrency context rules in [01 §7](01-dispatch-pipeline-contract.md) consuming P3/W04/W05 protection contracts | Task book reserves later rescheduling of callers across pCPUs | P3-W14 handoff; W04/W05 concurrency contracts | W06-DV07 review |

No row above is resolvable by inventing a crate, API, or ABI value in this
design; each is either an assigned prerequisite (with the failure boundaries
of [01 §2](01-dispatch-pipeline-contract.md)) or a W06-owned logical contract.

## Resolved design decisions and their authority

1. **Check-pipeline order is fixed:** classify-and-decode → structural
   validation → reference resolution → type check → authority/rights check →
   argument (Guest-data) validation → operation-state check → execute →
   structured result. Rationale: cheap structural checks fail fast; authority
   precedes any argument interpretation that could consult object state, so no
   untrusted input touches state before the caller is authorized; Guest-data
   validation is last because it is the most expensive and may fault. This is
   stage-local design freedom explicitly granted by plan work-sequence item 2
   ("integrated validation order"), constrained by the Coding Guidelines
   validate-before-use rule and W05's required ordering. Authority: this
   design; conflicts with a delivered W05 ordering contract are recorded, not
   absorbed.
2. **Authority is caller-associated and never a Guest argument.** The calling
   vCPU's security context is established by the dispatch context (Hypervisor
   side); the authority check consults W05's caller-associated capability
   state. P5 hypercalls carry object references, not capabilities; nothing a
   Guest writes can manufacture authority. Authority: ADR-013, ADR-051, the
   task-book invariant set, and the W05 plan; concrete lookup mechanics stay
   owned by W05.
3. **The integrated operation is a read-only object-attributes query class.**
   The minimal permitted call must exercise the full chain, which rules out
   discovery/version calls (no reference, rights, or Guest data). The expected
   candidate is a read-only query of an object's class and generation-stable
   attributes into a Guest-writable result buffer. Its identity, number, and
   encoding are owned by the W02 design and the factual ABI artifact, not by
   this design; W06 fixes only the semantic contract
   ([01 §6](01-dispatch-pipeline-contract.md)). If the delivered W02 supported
   set contains no call requiring reference + authority + Guest data, the
   task book's minimal closed loop cannot be satisfied — stop and record
   `Architecture Change Request`.
4. **Containment rule:** a Guest-caused condition (malformed, unknown,
   unsupported, version mismatch, invalid/stale/wrong-type reference,
   no-right, revoked, bad-state, invalid address, Guest-data fault) produces a
   structured Guest-facing result and never a Hypervisor panic. Resource
   exhaustion encountered while servicing a call produces a contained resource
   outcome plus a diagnostic. Only a detected Hypervisor invariant violation
   enters the fatal path, per the P0-W14 panic-classification contract, and it
   is never reported as a Guest error. Authority: task book (P5-T13/T14,
   plan out-of-scope list), detailed Plan-Agent guidance §55–§57, §93.
5. **The minimal operation performs no persistent state mutation.** Read-only
   semantics remove the rollback boundary from the first integrated path by
   construction; any future side-effecting operation must enter execution only
   after full validation and use a validate-then-commit shape, with state
   ownership remaining in W03/W04/W05. Rationale: plan step 3 requires one
   minimal operation, not a transactional framework; side-effect/rollback
   design beyond that is Reserved. Authority: this design within plan scope.
6. **Concurrency context:** dispatch runs in the VM-exit/synchronous-exception
   context of the calling pCPU. It must not block, sleep, allocate
   unboundedly, or perform heavy work; it holds no lock of its own across the
   operation and consumes the W04/W05 protection contracts as delivered. The
   caller's pCPU is not part of any authorization decision. Authority: Coding
   Guidelines (IRQ/VM-exit path rules), P3-W14 (no permanent single-pCPU
   assumption), task book Reserved list.
7. **Layering:** the Arch layer classifies the HVC trap and presents a
   standard exit reason; dispatch is architecture-independent Core logic that
   must not branch on QEMU, a board, or a platform name. Authority: ADR
   layering invariants, task book §1 closing paragraph.
8. **Observability split:** W06 defines the result-category taxonomy
   ([02 §4](02-containment-and-error-classification.md)) and guarantees a
   category for every dispatched call; counters, trace-event naming, and safe
   logging are integrated by W09 under the
   [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md) and
   [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md)
   contracts. Authority: task book package map ("W09 receives event/result
   categories" from W06) and the P0 diagnostics plans.

## Work breakdown and loading order

1. Load [01](01-dispatch-pipeline-contract.md) for the logical modules, the
   assumed prerequisite contracts with their failure boundaries, and the
   dispatch/outcome contracts.
2. Load [02](02-containment-and-error-classification.md) for the failure
   classification, containment rules, and the result-category taxonomy that
   downstream packages consume.
3. Execute the ordered steps in [03](03-implementation-workflow-and-validation.md);
   each step names its target, work, acceptance, failure/blocker handling, and
   evidence destination.
4. Record implementation facts in
   `../p5-w06-dispatch-permission-containment-record.md` (created when work
   starts) and all command output and run results in
   `../../verification/p5-w06-dispatch-permission-containment-verification.md`.
   Neither this design nor a record may claim W06 complete; completion is
   claimed only in verification material, for what was actually run.

## Explicitly excluded interfaces

No general dispatcher API for future hypercall families, no management or
machine ABI, no wire encoding or error-number assignment (W02 and the factual
ABI artifact own these), no object-table, handle, capability, rights, or
Guest-data representation (W04/W05/W03 own these), no telemetry transport or
counter API (W09 under P0-W12/W13), no VM lifecycle, scheduler, timer, IRQ, or
P6+ mechanism, and no host fuzz harness (W08). Any interface implied by prose
here but not contracted in [01](01-dispatch-pipeline-contract.md) must not be
implemented; raise the gap as a design conflict instead.

## Downstream handoff

- **W07** ([Validation Guest suite](../p5-w07-validation-guest-isolation-suite/README.md))
  receives the observable expected outcome for every acceptance class, as the
  stable scenario-outcome table in
  [02 §2](02-containment-and-error-classification.md), so Guest markers and
  dispatch outcomes agree.
- **W08** ([host fuzz/stress/SMP](../p5-w08-host-fuzz-stress-smp-baseline/README.md))
  receives the integrated host-side properties: the pipeline's seams as
  fuzzable units, the invariant that unclassified outcomes and Guest-input
  panics are failures, and the containment oracle.
- **W09** ([telemetry/regression](../p5-w09-telemetry-safe-logging-regression/README.md))
  receives the per-call result-category taxonomy of
  [02 §4](02-containment-and-error-classification.md) for counter and
  trace-event integration under P0-W12/W13.
- **W10** ([closeout](../p5-w10-closeout-p6-handoff/README.md)) receives the
  factual containment records: what was integrated, which classes are
  contained, and which boundaries remain with other packages.
- **P6** consumes, through W10 only, the evidenced facts that object references,
  caller-associated authority, and structured denial exist and are extensible;
  P6 designs its own interrupt-object semantics and must not infer a frozen
  management ABI from this package.
