# P3-W08 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tracked
tree. Prerequisite checks: the W04 slot reservation, W07's kind-1
reservation, W03's `OnlineSet`, W05's phase gate, and the W06 lock/ladder
contracts are available as agreed designs; the host-side test entry
(P0-W08 baseline) is available.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W04 `TlbReceptionSlot` is absent/undersized/misaligned, or W07's
  kind space landed without the reservation — cross-design conflicts;
  resolve with the owning design; do not invent substitute storage or a
  second wake path;
- W03/W05 surfaces differ from the assumed shapes
  ([01 §1.2](01-scope-and-foundations.md)) — resolve with their owners;
  no W08-private copies;
- implementing the transport appears to need a second in-flight request,
  a timer, a queue, or descriptor interpretation — design error; raise
  it (README decisions 1, 2, 5);
- a consumer (P4) asks for transport semantics changes — that is a W08
  design change made with the P4 design via
  [P3-W14](../p3-w14-p4-smp-handoff/README.md), not a local fork.

## 2. Ordered implementation steps

### Step 1 — mask type and slot contents

Target: the module the approved build design assigns.

Work: implement `TargetMask` and the `TlbReceptionSlot` contents +
transport `init_slots` per
[03-code-contracts-transport-request.md](03-code-contracts-transport-request.md)
§2–§4. Wire the init call into the boot sequence's global-init region
relative to W07's init per the boot sequence's ordering.

Acceptance: mask algebra total over the inventory (no out-of-range
panics); init validates all slots and records the W04→W08 handoff.

Failure/blocker: a slot failing validation is fatal boot-critical —
diagnose the W04 integration; do not skip CPUs.

Evidence: implementation record.

### Step 2 — target consumption and the no-op binding

Target: same module.

Work: implement `consume_pending_requests` and `TransportNoop` per
[03](03-code-contracts-transport-request.md) §5–§6, with the W06 AP-2
ordering pattern on the control CAS.

Acceptance: consumption completes a Pending request exactly once;
Empty/Completed reads write nothing; the seq-mismatch corruption path is
fatal and testable with a slot-fake.

Failure/blocker: any temptation to decode the descriptor is a boundary
violation — stop (README decision 1).

Evidence: verification record (W08-DV03 partial).

### Step 3 — initiation lock and request publication

Target: same module.

Work: implement the single-flight `SpinLock` (constructed with
`LadderClass::Infrastructure`) and the publish loop of
`initiate_transport` per
[04-code-contracts-transport-initiator.md](04-code-contracts-transport-initiator.md)
§3, including the W07 wake per target.

Acceptance: validation errors are side-effect-free; publication under the
lock sets Pending{seq} with the descriptor stable-before-pending (order
asserted in tests); two concurrent initiators serialize.

Failure/blocker: a CAS failure on Empty→Pending means slot-state
corruption or a double-publication bug — fatal per contract; do not
retry.

Evidence: verification record (W08-DV01, DV05 partial).

### Step 4 — collection, timeout, supersession

Target: same module.

Work: implement the bounded collection loop with the BW-4 interleaving,
`COLLECT_BOUND`, the `TimedOut{acked, unacked, excluded}` result, and the
supersession path (a fresh request after timeout).

Acceptance: a non-consuming target yields TimedOut with the exact unacked
set and a diagnosable slot state; a superseding request completes
normally afterward; accounting invariants
([04 §6](04-code-contracts-transport-initiator.md)) hold in every test.

Failure/blocker: a hang (bound not effective) is a contract violation —
fix the loop, not the bound.

Evidence: verification record (W08-DV06).

### Step 5 — scenario suite and concurrency evidence

Target: host-side tests per the P0-W08 baseline.

Work: implement the DV02/DV04/DV05 scenario suite: single/mask/broadcast
selection; exclusion reporting; concurrent initiators at the declared
thread counts; initiator-that-is-also-target (self-exclusion +
reactive-wait progress); induced non-consuming target.

Acceptance: all scenarios pass within declared limits with exact
accounting; the reactive-wait interleaving is demonstrably exercised
(the initiator-consumes-own-request case).

Failure/blocker: a deadlock observed in the harness is the highest-class
finding — capture, diagnose against
[02 §5](02-architecture-and-state.md), fix the protocol.

Evidence: verification record (W08-DV02, DV04, DV05).

### Step 6 — boundary review and closure

Work: run the matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md); perform the
DV07 boundary review (descriptor never decoded; no TLBI/barrier/cache
code; no Stage-2 types); confirm the handoff checklist. Record
implementation decisions in
`../p3-w08-tlb-shootdown-transport-record.md` and evidence in
`../../verification/p3-w08-tlb-shootdown-transport-verification.md` only
for what was actually performed; completion is claimed only in the
verification record, only for what was run.

## 3. Deferred-to-consumer obligations (recorded, not performed here)

- P4 (via W14) defines the descriptor interpretation and the real bound
  operation with its barriers, and decides broadcast-TLBI vs transport
  per operation; it may propose pipelining as a W08 design change.
- W11 owns the catalog for transport activity events and counter
  aggregation.
- W12/W13 exercise the transport at SMP scale and repeated boots; W08's
  host-side evidence is the base layer.
- W15 carries the semantic-gap statement into the stage documentation.
