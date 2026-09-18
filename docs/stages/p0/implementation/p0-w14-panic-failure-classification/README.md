# P0-W14 Panic / Failure Classification Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The architectural failure-class taxonomy (hypervisor invariant,
guest-caused, resource exhaustion, unsupported feature/hardware, platform
failure), the propagation and containment principles between them, and their
embedding into diagnostic and review requirements required by
[P0-W14](../../plans/p0-w14-panic-failure-classification.md).  
**Owner/change context:** P0-W14 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W14. It converts the bounded
work-package plan into one normative failure-classification document, minimal
discovery wiring, and a representative-scenario walkthrough proving the
taxonomy classifies later-stage failures without prescribing their
implementation. It deliberately does **not** define any Rust error enum,
panic handler, or guest-fault implementation (plan out-of-scope), does
**not** define diagnostic channel semantics or fatal-output content
([P0-W12](../p0-w12-logging-diagnostic-baseline/README.md) owns those; W14's
classes are consumed by them), does **not** design retry, backoff, or
per-subsystem recovery policies (owning designs decide those within the class
semantics), and does **not** authorize any code surface.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task
book, and P0-W14 plan. This document is the proposed detailed design for those
changes; it is not a completion record and contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W14 plan → this
design → Coding Guidelines. In particular:

- §12 fixes the split this taxonomy operationalizes: panic policy
  distinguishes fatal hypervisor invariant failures from guest-caused faults,
  and guest errors fault or stop the corresponding VM rather than the
  hypervisor. The implementation invariant states it as a MUST: a guest-
  caused fault affects only its VM/device context and does not trigger a
  global panic.
- ADR-007 (guests are untrusted; all guest-derived addresses, lengths,
  indices, and state transitions are validated) is why guest-caused is its
  own class: validation failure is an expected, contained outcome, not a
  hypervisor defect. ADR-013/ADR-051 (capability + rights authorization, no
  VM-ID privilege) make authorization denial an expected refusal, never an
  implicit failure escalation. ADR-035 (service/driver domain faults must not
  break the hypervisor) is the containment posture the class semantics
  preserve.
- ADR §13's error-vocabulary list (GuestFault, InvalidInput, PermissionDenied,
  ResourceExhausted, HardwareFailure, InvariantViolation) is a crate-boundary
  suggestion for future error types, not a P0 mandate; this design maps it
  informatively and binds the plan's five failure classes (work item 1) as
  the P0-governed taxonomy.
- The task-book outcome for W14 is: hypervisor invariant, guest, resource,
  unsupported-feature, and platform failures are **architecturally distinct**
  (P0-V09), so later designs can choose recovery, stop, or escalation paths
  from the classification alone.
- Prerequisite status: [W05](../p0-w05-documentation-baseline/README.md) and
  [W06](../p0-w06-adr-governance/README.md) are proposed designs, not
  deliveries; the current tree's `docs/README.md` header mandate and
  `AGENTS.md` conflict-label mandate are the fallback conventions. W10 and
  W12 (consumers) already reference this taxonomy by subject in their own
  designs; W14 must deliver a taxonomy they can cite without re-deciding.
- Consumer alignment: [W10](../p0-w10-unsafe-rust-governance/README.md) needs
  stable class identifiers its SAFETY justification template can reference;
  W12 needs the fatal-channel reservation and per-class diagnostic
  treatments; P1+ error models (exception entry, crash diagnostics, memory
  faults, hypercall error boundaries) need per-class response boundaries.

Classification: the classification document (five class definitions with
detection, containment, allowed/prohibited responses, and diagnostic
treatment; propagation and classification rules; diagnostic/review embedding;
the ADR §13 cross-map) and its discovery wiring are **Required** for W14
closure. Concrete error types/enums, panic-handler behavior, guest-fault
injection, per-subsystem recovery policies, and per-class trace-event names
([W13](../p0-w13-trace-event-namespace-baseline/README.md) registry) are
**Reserved** to the designs that introduce their subjects. Any code, crate,
or CI artifact is **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Failure sources and the semantic differences they must retain (work item 1) | [Classification contract](01-failure-classification-contract.md) §3 | P0-V09 (W14-DV01) |
| Guest-caused faults affect the corresponding guest, never escalate to global panic (work item 2) | [Classification contract](01-failure-classification-contract.md) §3 FC-GUEST, §4 | P0-V09 (W14-DV02) |
| Resource/capability/platform vs internal-invariant handling boundaries written into diagnostic/review requirements (work item 3) | [Classification contract](01-failure-classification-contract.md) §3, §6 | P0-V09 (W14-DV03) |
| Representative later scenarios confirm no implementation is pre-decided (work item 4) | [workflow](02-implementation-and-review.md) step 4 | P0-V09 (W14-DV04) |
| Document discoverability and coherence | [workflow](02-implementation-and-review.md) step 3 | P0-V09 (W14-DV05) |
| Downstream consumability by W10, W12, and P1+ low-level error models | [workflow](02-implementation-and-review.md) handoff checklist | W14 closure review (W14-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p0-implementation-designs` at `4e631ee`): no tracked document defines a
failure class, a containment rule, or a panic-worthiness rule. The ADR states
the split and the invariants; the Coding Guidelines state that a guest error
must not panic the hypervisor; neither is an applicable taxonomy a design can
cite per failure path. No Rust source exists, so no panic or error path
exists to classify. W01 is completed; W05/W06 (prerequisites) and W10/W12
(consumers) are proposed designs, several untracked parallel work referenced
by slug. Each ledger row below states the missing foundation the plan
outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Failure sources are architecturally distinct | ADR prose only; no operational per-class definitions | Five class definitions with detection points, containment scope, and allowed/prohibited responses | Conflation is prevented by per-class semantics a designer applies per failure path, not by principles alone | W14 (this design) within the plan's class list | W14-DV01 |
| Guest-caused containment is enforceable (work item 2) | §19 invariant text; no classification test or response boundary | FC-GUEST contract plus the classification test and escalation prohibition | The invariant must be applicable at design time: which paths are guest-caused, and what responses are allowed | W14 within §19/ADR-007 | W14-DV02 |
| Handling boundaries reach diagnostic/review requirements (work item 3) | No review requirement mentions failure classes | Per-class diagnostic treatments (consuming W12 channels by subject) and design/code review checklist items | Boundaries must be checkable when designs and code are reviewed, not rediscovered after incidents | W14 treatments; W12 channels; W10 hook | W14-DV03 |
| Classification without implementation pre-decision (work item 4) | Not exercised | Scenario walkthrough on five representative later-stage failures | Acceptance demands demonstrated taxonomy, demonstrated non-prescription | W14 | W14-DV04 |
| Discoverability (P0-V09) | No routing row | `docs/README.md` routing row and stage index row | Entry points must lead to the classification | W14 | W14-DV05 |
| W10/W12/P1+ can bind | Their designs reference this taxonomy by subject; nothing to bind yet | Stable class identifiers and handoff statements | Consumers cite identifiers; identifiers must exist and be stable | W14 delivers; consumers cite | W14-DV06 |

No row requires an error type, handler, or guest-fault mechanism, so no
decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Normative home:** `docs/security/failure-classification.md` is the sole
   normative home of the taxonomy, propagation rules, and review embedding.
   The security directory houses safety/threat-model/unsafe-audit records
   (its declared role), matching [W10](../p0-w10-unsafe-rust-governance/README.md)'s
   placement of the unsafe policy there; failure classification is a safety
   property of the same kind. [W05](../p0-w05-documentation-baseline/README.md)
   may re-home it; semantic ownership stays with the document.
2. **Five classes with stable identifiers.** FC-INVARIANT (hypervisor
   invariant violation), FC-GUEST (guest-caused fault), FC-RESOURCE (resource
   exhaustion), FC-UNSUPPORTED (unsupported feature/hardware capability), and
   FC-PLATFORM (platform/firmware failure) — the plan's class list, given
   short stable documentary identifiers so W10's SAFETY template, W12's
   treatments, and P1+ designs can cite them. Identifiers are documentation
   keys, not code symbols.
3. **Classify at detection; never reclassify to survive.** Every failure path
   in a future design names its class at the detection point. Two hard rules:
   an FC-INVARIANT must never be downgraded to a recoverable error (the
   hypervisor cannot certify correct operation it knows it does not have);
   and the classification test — if guest-controllable input can trigger a
   condition, that condition is not FC-INVARIANT (it is FC-GUEST plus, where
   applicable, a robustness defect to fix).
4. **Panic-worthiness is class-derived.** Only FC-INVARIANT exits (and
   FC-PLATFORM/FC-UNSUPPORTED escalations that the escalation rule promotes
   to invariant-threatening) may terminate the hypervisor. Everything else
   must have a designed containment path. W12's fatal channel is reserved
   accordingly; this design owns which classes may reach it, W12 owns what
   the output must contain.
5. **FC-GUEST containment semantics.** Effect scope: the faulting VM's
   context (vCPU, VM, or the involved device), never other VMs, never
   hypervisor-global state. Allowed responses: inject a fault to the guest,
   terminate the offending vCPU or VM, refuse the operation — per the owning
   design. Prohibited: global panic or hypervisor shutdown, silent retry
   loops that mask guest attack patterns, and cross-VM state corruption.
   Rate/telemetry treatment belongs to the owning designs within W12/W13
   governance.
6. **Authorization denial is an expected refusal, not a failure class.**
   Capability/rights check failures are designed negative outcomes of the
   authorization model (ADR-013/ADR-051); they are recorded and returned,
   not classified as failures. When an authorization mechanism itself is
   bypassed or corrupted, that is FC-INVARIANT. Input-validation failures
   take the class of the input's source: guest-derived → FC-GUEST; internal
   misuse by hypervisor code → a defect surfacing under FC-INVARIANT rules.
   Management-domain (Control/Service Domain) untrusted input applies the
   FC-GUEST containment principle by analogy; its final class assignment is
   owned by the P5 hypercall/management-ABI error-boundary design, recorded
   as an open item, not decided here.
7. **Per-class diagnostic treatment.** FC-INVARIANT → the fatal channel with
   W12's minimums; FC-GUEST → VM-scoped diagnostics plus trace/metric facts,
   never the fatal channel; FC-RESOURCE → metric facts plus the refusal
   response; FC-UNSUPPORTED → capability/property record plus a refusal or
   disabled path, never a silent fallback pretending support; FC-PLATFORM →
   platform/BSP-scoped diagnostics, escalating only under rule 8.
8. **Escalation rule (the only path upward).** A non-invariant failure
   escalates to fatal only when continuing would violate a hypervisor
   invariant (integrity of memory ownership, capability soundness, or
   exception-return correctness). The escalation is itself an FC-INVARIANT
   exit and must say what invariant was threatened. Ordinary unavailability
   (a device absent, a resource momentarily exhausted) is not escalation.
9. **ADR §13 cross-map is informative.** The document maps
   GuestFault→FC-GUEST, InvariantViolation→FC-INVARIANT,
   ResourceExhausted→FC-RESOURCE, HardwareFailure→FC-UNSUPPORTED/FC-PLATFORM,
   and notes InvalidInput/PermissionDenied as outcome vocabulary inside the
   class semantics (decision 6). The five classes bind P0 governance; finer
   error-type granularity is future designs' freedom within class semantics.
   Because §13 is a suggestion list, this mapping amends no ADR decision.

## Work breakdown and loading order

1. Read [the classification contract](01-failure-classification-contract.md)
   for the artifact groups, the five class contracts, propagation rules,
   diagnostic/review embedding, and the cross-map.
2. Apply the changes in the order stated in
   [the implementation workflow](02-implementation-and-review.md): verify
   prerequisite surfaces, author the classification document, wire discovery,
   run the scenario walkthrough, close.
3. Store actual review commands, output, environment, and result in
   `../../verification/p0-w14-panic-failure-classification-verification.md`,
   and record changed artifacts and any deviation in
   `../p0-w14-panic-failure-classification-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W14 complete.

## Design-level state and lifecycle

W14 adds no runtime state, registry, lock, allocation, or code path. The
authoritative state is one tracked classification document plus its
discovery links. Their documentary lifecycle:

```text
failure handling governed only by ADR prose and one-line coding rules
  -> failure-classification.md committed (five classes, propagation rules,
     diagnostic/review embedding, cross-map, thresholds)
  -> docs/README.md routing row + stage index row committed
  -> scenario walkthrough evidenced (five representative later-stage failures)
  -> W10 SAFETY template cites class identifiers; W12 fatal-channel
     reservation binds; P1+ designs classify every failure path
  -> taxonomy changes only through the document's thresholds
```

The classification document owns every taxonomy statement. A future design
whose failure path lacks a class, downgrades an invariant, or panics on guest
input is a review failure, not a local choice.

## Explicitly excluded interfaces

No Rust error enum, panic handler, fault-injection mechanism, result-type
convention, trait, function, module, crate, public API, ABI, or CI workflow
is designed or authorized by W14. The class identifiers are documentary keys.
The scenario walkthrough names no implementation. If implementing W14 appears
to require any of these, that is a scope boundary: stop and record.

## Downstream handoff

- **W10** receives the class identifiers its SAFETY justification template
  references (the failure-class consequence statement per unsafe segment) and
  the rule that unsafe-boundary failures classify under this taxonomy.
- **W12** receives the fatal-channel reservation (FC-INVARIANT-only
  producers) and the per-class diagnostic treatments its channels host.
- **P1+ low-level error models** (P1 exception entry and crash diagnostics,
  P2 memory allocation, P4 fault isolation, P5 hypercall error boundary, and
  their stage plan indexes) receive the requirement to name a class per
  failure path, the response boundaries per class, and the escalation rule;
  concrete error types remain their designs'.
- **W13** receives the subject areas its future event registry will host
  (guest faults, resource pressure, platform failures) without W14 declaring
  any name.
- **W05** may re-home the document; **W20** may later consume mechanical
  check candidates (for example: no panic construct on guest-input paths —
  the predicate, not the gate); **W22** maps this deliverable into the P1
  handoff package by its own plan scope.

The [stage implementation index](../README.md) row for this design is updated
truthfully as work proceeds; its status is "Proposed design; implementation
not claimed" until real evidence exists.
