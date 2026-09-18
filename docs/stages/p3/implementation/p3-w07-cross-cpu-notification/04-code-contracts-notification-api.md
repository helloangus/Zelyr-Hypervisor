# P3-W07 Code Contracts — API, Targeting, Accounting

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W07 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths are reserved to the workspace-owning
design.

## 1. `Notification` and `NotifyError`

```text
Name and stability: Notification { kind: NotificationKind, payload: u8,
    sequence: u32 } — internal value; produced by observe/poll.
Name and stability: NotifyError — enum { NotReady, UnknownTarget,
    TargetNotOnline, InvalidKind } — internal; the total outcome
    vocabulary of notify.
Purpose and caller: the event value consumers match on; the named error
    set W12's invalid/offline rows assert on.
Preconditions / postconditions: NotifyError is produced without any slot
    write or wake (fail-closed side-effect freedom).
Concurrency/allocation context: plain values; no allocation.
Errors and failure guarantee: n/a (these are the error/value types).
Security/authorization checks: the targeting errors are the
    authorization boundary for cross-CPU events at P3.
Logic: plain types.
Validation: W07-DV05.
```

## 2. `notify`

```text
Name and stability: notify(target: LogicalCpuId, kind: NotificationKind,
    payload: u8) -> Result<(), NotifyError> — internal; the package's
    entire send surface.
Purpose and caller: targeted CPU-to-CPU notification; callers: W08's
    transport (kind 1), W12's stress harness, any approved P3 consumer.
Inputs / outputs: target logical id, kind, payload; Ok or a named error.
Preconditions / postconditions: precondition — caller runs in thread
    context (no lock held — W06 BW-2; not in an exception handler — CR-4
    territory). Postconditions —
      Ok  ⇒ gates passed; event published (subject to defined
            coalescing); wake executed; returns once the CAS and sev are
            done (the target need not have run).
      Err ⇒ no slot write, no wake; error names the refused gate.
State and ownership change: target slot's event_word only.
Concurrency/allocation context: acquire-read of the W05 phase; W03
    online_set() snapshot (registry scan); CAS loop + sev; no allocation;
    no locks.
Errors and failure guarantee: side-effect-free refusal; no retry by W07.
Security/authorization checks: targeting universe = W03 `OnlineSet`
    (Eligibility::Eligible set) at send time; availability gate = W05
    `BootPhase == SmpReady` (acquire). Unknown ids, offline/failed/never-
    started CPUs, and pre-SmpReady sends are refused by construction.
Logic (pseudocode):

    notify(target, kind, payload):
        if boot_gate.phase().load(Acquire) != SmpReady:
            return Err(NotReady)               # W05 gate
        if !registry.online_set().contains(target):
            return Err(if registry.record(target).is_none()
                       { UnknownTarget } else { TargetNotOnline })
        if kind invalid: return Err(InvalidKind)
        slot = area_of(target)?.notification_slot   # W04 cross-CPU surface
        send_event(slot, kind, payload)?            # [03 §4](03-code-contracts-notification-slot.md)
        return Ok(())

Validation: W07-DV01/DV02 (happy path, ordering), W07-DV03 (self), DV04
    (concurrent), DV05 (each error row).
```

## 3. Self-notification (normative behavior)

```text
Name and stability: (behavior of notify when target == current logical id)
    — defined; no separate API.
Purpose and caller: consumers that use one code path for "wake everyone
    including me".
Behavior: gates evaluated identically (a CPU is in the OnlineSet it reads
    once SmpReady is declared — the snapshot includes the caller);
    send_event executes the same CAS on the caller's own slot word (local
    RMW); sev is a no-op for the running CPU and harmless to others.
    The event is observable by the caller's next poll/observe and appears
    in its accounting exactly like a remote event.
Preconditions / postconditions: as notify; additionally the caller must
    not be inside observe (no re-entrancy into its own slot mid-observe —
    thread-context rule makes this structural).
State and ownership change: own slot's event_word.
Concurrency/allocation context: as notify; all local.
Errors and failure guarantee: as notify.
Security/authorization checks: as notify.
Logic: notify with target = current(); no special casing.
Validation: W07-DV03.
```

## 4. Accounting surface

```text
Name and stability: slot_resident counters (arrivals_total,
    coalesced_total, arrivals_by_kind) plus the notify return value —
    internal; read by diagnostics/tests; aggregation owned by
    [P3-W11](../p3-w11-smp-observability/README.md) when it lands.
Purpose and caller: P3-V07's "explainable arrival/type accounting";
    callers: W12 assertions, W13 regression capture, W11 aggregation.
Invariants (the explainability contract):
    arrivals_total + coalesced_total
        == total sequences closed by the receiver (i.e., every send that
           passed the gates is either delivered or coalesced — none is
           silently lost);
    sum(arrivals_by_kind) == arrivals_total;
    coalesced events are counted but not kind-attributed (recorded
    limitation).
Preconditions / postconditions: counters are receiver-private; cross-CPU
    reads are diagnostic-grade (may observe mid-update values; consumers
    read for evidence, not for synchronization).
Concurrency/allocation context: Relaxed increments per W06 AP-3 (the
    counters feed no ordering decision); diagnostic reads are plain loads.
Errors and failure guarantee: an invariant breach found by a test is a
    protocol bug — diagnose, never adjust the counters to fit.
Security/authorization checks: n/a.
Logic: as observe ([03 §6](03-code-contracts-notification-slot.md)).
Validation: W07-DV06 (storm and gap tests assert the invariants exactly).
```

## 5. The non-RPC limit (normative statement)

W07's contract is: *at most one pending event per target; the latest
event's type/payload observable; concurrent sends may coalesce; events are
observable, never guaranteed delivered within any time bound; no
acknowledgement, no ordering across kinds, no payload beyond 8 bits, no
retry, no session or connection concept.* A consumer that needs any of the
excluded properties requires a design change to this package (new decision,
recorded) — not a protocol built on side channels of the slot (e.g., using
sequence gaps as a data channel is a misuse and a review failure).

## 6. `poll` and `wait`

```text
Name and stability: poll() -> Option<Notification> — internal;
    non-blocking observation of the caller's own slot.
Purpose and caller: consumption from any thread context, including while
    holding a lock *only if* the consumer's contract permits (the observer
    takes no lock and never waits, so polling does not violate BW-2 — but
    a consumer that acts on an event while holding a lock answers for its
    own ladder discipline).
Preconditions / postconditions: precondition — caller is the slot owner
    (post-install CPU). Postcondition — as observe.
Concurrency/allocation context: one acquire load; no allocation.
Errors and failure guarantee: None when no new event.
Security/authorization checks: ownership is structural.
Logic: observe(current().notification_slot).
Validation: W07-DV02.
```

```text
Name and stability: wait() -> Notification — internal; idle-context
    blocking observation (W06 BW-5's sanctioned instance).
Purpose and caller: the terminal idle state for an online CPU with no
    pending work; callers: per-CPU idle loops (the W02 `enter_secondary_idle`
    integration is that design's extension point; W12's wait-based stress
    consumers).
Inputs / outputs: none; returns the next delivered event (spurious wakes
    are tolerated and re-checked internally).
Preconditions / postconditions: precondition — thread context, no lock
    held, no unserviced reception duty of another protocol (BW-4: a CPU
    that also owes W08 completions must use poll-driven consumption, not
    an exclusive wait — the interleaving obligation is W08's design).
    Postcondition — exactly one event consumed per return; counters
    updated.
State and ownership change: receiver-private fields.
Concurrency/allocation context: loop { observe(); if event: return;
    wfe() } — `wfe` is the audited arch boundary (unsafe, P1 baseline
    discipline); no allocation; no lock.
Errors and failure guarantee: no error; a CPU in wait is by definition
    idle; wake failure behavior per the failure model (02 §6).
Security/authorization checks: n/a.
Logic: as above.
Validation: W07-DV02 (wake pairing), W07-DV06 (no event lost across
    wait/poll mixes).
```
