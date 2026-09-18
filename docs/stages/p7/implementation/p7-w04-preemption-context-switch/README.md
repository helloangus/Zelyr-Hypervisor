# P7-W04 Preemption and Context-Switch Correctness — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** P6-timer-driven scheduler deadlines, time-slice behavior, return
to EL2, scheduler reconsideration, and the required context/address-space/
timer/vIRQ/event preservation required by
[P7-W04](../../plans/p7-w04-preemption-context-switch.md).  
**Owner/change context:** P7-W04 implementation handoff; preemption and
switch-behavior authority for W05/W07/W09/W10 per the plan handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W04. The plan fixes *what*
must hold (a CPU-bound Guest is preempted and cannot monopolize a shared
pCPU; declared per-vCPU state remains isolated across repeated switches) and
defers the preemption model, switch sequencing, and ownership rules to this
design. It defines two cooperating mechanism areas — the deadline/preemption
bookkeeping around the P6 timer contract, and the vCPU-switch sequence that
wraps the P4 entry/exit and P6 timer/vIRQ state contracts — as
pseudocode-level contracts for a coding agent.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the supporting file for its assigned step:

- [Scope and foundations](01-scope-and-foundations.md): ledger, prerequisite
  contracts with failure boundaries (P6-W05/W13, P4-W09), itemized scope,
  and the explicit mechanism/policy split including what W04 refuses to
  decide.
- [Architecture and state](02-architecture-and-state.md): logical modules,
  the preemption-points model, the switch ownership table, concurrency and
  IRQ-context bounds, and isolation-class definitions.
- [Preemption code contracts](03-code-contracts-preemption.md): deadline
  arm/cancel, slice policy source, deadline-IRQ handling, reschedule intent,
  reconsideration trigger types.
- [Context-switch code contracts](04-code-contracts-context-switch.md): the
  switch sequence with per-step ownership, failure boundaries, and the
  isolation requirements.
- [Implementation workflow](05-implementation-workflow.md): ordered steps.
- [Validation and handoff](06-validation-and-handoff.md): validation matrix
  (P7-V08–V09), error/security/observability model, handoff checklist.

Before editing, the agent must follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P7 task book,
W01 register, the P7-W02 lifecycle design, and the P7-W04 plan). This design
proposes only.

## Authority, constraints, and scope classification

Governing order: ADR baseline → P7 task book → frozen P0–P6 contracts →
P7-W04 plan → this design. Binding constraints:

- **ADR §5:** every vCPU switch must explicitly handle arch context, virtual
  timer, vGIC state, lazy FP/SIMD (later), and TLB/VMID lifecycle. The
  architecture backend owns registers and virtualization controls; Core sees
  only standard `ExitReason` semantics. This design assigns every switch
  step to exactly one owner and has the scheduler touch no registers.
- **P6-W05 handoff (P7-IN-07):** P7 receives an evidenced per-pCPU
  monotonic-time and deadline-event mechanism — not a preemption policy.
  W04 builds the policy-bearing bookkeeping on that mechanism and owns no
  timer programming.
- **P6-W07/W08/W10 (P7-IN-08):** vIRQ pending state and event delivery are
  P6-owned and must survive arbitrary descheduling; W04 requires, but does
  not implement, that preservation.
- **P4-W04/W09 (P7-IN-05):** Guest entry/return mechanism, arch context
  save/restore at exit, Stage-2/VMID identity, and the stop/fault boundary
  are P4-owned. W04 sequences calls into them; it never redefines them.
- **W02 lifecycle authority**
  ([P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md)): every
  state change of a switch goes through the lifecycle engine; entry only
  via `admit_for_entry`; return only via `complete_exit`.
- **Coding Guidelines:** IRQ/VM-exit paths perform bounded work; the
  deadline handler does bookkeeping only and never performs the switch.

Classification:

- **Required:** preemption-point model (deadline expiry, Guest exit,
  reschedule request); slice policy source and default rule; deadline
  arm/cancel/re-arm discipline; bounded deadline-IRQ handling and the
  pCPU-local reschedule intent; the reconsideration trigger contract; the
  switch sequence with per-step ownership and failure boundaries; the
  isolation-classes list (P7-V09 basis); `DescheduleReason` semantics for
  W09.
- **Reserved:** lazy FP/SIMD save (ADR §5 explicitly later); any preemption
  of EL2-internal work; migration policy; weighted/RT preemption classes
  (ADR-017).
- **Out of Scope:** timer register programming and time representation
  (P6); the time-slice *value* beyond the declared selection rule (task
  book Out of Scope; value chosen at implementation, recorded, never a
  performance claim); tick-model alternatives (this design fixes the
  on-demand deadline model — recorded decision); register-save layout and
  assembly boundary (P4); pick policy/queue order (W05); pause/stop flows
  (W07); accounting/trace encoding (W09); guest workloads (W10).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02 lifecycle and P6 timer/event contracts | [workflow](05-implementation-workflow.md) step 1 | W04-DV01 review |
| Observable preemption and re-scheduling outcome | [preemption contracts](03-code-contracts-preemption.md) §1–§3; [architecture and state](02-architecture-and-state.md) §2 | P7-V08 |
| Timer-driven scheduler deadlines; return to EL2; reconsideration | [preemption contracts](03-code-contracts-preemption.md) §2/§4 | P7-V08 |
| Switch-isolation observations across repeated rotation | [context-switch contracts](04-code-contracts-context-switch.md) §2 (isolation classes); [validation](06-validation-and-handoff.md) DV03 | P7-V09 |
| Failure boundaries and observability with predecessor contracts | [context-switch contracts](04-code-contracts-context-switch.md) §3; [validation](06-validation-and-handoff.md) §2 | W04-DV05 review |
| Preemption/isolation evidence planning; handoff | [workflow](05-implementation-workflow.md) steps 6–7; [validation and handoff](06-validation-and-handoff.md) §3 | P7-V08/V09 evidence |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs`):
P0 documentation scaffold; no code. The P6 timer/vIRQ contracts and P4
entry/exit facts are plan-level only (W01 register P7-IN-05/07/08). W02's
lifecycle design and W03's placement design are proposed siblings this
package consumes. Every runtime-binding input is an assumed contract cited
by plan path with a failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Make CPU-bound Guest preemption … objectively verifiable" (plan Goal) | No deadline handling, no slice model, no preemption point exists | Deadline arm/cancel discipline on the P6 mechanism + slice policy source + bounded IRQ handler + reschedule intent | "CPU-bound Guest cannot monopolize" is a property of a mechanism, not a test afterthought | P7-W04 (bookkeeping); P6 owns the timer mechanism | W04-DV02/03 tests; P7-V08 |
| "Timer preemption, testable time-slice semantics" (task book Required) | No slice definition exists | Slice policy source (per-entity value from W05 policy; validated default rule) and an expiry determination | A slice must have a defined source, bound, and expiry predicate to be testable | Source: W05 policy shape, W04 contract; value chosen at implementation | W04-DV02; P7-V08 |
| "context preservation … required context/address-space/timer/vIRQ/event preservation" (plan Scope) | Ownership is stated in ADR §5 but no P7 sequence exists | The switch sequence with per-step owner and failure boundary; the isolation-classes list | Preservation is provable only if each state class has one owner and one transfer point | Owners: P4/P6 (mechanisms); sequencing: W04 | W04-DV03/04; P7-V09 |
| "return to EL2, scheduler reconsideration" (plan Scope) | W02's exit boundary exists as design; no reconsideration trigger model | Preemption-point enumeration + reconsideration trigger contract consumed by W05's loop | Reconsideration needs a defined set of triggers so the loop is reviewable | W04 triggers; W05 loop | W04-DV03; P7-V08 |
| "Review failure boundaries and observability needs with predecessor contracts" (work sequence 4) | Not done | Switch failure-boundary table + `DescheduleReason` semantics handed to W09 | Failure behavior must be designed before evidence can be planned | W04; rendering W09 | W04-DV05 review |

No row invents a crate, target, or runtime policy beyond this design's
recorded decisions.

## Resolved design decisions and their authority

1. **On-demand deadline model, no periodic tick.** Each dispatch arms one
   P6 per-pCPU deadline at `now + slice`; expiry is the only timer-driven
   preemption trigger; there is no periodic scheduler tick in P7.
   Rationale: P6-W05 delivers armable one-shot per-pCPU deadlines (not a
   tick facility); the task book's "testable time-slice semantics" needs
   exactly a per-dispatch deadline; fewer timer interactions mean fewer
   races for W06/W08. Authority basis: P7-IN-07 mechanism shape; stage-local
   choice recorded here because the plan defers the tick model to detailed
   design.
2. **Preemption is trap-boundary preemption.** A running Guest loses its
   pCPU only at an EL2 control point — after an exit (timer IRQ exit, any
   other exit) — never mid-instruction; the deadline IRQ manifests to the
   Guest as an IRQ-class exit, and EL2 then decides. Rationale: AArch64
   EL2 regains control only via exception entry (P1/P4 boundary); this is
   also the safety property that keeps register ownership with P4.
3. **Slice value selection rule.** `TimeSlice` is a per-dispatch value from
   the W05 policy source; the v0 default is a single boot-constant chosen at
   implementation time and recorded in the W04 record, selected large
   enough that timer handling overhead is small relative to the slice and
   small enough that an M:N rotation window (see W05 fairness) stays
   bounded in validation scenarios. The value is never a performance claim
   (task book: no KPI) and changing it is routine policy-parameter change,
   recorded.
4. **IRQ-context split.** The deadline handler executes in IRQ context and
   performs: record expiry, set the pCPU-local reschedule intent, mark
   trace. The switch itself executes in the pCPU's scheduler control
   context after the handler returns. Rationale: Coding Guidelines bound on
   IRQ work; keeps locking trivial in the handler.
5. **Switch sequencing ownership.** The switch is a pCPU-local sequence
   (guardrail: per-pCPU run-queue and slot ownership — W05 owns queues;
   the `current_vcpu` slot is W02's admission module; the sequence here
   orchestrates both and touches neither beyond their contracts). Register
   state: written only by P4 save/restore. Virtual timer: P6 save/restore.
   vGIC/LR and pending events: P6. Stage-2/VMID: P4 activation. Deadline:
   W04 arm/cancel. Lifecycle: W02 engine. Every step names its owner in
   [the switch contracts](04-code-contracts-context-switch.md) §1; a step
   with two owners is a design violation.
6. **Requeue-on-activation-failure, bounded.** If activating the next
   candidate fails (P4 address-space or P6 restore error), the candidate is
   requeued undispatched-this-round with a diagnostic; the loop returns to
   the picker or idles (W08). A candidate failing repeatedly is escalated
   to an invariant investigation record, not silently retried forever and
   not marked `Faulted` (the failure is not Guest-caused). Authority: P0
   panic policy boundary (fatal only for invariant violations).
7. **`DescheduleReason` semantics owned here.**
   `{ SliceExpired, Voluntary, Blocked, Paused, Stopped, Faulted,
   RescheduleRequest }` is the mechanism-side taxonomy every deschedule
   carries; W09 owns rendering/aggregation. Authority: W04 is the only
   producer of deschedules; fixing the taxonomy here prevents W09 from
   guessing.

## Work breakdown and loading order

1. Read this README; load supporting files per assigned step as listed
   above.
2. Implement in [the workflow](05-implementation-workflow.md) order:
   slice/deadline bookkeeping → deadline-IRQ handler → reconsideration
   triggers → switch sequence → isolation instrumentation → evidence.
3. Findings go to `../p7-w04-preemption-context-switch-record.md`; evidence
   to `../../verification/p7-w04-preemption-context-switch-verification.md`.
   Neither is created by this design; nothing here claims completion.

## Explicitly excluded interfaces

No public API, ABI, trace encoding, crate boundary, or assembly layout is
designed or authorized. This design contracts no timer registers, no
VMID allocator internals, no register-save layout, no LR manipulation, and
no guest-visible semantics. All names in
[the preemption contracts](03-code-contracts-preemption.md) and
[the switch contracts](04-code-contracts-context-switch.md) are internal,
stage-local design names. Queue topology and pick order are W05; block/wake
sourcing is W06; pause/stop flows are W07; remote reschedule and idle are
W08; counters and rendering are W09.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md) consumer map:

- **W05** receives the reconsideration trigger set and the requirement that
  its control loop consults the reschedule intent after every trigger; the
  slice policy source hook lives in its policy layer.
- **W07** receives: pause/stop of a *running* vCPU is realized as a
  reschedule request + control event applied at the pCPU owner's exit
  boundary — never by mutating a running vCPU cross-CPU.
- **W09** receives `DescheduleReason` semantics and the switch trace
  points; W09 renders and aggregates.
- **W10** receives the isolation-classes list as the workload coverage
  target (A→B→C→A rotation under CPU/HVC/timer/WFI work).
- **W08** receives the rule that remote reschedule requests surface as the
  `RescheduleRequest` preemption point on the target pCPU (transport: P3;
  semantics: W08).

W14 carries preemption/switch semantics into the P8 handoff; P8 may rely
only on evidenced semantics per the task book §7.
