# P3-W07 Code Contracts — Slot, Encoding, and Protocol

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W07 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; exact bit positions and Rust paths are fixed in the
implementation record. The layout is internal representation — never
serialized, never guest-visible.

## 1. `NotificationKind`

```text
Name and stability: NotificationKind — enum { Doorbell = 0, TlbTransport
    = 1, Reserved2 = 2, Reserved3 = 3 } over u8; internal; values fixed
    for the stage; values ≥ 4 are invalid and refused.
Purpose and caller: the minimal type information carried by an event;
    senders name a kind, receivers dispatch to the kind owner's contract.
Preconditions / postconditions: Doorbell has no payload semantics
    (payload ignored); TlbTransport's payload meaning is defined by
    [P3-W08](../p3-w08-tlb-shootdown-transport/README.md), which claims
    the value in its own design; Reserved2/Reserved3 exist so a consumer
    design claims a value without renumbering.
State and ownership change: n/a.
Concurrency/allocation context: plain value.
Errors and failure guarantee: from_u8(≥ 4) is a named invalid value,
    refused at the API boundary ([04 §2](04-code-contracts-notification-api.md)).
Security/authorization checks: n/a (host-only).
Logic: plain enum + checked conversion.
Validation: W07-DV01 (encoding table tests).
```

## 2. `NotificationSlot` (contents contract)

```text
Name and stability: NotificationSlot — contents owned by W07 from its
    init point; placement/sizing owned by
    [P3-W04](../p3-w04-per-cpu-runtime/README.md) (03 §5.1); internal;
    cache-line aligned; lifetime = whole boot (no Drop).
Purpose and caller: the target's entire notification state; senders
    touch only event_word; the owning CPU owns the rest.
Inputs / outputs: fields per the layout sketch in
    [02 §4](02-architecture-and-state.md): event_word (AtomicU64),
    last_seen (u32), arrivals_total, coalesced_total,
    arrivals_by_kind [u32; 4] (receiver-private).
Preconditions / postconditions: after `init` (§3): event_word == {seq 0,
    kind 0, payload 0, reserved 0}; counters zero; postcondition for all
    later operation — reserved bits stay zero (corruption = fatal
    invariant).
State and ownership change: W04 zero-fill → W07 init → live for boot.
Concurrency/allocation context: no allocation; the single cross-CPU RMW
    is the sender CAS on event_word; all other fields are
    receiver-private (single writer).
Errors and failure guarantee: corruption is fatal-class (P0-W14), never
    recovered.
Security/authorization checks: host-only; not guest-reachable.
Logic: structure only.
Validation: W07-DV01 (layout/encoding review), W07-DV06 (counter
    accounting tests).
```

## 3. `init_slots`

```text
Name and stability: init_slots(areas: &PerCpuSet) -> Result<(), InitError>
    — internal; called once during global initialization (single-threaded,
    pre-release), at the point the boot sequence orders after W04's
    allocate_all.
Purpose and caller: the W04→W07 ownership handoff point; lays down the
    initial slot values explicitly rather than trusting the zero-fill.
Inputs / outputs: the W04 per-CPU set; Ok or Err(InitError) naming the
    CPU/slot.
Preconditions / postconditions: precondition — boot phase Bootstrap
    (pre-release, single CPU). Postcondition — every candidate CPU's slot
    matches §2's initial postcondition; ownership of contents is W07's.
State and ownership change: contents ownership W04 → W07.
Concurrency/allocation context: single-threaded; no allocation beyond
    none; the P2 allocator is not used (slots already exist).
Errors and failure guarantee: an invalid/missing slot is fatal
    boot-critical (the W04 contract was violated); no partial init
    published.
Security/authorization checks: n/a.
Logic: for each area: validate alignment, write initial word/counters,
    record done-flag; re-validate (magic/self_ptr round-trip unchanged).
Validation: W07-DV01.
```

## 4. Sender protocol — `send_event` (internal primitive)

```text
Name and stability: send_event(slot: &NotificationSlot, kind:
    NotificationKind, payload: u8) -> Result<(), SendError> — internal;
    the CAS primitive under `notify`'s gates.
Purpose and caller: atomically publish one event into the target slot;
    caller is notify only (after gating).
Inputs / outputs: the target's slot (reached via the W04 cross-CPU
    surface by logical id), kind, payload; Ok or a gate error
    (gating errors are notify's; SendError covers only CAS-exhaustion,
    which at P3 is unbounded-retry and therefore unreachable — the
    variant exists so the contract is total).
Preconditions / postconditions: precondition — gates passed (SmpReady,
    target online; checked by notify). Postcondition — event_word has a
    sequence exactly one ahead of the pre-write value with this kind/
    payload, *unless* a concurrent sender overwrote it (defined
    coalescing); `sev` executed after the release write.
State and ownership change: event_word (sender-written field).
Concurrency/allocation context: CAS loop (Acquire read / Release write,
    per W06 AP-1/AP-2); no allocation; no lock (W06 BW-2 discipline);
    bounded work — two stores and a wake.
Errors and failure guarantee: no partial publication (CAS); on the
    impossible retry-exhaustion the error names it and no wake fires.
Security/authorization checks: none beyond notify's gates (host-only).
Logic (pseudocode):

    send_event(slot, kind, payload):
        loop:
            cur = slot.event_word.load(Acquire)
            next = pack(cur.seq.wrapping_add(1), kind, payload)
            if slot.event_word.compare_exchange(cur, next, Release,
                                                Acquire).is_ok(): break
        sev()                                  # broadcast wake; arch
                                               # boundary (unsafe, P1
                                               # baseline discipline)
        return Ok(())

Validation: W07-DV02 (ordering), W07-DV04 (concurrent senders).
```

## 5. Concurrent-sender semantics (normative statement)

- Two senders targeting one CPU both succeed eventually (CAS retry);
  their sequence numbers are distinct and consecutive relative to their
  own reads.
- The receiver observes only the latest kind/payload for any sequence gap
  it closes; every skipped sequence is counted in `coalesced_total`.
- Coalescing is therefore *countable but not attributable*: the kind of a
  coalesced event is not recorded. This is the recorded limitation that
  keeps the primitive minimal (README decision 2); a consumer needing
  per-event attribution needs a design change, not a local queue.
- Sequence wrap uses wrapping-distance comparison; safe at P3 scale,
  recorded.

## 6. Receiver protocol — `observe` (internal primitive)

```text
Name and stability: observe(slot: &NotificationSlot) -> Option<Notification>
    — internal; executed by the owning CPU only.
Purpose and caller: close the sequence gap, update accounting, return the
    delivered event; caller is poll and wait ([04 §4](04-code-contracts-notification-api.md)).
Inputs / outputs: the caller's own slot; Some(Notification{kind, payload,
    sequence}) when the sequence advanced, None otherwise.
Preconditions / postconditions: precondition — the caller is the slot's
    owner (enforced by calling through `current()`'s area, never via the
    cross-CPU table). Postcondition — counters reflect exactly the closed
    gap; last_seen == observed sequence.
State and ownership change: receiver-private fields only.
Concurrency/allocation context: acquire load; no allocation; no lock.
Errors and failure guarantee: reserved bits nonzero or an invalid kind in
    the word is corruption — fatal diagnostic; None means no new event
    (not an error).
Security/authorization checks: ownership check is structural (own area
    only).
Logic (pseudocode):

    observe(slot):
        word = slot.event_word.load(Acquire)
        (seq, kind, payload) = unpack(word)
        d = wrapping_distance(slot.last_seen, seq)
        if d == 0: return None
        slot.coalesced_total += d - 1
        slot.arrivals_total += 1
        slot.arrivals_by_kind[kind] += 1
        slot.last_seen = seq
        return Some(Notification{ kind, payload, sequence: seq })

Validation: W07-DV02 (single-variable ordering), W07-DV06 (accounting
    exactness incl. coalescing gaps).
```
