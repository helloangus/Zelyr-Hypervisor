# P7-W08 SMP Scheduler Architecture: Concurrency, Reconsideration, Idle

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W08 detailed design](README.md).

## 1. Logical module boundary

W08 contributes one scheduler-side logical module — the **per-pCPU scheduling
and reconsideration controller** — plus the per-pCPU idle state. It owns no
runqueue topology, no notification primitive, and no timer.

| Unit | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| pCPU scheduling loop | Run the single decision entry at preemption, IPI receipt, idle exit, and post-exit continuation | Per-pCPU scheduler context (P3-W04 reserved capacity) | Deadline events, reconsideration signals, exit dispositions | Dispatch decisions via L-1/L-3; idle entry | Selection policy internals (W05); block semantics (W06); pause semantics (W07) |
| Reconsideration controller | Coalesce, target-verify, transmit, receive, and re-check reconsideration requests | Per-pCPU reconsideration intent flag | Requests from W06 wakeup, W07 control, W04 deadline paths | Transport signals (P3-W07); handler re-runs the decision entry | Signal transport internals (P3/P6); placement policy (C-1 evaluation) |
| Idle controller | Own the designed idle state: entry protocol, wait, exit | Per-pCPU idle state and wake-source accounting | Decision entry with no eligible work; wake events (deadline, signal, notification) | Idle enter/exit; wake-source records | Power management; work stealing; timer programming |

## 2. Assumed upstream contracts and failure boundaries

All unevidenced in the current tree. Failure boundary per row: **stop the
affected step, record for P7-W01 reconciliation (P7-V01); Architecture Change
Request on contradiction — never patch locally.**

| ID | Assumed contract | Source (plan path) | Fails if |
|---|---|---|---|
| L-1/L-2/L-3 | Transition authority, single-running invariant, admission/return seam | [P7-W02](../../plans/p7-w02-scheduler-admission-lifecycle.md), design `../p7-w02-scheduler-admission-lifecycle/README.md` | The decision entry cannot dispatch or verify state |
| C-1 | Eligible-pCPU placement predicate | [P7-W03](../../plans/p7-w03-placement-configuration.md) | Remote requests cannot be placement-verified |
| S-2 | Enqueue seam (runqueue membership) | [P7-W05](../../plans/p7-w05-shared-mn-multivm.md), design `../p7-w05-shared-mn-multivm/README.md` | A woken/enqueued entity has no queue; W08 must not design one |
| S-3 | Reconsideration *requirement* (postcondition only) | [P7-W06](../p7-w06-block-wakeup/README.md) and [P7-W07](../p7-w07-pause-stop-fault/README.md) seams | No producer of reconsideration requests exists; W08's transport has no clients |
| PC-1 | Per-CPU state foundation with capacity reserved for scheduler/current-vCPU needs; no implicit global current-CPU state | [P3-W04](../../../p3/plans/p3-w04-per-cpu-runtime.md) | Per-pCPU contexts cannot be built without violating P3 ownership |
| PC-2 | Synchronization semantics: lock/IRQ rules, acquire/release expectations, non-sleeping contexts, baseline lock order | [P3-W06](../../../p3/plans/p3-w06-concurrency-synchronization.md) | Ordering pairings below have no authority; do not invent barrier spellings |
| PC-3 | Cross-CPU notification primitive: targeted delivery, self-notification defined, concurrent senders safe, invalid/offline-target outcomes defined | [P3-W07](../../../p3/plans/p3-w07-cross-cpu-notification.md) | The reconsideration signal has no transport; offline targets undefined |
| PC-4 | Evidenced Host SGI mechanism with target-attributed send/receipt/completion | [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md) | The physical layer under PC-3 is unevidenced; reconsideration latency bounds are untestable |
| PC-5 | Per-pCPU host deadline timer arm/cancel/rearm surface | [P6-W05](../../../p6/plans/p6-w05-el2-generic-timer.md) | Idle deadline folding cannot be expressed |
| PC-6 | P3 online-pCPU identity set used for targeting | [P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md) handoff | Targeting may reach nonexistent pCPUs |
| W6-2 | W06 wake path calls S-2 enqueue then S-3 reconsideration; deadline-home fold (its B-4) feeding PC-5 | [P7-W06](../p7-w06-block-wakeup/README.md), contracts in `../p7-w06-block-wakeup/02-code-contracts-block-path.md` | Idle deadline folding has no producer; remote wakes never signal |
| W7-3 | W07 control paths issue S-3 reconsideration for remote Running targets; VM-pause completion detection runs on exit paths | [P7-W07](../p7-w07-pause-stop-fault/README.md), contracts in `../p7-w07-pause-stop-fault/02-code-contracts-pause-resume.md` | Remote pause/stop requests never force the target out of Guest execution |

## 3. Per-pCPU scheduling model

Each online pCPU owns a scheduler context (PC-1) containing: its current-vCPU
registration (L-3), its runqueue membership view as delivered by S-2, its
reconsideration intent flag, its idle state, and its deadline obligations
view (folded via PC-5). Mutability rule (isolation):

- A pCPU's context is mutated **only** by that pCPU, except where a structure
  is explicitly declared shared (the intent flag is the only cross-CPU-
  written scheduler field in P7).
- Shared structures (enqueue targets, lifecycle states) are accessed only
  under PC-2's rules and the owning subsystem's authority (L-1, S-2).
- No global "current CPU" or global scheduler-context shortcut exists (PC-1
  prohibition); Core never iterates pCPUs to find work.

The scheduling loop has one decision entry (parent README decision 1):

```text
function scheduling_decision(pcpu):            # called at exactly four points:
    loop (bounded re-checks):                  #   1. preemption deadline (W04)
        entity = select_next(pcpu)             #   2. reconsideration handler
        if entity is Some:                     #   3. idle exit
            admit via L-3 (runs P7-W07 P-5 pause check, P7-W06 event poll)  #   4. post-exit continuation
            enter Guest via L-1/L-3
        else:
            enter_idle(pcpu)                   # I-1; returns only on a wake source
            continue                           # re-run decision after wake
```

`select_next` is the S-2/W05 surface; its policy internals are not W08's.
The loop's bounded re-check bounds exist to absorb arrivals during the
decision, not to poll: after a failed re-check, the pCPU goes idle rather
than spins.

## 4. Reconsideration protocol

```text
requester (any pCPU, any context incl. IRQ):
    set target.intent = pending          # release
    if target != current_pcpu:
        transport.send(target)           # P3-W07 primitive; one signal per
                                         # flag transition, coalescing storms
    # self-target: no signal; the local loop observes the flag at its next
    # decision point (exit path, deadline, idle exit)

handler (target pCPU, in its own interrupt context):
    bounded loop (declared upper bound, e.g. a small constant):
        target.intent = none             # clear FIRST (release)
        scheduling_decision(pcpu)        # observes enqueues made visible by
                                         # requesters' earlier release stores
        if target.intent == pending: continue   # work arrived during decision
        else: break
```

Properties: clearing before deciding prevents the lost-wakeup window
(a request coalesced during the decision is caught by the trailing
intent check); coalescing bounds handler invocations under request storms;
self-targets never cross the transport. Exact ordering spellings are
implemented under PC-2; this design fixes the pairing semantics only.

## 5. Cross-CPU effects served

- **Remote wakeup** (P7-W06 W-1): after enqueue, requests reconsideration of
  the pCPU(s) the C-1 eligible set names; if a named pCPU is idle, the signal
  is a designated idle wake source (I-2), so a sleeping pCPU receives it.
- **Remote pause/stop** (P7-W07 P-1/F-1): forces the owning pCPU to exit the
  Guest in bounded time; the request marker (release) precedes the signal so
  a handler decision cannot re-admit the marked vCPU (its admission check
  sees the marker).
- **Placement compliance**: every remote request is verified against C-1
  before signaling; a request naming an ineligible/offline pCPU is refused
  and diagnosed (an invariant bug, not retried).

## 6. Isolation and preservation rules under concurrent activity

- Concurrent distinct-vCPU execution (P7-V17) requires no scheduler-shared
  mutable state beyond the declared shared structures; reviewers verify the
  write sets of the decision entry, handler, and idle paths.
- Lifecycle and placement guarantees are never recomputed on remote paths:
  the remote pCPU applies the same L-1/L-3/C-1 checks as the local path —
  there is no "remote fast path" that skips them.
- Offline-target requests (PC-6) cannot arise from correct placement; their
  occurrence is a diagnosable invariant violation surfaced per the P0-W14
  boundary.
- Idle exit re-enters the same decision entry — no alternate selection rule
  exists for post-idle dispatch (preservation under P7-V18).

## 7. Failure and degradation behavior

- **Transport loss detection:** the P3-W07 surface reports send failure for
  an offline/invalid target; the scheduler records the refusal and — because
  requests are also observable at the target's next natural decision point
  (deadline, exit) — a missed signal degrades to bounded-latency discovery,
  never to a permanent stall, provided the target is online. A permanently
  stalled online target despite pending intent is an invariant violation to
  surface, not a poll loop to add.
- **Idle wake-source violation:** a wake observed with no matching recorded
  source is a diagnostic event (spurious wake); idle re-runs the decision
  regardless (correctness never depends on the wake being expected).
- **Deadline fold failure at idle entry:** idle is refused (stay in the
  decision loop's bounded path) per the P7-W06 B-4 degraded rule; record the
  blocked prerequisite.
