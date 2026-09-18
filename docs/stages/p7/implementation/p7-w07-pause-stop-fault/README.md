# P7-W07 Pause, Stop, and Fault Containment — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Pause/resume, VM pause completion, stop/fault scheduling exclusion,
remote-running-vCPU handling, and guest-fault containment required by
[P7-W07](../../plans/p7-w07-pause-stop-fault.md).  
**Owner/change context:** P7-W07 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W07. It converts the bounded
work-package plan into concrete mechanics: per-state pause outcomes for
Running, Runnable, and Blocked vCPUs; a VM pause completion model that never
blocks in a VM-exit path; resume semantics that preserve placement, pending
events, and pre-pause scheduling condition; stop and guest-fault transitions
that exclude a vCPU from scheduling without touching any other VM; and the
authorization ordering for every lifecycle control action. It deliberately does
**not** design snapshot/migration, VM fault policy, a management API, VM
restart semantics, or the scheduler implementation method.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| Pause/stop/fault model, state ownership, race and containment rules | [01-pause-stop-fault-architecture.md](01-pause-stop-fault-architecture.md) |
| Implement pause/resume and VM pause completion | [02-code-contracts-pause-resume.md](02-code-contracts-pause-resume.md) |
| Implement stop, guest-fault containment, and exclusion | [03-code-contracts-stop-fault.md](03-code-contracts-stop-fault.md) |
| Execute the ordered workflow | [04-implementation-workflow.md](04-implementation-workflow.md) |
| Plan or review validation and closure | [05-validation-and-handoff.md](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P7 task book](../../task-book-v0.1.md), the
[P7-W07 plan](../../plans/p7-w07-pause-stop-fault.md), and
[P7-W01](../p7-w01-entry-contract-reconciliation/README.md)'s recorded input
boundary. This document is a proposed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W07 plan → this design
→ Coding Guidelines. Binding constraints:

- ADR-013/051/052: authority is capability + rights + generation; there is no
  privileged VM ID, role, or first-VM shortcut. Every pause/resume/stop control
  action is authorized through the P5 capability model before any state
  effect ([P5-W05](../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md)).
- ADR-019 invariant: a Guest-caused fault affects only its own VM/vCPU context
  and must never become a global panic. Guest faults entering this design are
  recoverable VM-facing errors, not hypervisor failures.
- ADR-007 (untrusted Guests) and the Coding Guidelines' bounded-work rule
  shape the remote-pause and completion mechanics: no unbounded waiting in
  VM-exit or IRQ paths.
- The [P7-W02 plan](../../plans/p7-w02-scheduler-admission-lifecycle.md) owns
  lifecycle state meanings and transition legality; W07 implements specific
  transitions through that authority and invents none.
- The [P7-W03 plan](../../plans/p7-w03-placement-configuration.md) owns
  placement constraints; resume must preserve them, never recompute policy.
- The [P7-W04 plan](../../plans/p7-w04-preemption-context-switch.md) owns
  preemption; a paused Running vCPU leaves execution via the same exit/
  reconsideration machinery, not a second mechanism.

Classification:

- **Required:** pause of Running/Runnable/Blocked vCPUs with explicit outcomes;
  VM pause completion with no Guest executing; resume preserving eligibility,
  placement, and pending events; stop/fault scheduling exclusion; remote-running
  pause/stop requests; guest-fault containment; capability-authorized control
  ordering; the P7-V15/V16 evidence plan.
- **Reserved:** VM-level fault policy (what a faulted vCPU means for its VM's
  lifecycle), vCPU/VM restart after stop, snapshot/migration pause variants,
  delegation of lifecycle rights beyond P5's grant model, and any management
  API surface.
- **Out of Scope:** snapshot/migration protocol, VM fault policy, management
  API, scheduler implementation method, runqueue topology, preemption timing,
  and all P8+ mechanisms (PSCI-style CPU_ON/OFF presentation, machine ABI).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect lifecycle, placement, preemption inputs | [assumed contracts](01-pause-stop-fault-architecture.md) §2 | [workflow](04-implementation-workflow.md) step 1 review |
| Define pause completion and resume preservation | [pause/resume contracts](02-code-contracts-pause-resume.md) | P7-V15 |
| Define stop/fault exclusion and other-VM containment | [stop/fault contracts](03-code-contracts-stop-fault.md) | P7-V16 |
| Review remote execution and event interaction | [architecture](01-pause-stop-fault-architecture.md) §5–§6 | [workflow](04-implementation-workflow.md) step 4 review |
| Plan acceptance evidence and hand off the contract | [validation and handoff](05-validation-and-handoff.md) | P7-V15–V16 evidence locations; handoff checklist |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p7-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no implemented crates;
`docs/stages/p7/implementation/` holds only the stage README. P0–P6 are
planned, not implemented; everything W07 consumes is an **assumed contract**
cited by plan path, each with a failure boundary in
[01-pause-stop-fault-architecture.md](01-pause-stop-fault-architecture.md) §2.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Running/runnable/blocked pause safely (P7-V15) | Absent | Per-state pause paths through the W02 transition authority (L-1) | One pause request must have one defined outcome per lifecycle state | P7-W02 (states); P7-W07 (paths) | P7-V15 pause matrix |
| VM pause leaves no Guest executing (P7-V15) | Absent | VM-pause-pending generation + admission serialization + completion detection | Flag-setting alone cannot stop an already-running vCPU; a completion definition is required | P7-W07 (this design), using L-3 admission seam | P7-V15 VM-pause case |
| Resume preserves eligibility/placement/events (P7-V15) | Absent | Pre-pause context record + resume re-evaluation | A paused vCPU must resume no more and no less eligible than before | P7-W03 (placement C-1); P7-W06 (event record W-2 of its design); P7-W07 (preservation) | P7-V15 resume cases |
| Stopped/faulted excluded from scheduling (P7-V16) | Absent | Terminal-state exclusion enforced at admission and dequeue | Exclusion must be structural, not a convention callers may forget | P7-W02 (L-4 non-runnable exclusion); P7-W07 (enforcement) | P7-V16 |
| Guest fault does not corrupt other VM scheduling (P7-V16) | Absent | Contained `commit_guest_fault` transition consuming the P4-W06 classification | Fault reaction must mutate exactly one vCPU's scheduling state | P4-W06 (classification); P7-W07 (scheduling reaction) | P7-V16 containment |
| Remote-running-vCPU requests (plan scope) | Absent | Reconsideration-based remote completion (transport owned by P7-W08) | A remote Running vCPU must be forced out of Guest execution promptly | P3-W07/P6-W04 (Host mechanism); P7-W08 (transport); P7-W07 (semantics) | P7-V15 remote cases |
| Lifecycle control is authorized (task book §2; ADR-013) | Absent | P5 capability rights check with controlled denial classes before any effect | Unauthorized pause/stop must fail without state change | P5-W05/W06 (ordering, denials); P7-W07 (call ordering) | P7-V16 with P5 denial reuse |
| Evidence and handoff to W08–W11 | Verification directory empty (`.gitkeep` only) | Validation matrix and record paths defined here | Consumers rely only on declared, locatable evidence | P7-W07 | `../../verification/p7-w07-pause-stop-fault-verification.md` |

No row invents an upstream mechanism. If P7-W01's reconciliation (P7-V01)
marks any assumed input missing or contradictory, the affected step is blocked
and the issue is recorded; it is not repaired in W07.

## Resolved design decisions and their authority

1. **Per-state pause outcomes, one dispatch entry.** A pause request is
   authorized once, then dispatched on the vCPU's lifecycle state: Blocked
   transitions directly (no execution involved); Runnable is dequeued and
   transitions; Running is marked and forced out via reconsideration, with
   Paused/Stopped/Faulted/Offline producing explicit idempotent-success or
   controlled-denial outcomes. Rationale: P7-V15 requires the matrix to be
   safe in every state; a single entry keeps authorization and accounting
   uniform. Authority: plan scope; L-1 for transitions; stage-local dispatch
   design owned here.
2. **Pause of a Running vCPU is asynchronous, completion is defined.** The
   request records a pause-pending marker with release ordering and requests
   reconsideration of the target pCPU; the vCPU becomes `Paused` when the exit
   path confirms it no longer executes. Completion is *detected*, never
   awaited by spinning: the transition occurs on a scheduler path, and a
   control-plane caller observes completion through state, not by blocking in
   a VM-exit path. Rationale: the Coding Guidelines forbid long waits in
   VM-exit paths; ADR-016 preemption guarantees an upper bound (the running
   pCPU's next preemption/exit point). Authority: task book §1 ("pause/resume
   无竞态" spirit, bounded remote handling); Coding Guidelines; P7-W04.
3. **VM pause via pending generation + admission serialization.** A VM pause
   sets a per-VM pause-pending generation; every admission check (L-3 seam)
   refuses entry for member vCPUs of a paused generation; running members are
   individually paused per decision 2; VM pause is *complete* when, after the
   generation is set, no member vCPU is Running anywhere. The last departing
   member's exit path detects completion. Rationale: this gives P7-V15's "VM
   pause leaves no Guest executing" a checkable definition without spin-waits.
   Authority: plan scope item 2; stage-local mechanism owned here.
4. **Resume re-evaluates the pre-pause scheduling condition.** Resume restores
   `Runnable` — or `Blocked` when the vCPU was Blocked with no eligible event
   at pause time — preserving placement eligibility and the pending-event
   record. Resuming straight to `Runnable` in the blocked case would force a
   futile WFI round-trip; preserving `Blocked` keeps P7-V15's eligibility
   preservation exact. Requires `Paused→Blocked` in the W02 transition set
   (L-1 requirement; failure boundary in §2 of the architecture file).
   Authority: plan scope item 2; stage-local design freedom owned here within
   W02's transition authority.
5. **Stop and Faulted are terminal in P7.** No resume, restart, or re-admission
   exists in P7; object destruction remains with the P4/P5 lifecycle owners.
   Pending events on a Stopped/Faulted vCPU are retained only for diagnostic
   inspection. Rationale: plan excludes VM fault policy and restart semantics;
   half-open terminal states would create invalid-wake ambiguity for W06.
   Authority: plan out-of-scope list; stage-local design freedom owned here.
6. **Guest-fault reaction is unauthorized-but-automatic.** `Paused`/`Stopped`
   control transitions require P5 capability authorization; the
   `Running→Faulted` transition does **not** — it is the mandatory scheduling
   reaction to a P4-W06-classified GuestFault and must never be deniable by a
   capability. Rationale: ADR-019 containment is an invariant, not a service.
   Authority: P4-W06 handoff; ADR-019 invariant; stage-local classification.
7. **Containment is structural.** A guest-fault commit mutates only the
   faulted vCPU's scheduling state and its own accounting/diagnostic records.
   Other vCPUs, runqueues, and VMs are untouched; the faulted entity is
   excluded at the same structural admission filter that enforces all
   non-runnable states (L-4). Rationale: P7-V16 requires non-corruption of
   other VM scheduling; convention-based exclusion is un-auditable.
   Authority: ADR-019 invariant; P7-W02 L-4; stage-local design freedom.
8. **Remote requests reuse the W06/W08 reconsideration seam.** Remote
   pause/stop of a Running vCPU records the request and issues the same
   target-pCPU reconsideration request that wakeup and preemption use; W08
   owns the transport over the P3-W07/P6-W04 Host mechanism. W07 fixes only
   the required postcondition (target ceases Guest execution within bounded
   time). Authority: plan scope item 4;
   [P7-W06](../p7-w06-block-wakeup/README.md) seam S-3;
   [P7-W08](../p7-w08-smp-reschedule-idle/README.md).

## Work breakdown and loading order

1. Read [01-pause-stop-fault-architecture.md](01-pause-stop-fault-architecture.md)
   for the module boundary, assumed contracts with failure boundaries, the
   pause/stop/fault decision tables, and the containment model.
2. Implement pause/resume and VM pause completion per
   [02-code-contracts-pause-resume.md](02-code-contracts-pause-resume.md).
3. Implement stop, guest-fault containment, and exclusion per
   [03-code-contracts-stop-fault.md](03-code-contracts-stop-fault.md).
4. Follow [04-implementation-workflow.md](04-implementation-workflow.md).
5. Record planned and actual evidence per
   [05-validation-and-handoff.md](05-validation-and-handoff.md). Implementation
   notes go to `../p7-w07-pause-stop-fault-record.md` (created when work
   starts); evidence to `../../verification/p7-w07-pause-stop-fault-verification.md`.
   Neither this design nor the record may claim W07 complete.

## Explicitly excluded interfaces

No management API, capability encoding, snapshot/migration behavior, VM fault
policy, restart semantics, runqueue type, scheduling policy, public API, ABI,
or wire format is designed or authorized by W07. The P5 capability check is
consumed through its assumed seam, never re-implemented. Adding any excluded
surface is a scope conflict and is stopped at review. Cross-package seams this
design relies on or hands on are inventoried in
[01-pause-stop-fault-architecture.md](01-pause-stop-fault-architecture.md) §2
and [05-validation-and-handoff.md](05-validation-and-handoff.md) §3.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md), W08–W11 consume this package:

- **[P7-W08](../p7-w08-smp-reschedule-idle/README.md)** receives the
  remote-pause/stop postcondition (target pCPU ceases Guest execution in
  bounded time), the VM-pause completion-detection duty of its exit path, and
  the requirement that idle pCPUs honor control-driven reconsideration.
- **[P7-W09](../p7-w09-accounting-diagnostics/README.md)** receives the
  control/fault hook points (pause/resume/stop/fault with cause and authority
  outcome) and the `SwitchReason` vocabulary entries for control-driven
  stops.
- **[P7-W10](../p7-w10-validation-guest-suite/README.md)** receives the
  behavioral contract its pause/stop scenarios exercise, including the
  guest-observable consequence of a faulted vCPU (no further scheduling) and
  the boundary that a Guest cannot pause or stop another VM's vCPUs.
- **[P7-W11](../p7-w11-stress-invariants/README.md)** receives the pause/
  resume/stop/fault race classes and containment invariants its stress
  evidence (P7-V25/V26) amplifies.

Snapshot and fault policy remain later stages; nothing here constrains them
beyond the terminal-state and containment decisions recorded above.
