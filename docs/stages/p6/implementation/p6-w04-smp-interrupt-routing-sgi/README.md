# P6-W04 SMP Interrupt Routing and SGI — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The evidence-backed Host cross-pCPU SGI send/receipt/completion
mechanism and basic SPI routing-change behavior required by
[P6-W04](../../plans/p6-w04-smp-interrupt-routing-sgi.md).  
**Owner/change context:** P6-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W04. It converts the plan
into a typed SGI-target and send mechanism over the system-register SGI
interface, target-eligibility rules bound to the P3 lifecycle and W02
readiness ledger, determinate target accounting, and a quiesced SPI
route-change protocol. It deliberately does **not** design rescheduling
policy, complete remote-vCPU kick behavior, TLB-invalidation semantics,
IRQ load balancing, Guest SGIs, or scheduler wakeups; those belong to P7
and the virtual-interrupt packages.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) — the
system-register sequencing, typed-ID, lock-class, and bounded-work rules
are load-bearing here — then loads only the linked supporting file needed
for its assigned step. Before editing it must also follow the Coding
Guidelines preflight, including the repository
[AGENTS.md](../../../../../AGENTS.md), [documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P6 task book](../../task-book-v0.1.md), and the P6-W04 plan. This document
is the proposed detailed design; it is not a completion record and contains
no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W04 plan → this
design → Coding Guidelines. In particular:

- ADR-015 (SMP is an early foundation) and the task book Required outcome
  ("Cross-pCPU SGI and basic SPI routing operate with explicit targets and
  no scheduler-policy claim") govern; ADR-043/ADR-052 forbid
  board-conditional targeting.
- [P6-W02](../p6-w02-physical-gic-bring-up/README.md) supplies the readiness
  ledger and the initial determinate SPI routing state;
  [P6-W03](../p6-w03-physical-interrupt-lifecycle/README.md) supplies the
  classification, consumer registration, and the single
  acknowledge/complete path that SGI consumers run in. W04 builds on both
  without redefining them.
- The P3 contracts — pCPU lifecycle and online set, cross-CPU notification
  primitive, TLB-transport boundary, synchronization semantics — are
  assumed inputs
  ([p3-w03](../../../p3/plans/p3-w03-physical-cpu-lifecycle.md),
  [p3-w07](../../../p3/plans/p3-w07-cross-cpu-notification.md),
  [p3-w08](../../../p3/plans/p3-w08-tlb-shootdown-transport.md),
  [p3-w06](../../../p3/plans/p3-w06-concurrency-synchronization.md)); the
  failure boundary is in [scope and foundations](01-scope-and-foundations.md)
  §1. The task book's rule applies directly: P3's notification transport may
  be *used*, but any higher-level SGI protocol P6 needs is defined here.
- SGI ID numbering, target forms, and the SPI route-change protocol are
  Implementation Choice items owned by this design under the task book's
  Reserved/Implementation Choice classifications; where a choice would
  bind P7 policy, it is labeled Reserved instead.

Classification: the SGI send mechanism, target validation and accounting,
the P6 Host-SGI ID partition, and the SPI route-change protocol are
**Required** (detail in [01](01-scope-and-foundations.md) §3). Additional
Host-SGI message kinds beyond the declared partition, 1-of-N ("any CPU")
SPI routing, and affinity re-balancing are **Reserved** with triggers.
Guest SGIs, virtual interrupts, scheduler/TLB policy, remote-RPC semantics,
and GICv2/ITS mechanisms are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02–W03 + P3 contracts | [Scope and foundations](01-scope-and-foundations.md) §1 | P6-V04 (W04-DV01) |
| Approved design for bounded SGI send/receipt/completion and SPI routing-change behavior | [Architecture and state](02-architecture-and-state.md), [SGI send](03-code-contracts-sgi-send.md), [SPI routing](04-code-contracts-spi-routing.md) | P6-V04/V06 (W04-DV02) |
| Target eligibility and failure handling with online/offline pCPU state and IRQ lifecycle rules | [Scope and foundations](01-scope-and-foundations.md) §4, [SGI send](03-code-contracts-sgi-send.md) §3 | P6-V04 (W04-DV03) |
| CPU0→CPU1, CPU1→CPU0, multi-target, SPI re-route acceptance | [Validation and handoff](06-validation-and-handoff.md) §2 | P6-V04–P6-V06 (W04-DV04–DV07) |
| SGI remains a Host mechanism; no scheduler or Guest-SMP semantics | [Scope and foundations](01-scope-and-foundations.md) §5, [workflow](05-implementation-workflow.md) step 7 | P6-V04–V06 (W04-DV08) |
| Results recorded; trusted mechanism boundary handed to W11/W13/P7 | [Validation and handoff](06-validation-and-handoff.md) §4 | W04 closure (W04-DV09) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is the P0 documentation
scaffold — no runtime, no SMP lifecycle, no GIC code; the W01–W03 designs
are proposed, not implemented. W04 consumes assumed contracts (P3 online
set and notification primitive; W02 readiness and initial routing; W03
dispatch surface) and may exist only after their evidence is present (task
book P6-ENTRY gating and the plans-index ordering W02, W03 → W04).

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Evidence-backed cross-pCPU SGI | No SGI send path exists; P3 notification is a plan | Typed send mechanism over ICC_SGI1R with per-target accounting ([03](03-code-contracts-sgi-send.md) §2, §4) | "Target-attributed" evidence requires both a send-side record and a receipt-side record that can be correlated | W04; W03 provides receipt counting | W04-DV04/DV05 |
| Declared SGI target forms | None exist | `SgiTarget` type with validated forms and decomposition rules ([03](03-code-contracts-sgi-send.md) §2) | "Declared" forms must be enumerable and each must have one behavior | W04 | W04-DV02 |
| Target-accounting and completion | W03 counters exist as design | Accounting convergence rule (send records vs per-pCPU receipt counters) with drift reporting | P6-V05 requires "determinate target accounting", which is a cross-side property | W04 + W03 counters | W04-DV05 |
| Supported SPI target changes | W02 routes all SPIs to the boot pCPU; no change path exists | Quiesced route-change protocol under the distributor lock ([04](04-code-contracts-spi-routing.md) §3) | A route change racing delivery is exactly the indeterminacy P6-V06 forbids | W04 | W04-DV06 |
| Eligibility vs online/offline state | P3 lifecycle is a plan | Eligibility predicate over P3 online set ∩ W02 LocalReady set, evaluated atomically at send time | Sending to an ineligible target must have a named outcome, not undefined behavior | W04; P3/W02 supply the sets | W04-DV03 |
| No scheduler-policy claim | n/a | Explicit non-responsibility list and SGI-ID partition that carries no policy meaning ([01](01-scope-and-foundations.md) §5) | The mechanism must be usable by P7 without P6 having chosen policy | W04 vocabulary; P7 owns policy | W04-DV08 review |

No row requires inventing a crate boundary, target, or platform constant;
no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **SGIs are sent with ICC_SGI1R_EL1 only (system-register mechanism):**
   no GICD_SGIR-style path exists in the supported posture (W02 posture:
   ARE required), and no memory-based kick substitutes for it. Authority:
   ARE-required posture; task book SGI scope.
2. **Self-SGI uses the same path:** targeting the own pCPU goes through
   ICC_SGI1R like any other send (uniform accounting, one code path).
   Authority: stage-local design freedom; rationale: a second local-raise
   path would create a second accounting story.
3. **IRM (route-to-all-except-self) is a declared target form but is not
   used by P6 mechanisms' default paths:** P6-V04–V06 name explicit
   targets; the form exists because the architecture provides it and W11
   may declare it, but every P6-internal consumer uses explicit targets.
   Multi-affinity fan-out is decomposed into multiple SGI1R writes, each
   separately accounted.
4. **P6 Host-SGI ID partition (stage-local design freedom, recorded):**
   SGI 0..=7 are the assumed P3 notification-transport range (assumed
   contract from
   [p3-w07](../../../p3/plans/p3-w07-cross-cpu-notification.md); if the P3
   handoff names a different range, the P3 contract governs and this table
   is a recorded conflict to resolve, not a silent local renumbering);
   SGI 8 is the P6 generic Host event signal (single meaning: "check your
   per-CPU event state" — no message content, no policy); SGI 9 is
   reserved for P6 validation scenarios (W11); SGI 10..=15 unassigned
   (Reserved). Rationale: fixed low partition with one generic P6 signal
   keeps P7 wakeup design free while P6 stays testable.
5. **Target eligibility is online ∧ LocalReady, evaluated at send time:**
   ineligible targets yield a named rejection (no partial delivery for a
   validated multi-target set — the send is validated fully before the
   first SGI1R write). Authority: task book "target eligibility and
   failure handling with online/offline pCPU state".
6. **SGI completion is the W3 combined-EOI rule; W04 adds target
   accounting on top:** the sending pCPU records a send entry; each
   receiving pCPU's W03 dispatch records the receipt; drift between send
   and receipt totals is reportable, never auto-corrected, and no timeout
   is imposed (a not-yet-ready target legitimately holds an SGI pending at
   its GICR until it enables SGIs — W02's disabled baseline makes this
   deterministic rather than lost).
7. **SPI routing changes are quiesced and locked:** a route change
   requires the SPI disabled and inactive (checked, not assumed), is
   performed under the distributor-scoped lock, orders the IROUTER write
   before the re-enable write, and only ever targets IRM=0 explicit
   affinities. Authority: P6-V06 "changed route reaches the replacement
   target" with determinate accounting; 1-of-N routing is Reserved.
8. **No policy in the vocabulary:** the Host event SGI carries no message,
   no priority, and no scheduling meaning; P7 defines its own higher-level
   protocol over the mechanism. Authority: plan non-scope and handoff
   wording.

## Work breakdown and loading order

1. Read [scope and foundations](01-scope-and-foundations.md): assumed
   contracts, eligibility, SGI partition, non-policy rules.
2. Read [architecture and state](02-architecture-and-state.md): modules,
   accounting model, route state machine, concurrency.
3. Read [SGI send contracts](03-code-contracts-sgi-send.md) and
   [SPI routing contracts](04-code-contracts-spi-routing.md) before
   writing code.
4. Apply the changes in the order stated in the
   [implementation workflow](05-implementation-workflow.md).
5. Store actual commands, results, and run/not-run status in
   `../../verification/p6-w04-smp-interrupt-routing-sgi-verification.md`;
   record factual implementation status in
   `../p6-w04-smp-interrupt-routing-sgi-record.md` only when
   implementation begins. Neither this design nor a record may claim W04
   complete.

## Explicitly excluded interfaces

No scheduler-reschedule API, no run-queue or wakeup primitive, no
TLB-shootdown semantics, no message/RPC layer over SGIs, no load
balancing, no Guest SGI or virtual-interrupt surface, no GICv2 SGIR, no
ITS/MSI routing, no board-conditional affinity tables, and no new IRQ
entry path is designed or authorized by W04. The P3 notification
primitive's own semantics are P3's; W04 only transports and accounts.

## Downstream handoff

- **W11** receives the Host-SGI scenario inputs: the partition table
  (which IDs exist, which is the validation SGI), the declared target
  forms, and the accounting surface its CPU0↔CPU1 and multi-target
  scenarios assert on.
- **W13** receives the send/receipt accounting surface and the routing
  events for latency and regression evidence (P6-V24–V26).
- **P7** receives an evidenced Host mechanism (typed send, eligibility,
  accounting, the generic event SGI) and explicitly not a reschedule
  policy, a wakeup contract, or a frozen API; P7 designs its own protocol
  over this boundary per the task book §7.
- The Validation Guest never receives a Guest SGI from this package;
  Guest-visible SGI behavior is W07/W08/P8 scope.
