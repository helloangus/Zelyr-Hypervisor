# P7-W06 Block/Wakeup Architecture and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W06 detailed design](README.md).

## 1. Logical module boundary

W06 contributes one scheduler-side logical module — the **block/wakeup
controller** — plus two per-vCPU state records owned by existing W02-defined
objects. It owns no runqueue, no timer object, and no vIRQ structure.

| Unit | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Block path | Decide blocking on a WFI/WFE-class exit; poll eligibility; commit `Blocked`; release pCPU capacity | Per-vCPU block-intent marker | Exit classification (P4), pending-event view, lifecycle state | `Blocked` transition via W02 authority; next scheduler decision; exit hint | Exit classification itself; time-slice policy; what runs next |
| Wake path | Record wake events; convert eligibility; exclude ineligible states | Per-vCPU pending-event record | Events from timer IRQ, vIRQ injection, internal control, future Notification | `Blocked → Runnable` transition; enqueue via W05 seam; reconsideration request | Event production (P6); target-pCPU selection (W03/W05); transport (W08) |
| Deadline fold | Keep a blocked vCPU's guest-timer deadline observable while it has no pCPU | None (delegates) | vCPU deadline (P6-W06), home-pCPU host deadline (P6-W05) | Folded host deadline; `TimerExpiry` posting on IRQ | Timer programming, monotonic-time ownership (P6-W05) |

## 2. Assumed upstream contracts and failure boundaries

W06 consumes contracts that P0–P6 and sibling P7 packages are planned to
deliver. None is evidenced in the current tree. Each has a failure boundary:
**stop the affected step and raise the issue for W01 reconciliation
(P7-V01) / an Architecture Change Request — never patch it locally.**

| ID | Assumed contract | Source (plan path) | Fails if |
|---|---|---|---|
| L-1 | Transition authority validates and performs lifecycle transitions, rejecting invalid transitions explicitly | [P7-W02](../../plans/p7-w02-scheduler-admission-lifecycle.md), design `../p7-w02-scheduler-admission-lifecycle/README.md` | Transitions must be performed outside this authority, or legality for `Blocked→Runnable` / `Blocked→Paused` / `Paused→Blocked` is undefined |
| L-2 | Single-running invariant: a vCPU runs on at most one pCPU; a Running vCPU has exactly one owning pCPU | P7-W02 (P7-V04) | Wake could produce duplicate running; race tests cannot close the hole |
| L-3 | Scheduler-controlled admission/return seam: guest entry and exit return through scheduler control with the current-vCPU notion defined | P7-W02 (P7-V02) | The block path has no defined point at which it is called |
| P-1 | WFI/WFE exits are classified and delivered to EL2 control flow without redefining bring-up | [P4-W04](../../../p4/plans/p4-w04-vcpu-entry-exit.md), [P4-W05](../../../p4/plans/p4-w05-validation-guest.md) | Blocking exits cannot be identified; trap-and-schedule is unimplementable |
| P-2 | vCPU virtual-timer deadline state is vCPU-owned; expiry is eventually delivered even if the vCPU is not executing | [P6-W06](../../../p6/plans/p6-w06-guest-generic-timer.md) | Blocked-vCPU timer wake has no source |
| P-3 | Per-pCPU host deadline timer with arm/cancel/rearm owned by the running pCPU | [P6-W05](../../../p6/plans/p6-w05-el2-generic-timer.md) | The deadline fold cannot be expressed without violating P6 ownership |
| P-4 | Authorized vIRQ request/pending semantics per target vCPU, with unavailable-target outcomes defined | [P6-W07](../../../p6/plans/p6-w07-virtual-interrupt-core.md) | `VirtualIrq` wakeup lacks an authorized producer |
| S-1 | Acquire/release ordering expectations and non-sleeping-context rules for shared Host state | [P3-W06](../../../p3/plans/p3-w06-concurrency-synchronization.md) | The protocol orderings in §5 have no authority to cite; do not invent memory-order spellings |
| S-2 | Enqueue seam: a runnable entity can be inserted for scheduling under W03 placement constraints | [P7-W05](../../plans/p7-w05-shared-mn-multivm.md), design `../p7-w05-shared-mn-multivm/README.md` | A woken vCPU has nowhere to wait; W06 must not design the runqueue |
| S-3 | Reconsideration requirement: after enqueue, the eligible pCPU must be caused to re-evaluate; transport owned by W08 | [P7-W08](../../plans/p7-w08-smp-reschedule-idle.md), design `../p7-w08-smp-reschedule-idle/README.md` | A wake on a remote pCPU is never observed; if W08 does not fix this transport, W06's postcondition is unmet and the conflict is raised |
| C-1 | Eligible-pCPU placement predicate per vCPU (affinity/pinning/dedicated-shared) | [P7-W03](../../plans/p7-w03-placement-configuration.md) | Wake enqueue cannot verify placement compliance |

If the W02 frozen contract omits any transition used here (`Blocked→Runnable`,
`Blocked→Paused`, `Paused→Blocked`), that is a gap in L-1's authority, not a
license for W06 to invent transition legality; record it and stop.

## 3. Blocked eligibility

A vCPU is **blocked-eligible** when all of the following hold:

1. Its lifecycle state (L-1 authority) permits `Running → Blocked`.
2. The blocking exit is a P4-classified WFI or WFE class exit (assumed
   contract P-1); unknown synchronous exits never take the block path.
3. The eligibility poll (below) finds **no** eligible pending event.

An **eligible pending event** is a recorded wake event whose source implies
Guest-observable progress: a due vCPU timer deadline (P-2), an authorized
pending vIRQ targeted at this vCPU (P-4), a `Notification`-classified event, or
a pending internal control request (e.g. pause/stop needing the vCPU's
attention, per [P7-W07](../p7-w07-pause-stop-fault/README.md)).

A vCPU blocked without an eligible event must not busy-loop: after commit, the
pCPU runs the normal scheduler decision (dispatch another entity or enter
idle per P7-W08). The vCPU stays `Blocked` until a wake event arrives; there is
no timeout-based re-examination in P7 (time-slice preemption does not apply to
`Blocked` entities because they consume no capacity).

## 4. Per-vCPU wake-event state

Each vCPU carries, as part of its scheduler state (representation owned by the
module; layout is not an ABI):

- `wake_pending`: a set of at most four source flags
  (`TimerExpiry`, `VirtualIrq`, `Notification`, `Internal`), set with release
  ordering by producers, read with acquire ordering by consumers.
- `block_intent`: two-phase marker (`None`, `Intent`) manipulated only by the
  block path of the pCPU currently running the vCPU.
- One-word atomic access is the default expectation; any multi-word state must
  be moved under the lifecycle lock rather than by ad-hoc ordering (S-1).

Internal control requests set `wake_pending.Internal` and are then processed by
the control paths of [P7-W07](../p7-w07-pause-stop-fault/README.md); W06 only
guarantees that a blocked vCPU's pending control request is observable at the
defined consumption points.

## 5. The two-phase protocol (lost-wakeup prevention)

Blocker (on the pCPU running the vCPU):

```text
function try_block(vcpu, exit_hint):
    assert current_pcpu().current == vcpu              # L-3
    if lifecycle.state(vcpu) != Running: return NotBlocked  # e.g. pause raced in
    set block_intent = Intent                          # release
    events = read wake_pending (acquire)
    if any_eligible(events):                           # §3 rule 3
        block_intent = None
        return WokeImmediately(events)                 # caller re-decides; no capacity released
    perform Blocked transition via L-1                 # under lifecycle lock
    clear current-vCPU registration (L-3)
    record exit_hint for accounting (W09 hook)
    return Blocked
```

Waker (any pCPU, including IRQ context):

```text
function post_wake_event(vcpu, source):
    set wake_pending[source] (release, coalescing per source)
    if lifecycle.state(vcpu) == Blocked:               # acquire via L-1 lock
        perform Blocked → Runnable via L-1             # checks invalid-state exclusions
        enqueue via S-2 under C-1 placement
        request reconsideration of eligible pCPU(s) via S-3
    # else: event stays pending for the defined consumption points
    return outcome
```

Pairing guarantee: between the blocker's release-store of `Intent` and its
acquire-load of `wake_pending`, any waker's release-store of an event is
visible; if the waker instead observes `Blocked` already committed, the
transition happened under the lifecycle lock after the poll, so the waker's
eligibility change cannot be lost and cannot double-run (L-2). Exact barrier
spellings, lock scopes, and IRQ-masking rules are owned by the implementation
under S-1 and the Coding Guidelines; this design fixes the pairing, not the
instruction sequence.

## 6. Interaction classes (reviewed, implemented by W07/W08)

- **Pause/stop during block:** a control request for a `Blocked` vCPU either
  transitions it directly under L-1 (no execution needed) or leaves
  `wake_pending.Internal` for the control path; [P7-W07](../p7-w07-pause-stop-fault/README.md)
  owns the outcome classes. W06 guarantees only that the request is observable
  at consumption points.
- **Wake versus pause race:** L-1 serializes the eligibility transition against
  the pause transition; whichever wins, the resulting state is consistent and
  the event record survives for resume (P7-V15 "resume preserves pending
  events").
- **Cross-CPU wake:** `post_wake_event` may run on a different pCPU than the
  home pCPU; the enqueue and reconsideration steps (S-2/S-3) make the wake
  effective remotely. W06 imposes no transport requirement beyond S-3's
  postcondition.
- **Idle interlock:** after a block, the home pCPU may enter idle; the
  deadline fold (decision 5 of the README) and S-3 reconsideration are the two
  wake sources idle must honor. [P7-W08](../p7-w08-smp-reschedule-idle/README.md)
  owns idle entry ordering.

## 7. Failure and degradation behavior

- **Wake for a nonexistent or offline target:** the vCPU object lifetime is
  owned by the VM/vCPU lifecycle (P4/P5 boundary); `post_wake_event` on a
  destroyed vCPU is a caller bug (hypervisor-internal), reported as an
  invariant violation per the P0-W14 panic policy boundary, not absorbed
  silently. Guest-initiated wake requests reach this path only through
  authorized producers (P-4, P5 dispatch), so untrusted input cannot create it.
- **Spurious or unclassifiable blocking exits:** do not block; return to the
  scheduler decision with the exit unhandled by W06 and diagnosed per the
  P4-W06 fault-classification boundary. Blocking on an unknown exit would hide
  Guest-visible state changes.
- **Deadline fold failure (P-3 unavailable):** block still proceeds, but idle
  entry must not be permitted for the home pCPU until the deadline is
  observable; this degraded combination is a blocked prerequisite to record,
  not a local workaround.
