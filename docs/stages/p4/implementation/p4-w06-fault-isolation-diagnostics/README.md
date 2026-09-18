# P4-W06 Guest Fault Isolation and Diagnostics — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Exit categorization, Stage-2 translation/permission fault diagnosis,
Guest-versus-Hypervisor fault separation, faulting-address inspection, and the
positive/negative isolation expectation set required by
[P4-W06](../../plans/p4-w06-fault-isolation-diagnostics.md)
(P4-E02, P4-F01–F04, P4-G01–G04).  
**Owner/change context:** P4-W06 implementation handoff; this design owns the
P4 exit-classification detail, the fault-diagnostic record, the isolation
expectation matrix, and the Guest-fault containment boundary on top of the
[P4-W04](../p4-w04-vcpu-entry-exit/README.md) exit frame.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W06. It converts the bounded
work-package plan into the diagnostic object model, the full classification and
diagnosis contracts over the W04 exit frame, the isolation expectation matrix
that makes P4-V06–P4-V09 objectively checkable, and the implementation
workflow, with pseudocode rather than production code. It deliberately does
**not** redesign the world-switch or its action policy
([P4-W04](../p4-w04-vcpu-entry-exit/README.md) stays the action authority and
`classify_minimal`/`decide` remain unchanged), the Stage-2 mapper or its
ledger ([P4-W02](../p4-w02-stage2-address-space/README.md) stays the only
mapping-state source), the Validation Guest scenario bodies and marker
protocol ([P4-W05](../p4-w05-validation-guest/README.md) stays the trigger
owner), the repeat driver and telemetry counters
([P4-W07](../p4-w07-repeatability-telemetry/README.md)), the automation
harness ([P4-W08](../p4-w08-qemu-integration-regression/README.md)), the final
generic `ExitReason` API, a management error ABI, device/DMA isolation, IOMMU,
or a policy for all future VM lifecycle failures.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — scope
  classification, assumed upstream contracts with failure boundaries, the
  authority analysis for contested areas, and the resolved design decisions
  (D1–D10). Load first; every other file depends on it.
- [02-architecture-and-state.md](02-architecture-and-state.md) — logical
  modules, diagnostic value objects and ownership, the classification flow,
  the Guest/Hypervisor fault-domain model, containment ordering, and the
  isolation expectation model (IS-series). Load for architecture work.
- [03-code-contracts-exit-classification.md](03-code-contracts-exit-classification.md)
  — syndrome decode, faulting-IPA reconstruction, and the `ExitDiagnostic`
  contract with pseudocode. Load for the classification work area.
- [04-code-contracts-diagnostics-isolation.md](04-code-contracts-diagnostics-isolation.md)
  — the expectation matrix types, expectation matching, the bounded diagnostic
  report, and the ledger-derived mapping dump, with pseudocode. Load for the
  diagnostics/isolation work area.
- [05-implementation-workflow.md](05-implementation-workflow.md) — ordered
  implementation steps with acceptance and failure handling.
- [06-validation-and-handoff.md](06-validation-and-handoff.md) — the
  validation matrix (P4-V06, P4-V07, P4-V08, P4-V09 inputs), the
  error/security/observability model, and the handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P4 task book, the
P4-W06 plan, and the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md)
entry review result). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W06 plan](../../plans/p4-w06-fault-isolation-diagnostics.md) → this design
→ Coding Guidelines. Binding constraints include:

- ADR-007 and ADR §19: Guest input is untrusted; a Guest-caused fault is a
  recoverable, VM-facing classified event and may never become a Hypervisor
  panic; only Hypervisor invariant violations are fatal. This design is the
  P4 place where that separation is made explicit and checkable.
- ADR §12: the panic/failure policy distinguishes fatal hypervisor invariants
  from Guest-caused faults; the P0 failure-classification contract
  ([P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md), assumed
  per W01 row R19) supplies the escalation semantics.
- ADR-048: structured telemetry is a designed interface; diagnostics route
  through the P0 logging/trace baseline (W01 rows R20, A8), never free-form
  prints.
- W04's exit frame and `classify_minimal`/`decide` are frozen seams: W06
  consumes the frame verbatim, never changes an action decision, and never
  re-captures registers.
- W02's mapping ledger and `query` snapshot are the only mapping-state source
  (W02 decision D9); W06 performs no hardware table walk and no descriptor
  decode of its own.
- W05's scenario table and marker grammar are the trigger surface; W06 refines
  expected outcomes but changes nothing in the W05 artifacts (change rules in
  W05 §6 apply jointly).
- Task book §1 Out of scope for W06: final `ExitReason` API, management error
  ABI, general debug monitor, security certification, device/DMA isolation,
  IOMMU, Guest interrupt delivery, and a policy for all future VM lifecycle
  failures.

Classification: the diagnostic types and contracts
([02](02-architecture-and-state.md), [03](03-code-contracts-exit-classification.md),
[04](04-code-contracts-diagnostics-isolation.md)) and the isolation
expectation matrix are **Required** for P4-E02, P4-F01–F04, and the W06 share
of P4-G01–G04. A hardware page-table walk for diagnostics, a symbolized or
interactive debug monitor, Guest-side self-diagnosis, fault-injection
tooling beyond the W05 triggers, and cross-pCPU fault handling are
**Reserved** with recorded re-entry points. The final `ExitReason` software
API, management error ABI, vGIC/IRQ fault classes, DMA/IOMMU isolation,
post-mortem tooling beyond the retained diagnostic, and any policy for
non-P4 Guest error handling are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-E02 exit categorization for all P4 exits | [classification contracts](03-code-contracts-exit-classification.md) §3 | P4-V06 (defined exits distinguishable) |
| P4-F01 translation-fault diagnosis with required context | [classification contracts](03-code-contracts-exit-classification.md) §3–§4; [diagnostics contracts](04-code-contracts-diagnostics-isolation.md) §3 | P4-V06, P4-V07 |
| P4-F02 permission-fault diagnosis (write vs execute distinguishable) | [classification contracts](03-code-contracts-exit-classification.md) §3; [expectation matrix](02-architecture-and-state.md) §6 (IS-04/IS-05) | P4-V06, P4-V08 |
| P4-F03 Guest-versus-Hypervisor fault distinction | [architecture](02-architecture-and-state.md) §4 (fault-domain model); [foundations](01-scope-and-foundations.md) D3, D5 | P4-V06, P4-V09 (containment review) |
| P4-F04 faulting-address inspection against mapping state | [classification contracts](03-code-contracts-exit-classification.md) §3.4; [diagnostics contracts](04-code-contracts-diagnostics-isolation.md) §3.4–§3.5 | P4-V06, P4-V07 |
| P4-G01 selected Hypervisor-owned-range accesses blocked and diagnosable | [expectation matrix](02-architecture-and-state.md) §6 (IS-03); [workflow](05-implementation-workflow.md) steps 3 and 6 | P4-V07 |
| P4-G02 Guest-RAM boundary blocked and diagnosable | [expectation matrix](02-architecture-and-state.md) §6 (IS-02) | P4-V07 |
| P4-G03 execute enforcement (XN/unmapped execute) | [expectation matrix](02-architecture-and-state.md) §6 (IS-05) | P4-V08 |
| P4-G04 write enforcement (read-only window) | [expectation matrix](02-architecture-and-state.md) §6 (IS-04) | P4-V08 |
| Containment review; unknown synchronous conditions diagnosable | [foundations](01-scope-and-foundations.md) D2, D3; [validation](06-validation-and-handoff.md) DV09–DV10 | P4-V09 |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs`):
documentation-only repository — no hypervisor or Guest code, no EL2 runtime,
no Stage-2 mapper, no world switch. Sibling designs W01–W05 exist as proposed
designs with no implementation records; P0–P3 packages are planned and
undelivered except P0-W01. The fine-grained P4-E/F/G requirement wording of
the superseded root source task book is not tracked in this repository (same
provenance finding as W01 item A9); this design takes requirement meaning from
the tracked task book §5 traceability rows and the W06 plan scope sentences.
Everything below is therefore an assumed contract or a P4-internal
deliverable, never an observable artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Defined exits are distinguishable (P4-V06) | W04's minimal `ExitClass` exists only as a proposed design; no decode detail exists anywhere | W06's syndrome decode and `ExitDiagnostic` over the W04 frame ([03](03-code-contracts-exit-classification.md)) | distinguishability needs more than the minimal class: access type, status family, and raw syndrome retained | W06 (this design); W04 frame (assumed contract M2) | W06-DV01/DV02 decode tests; P4-V06 via W05/W08 |
| Faults carry Guest/vCPU, PC, state, IPA, access, mapping, reason context (plan step 3) | Nothing exists | The `ExitDiagnostic` record fields and the faulting-IPA reconstruction contract ([03](03-code-contracts-exit-classification.md) §3.3–§3.4) | "required context" is only meaningful as named fields with sources | W06; W02 `query` (assumed M4) | DV03 field-completeness review; P4-V06 |
| Guest faults never become Hypervisor panics; Hypervisor invariants stay fatal (P4-V09, ADR §19) | The type split (`ExitInfo`/`StopCause` vs errors) is W04-proposed only; no runtime boundary exists | The fault-domain model and containment-ordering rules ([02](02-architecture-and-state.md) §4–§5) | containment must be structural (capture path), not convention | W06 model; W04 stub discipline (assumed M2); P0-W14 escalation (assumed M5) | DV09 containment review; P4-V09 |
| Unmapped, RAM-boundary, Hypervisor-owned-range accesses blocked with EL2 diagnostically live (P4-V07) | Nothing exists; W05's VG-004 covers only the generic unmapped probe | The IS-series expectation matrix ([02](02-architecture-and-state.md) §6) and its probe-class assertions | V07's three access classes need per-class expectations, not one generic unmapped row | W06 expectations; W05 triggers (assumed M3); probe carriage via W03/W05 channel (open item O1) | DV05–DV07; P4-V07 |
| Read-only write and execute comparison controlled and distinguishable; no stale-translation reliance (P4-V08) | W02 protect/ordering design exists as a proposal; no runtime evidence exists | IS-04/IS-05 expectations incl. mapping-agreement cross-check ([02](02-architecture-and-state.md) §6; [04](04-code-contracts-diagnostics-isolation.md) §3.4) | "distinguishable" is a diagnosis property; "not stale" needs the ledger-vs-hardware agreement check | W06; W02 protect/BBM (assumed M4) | DV08; P4-V08 |
| Controlled Guest faults stay VM-facing; WFI/WFE defined (P4-V09 joint with W04/W05) | Nothing exists | Verdict vocabulary incl. `Unclassified` fail-closed behavior ([03](03-code-contracts-exit-classification.md) §3.5) | unknown conditions must be diagnosable without becoming fatal | W06; W04 action policy (frozen seam) | DV09/DV10; P4-V09 |

No row above requires a decision this design is not authorized to make; the
decode detail, diagnostic record shape, and expectation matrix are stage-local
design freedom inside the plan's declared scope. The two recorded
cross-design dependencies (probe carriage, expectation refinement review) are
handled as open items O1/O2 in [01 §5](01-scope-and-foundations.md), not
silently absorbed.

## Resolved design decisions and their authority

Summarized here; full rationale and authority citations in
[01 §4](01-scope-and-foundations.md):

1. **Two-layer classification:** W04's `classify_minimal`/`decide` stays the
   sole action authority; W06 adds a descriptive `ExitDiagnostic` layer.
   Plan scope (W06 owns classification/diagnosis; W04 owns actions).
2. **Pure, total, allocation-free diagnosis** from frame data only; unknown
   values fail closed to a retained-raw `Unclassified` verdict, never to
   re-entry or panic. ADR-007; W01 A2.
3. **Structural fault-domain separation:** a `GuestExitFrame` exists only via
   the Guest-exit stub, so Guest domain is a capture-path property; EL2-context
   faults belong to the P1 fatal path and are never reclassified as Guest
   faults. ADR §19; W04 stub contracts.
4. **Faulting-IPA reconstruction is W06-owned** (FAR/HPFAR composition per the
   pinned architecture revision); an unavailable address is reported as
   explicitly unavailable, never guessed. QEMU divergences become
   Specification Investigation items (W01 A7).
5. **Mapping cross-check via W02 `query` only;** a ledger/hardware disagreement
   is a failed isolation-invariant evidence (run failed, diagnosis retained,
   invariant-review item), not a runtime panic and not an ignored anomaly.
   W02 D9; P4-V07/V08 wording.
6. **IS-series expectation matrix owned by W06** as a refinement of W05's
   expected-outcome column, honored through W05's joint-change rule; probe
   addresses ride the W03/W05 boot-info channel. Plan work sequence item 4.
7. **Containment-first ordering:** W04's stop decision executes before heavy
   diagnostics; report and dump run after the stop, in quiescent context; no
   diagnostic work in the capture path. W04 stub discipline; Coding Guidelines
   (bounded exit-path work).
8. **Ledger-derived diagnostic dump only;** a hardware page-table walk is
   Reserved behind a future W02 seam. W02 D9.
9. **Diagnostics route through the P0 logging/trace baseline** with stage-local
   `diag.*` event names; the human report is a bounded, truncation-annotated
   buffer write, never an unbounded print. W01 A8; ADR-048.
10. **W06 adds no `unsafe`:** classification, matching, formatting, and the
    dump are safe Rust over the W04 frame and W02 query results; any
    implementation-time discovery of an `unsafe` need is a recorded design
    deviation with its own SAFETY justification and inventory entry.

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): the
   Required/Reserved/Out-of-Scope detail, assumed contracts M1–M6 with failure
   boundaries, contested-area analysis, decisions D1–D10, and open items
   O1–O2.
2. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module map, the diagnostic value objects, the classification flow, the
   fault-domain model, and the IS-series expectation matrix that the contracts
   implement.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md),
   loading [03-code-contracts-exit-classification.md](03-code-contracts-exit-classification.md)
   for decode/diagnosis and
   [04-code-contracts-diagnostics-isolation.md](04-code-contracts-diagnostics-isolation.md)
   for expectations, report, and dump.
4. Record validation in
   `../../verification/p4-w06-fault-isolation-diagnostics-verification.md` and
   implementation facts in `../p4-w06-fault-isolation-diagnostics-record.md`
   only when work starts; neither this design nor the records may claim W06
   complete.

## Explicitly excluded interfaces

Not designed or authorized by W06: the final generic `ExitReason` API and its
ADR-level taxonomy (the P4 exit classes remain temporary vocabulary recorded
as facts by [P4-W09](../p4-w09-closeout-p5-handoff/README.md)); any
hypercall/HVC or management error ABI (P5); any change to W04's action policy,
exit frame layout, or world-switch; any change to W02's mapper, ledger, or
`query` semantics; any change to W05's scenario bodies, marker grammar, or
table (only refinements via the joint-change rule); vGIC/IRQ/timer fault
classes (P6); scheduler-visibility of fault states (P7); symbolization,
interactive debugging, or a persistent event log transport (P0-W12/W13 own the
baseline); device/DMA/IOMMU isolation; and any QEMU-conditional behavior in
Core (W01 A7).

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W07** (design: `../p4-w07-repeatability-telemetry/README.md`) receives
  the categorized diagnostic events (`diag.*` correlation fields), the
  `ExitDiagnostic`/verdict vocabulary for its counters and fault-correlation
  records, and the containment-first ordering constraint its driver must
  respect.
- **P4-W08** (design: `../p4-w08-qemu-integration-regression/README.md`)
  receives the stable expected diagnostics: the IS-series expectation matrix,
  the match-verdict vocabulary, and the requirement that every isolation
  scenario has a determinate expected `ExitDiagnostic` shape in the run
  record.
- **P4-W09** (design: `../p4-w09-closeout-p5-handoff/README.md`) receives
  only implemented facts: the diagnostic record fields, the decode coverage,
  the containment review result, and the retained limitation list (for
  example, no hardware walk, no host-fault diagnosis beyond the P1 path).
- **P5** inherits the evidenced Guest-fault containment boundary and the
  Guest-versus-invariant distinction as facts; it owns all new ABI and error
  semantics on top and may not treat P4 diagnostic formats as frozen.
