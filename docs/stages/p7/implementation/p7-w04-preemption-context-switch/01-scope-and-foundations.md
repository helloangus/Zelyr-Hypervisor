# P7-W04 Scope, Foundations, and Prerequisite Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W04 detailed design](README.md).

## 1. Foundation analysis

For the plan goal ("CPU-bound Guest preemption and vCPU-switch isolation
objectively verifiable") to be true, these concrete artifacts must exist:

1. A deadline discipline on the P6 per-pCPU timer: arm at dispatch, cancel
   or re-arm at switch, expiry recorded — without it, "the deadline returns
   a non-exiting Guest to EL2" (P7-V08) has no mechanism.
2. A slice policy source with a validated, recorded default — a testable
   slice needs a defined value source and expiry rule.
3. A bounded deadline-IRQ handler plus a pCPU-local reschedule intent —
   separating IRQ bookkeeping from switching is what makes both verifiable.
4. An enumerated preemption-point/reconsideration-trigger set — the loop's
   reconsideration behavior must be reviewable against a closed trigger
   list.
5. A switch sequence where each state class has exactly one owner and one
   transfer point — P7-V09's A→B→C→A isolation is provable only against an
   ownership table.
6. Switch instrumentation carrying reasons and per-class isolation evidence
   hooks for W09/W10.

Items 1–6 are this design's foundation deliverables; none exists today.

## 2. Prerequisite contracts and failure boundaries

| Prerequisite | Delivering package / plan path | What W04 assumes | Failure boundary |
|---|---|---|---|
| Per-pCPU deadline timer (arm/cancel/rearm, monotonic time) | P6-W05 via P6-W13 (`../../../../stages/p6/plans/p6-w13-telemetry-regression-handoff.md`); P7-IN-07 | an armable per-pCPU one-shot deadline with IRQ delivery and a monotonic time source | no armable deadline → preemption as specified is impossible; W04 blocked; a tick-only P6 would be a contract mismatch → ACR |
| vCPU virtual timer save/restore | P6-W05/W06 via P6-W13; P7-IN-07 | per-vCPU virtual-timer state survives arbitrary deschedule/switch | differs → switch isolation (P7-V09) blocked for the timer class; ACR |
| vIRQ pending/event preservation; LR/maintenance ownership | P6-W07/W08/W10 via P6-W13; P7-IN-08 | pending vIRQ state is per-vCPU and preserved across switches; EL2 never loses events by descheduling | differs → isolation blocked for the vIRQ/event class; ACR |
| Guest entry/return mechanism; arch context save at exit; Stage-2/VMID activation | P4-W04 via P4-W09 (`../../../../stages/p4/plans/p4-w09-closeout-p5-handoff.md`); P7-IN-05 | an entry call per dispatch; context already saved when EL2 regains control; an address-space activation call per switch | differs → switch sequence cannot bind; W04 blocked; do not redesign P4 |
| Lifecycle engine, gate, exit boundary | P7-W02 design (`../p7-w02-scheduler-admission-lifecycle/README.md`) | transitions via engine; entry via gate; return via `complete_exit`; `Deschedule` event | differs → renegotiate at design level; never bypass the engine |
| Slice policy source | P7-W05 design (`../p7-w05-shared-mn-multivm/README.md`) | a per-entity slice query in the policy layer | absent → W04 uses the recorded default constant for all entities and records the deviation |
| Trace namespace and event budget | P0 (P7-IN-01) | switch/deadline events fit the namespace and IRQ-safe emission rules | differs → W09/W04 seam review; no out-of-namespace events |

## 3. Itemized scope classification

**Required:** R1 `TimeSlice` type + validation + recorded default rule; R2
slice policy source seam (W05 hook, W04 contract); R3 deadline arm/cancel/
re-arm discipline per dispatch/switch; R4 bounded deadline-IRQ handler +
reschedule intent; R5 preemption-point/trigger enumeration (`DeadlineExpired`,
`GuestExit`, `RescheduleRequest`); R6 reconsideration trigger contract for
the control loop; R7 switch sequence with per-step owner + failure
boundaries; R8 isolation-classes list; R9 `DescheduleReason` semantics; R10
switch/deadline trace points (semantics).

**Reserved:** lazy FP/SIMD (ADR §5); preemption of EL2-internal work;
migration/balancing policy; weighted or RT-aware preemption (ADR-017);
slice handout schemes (quotas).

**Out of Scope:** timer register programming/time representation (P6);
slice value as a KPI; periodic tick alternatives (decided against — README
decision 1); register-save layout/assembly (P4); queue/pick policy (W05);
block/wake sourcing (W06); pause/stop flows (W07); remote transport (W08/
P3); counter definitions/encoding (W09); guest workloads (W10).

## 4. Mechanism vs policy split

| Concern | Classification | Owner |
|---|---|---|
| Deadline discipline, IRQ handler, intent flag, triggers, switch sequence | Mechanism | W04 |
| Slice value (default constant) | Policy parameter (P7 stage scope) | W04 records; chosen at implementation |
| Per-entity slice differentiation | Policy | W05 (v0: uniform) |
| Requeue position after deschedule | Policy | W05 |
| Whether slice expiry forces a switch | Mechanism (yes: expiry always forces reconsideration) with policy deciding the next entity | Split W04/W05 |
| Timer programming, time source | Predecessor mechanism | P6 |
| Register/timer/vGIC state transfer | Predecessor mechanism | P4/P6 |

Review rule: code in the preemption/switch modules that encodes pick order,
requeue position, or differentiated slices is a review failure (W05 scope);
code in the policy layer that touches the timer or registers is a layering
failure (W04/P6 scope).

## 5. Guardrail restatements binding this package

- Physical CPU and vCPU are separate objects; the deadline and intent flag
  belong to the *pCPU's* scheduler state; the slice consumption belongs to
  the *vCPU's* scheduling view; neither ever collapses into the other.
- Per-pCPU run-queue ownership is W05's; the switch sequence only ever
  dequeues/enqueues through W05-owned operations, never by touching queue
  internals.
- The `current_vcpu` slot and lifecycle state are W02's; the switch
  orchestrates through the gate/exit/engine contracts only.
- The scheduler never touches arch registers, timer registers, GIC/LR
  state, or VMID allocator internals; it invokes predecessor mechanisms.
