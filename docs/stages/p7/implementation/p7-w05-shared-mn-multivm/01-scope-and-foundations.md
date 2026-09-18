# P7-W05 Scope, Foundations, and Prerequisite Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W05 detailed design](README.md).

## 1. Foundation analysis

For the plan goal ("correct shared-CPU M:N and multi-VM progress with basic
no-starvation fairness") to be true, these concrete artifacts must exist:

1. A schedulable-entity view that exposes exactly what a policy may see
   (identity, placement, dispatchability) — without it, policy and
   mechanism blur.
2. Per-pCPU run queues with a single-owner discipline and bounded capacity
   — M:N rotation is impossible without queues, and "no duplicate
   execution" needs queue-level exclusion.
3. A policy seam with the v0 round-robin implementation — rotation must be
   real code behind a replaceable seam (ADR-017/057).
4. A deterministic enqueue-placement rule respecting W03 eligibility —
   "CPU affinity 生效" (task book exit) must hold at queue-insertion time,
   not only at gate time.
5. A control loop binding pick → gate → enter → exit → reconsider — the
   loop is where M:N actually happens; it must be one reviewable function.
6. The scenario table and fairness condition — the plan's matrices and
   "sustained nonzero execution" must be decidable definitions consumed
   identically by W10/W11/W12.
7. Prohibited-execution assertions (duplicate run, non-runnable dispatch)
   wired to the W02 checker.

Items 1–7 are this design's foundation deliverables; none exists today.

## 2. Prerequisite contracts and failure boundaries

| Prerequisite | Delivering package / plan path | What W05 assumes | Failure boundary |
|---|---|---|---|
| Lifecycle engine, gate, invariants | P7-W02 design (`../p7-w02-scheduler-admission-lifecycle/README.md`) | `admit_for_entry`, `complete_exit`, engine transitions, INV-1/3, leaf-lock rule | gate/engine semantics differ → renegotiate at design level; queue discipline would need redesign |
| Placement eligibility + ledger | P7-W03 design (`../p7-w03-placement-configuration/README.md`) | `is_eligible`, frozen placements, exclusivity | differs → enqueue rule and scenario table invalid; blocked |
| Preemption/switch contracts | P7-W04 design (`../p7-w04-preemption-context-switch/README.md`) | triggers, `switch_to`, `DescheduleReason`, slice source seam | differs → loop cannot bind; blocked |
| pCPU identity, online registry, cross-CPU notification transport | P3-W14 + P3-W07 (`../../../../stages/p3/plans/p3-w14-p4-smp-handoff.md`); P7-IN-04 | stable online set; a transport that can carry a reschedule request to a pCPU (semantics W08's) | transport absent → multi-pCPU matrices blocked at 1-pCPU scope; record |
| Multi-vCPU/multi-VM object facts | P4-W09 (`../../../../stages/p4/plans/p4-w09-closeout-p5-handoff.md`); P7-IN-05 | VM and vCPU objects exist independently; multiple vCPUs per VM addressable | differs → scenario table blocked |
| Block/wakeup protocol | P7-W06 design (`../p7-w06-block-wakeup/README.md`) | no-lost-wakeup invariant; enqueue follows runnable transition | protocol changes → queue observability windows change; joint review required |
| Trace/accounting hooks | P0 + P7-W09 design (`../p7-w09-accounting-diagnostics/README.md`) | trace marks available at loop points; queue-depth sampling acceptable | differs → observability seam review; mechanism work proceeds |

## 3. Itemized scope classification

**Required:** R1 `SchedulingEntity` view; R2 per-pCPU `RunQueue`
(single-owner, bounded, duplicate-free); R3 `SchedulingPolicy` seam +
`RoundRobin` v0; R4 `choose_enqueue_pcpu` v0 rule; R5 control loop with
reconsideration/idle hooks; R6 M:N scenario table (1/2, 2/4, 4/8; two-VM);
R7 v0 fairness condition + assertion hook; R8 prohibited-execution
assertions; R9 queue/loop trace points (semantics).

**Reserved:** proportional fairness/weights/quotas/shares; runtime policy
switching; work stealing; load balancing; NUMA; RT/partition classes
(ADR-017); migration policy beyond R4; multi-queue per pCPU; priority
classes.

**Out of Scope:** idle-loop mechanism and remote-reschedule transport
semantics (W08); wakeup event sourcing/races (W06); pause/stop flows
(W07); counter definitions/encoding (W09); guest workloads (W10); stress
execution (W11); automated matrix execution (W12); performance
conclusions (W13).

## 4. ADR-016/ADR-057 conformance statement

- **ADR-016:** the design supports dedicated (all-pinned) VMs, shared
  vCPUs, affinity masks, and mixtures on one system with one vCPU type and
  one policy seam; M:N is the normal mode, 1:1 is a special case; nothing
  in the queue/loop assumes vCPU count ≤ pCPU count.
- **ADR-057 (open, kept visible):** round-robin is the v0 policy behind
  the seam, chosen because the ADR stage tree names it as the first M:N
  scheduler and because it satisfies the v0 fairness condition trivially.
  The final default algorithm remains an open ADR-057 decision for a later
  benchmarked choice; replacing `RoundRobin` behind the seam is the
  intended extension path and must not require loop or queue changes.
- **ADR-017:** the seam is the anti-lock-in mechanism; adding a policy
  class later is a design change at the seam, not a rewrite.

## 5. Mechanism vs policy split

| Concern | Classification | Owner |
|---|---|---|
| Queue structure, discipline, capacity | Mechanism | W05 |
| Control loop shape, hook points | Mechanism | W05 |
| Pick order within a queue | Policy | `RoundRobin` v0 (W05's, replaceable) |
| Requeue position (always tail) | Policy (v0) | `RoundRobin` |
| Enqueue target choice | Policy (v0 rule recorded) | W05 `choose_enqueue_pcpu`; revisitable |
| Idle behavior | Mechanism hook; policy in W08 | split |
| Slice values | Policy | W04 contract / W05 source |
| Fairness target (what must hold) | Stage policy definition | W05 |
| Fairness enforcement beyond v0 RR (weights) | Reserved | later |

Review rule: loop/queue code that encodes anything from the Reserved rows,
or policy code that touches queue internals or the gate, is a review
failure.
