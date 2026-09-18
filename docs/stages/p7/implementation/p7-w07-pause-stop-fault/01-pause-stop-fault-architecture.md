# P7-W07 Pause/Stop/Fault Architecture and Containment Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W07 detailed design](README.md).

## 1. Logical module boundary

W07 contributes one scheduler-side logical module — the **lifecycle control
path** — plus two markers consumed at existing seams (admission and exit). It
owns no capability store, no runqueue, and no fault-classification logic.

| Unit | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Control dispatcher | Authorize and dispatch pause/resume/stop on one vCPU or a VM | None of its own (delegates) | Control request, caller authority (P5), target handle (P5) | Per-state outcome; transitions via L-1; reconsideration via S-3 | Capability semantics (P5); what happens after exclusion |
| Pause completion | Detect that a pause/VM-pause request has fully taken effect | Per-VM pause-pending generation; per-vCPU pause-pending marker | Exit/admission path observations | `Paused` commits; VM-pause completion detection | Forcing a remote pCPU to exit (S-3 transport, W08) |
| Resume path | Restore eligibility from preserved pre-pause context | Pre-pause context record | Resume request, preserved context, pending events | `Paused→Runnable/Blocked`; re-enqueue via S-2 | Placement policy (C-1); event production (W06 sources) |
| Fault/stop commit | Convert classified exits and control stops into terminal exclusion | None (transitions only) | P4-W06 classification; authorized stop request | `Running→Stopped/Faulted` via L-1; exclusion; diagnostics hooks | Fault classification (P4-W06); VM-level fault policy (later) |

## 2. Assumed upstream contracts and failure boundaries

All are unevidenced in the current tree (P0 documentation scaffold). Failure
boundary for each: **stop the affected step, record the gap for P7-W01
reconciliation (P7-V01); raise an Architecture Change Request if a delivered
contract contradicts the assumption. Never patch locally.**

| ID | Assumed contract | Source (plan path) | Fails if |
|---|---|---|---|
| L-1 | Transition authority validates and performs transitions; rejection is explicit. Required edges include `Running→Paused`, `Blocked→Paused`, `Runnable→Paused`, `Paused→Runnable`, `Paused→Blocked`, `Running→Stopped`, `Running→Faulted` | [P7-W02](../../plans/p7-w02-scheduler-admission-lifecycle.md), design `../p7-w02-scheduler-admission-lifecycle/README.md` | Any required edge is missing — a gap in W02's authority, not a license to invent it |
| L-2 | Single-running invariant (a vCPU runs on at most one pCPU) | P7-W02 (P7-V04) | Pause completion cannot be defined ("no longer executing" is ambiguous) |
| L-3 | Scheduler-controlled admission/return seam with a pre-entry check point | P7-W02 (P7-V02) | The pause-pending generation cannot be enforced before Guest entry |
| L-4 | Non-runnable states are structurally excluded from Guest entry | P7-W02 (P7-V04) | Stopped/Faulted exclusion would be convention-based |
| C-1 | Eligible-pCPU placement predicate per vCPU | [P7-W03](../../plans/p7-w03-placement-configuration.md) | Resume cannot preserve placement; empty eligibility must be unrepresentable |
| S-2 | Enqueue seam for runnable entities | [P7-W05](../../plans/p7-w05-shared-mn-multivm.md) | Resumed vCPUs have nowhere to wait |
| S-3 | Reconsideration requirement with transport owned by W08 over P3-W07/P6-W04 Host mechanisms | [P7-W06](../p7-w06-block-wakeup/README.md) seam; [P7-W08](../p7-w08-smp-reschedule-idle/README.md); [P3-W07](../../../p3/plans/p3-w07-cross-cpu-notification.md); [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md) | A remote Running vCPU cannot be forced out of Guest execution in bounded time |
| A-1 | P5 capability rights check with operation-level rights and controlled denial classes; no role/VM-ID bypass | [P5-W05](../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md), [P5-W06](../../../p5/plans/p5-w06-dispatch-permission-containment.md) | Control actions would be authorized by convention; ADR-013 violated |
| A-2 | P5 handle/object-table semantics: invalid, stale, wrong-type, destroyed references are denied explicitly | [P5-W04](../../../p5/plans/p5-w04-handle-lifecycle-type-safety.md) | Pause of a destroyed vCPU would be undefined |
| F-0 | P4-W06 fault classification delivers a GuestFault-class exit with required context (VM/vCPU, reason, addresses) and distinguishes Guest faults from Hypervisor invariant failures | [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md) | `commit_guest_fault` has no trusted trigger |
| P-0 | P0-W14 panic/failure classification: Guest-caused faults are recoverable VM-facing errors | P0-W14 plan (`docs/stages/p0/plans/p0-w14-panic-failure-classification.md`), ADR-019 invariant | Fault handling would escalate to global panic |
| W6-1 | W06 wake-event record survives pause; `post_internal_event` is the control-path wake adapter | [P7-W06](../p7-w06-block-wakeup/README.md), its design `../p7-w06-block-wakeup/README.md` | Pause/resume cannot preserve pending events (P7-V15) |

If the W02 frozen contract lacks `Paused→Blocked` (needed by resume decision 4
of the parent README) or any other listed edge, W07 stops for W02 authority —
the transition set is extended only in W02's design, never here.

## 3. Pause decision table

After authorization (A-1) succeeds for the target handle (A-2), the dispatcher
serializes on the lifecycle authority and dispatches:

| Observed state | Action | Outcome |
|---|---|---|
| Running (this pCPU) | Record pause-pending; run the local completion check at the exit path of the in-flight execution | `PauseAccepted` → `Paused` after the vCPU stops executing |
| Running (remote pCPU) | Record pause-pending; issue S-3 reconsideration to the owning pCPU | `PauseAccepted`; completion as above, bounded by preemption (P7-W04) |
| Runnable | Dequeue via S-2 cooperation; transition `Runnable→Paused` under L-1 | `Paused` immediately |
| Blocked | Transition `Blocked→Paused` under L-1; pending events retained | `Paused` immediately |
| Paused | No-op | `PausedAlready` (explicit idempotent success) |
| Stopped / Faulted / Offline | No state change | `ControlledDenied(WrongState)` |

A pause-pending marker is consumed at the exit path: the exit path transitions
`Running→Paused` (L-1) instead of re-admitting, records the pre-pause context
(§4), and updates the VM-pause completion detector (§5). Race rule: if a wake
or dispatch races the marker, L-1 serialization decides once — either the vCPU
entered (still Running; the marker catches it at the next exit and reconsideration
forces that exit) or it did not (pause completes immediately). Both orders are
safe; neither loses the request.

## 4. Pre-pause context and resume re-evaluation

At the `Paused` commit, the exit path records:

- pre-pause scheduling condition: `WasRunnableOrRunning` or
  `WasBlockedWithoutEvent`;
- placement eligibility as it stood (not recomputed at resume);
- the pending-event record reference (retained; W6-1 guarantees it survived
  pause).

`resume` (authorized, A-1) then re-evaluates under L-1:

```text
eligible_events = pending events ∩ eligible classes (P7-W06 §3 rule)
if state != Paused: return ControlledDenied(WrongState)      # no silent success
if eligible_events nonempty or condition == WasRunnableOrRunning:
    transition Paused→Runnable; enqueue via S-2 under C-1
else:
    transition Paused→Blocked                                # stays off the runqueue
restore nothing else; placement and events were never lost
```

`VM resume` is the per-member composition: every member is resumed; a member
in `Stopped/Faulted` produces `ControlledDenied(WrongState)` for itself while
the remaining members still resume — a partially stopped VM is a fact to
report, not smoothed over. The VM-pause generation is cleared only when the
composition reports no `PausedAlready`-style ambiguity, i.e. every member
returned a terminal outcome.

## 5. VM pause completion model

```text
request_vm_pause(vm, authority):
    authorize(A-1) for the VM control right
    set vm.pause_generation += 1            # release, under lifecycle lock
    for each member vcpu: dispatch pause per §3
    return PausePending(vm.pause_generation)   # acceptance, not completion

completion detection (runs on scheduler exit paths, never spun on):
    when a member vCPU stops executing and no member is Running anywhere
    (verified under the lifecycle lock after the generation was set):
        vm.observed_pause_generation = vm.pause_generation   # complete
```

Properties this model guarantees:

- After the generation is set, no admission check (L-3) admits a member vCPU:
  *no new Guest execution begins*.
- Every member already executing exits at its next exit point, forced by S-3
  reconsideration if necessary, and is individually paused: *existing Guest
  execution ends in bounded time* (bounded by P7-W04 preemption; no unbounded
  spin anywhere).
- Completion is a checkable predicate ("no member Running, generation
  observed"), evaluated on scheduler paths, observable by control-plane
  callers through VM state. No VM-exit or IRQ path ever waits for it.

## 6. Remote-running handling and event interaction

- The remote path records the request first, then requests reconsideration
  (S-3) of the owning pCPU. Ordering matters: a reconsideration arriving
  before the marker is visible would let the target re-enter untouched; the
  marker store (release) precedes the reconsideration request.
- A control request arriving for a Blocked vCPU uses W06's
  `post_internal_event` (W6-1) only when the control path needs the vCPU to
  observe something at a consumption point; pure state transitions
  (Blocked→Paused) do not wake it. This keeps control actions from causing
  needless scheduling churn.
- A pause racing a wake: L-1 serializes; the event record survives into
  `Paused` (W6-1), and §4's resume re-evaluation honors it — this is exactly
  P7-V15's "resume preserves pending events".

## 7. Stop/fault containment model

- **Stop (authorized control):** record request; if Running, force exit via
  S-3; at the exit path, transition `Running→Stopped` (L-1); exclusion
  enforced by L-4. Pending events are retained for diagnostics only.
- **Guest fault (automatic):** the exit path receives an F-0-classified
  GuestFault; `commit_guest_fault` transitions `Running→Faulted` (L-1),
  excludes structurally (L-4), and updates only that vCPU's accounting and
  diagnostic snapshot (P7-W09 hooks). No authorization applies (parent README
  decision 6); no other VM's or vCPU's scheduling state is read or written.
  The hypervisor does not panic (P-0); the fault is a VM-facing event.
- **Other-VM non-corruption:** the fault commit must not touch runqueues,
  other vCPUs' state, shared counters (beyond its own records), or placement
  data. Reviewers verify this by inspecting the commit path's write set;
  P7-V16 and P7-W11 race stress check it dynamically.
