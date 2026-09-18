# P3-W08 Code Contracts — Request Encoding and Target Consumption

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; exact bit positions and Rust paths are fixed in the
implementation record. The layout is internal representation — never
serialized, never guest-visible.

## 1. `TransportRequestSeq`

```text
Name and stability: TransportRequestSeq — newtype over u16, wrapping;
    internal; assigned by the initiator under the single-flight lock.
Purpose and caller: tags a request so superseded/stale slot state is
    detectable; callers: initiate ([04 §3](04-code-contracts-transport-initiator.md)),
    consume (§5 below).
Preconditions / postconditions: monotonic per boot under single-flight;
    wrap handled by equality comparison only (never ordering) — a target
    matches `control.seq == expected`, so wrap is safe.
Concurrency/allocation context: plain value.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic: plain newtype.
Validation: W08-DV01.
```

## 2. `TargetMask`

```text
Name and stability: TargetMask — newtype over a fixed-capacity bitset
    (one bit per dense LogicalCpuId, capacity = W01 inventory bound ≤ 8);
    internal.
Purpose and caller: expresses single-target (one bit) and mask/broadcast
    requests; callers: initiate, W12 stimuli, diagnostics.
Inputs / outputs: constructors — single(logical), from_iter(logicals),
    broadcast_minus_initiator(); queries — contains, is_empty, count,
    iteration in logical order; set algebra — intersect (with online
    set), union, complement-within-inventory.
Preconditions / postconditions: bit indices are valid logical ids only;
    construction from out-of-range ids is a checked error (never a panicking
    index on untrusted input — masks may come from caller-provided
    selections).
State and ownership change: none (value type).
Concurrency/allocation context: stack value; no allocation.
Errors and failure guarantee: construction errors name the offending id.
Security/authorization checks: n/a (host-only); the *authorization*
    boundary is the online-set reduction in initiate, not the type.
Logic: dense bitset; density invariant from
    [P3-W01](../p3-w01-cpu-topology-inputs/README.md) makes logical id →
    bit index total.
Validation: W08-DV01/DV02 (algebra + exclusion tests).
```

## 3. `TlbReceptionSlot` (contents contract)

```text
Name and stability: TlbReceptionSlot — contents owned by W08 from its
    init point; placement/sizing owned by
    [P3-W04](../p3-w04-per-cpu-runtime/README.md) (03 §5.2); internal;
    cache-line aligned; lifetime = whole boot.
Purpose and caller: the target's entire transport state; initiator
    touches control+descriptor on the Empty→Pending edge; target touches
    control on the Pending→Completed edge.
Inputs / outputs: fields —
    control: AtomicU32 { state: u8 {Empty=0, Pending=1, Completed=2},
                         seq: TransportRequestSeq (u16), reserved: u8 }
    descriptor: AtomicU64 (opaque; stable while Pending)
    completions: u32 (target-private count)
Preconditions / postconditions: after init (§4): state Empty, seq
    arbitrary-but-recorded (0), descriptor 0, reserved 0. Invariant —
    descriptor is written only before the Pending CAS and read only
    while Pending/Completed of the same seq.
State and ownership change: W04 zero-fill → W08 init → live for boot.
Concurrency/allocation context: no allocation; the only cross-CPU RMWs
    are the two CAS edges (initiator-owned and target-owned
    respectively); completions is single-writer.
Errors and failure guarantee: illegal control encodings are fatal-class
    (P0-W14) with CPU attribution.
Security/authorization checks: host-only.
Logic: structure per [02 §3](02-architecture-and-state.md) §3.
Validation: W08-DV01 (layout/encoding), W08-DV03 (state machine).
```

## 4. `init_slots` (transport)

```text
Name and stability: init_slots(areas: &PerCpuSet) -> Result<(), InitError>
    — internal; called once during global initialization
    (single-threaded, pre-release), ordered with W07's init per the boot
    sequence.
Purpose and caller: the W04→W08 contents handoff for the TLB slot.
Inputs / outputs: the W04 per-CPU set; Ok or Err naming CPU/slot.
Preconditions / postconditions: precondition — boot phase Bootstrap.
    Postcondition — every candidate slot matches §3's initial
    postcondition; contents ownership is W08's.
State and ownership change: W04 → W08.
Concurrency/allocation context: single-threaded; no allocation.
Errors and failure guarantee: invalid slot = fatal boot-critical; no
    partial init.
Security/authorization checks: n/a.
Logic: validate alignment; write initial control/descriptor/completions;
    re-validate.
Validation: W08-DV01.
```

## 5. `consume_pending_requests` (target side)

```text
Name and stability: consume_pending_requests() -> usize — internal;
    executed by a CPU for its own slot only.
Purpose and caller: the target half of the protocol; callers: the
    owning CPU's idle/poll path on W07 kind-1 wake, and — binding per
    W06 BW-4 — interleaved into any W08 wait loop the CPU runs
    ([04 §5](04-code-contracts-transport-initiator.md)).
Inputs / outputs: none; returns the number of requests consumed
    (0 or 1 at P3; the count keeps the contract total if P4 ever
    pipelines).
Preconditions / postconditions: precondition — caller is the slot owner
    (calls through `current()`'s area, never the cross-CPU table).
    Postcondition — if control read Pending{seq}: descriptor read
    (acquire), bound operation executed (P3: TransportNoop), control
    CAS Pending→Completed (release) for the same seq, completions
    incremented; returns 1. If Empty/Completed: returns 0, no writes.
State and ownership change: own slot per above.
Concurrency/allocation context: acquire loads; one CAS; no allocation;
    no lock; the bound operation is invoked with no lock held and no
    W06 ladder obligation (P3's placeholder does nothing; P4's binding
    inherits the same context rule).
Errors and failure guarantee: an illegal control encoding or a seq
    mismatch against the descriptor's pairing window is corruption —
    fatal diagnostic; consumption never "skips" a Pending request
    without either completing it or taking the fatal path.
Security/authorization checks: host-only; the request is hypervisor-
    originated by construction (no guest path), so no untrusted-input
    parsing occurs at P3 — recorded so P4 knows where the trust boundary
    sits when it defines descriptor content.
Logic (pseudocode):

    consume_pending_requests():
        slot = current().tlb_slot
        c = slot.control.load(Acquire)
        if c.state != Pending: return 0
        desc = slot.descriptor.load(Acquire)      # stable while Pending
        bound_operation(desc)                     # P3: TransportNoop
        if slot.control.compare_exchange(
               pack(Pending, c.seq), pack(Completed, c.seq),
               Release, Acquire).is_err():
            fatal_invariant()                     # single-flight makes
                                                  # contention impossible
        slot.completions += 1
        return 1

Validation: W08-DV03 (consume/complete ordering; single consumption per
    request), W08-DV06 (non-consuming target → timeout path).
```

## 6. `TransportNoop` (the P3 bound operation)

```text
Name and stability: TransportNoop — internal placeholder binding; the
    ONLY bound operation authorized at P3.
Purpose and caller: makes the transport executable and testable without
    implementing any invalidation; caller: consume_pending_requests.
Inputs / outputs: the descriptor word; ignored (documented).
Preconditions / postconditions: none; cannot fail; performs no TLBI, no
    barrier, no cache maintenance.
State and ownership change: none.
Concurrency/allocation context: none.
Errors and failure guarantee: none.
Security/authorization checks: none.
Boundary statement: replacing this binding with a real invalidation
    operation is P4's design (descriptor meaning + operation + barriers),
    arriving through [P3-W14](../p3-w14-p4-smp-handoff/README.md); any P3
    code that decodes `desc` or executes an architectural invalidate is a
    scope violation to stop at review.
Validation: W08-DV07 (boundary review: descriptor never decoded).
```
