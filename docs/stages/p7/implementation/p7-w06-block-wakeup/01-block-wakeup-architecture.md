# P7-W06 Block/Wakeup Architecture and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed; owner-selected handshake direction.
**Scope:** W06 block/wakeup state ownership, per-vCPU coordination state,
block/wake race protocol, interactions, and failure boundaries.
**Version:** v0.2
**Owner/change context:** P7-W06 design amendment following owner direction
on AUD-001, 2026-09-28.
**Parent:** [P7-W06 detailed design](README.md).
**Supersedes:** None; refines the existing proposed state model in this path.

## 1. Logical module boundary

W06 contributes one scheduler-side logical module — the **block/wakeup
controller** — plus one per-vCPU block/wake coordination word and its event
bits. W02-defined objects retain lifecycle ownership. W06 owns no runqueue, no
timer object, and no vIRQ structure.

| Unit | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Block path | Decide blocking on a WFI/WFE-class exit; poll eligibility; commit `Blocked`; release pCPU capacity | Per-vCPU block/wake coordination word (phase and event bits) | Exit classification (P4), event bits, lifecycle state | `Blocked` transition via W02 authority; next scheduler decision; exit hint | Exit classification itself; time-slice policy; what runs next |
| Wake path | Record wake events; participate in the block handshake; convert eligibility; exclude ineligible states | Same per-vCPU block/wake coordination word | Events from timer IRQ, vIRQ injection, internal control, future Notification | W02 lifecycle transition; enqueue via W05 seam; reconsideration request | Event production (P6); target-pCPU selection (W03/W05); transport (W08) |
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
module; layout is not an ABI), one `block_wake` coordination word. It packs a
phase (`Open`, `Intent`, `WakeDuringIntent`, `Committing`, `WakeDuringCommit`,
or `Blocked`) and up to four source bits (`TimerExpiry`, `VirtualIrq`,
`Notification`, `Internal`). The phase and event bits must share one atomic
compare-exchange domain. Separate atomics for the event set and block intent
do not establish a shared winner and permit the audited lost wakeup.

The word is internal and not an ABI. P3-W06 owns the concrete acquire/release
spellings and IRQ-context rules (S-1); W02 remains the sole lifecycle
transition authority. A representation that splits these fields must prove an
equivalent shared linearization protocol under S-1 and amend this design before
implementation.

| Phase | Owner / permitted transition | Wake arriving in this phase | Required completion |
|---|---|---|---|
| `Open` | Blocker may claim `Intent`; waker records only | The event bit changes the shared word, so a no-event `Open→Intent` CAS cannot silently overwrite it | Blocker re-evaluates the event bits; waker leaves them pending |
| `Intent` | Blocker may claim `Committing` | Waker atomically claims `WakeDuringIntent` while recording its bit | Blocker aborts before `Running→Blocked`, returns the eligible event set, and restores `Open` |
| `WakeDuringIntent` | Waker has published the event; blocker owns abort | Additional wakers only coalesce/add source bits | Blocker does not commit `Blocked`; event remains observable or is returned to the caller |
| `Committing` | Blocker owns the W02 `Running→Blocked` attempt | Waker atomically claims `WakeDuringCommit` while recording its bit | Blocker resolves the W02 result; on a successful block it completes the wake through W02 and enqueues only if the resulting state is eligible |
| `WakeDuringCommit` | Blocker owns wake completion; waker returns after publication | Additional wakers coalesce/add bits without taking lifecycle action | Blocker resolves pause/exclusion through W02/W07, completes an eligible wake exactly once, then restores `Open` |
| `Blocked` | W02 block transition is committed; waker owns wake | Waker records the bit and asks W02 to transition `Blocked→Runnable` | Only the successful W02 transition enqueues; rejected/excluded states retain the event per W07 |

Internal control requests set the `Internal` bit in `block_wake` and are then processed by
the control paths of [P7-W07](../p7-w07-pause-stop-fault/README.md); W06 only
guarantees that a blocked vCPU's pending control request is observable at the
defined consumption points.

## 5. Shared atomic block/wake handshake (lost-wakeup prevention)

Blocker (on the pCPU running the vCPU):

```text
function try_block(vcpu, exit_hint):
    assert current_pcpu().current == vcpu              # L-3
    if lifecycle.state(vcpu) != Running: return NotBlocked  # e.g. pause raced in
    if !CAS(block_wake, Open+no_eligible_bits, Intent+same_bits):
        return WokeImmediately(consume_eligible_bits()) # event won before intent
    if any_eligible(read_pending_bits(block_wake)):
        atomically_clear_intent_and_consume_eligible_bits()
        return WokeImmediately(events)                  # no capacity released
    if !CAS(block_wake, Intent+bits, Committing+bits):
        # A waker changed Intent to WakeDuringIntent; it owns no L-1 transition.
        atomically_return_to_Open_and_consume_eligible_bits()
        return WokeImmediately(events)
    if !lifecycle.transition(vcpu, Running, Blocked):    # L-1
        atomically_return_to_Open_preserving_pending_bits()
        return NotBlocked(TransitionRejected)
    if !CAS(block_wake, Committing+bits, Blocked+bits):
        # WakeDuringCommit transfers completion to this blocker.
        if lifecycle.transition(vcpu, Blocked, Runnable): # L-1, W07 exclusions apply
            enqueue(vcpu)                                 # S-2/C-1, only on success
        else:
            preserve event bits for the W07 resume/control path
        atomically_return_to_Open_preserving_pending_bits()
        clear current-vCPU registration
        record block and wake outcomes
        return WokeDuringCommit
    clear current-vCPU registration (L-3)
    record exit_hint for accounting (W09 hook)
    return Blocked
```

Waker (any pCPU, including IRQ context):

```text
function post_wake_event(vcpu, source):
    phase = atomically_record_source_and_observe_or_claim_phase(source)
    # One CAS loop updates the phase and pending-source bit together.
    if phase == WakeDuringIntent:
        return EventRecorded                           # blocker must abort
    if phase == WakeDuringCommit:
        return EventRecorded                           # blocker must finish wake
    if phase == Blocked:
        perform Blocked → Runnable via L-1             # checks invalid-state exclusions
        enqueue via S-2 under C-1 placement
        request reconsideration of eligible pCPU(s) via S-3
        atomically_return_phase_to_Open_preserving_pending_bits()
    # Open and excluded lifecycle states retain the bit for their defined
    # consumption point and do not claim block completion.
    return outcome
```

Pairing guarantee: the event bits and phase share one atomic modification
order. An event recorded before `Open → Intent` prevents the no-event CAS from
matching; an event during `Intent` wins `Intent → WakeDuringIntent` or makes
the blocker's claim fail; an event during `Committing` transfers wake
completion to the blocker; and an event after `Blocked` is published leaves
the waker responsible for W02's transition. Thus no return path can observe
`Running`, drop the event, and then allow `Blocked` to be published without
either side owning the wake. P3-W06 must review the compare-exchange loop,
event-bit consumption, and IRQ rules against this state machine. This is a
design proof obligation, not an implementation or runtime claim.

## 6. Interaction classes (reviewed, implemented by W07/W08)

- **Pause/stop during block:** a control request for a `Blocked` vCPU either
  transitions it directly under L-1 (no execution needed) or leaves
  the pending `Internal` bit in `block_wake` for the control path; [P7-W07](../p7-w07-pause-stop-fault/README.md)
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
