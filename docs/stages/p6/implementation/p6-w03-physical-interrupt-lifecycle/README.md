# P6-W03 Physical Interrupt Lifecycle — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The safe, diagnosable Host physical-IRQ lifecycle from receipt
through classification, dispatch outcome, and completion, required by
[P6-W03](../../plans/p6-w03-physical-interrupt-lifecycle.md).  
**Owner/change context:** P6-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W03. It converts the plan
into an identification/classification layer over the acknowledged CPU
interface, a bounded dispatch loop with one completion owner, a minimal
consumer-registration surface, and the safe spurious/unknown/consumerless
outcomes. It deliberately does **not** design SGI routing policy or SPI
route changes (W04), timer event semantics (W05), virtual interrupts
(W07/W08), a device-driver framework, production DoS controls, or scheduler
wakeup semantics.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) — the
system-register sequencing, interrupt-context bounding, typed-ID, and
bounded-lock rules are load-bearing here — then loads only the linked
supporting file needed for its assigned step. Before editing it must also
follow the Coding Guidelines preflight, including the repository
[AGENTS.md](../../../../../AGENTS.md), [documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P6 task book](../../task-book-v0.1.md), and the P6-W03 plan. This document
is the proposed detailed design; it is not a completion record and contains
no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W03 plan → this
design → Coding Guidelines. In particular:

- ADR-032's physical-chain baseline and the task book Required outcome
  ("safely classifies, dispatches, completes, and diagnoses SGI/PPI/SPI and
  spurious/unknown inputs") govern the lifecycle; ADR-007's Guest-untrusted
  discipline and ADR-048's telemetry requirement shape the error and
  observability model.
- [P6-W02](../p6-w02-physical-gic-bring-up/README.md) owns bring-up and the
  readiness ledger; W03 consumes the acknowledged Host IRQ boundary (every
  interrupt disabled, combined-EOI posture, per-pCPU readiness) and the
  register-access surface
  ([W02 03](../p6-w02-physical-gic-bring-up/03-code-contracts-register-access.md)).
  W03 must not re-initialize hardware or relax W02's initial state.
- The P1 exception-entry contract
  ([p1-w05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md)) and
  P3 CPU-local/synchronization contracts
  ([p3-w09](../../../p3/plans/p3-w09-cpu-local-exception-interrupt.md),
  [p3-w06](../../../p3/plans/p3-w06-concurrency-synchronization.md),
  [p3-w04](../../../p3/plans/p3-w04-per-cpu-runtime.md)) are assumed inputs
  with the failure boundary in [scope and foundations](01-scope-and-foundations.md)
  §1.
- Register field interpretation (IAR special values, priority widths) is
  taken from the GIC specification revision pinned in the W01 record; this
  design fixes names, ranges, ownership, and behavior, and marks exact
  special-value encodings as a Specification Investigation checkpoint at
  implementation.

Classification: the identification layer, dispatch/completion loop, consumer
registry, safe unknown/spurious outcomes, counters, and bounded storm
containment are **Required** (detail in
[01](01-scope-and-foundations.md) §3). Group-0/Secure interrupt handling,
FIQ-specific semantics beyond the unexpected boundary, runtime consumer
deregistration, level/edge trigger management, and priority-preemption
policy are **Reserved** with triggers. Guest injection, vIRQ, List
Registers, maintenance handling, SGI send policy, SPI routing policy,
device-driver handler frameworks, scheduler wakeups, and production
IRQ-DoS resistance are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02 readiness + P1/P3 contracts | [Scope and foundations](01-scope-and-foundations.md) §1 | P6-V20 (W03-DV01) |
| Bounded Host IRQ lifecycle, state authority, completion responsibility, error boundaries | [Architecture and state](02-architecture-and-state.md) §2–§5 | P6-V20 (W03-DV02) |
| Classification and dispatch outcomes with per-pCPU diagnostics and source categories | [Identification](03-code-contracts-identification.md), [dispatch/completion](04-code-contracts-dispatch-completion.md) | P6-V20 (W03-DV03) |
| Ordinary, repeated, simultaneous, spurious, unknown, unconsumed acceptance | [Validation and handoff](06-validation-and-handoff.md) §2 | P6-V20 (W03-DV04–DV07) |
| Guest isolation, no-invalid-index/handler, storm containment, telemetry | [Scope and foundations](01-scope-and-foundations.md) §4, [dispatch/completion](04-code-contracts-dispatch-completion.md) §5 | P6-V20/V22 (W03-DV08, DV09) |
| Lifecycle contract handed to W04/W05/W07/W11–W13 | [Validation and handoff](06-validation-and-handoff.md) §4 | W03 closure (W03-DV10) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is the P0 documentation
scaffold — no runtime, no EL2 exception vectors, no P3 SMP lifecycle, no GIC
code. The W01/W02 designs are proposed, not implemented. W03 therefore
consumes assumed contracts end to end and may exist only after the W02
readiness evidence and P1/P3 entry conditions are actually present (task
book P6-ENTRY gating).

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Safe lifecycle from receipt to completion | No IRQ entry-to-EOI path exists anywhere | Entry bridge contract over the P1 exception baseline + the acknowledge/complete loop ([02](02-architecture-and-state.md) §2, [04](04-code-contracts-dispatch-completion.md) §3) | Receipt is only defined relative to the P1 vector entry; completion is only defined by an explicit EOI owner | P1 supplies entry; W03 owns the loop | W03-DV03 |
| Classification of SGI/PPI/SPI | No typed interrupt-ID surface exists at runtime | `PhysicalIntId` family with validated construction over the W02-enabled range ([03](03-code-contracts-identification.md) §2) | "No invalid index/handler" is a type-and-range property, not a convention | W03 (IDs extend the P0 newtype baseline) | W03-DV03 |
| Dispatch outcomes with one completion owner | Nothing exists | Registry with one consumer per ID and an EOI-after-handler rule under a fixed posture ([04](04-code-contracts-dispatch-completion.md) §2, §3) | Ambiguous completion ownership is the defect class (double-EOI, missing-EOI) the outcome forbids | W03 | W03-DV04 |
| Safe spurious/unknown/consumerless outcomes | Nothing exists | Named outcome paths, never an unvalidated handler call or index ([03](03-code-contracts-identification.md) §4, [04](04-code-contracts-dispatch-completion.md) §4) | Spurious/unknown handling must be structural: no path from an unvalidated ID to memory access or handler invocation | W03 | W03-DV05 |
| Storm smoke preserves invariants | Nothing exists | Per-entry dispatch bound, per-ID/per-pCPU counters, threshold events ([04](04-code-contracts-dispatch-completion.md) §5) | Containment expectations must be bounded and observable before W12 can test them | W03; W12 consumes | W03-DV09 |
| Diagnosability per pCPU | P3 CPU-local diagnostics are plans | Counter records in per-CPU storage; no shared-scratch state | A storm must be attributable to a pCPU and an ID class | P3 storage; W03 counters | W03-DV09 |

No row requires inventing a crate boundary, target, or platform constant;
no decision blocker is outstanding for this design.

## Resolved design decisions and their authority

1. **Acknowledgement via the Group-1 NS interface with combined EOI
   (EOImode=0 posture inherited from W02):** `ICC_IAR1_EL1` acknowledges;
   `ICC_EOIR1_EL1` with the same value drops priority and deactivates in
   one step. Authority: W02 decision 5; the run/deactivate split (DIR) is
   Reserved for W08's presentation needs. Consequence: the acknowledging
   pCPU is the sole completion owner; no cross-CPU completion exists.
2. **Handler-then-EOI ordering:** the consumer runs before completion so
   the interrupt remains active during handling (no same-ID re-entry at
   the CPU; re-asserted level state is observed after completion).
   Authority: stage-local design freedom (task book Implementation Choice
   class); recorded for W03 closure.
3. **Flat, non-nested handling in P6:** binary-point/no-preemption posture
   from W02 means a running handler is not preempted by another Group-1
   interrupt on the same pCPU; nested-host-IRQ support is not designed.
   Consequence: consumer code must be bounded ([04](04-code-contracts-dispatch-completion.md)
   §2 constraints); long work must be deferred to the consumer's own
   bottom half (its design's responsibility, not W03's).
4. **One consumer slot per interrupt ID; registration-before-enable:** a
   consumer is registered by its owning subsystem before the source is
   enabled; the slot is published (release) before the enabling write;
   deregistration is Reserved (no P6 consumer needs it; removal ordering
   is a future design). Authority: W02 INV-1 continued.
5. **Consumer callback is the minimal stage-local surface:** a fixed
   function shape taking an immutable host-IRQ context; no device-model,
   no return-status protocol, no chained handlers in P6. Authority: the
   plan excludes "detailed handler APIs"; this minimum is owned by this
   design with the rationale that W05 (timer PPI) and W08 (maintenance
   PPI) need exactly one bounded entry each.
6. **Special IAR values decode symbolically:** no-pending/spurious and
   implementation-reserved bands are recognized as bands, not magic
   numbers; exact encodings are transcribed from the pinned revision at
   implementation and exercised by unit tests. Authority: task book
   Specification Investigation class.
7. **Unknown and consumerless interrupts complete, count, and diagnose —
   they never panic and never call through an unvalidated slot.** A
   panic on unknown input would convert a platform/guest-caused event
   into a hypervisor invariant failure (ADR-007 discipline; task book
   P6-V20).
8. **Storm containment is bounded work per entry plus observability:** a
   fixed per-entry dispatch bound with hardware re-delivery for the
   remainder, counters, and threshold events; no rate limiting, masking
   policy, or DoS resistance is claimed (task book P6-V22 wording; W12
   owns the robustness evidence).

## Work breakdown and loading order

1. Read [scope and foundations](01-scope-and-foundations.md): assumed
   contracts, entry gating, lifecycle boundary, isolation rules.
2. Read [architecture and state](02-architecture-and-state.md): modules,
   state authority table, per-IRQ lifecycle, concurrency.
3. Read [identification contracts](03-code-contracts-identification.md)
   then [dispatch/completion contracts](04-code-contracts-dispatch-completion.md)
   before writing any code.
4. Apply the changes in the order stated in the
   [implementation workflow](05-implementation-workflow.md).
5. Store actual commands, results, and run/not-run status in
   `../../verification/p6-w03-physical-interrupt-lifecycle-verification.md`;
   record factual implementation status in
   `../p6-w03-physical-interrupt-lifecycle-record.md` only when
   implementation begins. Neither this design nor a record may claim W03
   complete.

## Explicitly excluded interfaces

No SGI send primitive, no SPI route-change API, no trigger-type
(level/edge) configuration surface, no interrupt-balancing, no scheduler
wakeup hook, no virtual-IRQ or List-Register interface, no Guest-visible
state, no device-model dispatch framework, no new IRQ-entry assembly (the
P1 vector entry is the only entry path), and no platform-name branch is
designed or authorized by W03. The maintenance interrupt and timer PPI
*semantics* are W05/W08/W09; W03 provides their registration/enable/
completion mechanics only.

## Downstream handoff

- **W04** receives the classification/consumer surface for Host SGIs and
  the completion rule its SGI consumers are bound by; routing decisions
  stay W04-owned.
- **W05** receives the PPI registration/enable path and the per-entry
  dispatch context its timer handler runs in.
- **W07** receives the Host receive/classify/complete boundary that its
  vIRQ producers must not bypass or redefine.
- **W11** receives the observable Host-IRQ behaviors its Guest scenarios
  exercise (completion, masking by disable, spurious safety).
- **W12** receives the counter/bound structure and the named unknown/
  spurious outcomes as its robustness baseline.
- **W13** receives the counter/event surface for correlation and the
  latency-measurement hooks' location (measurement itself is W13's).
- No consumer receives handler-API freedom beyond the minimal callback, a
  second completion owner, or a machine-specific IRQ layout.
