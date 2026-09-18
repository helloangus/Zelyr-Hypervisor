# P7-W05 Shared M:N and Multi-VM Scheduling — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Correct shared-CPU M:N operation, multi-VM progress, and basic
no-starvation fairness as required by
[P7-W05](../../plans/p7-w05-shared-mn-multivm.md).  
**Owner/change context:** P7-W05 implementation handoff; shared-scheduler
semantics authority for W08/W10/W11 per the plan handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W05. The plan fixes *what*
must hold (the 1/2, 2/4, 4/8 M:N matrices make progress; two VMs preserve
opportunities under competing workloads; every equal-class continuously
runnable vCPU receives sustained nonzero execution; duplicate and
non-runnable execution are prohibited) and defers the scheduler algorithm
realization, run-queue discipline, and dispatch-loop shape to this design.
It defines the per-pCPU run-queue ownership model, the round-robin v0
policy behind a minimal policy seam, the control loop that ties together
the W02 gate, W04 preemption/switch, and W03 eligibility, and the scenario/
fairness contract that W08/W10/W11 consume.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then
loads only the supporting file for its assigned step:

- [Scope and foundations](01-scope-and-foundations.md): ledger, prerequisite
  contracts with failure boundaries, itemized scope, and the explicit
  mechanism/policy split (which decisions this design owns vs. leaves to
  ADR-057 evolution).
- [Architecture and state](02-architecture-and-state.md): logical modules,
  the per-pCPU scheduler object model, run-queue ownership, concurrency and
  lock discipline, the M:N scenario definitions, and the fairness
  condition.
- [Shared-scheduler code contracts](03-code-contracts-shared-scheduler.md):
  full contracts with pseudocode — `SchedulingEntity`, `RunQueue`, policy
  seam (`RoundRobin` v0), `choose_enqueue_pcpu`, the control loop, and the
  scenario/invariant assertion hooks.
- [Implementation workflow](04-implementation-workflow.md): ordered steps.
- [Validation and handoff](05-validation-and-handoff.md): validation matrix
  (P7-V10–V12), error/security/observability model, handoff checklist.

Before editing, the agent must follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P7 task book,
W01 register, the P7-W02/W03/W04 designs, and the P7-W05 plan). This design
proposes only.

## Authority, constraints, and scope classification

Governing order: ADR baseline → P7 task book → frozen P0–P6 contracts →
P7-W05 plan → this design. Binding constraints:

- **ADR-016/§5:** vCPU is the only schedulable entity; VM never enters the
  scheduler; M:N overcommit with affinity/pinning preserved; evolution
  path open to weighted/fair/RT/partition later.
- **ADR-017:** the scheduler API must not lock in a single policy — hence
  the minimal policy seam (decision 2).
- **ADR-057 (open):** the final default algorithm is deliberately
  undecided pending benchmarks; the ADR stage tree names round-robin as
  the first M:N scheduler, which this design fixes as v0 while keeping
  ADR-057 visible.
- **Task book:** M>N stable operation; pause/resume race-free; affinity
  effective; "prohibited duplicate/non-runnable execution" (plan wording)
  — realized as the W02 invariants plus queue discipline here.
- **W02/W03/W04 authorities:** dispatch goes through `admit_for_entry`;
  eligibility through W03's `is_eligible`; deschedules/switches through
  W04's contracts; all lifecycle changes through the W02 engine.

Classification:

- **Required:** `SchedulingEntity` view; per-pCPU `RunQueue` with
  ownership discipline and bounded capacity; the minimal
  `SchedulingPolicy` seam with the `RoundRobin` v0 implementation;
  `choose_enqueue_pcpu` v0 rule; the per-pCPU control loop with the
  reconsideration hook; the M:N scenario boundary (1/2, 2/4, 4/8, two-VM)
  as consumer-facing definitions; the v0 fairness condition (sustained
  nonzero execution within a bounded window); prohibited-execution
  assertions.
- **Reserved:** proportional fairness, weights, quotas, shares, policy
  switching at runtime, advanced balancing, work stealing, NUMA, RT
  classes (ADR-017), migration policy beyond the v0 enqueue rule.
- **Out of Scope:** remote-reschedule transport and idle-loop design
  ([P7-W08](../p7-w08-smp-reschedule-idle/README.md) — W05 defines the
  hook points and the enqueue rule only); wakeup event sourcing and races
  ([P7-W06](../p7-w06-block-wakeup/README.md)); pause/stop flows
  ([P7-W07](../p7-w07-pause-stop-fault/README.md)); counters/trace
  encoding ([P7-W09](../p7-w09-accounting-diagnostics/README.md)); guest
  workloads (W10); stress campaigns (W11); performance conclusions (W13).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect placement and preemption contracts | [workflow](04-implementation-workflow.md) step 1 | W05-DV01 review |
| Required M:N and multi-VM scenario boundary | [architecture and state](02-architecture-and-state.md) §5; [contracts](03-code-contracts-shared-scheduler.md) §5 | P7-V10/V11 |
| Progress/fairness conditions; prohibited execution | [architecture and state](02-architecture-and-state.md) §6; [contracts](03-code-contracts-shared-scheduler.md) §4/§5 | P7-V10–V12 |
| Review against ADR-016 and ADR-057 | [scope and foundations](01-scope-and-foundations.md) §4 | W05-DV02 review |
| Matrix and fairness evidence planning; handoff to W08/W10/W11 | [workflow](04-implementation-workflow.md) steps 6–7; [validation and handoff](05-validation-and-handoff.md) §3 | P7-V10–V12 evidence |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p7-implementation-designs`):
P0 documentation scaffold; no code. The consumed sibling designs (W02
lifecycle, W03 placement, W04 preemption/switch) are proposed designs; the
P3 notification transport and P4 multi-vCPU facts are plan-level assumed
contracts (W01 register P7-IN-04/05). Every runtime-binding input below is
cited by path with a failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| "Establish correct shared-CPU M:N … progress" (plan Goal) | No run queue, no loop, no policy exists | Per-pCPU `RunQueue` with ownership discipline; control loop binding gate/pick/switch; v0 RR policy | M:N progress is a property of a queue+loop mechanism, not of tests alone | P7-W05 (this design) | W05-DV03/04; P7-V10 |
| "1/2, 2/4, 4/8 matrices" (plan Scope) | No scenario definitions exist | Consumer-facing scenario table (topology, workload class, pass condition) used by W10/W11/W12 | Scenarios must be defined once, identically, for all evidence producers | P7-W05 defines; W10/W11/W12 run | W05-DV05 review; P7-V10 |
| "two-VM competition … preserve opportunities" (plan Scope) | No multi-VM semantics exist | Multi-VM rule set: VM non-schedulable; per-vCPU competition only; opportunity preservation condition | Multi-VM fairness needs a defined competition unit (vCPU, not VM) | P7-W05 | W05-DV05; P7-V11 |
| "basic no-starvation fairness" (plan Scope) | No fairness condition exists | v0 fairness condition with a bounded observation window derived from slice and eligible-set size | "Sustained nonzero execution" must be decidable in a test | P7-W05 (condition); W05/W11 measure | W05-DV05; P7-V12 |
| "prohibited duplicate/non-runnable execution" (plan Scope) | W02 invariants exist as design; queue-side discipline missing | Queue discipline (no duplicate enqueue; pop-then-verify) mapping onto INV-1/INV-3 | Prohibition must hold at queue level, not only at gate level | P7-W05 discipline; W02 invariants | W05-DV03/04 |

No row invents a crate, target, or runtime policy beyond this design's
recorded decisions.

## Resolved design decisions and their authority

1. **v0 algorithm: round-robin over per-pCPU FIFO run queues.** Pop-front
   at pick; push-tail at every deschedule (regardless of remaining slice).
   Rationale: the ADR stage tree names RR as the first M:N scheduler; it
   trivially satisfies the v0 fairness condition (continuous rotation gives
   every eligible runnable vCPU a turn within `N × slice`); ADR-057 keeps
   the final default open. Requeue-at-tail-even-with-slice-left is the
   simplest rule that cannot starve; "continue current with remaining
   slice" is a policy refinement deliberately deferred (Reserved).
2. **Per-pCPU queue ownership (guardrail).** Each online pCPU's scheduler
   owns exactly one run queue; no other pCPU mutates it except through the
   owner-side enqueue discipline (a cross-CPU wake enqueues via the
   owner-side `enqueue` operation; the remote kick is W08's). Queue
   internals are never accessed by W02/W04; the switch orchestrates via
   W05 operations. This realizes the guardrail "per-pCPU run-queue
   ownership" mechanically.
3. **Minimal policy seam, not a framework.** `SchedulingPolicy` exposes
   exactly `pick_next`, `on_dispatch`, `on_deschedule` over one pCPU's
   queue (contracts §3.2). Rationale: ADR-017 requires that the API not
   lock in one policy, but a rich policy framework would be speculative
   architecture (Plan guide: no unapproved abstractions). Later classes
   (weighted/RT/partition) extend the seam by design change, not by v0
   speculation.
4. **Enqueue eligibility and placement v0 rule.** A vCPU is enqueued only
   on a pCPU where `is_eligible` (W03) holds; `choose_enqueue_pcpu` (v0):
   prefer the pCPU where the event occurred if eligible, else the
   lowest-id eligible online pCPU in the placement set, else do not enqueue
   (and rely on W08's wake-on-eligibility hook). Rationale: deterministic,
   testable, affinity-respecting; the "else" branch is the explicit
   handoff to W08's idle/wake responsibility, not a lost vCPU.
5. **Queue discipline maps to W02 invariants.** Enqueue accepts only
   `Runnable` entities and rejects duplicates (an entity is in at most one
   queue); pick pops then re-verifies dispatchability (state may have
   changed between enqueue and pop); a failed verification re-enqueues or
   skips per state. This closes the queue-side of INV-1/INV-3; the
   gate-side is W02's. The transition-then-enqueue ordering (W02 §5)
   remains: cross-CPU observability windows between the two steps are the
   wakeup-race surface owned by W06 under the frozen no-lost-wakeup
   invariant (P7-V14).
6. **Bounded queue capacity.** Capacity = the configured vCPU population
   (a stage-configuration constant); enqueue overflow is a
   `ResourceExhausted`-class configuration error (it can only occur on a
   configuration bug: more runnable entities than vCPUs cannot exist).
   Rationale: bounded memory, no heap on the hot path, trivially auditable.
7. **Idle is a hook, not a policy, in W05.** When pick yields nothing, the
   loop calls the idle hook owned by W08 and re-checks on its return
   criterion. W05 defines only the hook contract; busy-loop idle is
   prohibited (task book P7-V18) — enforced by W08's design and validated
   there.
8. **Fairness condition (v0).** In any observation window of length
   `W = (N_eligible + 1) × slice_default + handling_slack` (constants
   declared, `handling_slack` recorded at implementation), every vCPU that
   is continuously `Runnable` and eligible on the measured pCPU set
   accumulates strictly positive scheduled time. This is the decidable
   form of P7-V12 "sustained nonzero execution"; proportional fairness is
   explicitly not claimed (task book). Measurement method and tolerance
   recording are W11/W13 material; W05 owns the condition's definition and
   its assertion hook.

## Work breakdown and loading order

1. Read this README; load supporting files per assigned step as listed
   above.
2. Implement in [the workflow](04-implementation-workflow.md) order:
   entity/queue → policy seam → enqueue rule → control loop → scenario
   hooks → integration → evidence.
3. Findings go to `../p7-w05-shared-mn-multivm-record.md`; evidence to
   `../../verification/p7-w05-shared-mn-multivm-verification.md`. Neither
   is created by this design; nothing here claims completion.

## Explicitly excluded interfaces

No public API, ABI, trace encoding, crate boundary, or configuration
format is designed or authorized. All names in
[the shared-scheduler contracts](03-code-contracts-shared-scheduler.md)
are internal, stage-local design names. Work stealing, balancing, weight
arithmetic, RT classes, and runtime policy switching are Reserved and must
not appear. Idle behavior, remote transport, wakeup protocols, pause/stop
flows, counters, encodings, and workloads belong to W06–W10 designs. A
need to consult another pCPU's queue internals (as opposed to its
published operations) is a design violation of decision 2.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md) consumer map:

- **W08** receives: the idle hook contract, the `RescheduleRequest`
  surface point on the target pCPU, and the enqueue-else rule's
  "wake-on-eligibility" obligation (a vCPU that could not be enqueued for
  lack of an eligible online pCPU must be retried when one becomes
  available — W08 owns that mechanism).
- **W10** receives the M:N scenario table as the workload-coverage target.
- **W11** receives the fairness condition, the prohibited-execution
  assertions, and the scenario table as stress targets; W12 inherits both
  for the automated matrix.
- **W09** receives the queue/loop trace points and accounting hooks
  (dispatch/deschedule marks with reasons — W04's taxonomy — plus queue
  depth samples for its runqueue-depth telemetry).

W14 carries the shared-scheduler semantics record into the P8 handoff; P8
(scheduler-Linux integration, P8-W11) may rely only on evidenced semantics
per the task book §7.
