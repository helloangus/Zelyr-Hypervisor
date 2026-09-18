# P3-W11 Code Contracts — Counter Block

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W11 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths are reserved to the workspace-owning
design. The counter layout is internal representation, not an ABI: it is
never serialized, persisted, or exposed to a guest (Coding Guidelines rule).

## 1. `SmpCounterId`

```text
Name and stability: SmpCounterId — exhaustive enum of the W11 counters;
    internal; fixed discriminants equal to slot index (see §2).
Purpose and caller: compile-time-complete naming of every counter;
    callers: increment API, snapshot, dump, slot map.
Inputs / outputs: none (unit variants).
Preconditions / postconditions: discriminants < COUNTER_CAPACITY; the
    capacity constant equals the W04-recorded block capacity.
State and ownership change: none.
Concurrency/allocation context: Copy; no allocation; no synchronization.
Errors and failure guarantee: none.
Security/authorization checks: none.
Logic: enum with one variant per §2 row; a `COUNT` associated constant.
Validation: W11-DV02 capacity-fit and slot-map tests.
```

## 2. Counter catalog

| Slot order | Id | Meaning | Incremented by | Counting rule |
|---|---|---|---|---|
| 0 | `lifecycle_transitions` | lifecycle transitions recorded for this CPU's registry record | W03 transition operations (this CPU's record) | +1 per transition, including failures where the table moves a record to `Failed` |
| 1 | `start_requests_sent` | CPU-start requests this CPU issued | W02 requester (boot CPU only) | +1 per CPU_ON issued, success or refusal |
| 2 | `start_outcomes_entered` | secondaries observed entering EL2 on this CPU's entry path | W02 secondary entry | +1 per entry that passes identity confirmation |
| 3 | `start_outcomes_failed` | start failures this CPU reported (as requester or entrant) | W02 outcome paths | +1 per terminal failed outcome |
| 4 | `notifications_sent` | notification sends performed by this CPU | W07 send path | +1 per send attempt, including refused/invalid targets |
| 5 | `notifications_received` | notifications received by this CPU | W07 reception path | +1 per accepted reception (refusals are not receptions) |
| 6 | `tlb_requests_sent` | TLB-transport requests issued by this CPU | W08 request path | +1 per request attempt |
| 7 | `tlb_requests_received` | TLB-transport requests received | W08 reception path | +1 per received request |
| 8 | `tlb_completions` | acknowledgements/completions observed | W08 completion path | +1 per completion observed for this CPU's requests |
| 9 | `contention_events` | acquisitions observed contended at a W06-designated site | W06 observation call | +1 per contended acquisition |
| 10 | `contention_wait_iterations` | accumulated poll/wait iterations before acquisition | W06 observation call | += iterations (single add at acquisition, not per iteration) |
| 11 | `contention_max_wait_iterations` | high-water of single-acquisition wait iterations | W06 observation call | store-max (see §4 contract) |
| 12.. | reserved slots | untouched until a catalog change | — | W04 capacity minus used slots; adding counters is a W11 design change |

Notes: slot 3 counts failures *reported by* the CPU (attribution rule: a
counter lives where the activity ran); the requester-side and entrant-side
failures are distinguishable by events, not by more counters. Refused
notification sends count in slot 4 because the attempt ran on this CPU —
accounting identities used by W12 are defined over both counters plus the
refusal events, and are stated in W12's scenario contract, not here.

Wrap analysis: the largest plausible per-boot increment source is
`contention_wait_iterations` (poll loops). At any rate a P3 boot can
sustain, the u64 bound is unreachable by many orders of magnitude; the
atomic add's defined wrap behavior is the documented last resort and
wrap-alarm monitoring is Reserved (entry README scope classification).

## 3. `increment`

```text
Name and stability: increment(id: SmpCounterId) — internal;
    inline-able hot-path helper.
Purpose and caller: the only mutation path for any counter; callers are
    the owning packages' code running on the owning CPU (W02/W03/W06/W07/W08
    at their designated points).
Inputs / outputs: counter id; no return value (infallible).
Preconditions / postconditions: the caller executes on the CPU that owns
    the target block (enforced by implementation addressing through the
    CPU-local mechanism, not by an argument); postcondition — the slot is
    advanced by exactly one even under same-CPU exception reentrancy.
State and ownership change: one u64 slot +1 (wrapping add defined).
Concurrency/allocation context: single atomic fetch-add, Relaxed;
    no allocation; no lock; no call-out — legal in exception/interrupt
    context.
Errors and failure guarantee: none; an unrepresentable call (cross-CPU) is
    impossible by construction — the API addresses through the caller's
    own area only.
Security/authorization checks: none (not guest-reachable).
Logic (pseudocode):

    increment(id):
        area = current()                       # W04 accessor
        slot = area.counters.slot(id)          # fixed offset map
        slot.fetch_add(1, Relaxed)

Validation: W11-DV02 reentrancy test (increment interrupted by a
    simulated exception that also increments; total = 2).
```

## 4. `record_contention` (W06 seam)

```text
Name and stability: record_contention(site: ContentionSite, wait_iterations:
    u64) — internal; the contention-observation seam.
Purpose and caller: lets [P3-W06](../p3-w06-concurrency-synchronization/README.md)
    report a contended acquisition without knowing counter mechanics;
    caller is W06 at its designated sites, after acquisition completes
    (never while holding the lock).
Inputs / outputs: a site identifier (exhaustive enum shared with W06:
    boot rendezvous poll, registry record poll, notification reception
    poll, TLB reception poll, plus a Reserved tail) and the iteration
    count of that single acquisition.
Preconditions / postconditions: caller on its own CPU; wait_iterations >
    0 (a zero value is a W06 contract violation — uncontended acquisitions
    are not reported); postcondition — contention_events +1,
    contention_wait_iterations += wait_iterations,
    contention_max_wait_iterations raised to max(old, wait_iterations).
State and ownership change: three slots as above.
Concurrency/allocation context: same as increment (§3); the store-max is
    a fetch-max loop, correct under same-CPU reentrancy.
Errors and failure guarantee: none.
Security/authorization checks: none.
Logic (pseudocode):

    record_contention(site, iters):
        increment(contention_events)            # site recorded via event, §4 of catalog file
        slot = current().counters.slot(contention_wait_iterations)
        slot.fetch_add(iters, Relaxed)
        max_slot = ...slot(contention_max_wait_iterations)
        loop: old = max_slot.load(Relaxed)
              if old >= iters: break
              if max_slot.compare_exchange_weak(old, iters, Relaxed, Relaxed).is_ok(): break

Validation: W11-DV02 unit tests incl. high-water update under reentrancy.
```

The per-site breakdown is carried by the contention *event*
([04](04-code-contracts-event-catalog.md) §4), not by per-site counters —
capacity is reserved for accounting totals, and per-site totals are
derivable from `DebugOnly` events when they are enabled.

## 5. `snapshot` / `aggregate`

```text
Name and stability: snapshot() -> SmpCounterSnapshot; and
    SmpCounterSnapshot::aggregate() -> SmpCounterSums — internal;
    diagnostic-time read surface.
Purpose and caller: the evidence surface; callers: boot diagnostics at
    the SMP-ready dump point, [P3-W12](../p3-w12-smp-stress-failure-tests/README.md)
    checkpoints, [P3-W13](../p3-w13-qemu-smp-regression/README.md)
    assertions, fatal-path consumers via W09's contract.
Inputs / outputs: none; snapshot returns per-CPU rows (attribution triple
    from the area header + all slot values); aggregate returns per-slot
    sums over the rows.
Preconditions / postconditions: precondition — caller is outside
    exception context and global initialization is published (W05 phase
    check); rows cover exactly W03's `OnlineSet` unless `include: All`
    is requested for diagnostics (failed/quarantined CPUs renderable but
    excluded from sums by default). Postcondition — each row's slot
    values are atomic per-slot reads; no coherence is claimed across
    slots or CPUs (architecture §5).
State and ownership change: none (read-only; fixed-capacity result,
    stack-constructible, no allocation).
Concurrency/allocation context: acquire loads through the W04 read-only
    lookup table; iteration bounded by the topology bound (≤ 8).
Errors and failure guarantee: none; an unreadable area header is an
    invariant violation taking the fatal diagnostic path (it would mean
    W04's postconditions broke).
Security/authorization checks: none (diagnostic surface, not
    guest-reachable; never serialized).
Logic (pseudocode):

    snapshot():
        require_published()                     # W05 gate
        for logical in online_set().iteration_order():   # W03 OnlineSet
            area = lookup(logical)              # W04 read-only table
            row = Row { attribution: area.header.attribution(),
                        slots: read each slot once, acquire, fixed order }
        return Snapshot { rows }

Validation: W11-DV02 aggregation tests over fake blocks; W11-DV05 capture.
```

## 6. `render_counter_dump`

```text
Name and stability: render_counter_dump(sink) — diagnostic output per the
    P0-W12 governance; internal.
Purpose and caller: one line per rendered CPU row (attribution triple,
    non-zero counters, and zero counters as declared zeros) plus the
    aggregate line; callers: boot sequence at the SMP-ready dump point,
    on-demand diagnostics, fatal path if W09 consumes it.
Inputs / outputs: a diagnostic sink per the channel contract; no return.
Preconditions / postconditions: same preconditions as snapshot;
    rendering is a pure function of the snapshot taken.
State and ownership change: none.
Concurrency/allocation context: diagnostic time; formatting per the
    channel's no-allocation rules for the crash-dump channel (rendering
    must remain legal on the fatal path per the P0-W12 crash rules).
Errors and failure guarantee: none.
Security/authorization checks: content limited to typed ids, roles, and
    counts — no platform names, no addresses beyond what W03/W04 dumps
    already render.
Logic: snapshot; for each row render "logical=… hardware=… role=…
    counters…"; render aggregate line; ordering by logical id.
Validation: W11-DV05 capture review (P3-V11 evidence surface).
```
