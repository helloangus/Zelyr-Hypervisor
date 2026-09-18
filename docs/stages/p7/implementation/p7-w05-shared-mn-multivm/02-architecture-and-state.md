# P7-W05 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W05 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `entity` | the scheduler-facing view of a vCPU | none (view) | vCPU identity + frozen placement + dispatchability | `SchedulingEntity` refs | lifecycle state (W02), placement values (W03) |
| `runqueue` | per-pCPU FIFO queue with discipline | the queue (bounded ring of entity refs) + per-queue lock | enqueue/remove/pop requests from owner-side paths | popped candidates; queue stats | eligibility rules (calls W03), lifecycle (calls W02 predicates) |
| `policy` | pick/requeue decisions behind the seam | per-queue policy cursor (RR: implicit in FIFO) | queue view, events (`on_dispatch`, `on_deschedule`) | next candidate or None | queues themselves, gate, timers |
| `enqueue` | target choice + owner-side insert | none | woken/descheduled entity, event pCPU | enqueue outcome | wakeup protocols (W06), remote transport (W08) |
| `loopctl` | the per-pCPU control loop | none (orchestration) | triggers (W04), intent, gate, switch, idle hook | Guest execution windows | pick policy (policy module), idle mechanism (W08) |

Dependency direction (normative): `loopctl` → `policy`/`runqueue`/
`enqueue` + W02/W04 contracts; `enqueue` → `runqueue` + W03 eligibility;
`policy` → `runqueue` view + `entity`. Nothing depends on W08/W09
internals; hooks point outward.

## 2. Core objects and ownership

| Object | Owner | Mutable state | Writers | Lifetime |
|---|---|---|---|---|
| `PcpuScheduler` | one per online pCPU (attached to the P3 per-CPU area, P7-IN-04) | `RunQueue`, policy cursor, idle-hook link | owner pCPU paths only | pCPU online lifetime |
| `RunQueue` | its `PcpuScheduler` | bounded ring of entity refs, length | enqueue/remove/pop under the queue lock | as owner |
| `SchedulingEntity` | created with the vCPU configuration; immutable view | none | none (values frozen at configuration) | vCPU lifetime |
| fairness assertion state | per-measurement-window record (validation instrumentation) | window accumulators | measurement hooks (W11 harness; sampled) | measurement lifetime |

Single-writer rule: a queue has exactly one writer domain — the owner
pCPU's paths (loop, switch-side requeue, owner-side wakeup insert). The
cross-CPU wakeup path delivers an insert *request* through the owner-side
enqueue operation (executed by the requester but only through the queue's
locked operations — no direct ring access). Remote *kicks* (tell the owner
to look) are W08's transport.

## 3. Concurrency and lock discipline

- **Queue lock:** one per queue; held only for ring operations (O(1));
  never held while taking a lifecycle lock, the ledger, or another queue's
  lock (extends the P3-W06 order; W02's leaf rule preserved: gate is
  called after pop, with the queue lock released).
- **Pop-then-verify:** the picker pops a candidate under the queue lock,
  releases, then the gate re-verifies state/eligibility (W02 `admit_for_entry`);
  a rejection requeues the candidate at tail (under the queue lock again)
  and the loop proceeds — bounding any single-candidate retry to one
  pass and delegating sustained-failure handling to the idle/failure
  rules (W08, and the W04 activation-containment pattern).
- **Enqueue-after-transition:** an entity is enqueued only after its
  `Runnable` transition has been accepted by the engine (W02 §5 ordering);
  the observability window between transition and enqueue is the
  W06-owned wakeup-race surface under the frozen no-lost-wakeup invariant.
- **Duplicate exclusion:** enqueue rejects an entity already present (ring
  scan is O(n) with n ≤ capacity; acceptable at configuration-bounded n;
  a per-entity queued-flag maintained under the queue lock is the
  permitted implementation refinement).
- **IRQ context:** no queue mutation in IRQ context; wakeups from P6
  events land via the W06 protocol's bounded post-IRQ step, which uses
  the same owner-side enqueue operation.
- **Allocation:** none on the hot path; ring storage is allocated at
  scheduler init (configuration-bounded capacity, README decision 6).

## 4. M:N scenario definitions (consumer-facing)

Topology is written pCPU:vCPU. All scenarios run on the QEMU reference
platform (P7-IN-01) with the Validation Guest suite (W10) supplying
workloads; W11 executes stress variants; W12 automates the declared
matrix.

| Scenario | Topology | Composition | Progress condition (per P7-V10/V11) |
|---|---|---|---|
| S1 | 1:2 | 1 VM, 2 shared vCPUs, CPU-bound | both vCPUs accumulate positive scheduled time in every fairness window |
| S2 | 2:4 | 1 VM, 4 shared vCPUs over 2 pCPUs | all four progress; each pCPU alternates between its eligible set |
| S3 | 4:8 | 1 VM, 8 shared vCPUs over 4 pCPUs (2× overcommit) | all eight progress; no duplicate or non-runnable execution (checker clean) |
| S4 | 4:8 | 2 VMs × 4 shared vCPUs, mixed CPU/HVC/WFI workloads | every vCPU in both VMs progresses; no VM is denied opportunities while eligible vCPUs exist (opportunity preservation, P7-V11) |
| S5 | 4:4 + pinned | shared matrix plus a pinned/dedicated pair | pinned pair stays on its pCPUs (P7-V05 support); shared set progresses on the remainder |
| S6 | 4:8 | S3 + block/wake churn (WFI-heavy) | progress maintained under churn; wakeup correctness is W06/W08 evidence |

Composition rules: VM count and vCPU counts are stage-configuration
inputs; no scenario requires dynamic VM creation (Reserved). Scenarios are
definitions only — running them is W10/W11/W12 scope.

## 5. Fairness condition (v0, normative definition)

For a measurement window of length

```text
W = (N + 1) * DEFAULT_SLICE + SLACK
```

where `N` = the number of vCPUs continuously `Runnable` and eligible on
the measured pCPU set during the window, `DEFAULT_SLICE` = the recorded
W04 default, and `SLACK` = a recorded constant covering switch/handling
overhead (bounded, measured in validation, never a KPI):

- every such vCPU accumulates strictly positive scheduled time within the
  window (P7-V12 "sustained nonzero execution");
- no proportional or weighted claim is made or tested;
- a vCPU that blocks, is paused, or loses eligibility mid-window is exempt
  for that window (it is not "continuously runnable").

The condition is checkable by assertion in W11's harness and by the loop's
instrumentation; W05 owns the definition and the assertion hook, not the
campaign. RR makes the condition hold by construction (continuous
rotation); the test exists to catch integration defects (lost enqueues,
broken eligibility), not to prove RR optimal.

## 6. Failure model

| Condition | Classification | Behavior |
|---|---|---|
| Gate rejects popped candidate | recoverable, expected | requeue at tail; continue; counts as a reconsideration pass |
| Enqueue of non-`Runnable` entity attempted | programming error | rejected by discipline; audit-grade trace; invariant check |
| Duplicate enqueue attempted | programming error | rejected; audit-grade trace |
| Queue capacity exceeded | configuration invariant violation | `ResourceExhausted`-class error at init; at runtime it is invariant-grade (cannot occur with bounded population) |
| Pick finds queue non-empty but nothing dispatchable (all raced away) | recoverable | loop re-runs pick after requeues; if sustained, falls into idle-hook path (W08) with recorded diagnostic |
| Idle hook misbehaves (busy loop) | W08 design failure | detected by P7-V18 evidence, not by W05 code; W05's hook contract records the prohibition |

No failure path in W05 panics; invariant-grade rows escalate through the
W02 audit/fatal discipline.
