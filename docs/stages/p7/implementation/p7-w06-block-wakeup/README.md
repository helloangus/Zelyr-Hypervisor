# P7-W06 Blocking and Event Wakeup — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Scheduler-visible WFI/WFE blocking, eligible-event wakeup, and
lost-wakeup/duplicate-running prevention required by
[P7-W06](../../plans/p7-w06-block-wakeup.md).  
**Owner/change context:** P7-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W06. It converts the bounded
work-package plan into concrete blocking and wakeup mechanics: the conditions
under which a vCPU exit makes it `Blocked`, the per-vCPU wake-event model with
enumerated wakeup sources, the two-phase block/wake protocol that prevents lost
wakeups, and the exclusion rules that stop ineligible vCPUs from being woken
into execution. It deliberately does **not** design P6 event-delivery
internals, generic wait-queue structures, a Notification IPC object, runqueue
topology, device models, or any scheduling policy; those remain with P6,
P7-W05, and later stages.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| Understand the block/wake model, state ownership, and race classes | [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) |
| Implement the blocking path (WFI/WFE exit → `Blocked`) | [02-code-contracts-block-path.md](02-code-contracts-block-path.md) |
| Implement the wakeup path (event → `Runnable`) | [03-code-contracts-wakeup-path.md](03-code-contracts-wakeup-path.md) |
| Execute the ordered workflow | [04-implementation-workflow.md](04-implementation-workflow.md) |
| Plan or review validation and closure | [05-validation-and-handoff.md](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index, [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P7 task book](../../task-book-v0.1.md), the [P7-W06 plan](../../plans/p7-w06-block-wakeup.md),
and [P7-W01](../p7-w01-entry-contract-reconciliation/README.md)'s recorded input
boundary. This document is a proposed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W06 plan → this design
→ Coding Guidelines. Binding constraints:

- ADR-016 (static pinned evolving to preemptive M:N) and the task-book goal
  ("a blocked Guest releases its pCPU") require that blocking is
  scheduler-visible: the hypervisor traps the blocking exit and returns the
  pCPU capacity to the scheduler. ADR-017 reserves RT scheduling and must not
  be pre-empted by blocking mechanics.
- ADR-007 makes Guest workloads and event timing untrusted: a Guest cannot
  cause a lost wakeup, a duplicate run, or an invalid wake by its own exit or
  timer behavior.
- ADR-048 makes structured telemetry first-class; every blocking and wakeup
  event is accounted (hooks for [P7-W09](../p7-w09-accounting-diagnostics/README.md)).
- The [P7-W02 plan](../../plans/p7-w02-scheduler-admission-lifecycle.md) owns
  lifecycle semantics and transition legality; this design consumes that
  authority and must not redefine state meanings.
- The P6 timer and vIRQ contracts
  ([P6-W05](../../../p6/plans/p6-w05-el2-generic-timer.md),
  [P6-W06](../../../p6/plans/p6-w06-guest-generic-timer.md),
  [P7-W06 input from P6-W07](../../../p6/plans/p6-w07-virtual-interrupt-core.md))
  supply event mechanisms; this design consumes them and supplies only the
  scheduler-side wakeup reaction ("scheduler wakeups" are explicitly outside
  P6-W07).

Classification:

- **Required:** scheduler-visible block on WFI/WFE-style exits; blocked
  eligibility definition; timer, vIRQ, Notification-classified, and internal
  event wakeup; before/during/after-block race safety; invalid-state wakeup
  exclusion; pCPU-capacity release; block/wake accounting hooks; the P7-V13 and
  P7-V14 evidence plan.
- **Reserved:** the Notification IPC *object* and its user-visible semantics
  (ADR-034, later stages) — only the wakeup seam classified for a future
  Notification source exists here; guest-visible paravirtual wait/wake
  channels; any cross-vCPU wait/notify primitive between Guests.
- **Out of Scope:** P6 event-delivery mechanics (vIRQ pending queues, List
  Registers, timer programming), generic wait-queue data structures, wakeup
  APIs exposed to Guests or management domains, device-model design, runqueue
  topology and fairness policy (P7-W05), remote-reschedule transport internals
  (P7-W08), preemption time-slice values (P7-W04), and any public/management
  ABI.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W02 lifecycle and P6 event contracts | [assumed contracts](01-block-wakeup-architecture.md) §2 | [workflow](04-implementation-workflow.md) step 1 review |
| Block release of pCPU capacity (WFI/WFE) | [block-path contracts](02-code-contracts-block-path.md) | P7-V13 |
| Blocked eligibility and event-to-runnable behavior | [architecture](01-block-wakeup-architecture.md) §3–§4, [wakeup contracts](03-code-contracts-wakeup-path.md) | P7-V13, P7-V14 |
| Timer, vIRQ, Notification, internal-event wakeup | [wakeup contracts](03-code-contracts-wakeup-path.md) §2–§5 | P7-V14 |
| Before/during/after-block race acceptance | [architecture](01-block-wakeup-architecture.md) §5, protocol contracts §2/§4 | P7-V14; race-density stress is [P7-W11](../p7-w11-stress-invariants/README.md) |
| Invalid-state wakeup exclusion | [wakeup contracts](03-code-contracts-wakeup-path.md) §6 | P7-V14 |
| Cross-CPU and pause interaction requirements (review only) | [architecture](01-block-wakeup-architecture.md) §6 | [workflow](04-implementation-workflow.md) step 4 review |
| Block/wakeup evidence and handoff to W08–W11 | [validation and handoff](05-validation-and-handoff.md) | P7-V13–V14 evidence locations; handoff checklist |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p7-implementation-designs`): the repository is a P0 documentation
scaffold. `git ls-files` shows no Cargo workspace, no Rust sources, no
implemented crates, and `docs/stages/p7/implementation/` contains only the
stage README. P0–P6 packages are planned, not implemented. Everything P7-W06
consumes is therefore an **assumed contract** with an explicit failure
boundary, cited by plan path; no contract below is evidenced in this tree.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| No-event WFI/WFE releases pCPU (P7-V13) | No code exists; P4 plans WFI/WFE exit handling ([P4-W04](../../../p4/plans/p4-w04-vcpu-entry-exit.md)) | Assumed P4 exit classification delivering WFI/WFE as distinct exit conditions; block path defined here | Without classified blocking exits there is nothing for the scheduler to observe | P4 (classification); P7-W06 (block mechanics) | P7-V13 block evidence |
| Preexisting event does not strand vCPU (P7-V13) | Absent | Pre-block eligibility poll consuming P6 pending-event facts (assumed P6-W06/W07 contracts) | Blocking with a pending eligible event must abort before capacity release | P6 (event facts); P7-W06 (poll) | P7-V13 |
| Applicable events eventually wake eligible vCPUs (P7-V14) | Absent | Wake-event post/consume protocol with enumerated sources | Wakeup must have one entry point all sources use, or sources diverge | P7-W06 (this design) | P7-V14 wake evidence |
| No lost wakeup (P7-V14) | Absent | Two-phase block/wake ordering protocol | A wake between eligibility poll and state change is otherwise lost | P7-W06; ordering rules per [P3-W06](../../../p3/plans/p3-w06-concurrency-synchronization.md) | P7-V14; P7-V25 race stress (W11) |
| No duplicate running (P7-V14) | Absent | W02 single-running invariant (assumed contract L-1/L-2) + wake transitions only through it | A wake must never create a second concurrent execution of one vCPU | P7-W02; P7-W06 consumes | P7-V14 with P7-V04 invariant checks |
| Invalid-state wakeup excluded (P7-V14) | Absent | Wake-exclusion rules keyed on W02 lifecycle states | Paused/Stopped/Faulted vCPUs must gain pending events, not eligibility | P7-W02 (states); P7-W06 (rules) | P7-V14 invalid-wake cases |
| Blocked Guest timer still wakes (task book §2 wakeup sources) | Absent | Deadline-home rule folding vCPU deadlines into the last-owning pCPU's host deadline (P6-W05 per-pCPU ownership) | A blocked vCPU's deadline must remain observable by some online pCPU | P6-W05 (timer ownership); P7-W06 (fold rule) | P7-V14 timer-wake case |
| Evidence and handoff to W08–W11 | Verification directory empty (`.gitkeep` only) | Validation matrix and record paths defined here | Consumers may rely only on declared, locatable evidence | P7-W06 | `../../verification/p7-w06-block-wakeup-verification.md` |

No row requires inventing an upstream mechanism: each prerequisite is an
assumed contract owned by its plan, and each has a failure boundary in
[01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §2. If W01's
reconciliation (P7-V01) records any prerequisite as missing or contradictory,
the affected step is blocked and an Architecture Change Request is raised; it
is not repaired here.

## Resolved design decisions and their authority

1. **Trap-and-schedule blocking.** A Guest WFI or WFE that reaches EL2 as a
   blocking-class exit makes the vCPU `Blocked`; the pCPU immediately runs the
   scheduler decision instead of re-entering the same Guest. Rationale: the
   task book requires "scheduler-visible WFI/WFE blocking" and a blocked Guest
   that releases its pCPU; P4 already delivers classified WFI/WFE exits.
   Authority: task book §1/§2; P4-W04 handoff.
2. **One blocking state, two exit hints.** WFI and WFE both produce the single
   `Blocked` lifecycle state; the exit distinction is retained only as a
   bounded trace/accounting hint. Rationale: W02 owns lifecycle states; adding
   scheduler-internal wait classes would duplicate lifecycle authority.
   Authority: P7-W02 plan scope; stage-local design freedom owned here for the
   hint.
3. **Coalescing wake-event set with four enumerated sources.** Each vCPU
   carries a pending-event record with sources `TimerExpiry`, `VirtualIrq`,
   `Notification`, `Internal`. Multiple arrivals of one source coalesce;
   distinct sources are independently visible until consumed. Rationale: P7-V14
   names exactly these sources; coalescing bounds IRQ-context work.
   Authority: task book §2 Required; stage-local naming owned here.
4. **Two-phase block/wake protocol.** The blocker sets a block intent, re-checks
   pending events, and aborts the block if any are eligible; the waker records
   the event with release ordering, then checks the intent/state and performs
   the eligibility transition. Exact memory-order spellings follow P3-W06's
   handed-down rules. Rationale: this is the minimal protocol that closes the
   before/during/after race window. Authority: plan scope item 3;
   [P3-W06](../../../p3/plans/p3-w06-concurrency-synchronization.md) for
   synchronization semantics.
5. **Deadline home for blocked vCPUs.** A vCPU's Guest-timer deadline stays
   folded into the host per-pCPU deadline of the pCPU that last ran it ("home
   pCPU"); a deadline IRQ there posts `TimerExpiry` for the vCPU and lets
   placement decide where it next runs. Rationale: P6-W05 fixes per-pCPU timer
   ownership, so no cross-pCPU timer mutation may be introduced; the home-pCPU
   fold preserves P6 authority while guaranteeing eventual wake.
   Authority: [P6-W05](../../../p6/plans/p6-w05-el2-generic-timer.md) handoff;
   fold rule is stage-local design freedom owned here.
6. **Notification seam only.** The wakeup entry accepts a
   `Notification`-classified source so the ADR-034 Notification object can
   later call it unchanged; the object itself, its IPC semantics, and any
   user-facing wakeup API are Reserved for later stages. Authority: ADR-034;
   task book §2 lists Notification as a required wakeup source.
7. **Wakeup is eligibility-only; placement is not chosen here.** A successful
   wake transitions `Blocked → Runnable` through the W02 transition authority
   and inserts the entity through the P7-W05 enqueue seam; selecting the target
   pCPU and issuing remote reconsideration belong to placement and
   [P7-W08](../p7-w08-smp-reschedule-idle/README.md). Rationale: mechanism
   (wake) and policy (where to run) stay separable per the task book and
   ADR-016/057. Authority: plan out-of-scope list; P7-W03/W05/W08 plans.
8. **Event consumption at defined boundaries.** Pending events are consumed at
   block-entry re-check and at Guest re-entry eligibility evaluation, never
   opportunistically inside unrelated code. Rationale: consumption points are
   the only places where "event seen by the vCPU" can be defined without
   races. Authority: stage-local design freedom owned here, required for
   P7-V14 determinism.

## Work breakdown and loading order

1. Read [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) for
   the module boundary, assumed upstream contracts with their failure
   boundaries, the blocked-eligibility model, and the race classes.
2. Implement the blocking path per
   [02-code-contracts-block-path.md](02-code-contracts-block-path.md).
3. Implement the wakeup path per
   [03-code-contracts-wakeup-path.md](03-code-contracts-wakeup-path.md),
   including the two-phase protocol and exclusion rules.
4. Follow [04-implementation-workflow.md](04-implementation-workflow.md) for
   step order, acceptance conditions, and blocker handling.
5. Record planned and actual evidence per
   [05-validation-and-handoff.md](05-validation-and-handoff.md). Implementation
   notes go to `../p7-w06-block-wakeup-record.md` (created when work starts);
   evidence to `../../verification/p7-w06-block-wakeup-verification.md`.
   Neither this design nor the record may claim W06 complete.

## Explicitly excluded interfaces

No P6 event-delivery mechanism, generic wait-queue type, Guest-visible wait or
wake API, management API, runqueue type, scheduling policy, public API, ABI, or
wire format is designed or authorized by W06. The `Notification` IPC object,
wait-queue structures, and wakeup APIs named in the plan's out-of-scope list
are excluded; adding any of them is a scope conflict and is stopped at review.
The only cross-package seams this design relies on are the assumed contracts
listed in [01-block-wakeup-architecture.md](01-block-wakeup-architecture.md) §2
and the seams it hands to W05/W08/W09 in
[05-validation-and-handoff.md](05-validation-and-handoff.md).

## Downstream handoff

Per the [P7 plan index](../../plans/README.md), W08–W11 consume this package:

- **[P7-W08](../p7-w08-smp-reschedule-idle/README.md)** receives the wake-path
  requirement that an eligible pCPU must be caused to reconsider after a wake,
  the deadline-home/idle interlock, and the requirement that idle never
  busy-waits. W08 owns the local/remote reconsideration transport over the
  P3-W07 notification primitive.
- **[P7-W09](../p7-w09-accounting-diagnostics/README.md)** receives the block,
  wake, and wake-source hook points that its counters and trace events attach
  to, plus the exit-hint vocabulary (WFI vs WFE).
- **[P7-W10](../p7-w10-validation-guest-suite/README.md)** receives the
  behavioral contract its WFI/wakeup scenarios exercise: blocking releases the
  pCPU, each enumerated source wakes, and no stranded or invalid wakes occur.
- **[P7-W11](../p7-w11-stress-invariants/README.md)** receives the race classes
  (before/during/after-block, concurrent sources, invalid-state wakes) that its
  race stress (P7-V25) amplifies.

Event-delivery mechanics remain P6 property; W08–W11 may rely on the behavioral
contract only, not on any P6 redesign.
