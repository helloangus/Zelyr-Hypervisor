# P3-W07 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W07 detailed design](README.md).

## 1. The sender/receiver split

Notification has exactly two roles per event, and every byte of slot state
belongs to one of them:

```text
SENDER (any online CPU)                        RECEIVER (target CPU)
--------------------------                     ------------------------
gate: SmpReady (acquire-read, W05)             idle loop or poll point
gate: target in OnlineSet (W03)                    |
    |                                              v
    v                                          poll(): observe slot word
CAS slot word (release):                           (acquire)
  seq+1, kind, payload                          vs local last_seen
    |                                              |
    v                                          new sequence:
sev()  (broadcast wake)                        update last_seen, bump
                                               arrival/coalesced/kind
                                               counters, dispatch to the
                                               kind's registered consumer
                                               contract (W08 at P3)
```

There is no sender-side per-target state: a send is a pure function of the
target's slot word plus the two gates. All mutable slot state is either
sender-written (the packed event word) or receiver-private (counters and
`last_seen`), so no cross-CPU read-modify-write ever occurs except the
sender's CAS on the single event word.

## 2. Logical modules

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. Slot layout and encoding | packed word format, kind space, counter placement | W04 slot reservation | the layout contract | none (layout is normative) | slot placement/sizing (W04); consumers' payload meaning |
| B. Send path | gates, CAS protocol, wake, error outcomes | target id, kind, payload | Ok/errors; event visible to receiver | the event word (CAS) | what happens after arrival |
| C. Receive path | poll/wait, last-seen tracking, counters, dispatch to the kind owner's contract | own slot word | `Notification` values; counters | `last_seen`, arrival/coalesced/per-kind counters | consumer protocol logic (W08's) |
| D. Integration | kind registry rules, W02/W05 extension notes, evidence surfaces | consumer designs | citable integration map | none | catalog (W11); scheduler (P7) |

## 3. Objects and ownership

| Object | Count | Owner | Writers | Readers |
|---|---|---|---|---|
| Packed event word | one per CPU (in `NotificationSlot`) | W07 (contents); W04 (placement) | any online CPU, CAS-serialized (hardware) | the owning CPU (acquire) |
| `last_seen` sequence | one per CPU | W07 | the owning CPU only | the owning CPU |
| Arrival / coalesced / per-kind counters | per CPU | W07 | the owning CPU only | cross-CPU diagnostics read-only (W11/W12 surfaces) |
| Kind registry (kind → owning design) | one per stage | W07 (normative table) | design changes only | consumers, reviewers |

No locks exist in W07. Every field is single-writer except the event word,
whose writer-set race is resolved by the hardware CAS (the defined
concurrent-sender behavior). This is the W06 atomic-policy discipline
(AP-1 with the AP-2 CAS form), not a new ordering invention.

## 4. Slot layout (normative sketch)

```text
NotificationSlot (cache-line aligned; placement W04's, contents W07's)
┌──────────────────────────────────────────────┐
│ event_word: AtomicU64                        │  packed:
│   bits 31:00  sequence (u32, wrapping)       │   sender CAS target
│   bits 39:32  kind     (u8)                  │
│   bits 47:40  payload  (u8)                  │
│   bits 63:48  reserved (zero)                │
│ last_seen: u32            (receiver-private) │
│ arrivals_total: u32       (receiver-private) │
│ coalesced_total: u32      (receiver-private) │
│ arrivals_by_kind: [u32; 4] (receiver-private)│
└──────────────────────────────────────────────┘
```

Exact bit positions and the Rust encoding are implementation-record
material; the field set, widths, single-writer split, and wrapping
semantics are the contract. The layout is internal representation, never
serialized or guest-visible (Coding Guidelines). Sequence comparison uses
wrapping-distance arithmetic so u32 wrap is safe (documented); a receiver
that observes a wrap is indistinguishable from a very fast sender within
one wrap window — accepted and recorded, since u32 events per observation
gap is far beyond P3's scale.

## 5. Protocol mechanics (summary; contracts in 03/04)

- **Send:** CAS loop on `event_word`: read current (acquire), write
  `{seq+1, kind, payload}` with release. Two racing senders both
  eventually succeed (retry on CAS failure); the second writer's
  kind/payload overwrites the first's — the first is coalesced, and the
  sequence advance by 2 is how the receiver counts it.
- **Observe:** receiver reads `event_word` (acquire); if the sequence is
  ahead of `last_seen` by wrapping-distance d, then d−1 events were
  coalesced (`coalesced_total += d−1`), one event is delivered
  (kind/payload of the observed word), `arrivals_total += 1`,
  `arrivals_by_kind[kind] += 1`, `last_seen = seq`.
- **Wake:** `sev()` after a successful send (doorbell semantics even for
  kind ≥ 1; consumers decide whether they need it). Receivers in `wait`
  re-check on wake; spurious wakes are legal and must be tolerated.
- **Gates:** both checks are fail-closed reads (W05 phase via acquire;
  W03 `OnlineSet` snapshot). A refused send performs no slot write and no
  wake.

## 6. Failure model

- **Refused sends** (`NotReady`, `UnknownTarget`, `TargetNotOnline`,
  `InvalidKind`) are diagnosable, side-effect-free, and never retried by
  W07 — retry policy belongs to the consumer.
- **A target that never polls/waits** keeps its event pending in the slot
  indefinitely (no expiry at P3); the event is not lost until over-written
  by a later send (coalescing). This is the documented non-RPC limit, not
  a failure.
- **Slot corruption** (reserved bits nonzero, impossible sequence
  regression) is an invariant violation — fatal diagnostic path with CPU
  attribution per the P0-W14 classification.
- **WFE never waking** (platform anomaly) degrades the `wait` user to a
  hung idle — at P3 the coordinator/secondary boot paths still use bounded
  polls (W02/W05), so a wake failure is observable as boot stalls, not
  silent loss. Recorded limitation; the timer-based watchdog is P6's.

## 7. Consumer integration map

| Consumer | Surface consumed | Contract point |
|---|---|---|
| W08 (TLB transport) | kind 1 reservation; `notify` per target; `poll` consumption on targets; wake | [04](04-code-contracts-notification-api.md) §2, §6; [03](03-code-contracts-notification-slot.md) §3 |
| W11 (observability) | counters (arrivals/coalesced/per-kind), kind vocabulary, activity events | [03](03-code-contracts-notification-slot.md) §4; [04](04-code-contracts-notification-api.md) §4 |
| W12 (stress) | notify/poll/wait as stimulus; counters as explainable accounting; concurrent/self/invalid behaviors | [04](04-code-contracts-notification-api.md) §3–§5 |
| W13 (regression) | scenario definitions as matrix row inputs | [06-validation-and-handoff.md](06-validation-and-handoff.md) §3 |
| W14/P4 (handoff) | notify contract as the cross-CPU notification foundation | entry README handoff |
