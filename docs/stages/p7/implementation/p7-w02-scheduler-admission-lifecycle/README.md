# P7-W02 Scheduler Admission and Lifecycle — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The scheduler-controlled normal Guest entry/exit admission boundary,
the explicit vCPU run-state lifecycle with legal and rejected transitions, and
the core single-running invariants required by
[P7-W02](../../plans/p7-w02-scheduler-admission-lifecycle.md).  
**Owner/change context:** P7-W02 implementation handoff; this design is the
P7 lifecycle authority per the plan handoff ("W03–W11 can rely on lifecycle
authority and invariants").  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W02. The plan fixes *what*
must hold (scheduler-controlled normal execution, an explicit seven-state
lifecycle, objective transition validity, checkable invariants) and defers
*how* to this design. This design therefore fixes the run-state set and
transition table, the admission gate through which all normal Guest
entry/exit passes, the ownership of every mutable lifecycle fact, the
concurrency discipline, and the invariant set — as pseudocode-level contracts
for a coding agent, not runnable production code.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the supporting file named for its assigned step:

- For the requirement ledger, prerequisite contracts and their failure
  boundaries, and the itemized scope classification, load
  [scope and foundations](01-scope-and-foundations.md).
- For the logical modules, object ownership, state machines, concurrency and
  security model, load [architecture and state](02-architecture-and-state.md).
- For the full function/type/data-structure contracts with pseudocode
  (lifecycle engine, admission gate, invariant audit), load
  [lifecycle code contracts](03-code-contracts-lifecycle.md).
- For the ordered implementation steps, load
  [the implementation workflow](04-implementation-workflow.md).
- For the validation matrix, error/security/observability model, and handoff
  checklist, load [validation and handoff](05-validation-and-handoff.md).

Before editing anything, the agent must also follow the Coding Guidelines
preflight (repository `AGENTS.md`, documentation index, ADR baseline, P7 task
book, P7-W01 register, and the P7-W02 plan). This design proposes; it does not
implement and claims nothing.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → frozen P0–P6 handoff
contracts → P7-W02 plan → this design. Binding constraints:

- **ADR-016 / task book:** vCPU is the sole schedulable Guest execution
  entity; VM is a lifecycle container and never enters the scheduler.
  Evolution from static binding to preemptive M:N must preserve pinned
  operation.
- **ADR §4.1** fixes the vCPU state names: `Offline, Runnable, Running,
  Blocked, Paused, Stopped, Faulted`. This design may define the transition
  edges between them (stage-local design freedom — the task book explicitly
  defers transition semantics and representation to detailed design) but may
  not rename, split, or add states.
- **Task book §2:** "Scheduler-controlled normal Guest entry and return after
  exit, with only bounded early-boot/bring-up/emergency-debug exceptions."
  The exception *classes* come from the W01 register §E
  (P7-IN boundary, classes E1–E3); this design owns the gate mechanics and
  the instance enumeration.
- **Mechanism/policy separation (Plan guide guardrail):** the lifecycle
  engine and admission gate are policy-free mechanism; scheduling policy
  (which vCPU next, slices) lives in the policy layer owned by
  [P7-W05](../p7-w05-shared-mn-multivm/README.md). Policy must not leak into
  the lifecycle contracts defined here.
- **Authorization (ADR-013, P5):** external lifecycle control actions
  (pause/resume/stop and later management controls) authorize through the P5
  capability/handle + rights + generation model. This engine treats an
  authorized request as a precondition and records authority metadata; it
  never substitutes roles or VM ids for capability checks.
- **Failure containment (ADR-007/012 invariants, P5 taxonomy):** a
  Guest-caused fault moves the vCPU to `Faulted` and never panics the
  hypervisor; a hypervisor invariant violation is a fatal crash per the P0
  panic policy. The two are distinct classifications and this design keeps
  them distinct.
- **Concurrency (P3-W06):** scheduler code runs on pCPUs that take interrupts
  and timer events (P6); lock holds are bounded, IRQ-context work is bounded,
  and no allocation occurs in IRQ context (Coding Guidelines).

Classification:

- **Required:** `VcpuRunState` and `LifecycleEvent` definitions; the
  transition legality table with rejected-invalid behavior; the admission
  gate `admit_for_entry` and exit boundary `complete_exit`; the bypass-class
  register (E1–E3 instances); the invariants INV-1…INV-5 with a host-runnable
  checker; the transition-authority context; telemetry semantics handed to
  [P7-W09](../p7-w09-accounting-diagnostics/README.md).
- **Reserved:** re-entry of `Stopped` vCPUs (restart) — P7 treats
  `Stopped`/`Faulted` as terminal; dynamic VM/vCPU creation is a later stage
  (task book Reserved). The gate must not preclude them, but no restart path
  is designed.
- **Out of Scope:** runqueue container design and pick policy
  ([P7-W05](../p7-w05-shared-mn-multivm/README.md)); placement eligibility
  rules ([P7-W03](../p7-w03-placement-configuration/README.md) — W02 consumes
  the eligibility predicate as an interface); preemption/deadline mechanics
  ([P7-W04](../p7-w04-preemption-context-switch/README.md)); block/wakeup
  event semantics ([P7-W06](../p7-w06-block-wakeup/README.md)); pause/stop
  policy, remote-running handling, and fault containment flows
  ([P7-W07](../p7-w07-pause-stop-fault/README.md)); cross-CPU reschedule and
  idle behavior ([P7-W08](../p7-w08-smp-reschedule-idle/README.md)); trace
  encoding and counter definitions (P7-W09); any management ABI, Control
  Domain policy, public API stability, crate/file layout, or trace binary
  format.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| W01 entry and authority assumptions | [scope and foundations](01-scope-and-foundations.md) §2 | P7-V02 precondition (review) |
| Normal execution is scheduler-controlled (entry + return) | [contracts](03-code-contracts-lifecycle.md) §4 (admission gate, exit boundary, bypass register) | P7-V02 |
| Lifecycle semantics: state set, legal transitions, rejected invalid transitions | [contracts](03-code-contracts-lifecycle.md) §2–§3 (state, events, transition table) | P7-V03 |
| Core single-running invariants, checkable | [architecture and state](02-architecture-and-state.md) §5; [contracts](03-code-contracts-lifecycle.md) §6 (checker) | P7-V04 |
| Integration with predecessor vCPU and failure terminology, without redefining | [scope and foundations](01-scope-and-foundations.md) §3; [architecture and state](02-architecture-and-state.md) §4 | P7-V02/V04 review |
| Lifecycle/property evidence planning; handoff to W03–W11 | [workflow](04-implementation-workflow.md) steps 6–7; [validation and handoff](05-validation-and-handoff.md) §3 | P7-V03/V04; handoff checklist |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs`,
audit via `git ls-files`): P0 documentation scaffold; no Cargo workspace, no
Rust sources, no CI. P1–P6 implementation/verification directories contain
only `.gitkeep`. The P7 stage `implementation/` index exists but is a
coordinator-owned placeholder. Therefore every P0–P6 contract this design
consumes is an **assumed contract** cited by plan path, with a failure
boundary if delivered differently; none is evidenced today. The W01 register
(`../p7-w01-entry-contract-reconciliation-record.md`, created when W01 runs)
is the authoritative input map this design assumes.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Define a scheduler-controlled normal execution admission boundary" (plan Goal) | P4 delivers a direct, non-scheduler first-entry path; no P7 gate exists | Admission gate + exit boundary contracts, plus the bypass-class instance register derived from W01 §E | Without a single gate and an enumerated bypass list, "normal entry is scheduler-controlled" is unverifiable | P7-W02 (this design); classes from W01 | W02-DV02 review; P7-V02 tests |
| "an explicit, testable vCPU lifecycle" (plan Goal) | ADR §4.1 names the seven states; no transition table, no engine, no tests exist | `VcpuRunState`/`LifecycleEvent` types, a total legality table, a transition engine with typed rejection, host-side table tests | A lifecycle is testable only when every (state, event) pair has a defined outcome | P7-W02 (edges); ADR owns the state set | W02-DV03 table tests; P7-V03 |
| "core single-running invariants" (plan Scope) | Stated in task book Required; not defined operationally | Invariants INV-1…INV-5 with a checker runnable host-side and as QEMU assertions | "Checkable" requires a named, mechanical check per invariant | P7-W02; measurement consumers W09/W11 | W02-DV04 checker tests; P7-V04 |
| "Integrate the contract with predecessor vCPU and failure terminology without redefining it" (work sequence 3) | P4/P5 contracts are plan-level, unevidenced | Mapping tables: P4 vCPU execution facts → P7 states; P5 fault taxonomy → `Faulted` vs fatal; P3 pCPU states → gate eligibility input | Integration without redefinition requires the mapping to be explicit and reviewable | P7-W02 documents; P4/P5 own the mapped contracts | W02-DV02 mapping review |
| "Record the accepted boundary and hand it to placement, preemption, wakeup, pause, observability, and stress packages" (work sequence 5) | Not done | Handoff section naming W03–W11 consumption points and frozen invariants | Handoff requires named consumption points, not prose | P7-W02 | W02-DV07 consumability review |

No row requires inventing a crate, target, or runtime policy beyond this
design's recorded decisions.

## Resolved design decisions and their authority

1. **State set and naming.** Exactly the ADR §4.1 vCPU states, spelled
   `Offline, Runnable, Running, Blocked, Paused, Stopped, Faulted`, carried in
   a dedicated `VcpuRunState` enum (no booleans, no sentinel integers — Plan
   guide prohibition on hidden state machines). Authority: ADR fixes the set;
   the type realization is stage-local.
2. **Transition edges.** The full legality table in
   [lifecycle contracts](03-code-contracts-lifecycle.md) §3, including edges
   the ADR's linear presentation does not show but the task book requires:
   `Running→Runnable` (deschedule), `Running/Runnable/Blocked→Paused` (task
   book P7-V15 "running/runnable/blocked pause safely"), and
   `Paused→Runnable` (resume with preserved eligibility). This reads ADR
   §4.1 as a state *set*, not a chain; the task book's pause/resume matrix
   requires the additional edges. Authority: task book Required + P7-V15;
   recorded here because the ADR text is ambiguous.
3. **Terminality.** `Stopped` and `Faulted` have no outgoing edges in P7;
   vCPU restart is Reserved. Authority: task book Reserved list ("dynamic
   VM/vCPU creation" and no P7 restart requirement).
4. **Admission model.** Normal Guest entry is permitted only through
   `admit_for_entry` (the gate), which (a) re-checks placement eligibility
   via the W03 predicate interface, (b) performs the `Dispatch` transition,
   and (c) reserves the single `current_vcpu` slot on the pCPU. Normal return
   always passes `complete_exit`, which returns control to the scheduler
   control loop; no exit path re-enters the Guest directly. Bypasses are
   enumerated statically against the W01 E1–E3 classes and reviewed.
   Authority: task book Required bullet.
5. **Scheduler activation.** The lifecycle engine has an explicit scheduler
   mode: before `activate_scheduler`, bypass classes E1/E2 are legal and the
   gate is not yet enforcing; after activation they are empty and any
   non-registered entry path is an invariant violation (E3 remains, by
   definition outside normal scheduling). Authority: task book bounded-
   exception wording; mechanism owned here.
6. **Lock discipline.** One per-vCPU lifecycle lock; it is a *leaf* lock:
   scheduler code never acquires another lock while holding it, and queue
   mutations (W05) happen outside lifecycle critical sections. Rationale:
   makes the transition engine analyzable and lets W05/W06/W07/W08 extend
   the P3-W06 lock order without nesting cycles. The full scheduler lock
   order is assembled by W05/W08; this design fixes only the leaf rule.
7. **IRQ-context rule.** `LifecycleEvent`s raised from IRQ/timer context
   (e.g., wake events arriving from P6) must use bounded, non-allocating
   paths; the engine's IRQ-context entry points perform table lookup plus a
   state store and nothing else. Deeper wakeup semantics (races, deferred
   events) are [P7-W06](../p7-w06-block-wakeup/README.md). Authority: Coding
   Guidelines bound on IRQ/VM-exit path work.
8. **Authority metadata, not re-verification.** Control events (`Pause`,
   `Resume`, `Stop`) carry a `TransitionContext` with the caller role and the
   capability receipt obtained upstream from the P5 rights check (hook owned
   by W07/W03 designs); the engine records it and rejects context-less
   control events. The engine does not re-implement capability checks — that
   would duplicate P5 authority in violation of ADR-013 layering.
9. **Invariants.** INV-1 (no double-run), INV-2 (Running ⇔ exactly one
   pCPU's `current_vcpu`), INV-3 (only `Running` vCPUs execute Guest code),
   INV-4 (all state changes pass the engine — no direct field writes), INV-5
   (transition/accounting monotonicity observable by W09). Checker contract
   in [lifecycle contracts](03-code-contracts-lifecycle.md) §6; runtime
   measurement belongs to W09/W11.
10. **Module and dependency direction.** Logical modules `lifecycle`
    (engine + table), `admission` (gate, exit boundary, bypass register),
    `audit` (checker). Dependency rule: `audit`/`admission`/policy (W05) may
    depend on `lifecycle`; `lifecycle` depends only on ids, error, trace, and
    P3 synchronization primitives — never on placement, runqueues, or
    policy. Physical CPU and vCPU remain separate objects; the pCPU-side
    `current_vcpu` slot lives in the per-pCPU scheduler state (P3-W04
    reserved capacity), not in the vCPU. Crate/file placement follows the
    repository's approved layout decisions at implementation time; this
    design fixes responsibilities and dependency direction only.

## Work breakdown and loading order

1. Read this README; then load supporting files per assigned step as listed
   under *Purpose and use*.
2. Implement in the order given in
   [the implementation workflow](04-implementation-workflow.md): foundations
   (types, table) → engine → gate/boundary → activation/bypass register →
   checker → integration seams → evidence.
3. Implementation findings go to
   `../p7-w02-scheduler-admission-lifecycle-record.md`; review/test evidence
   goes to
   `../../verification/p7-w02-scheduler-admission-lifecycle-verification.md`.
   Neither file is created by this design; no completion may be claimed from
   this design.

## Explicitly excluded interfaces

This design authorizes no public API, no management/wire ABI, no trace binary
encoding, no crate boundary, and no configuration format. All names in
[the lifecycle contracts](03-code-contracts-lifecycle.md) are internal,
stage-local design names: an implementation may adjust mechanical naming
(field names, module file names) provided the contract semantics, ownership,
and dependency direction are preserved; any change to the state set, edge
set, gate semantics, or invariants is a new design decision, not a
refactor. Runqueue internals, pick policy, deadline/timer mechanics,
wakeup/pause/fault flows, and idle/remote-reschedule behavior are owned by
W05–W08 respectively and are deliberately not contracted here beyond the
seam interfaces this design consumes (eligibility predicate, P4 entry/exit
mechanism, P3 lock primitives).

## Downstream handoff

Per the [P7 plan index](../../plans/README.md) consumer map:

- **W03** receives the lifecycle authority: placement changes attach only at
  configuration time on `Offline` vCPUs; `is_dispatchable` (state =
  `Runnable`) is the gate input W03's eligibility predicate feeds.
- **W04** receives: preemption descheduling must issue `Deschedule` (or a
  terminal/`Block`/`Pause` event) through the engine; `admit_for_entry` is
  the only entry path after scheduler activation; INV-1/INV-2 are frozen.
- **W05** receives: the picker dispatches only through the gate; the queue
  discipline must preserve INV-3/INV-4 and the leaf-lock rule; the policy
  layer owns everything W02 does not fix.
- **W06** receives: `Block` is legal only from `Running`; `Wake` only from
  `Blocked`; wakeup races must preserve "a `Runnable` vCPU is discoverable
  for dispatch" without new run-states (auxiliary bookkeeping must go
  through declared mechanisms, not hidden state).
- **W07** receives: `Pause` legal from `Running`/`Runnable`/`Blocked`;
  `Resume` only from `Paused`, restoring `Runnable` with eligibility and
  placement preserved; `Stop` legal from every non-terminal state; remote
  handling of a `Running` vCPU must route through the pCPU owner, never
  mutate lifecycle state cross-CPU directly.
- **W09** receives the telemetry semantics (events and reasons) from the
  contracts file; W09 owns encoding, aggregation, and presentation.
- **W11** receives INV-1…INV-5 as the stress/property targets; W02's checker
  is the reference implementation of "checkable".

W14 consumes the accepted-boundary record as part of the P8 handoff; P8 may
rely only on evidenced semantics per the task book §7.
