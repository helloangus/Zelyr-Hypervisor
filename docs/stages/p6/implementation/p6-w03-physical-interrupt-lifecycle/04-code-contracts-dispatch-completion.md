# P6-W03 Code Contracts — Dispatch, Completion, Registration, Bounds

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W03 detailed design](README.md).  
**Convention:** checklist §3 contract template. Pseudocode is design logic.

## 1. `handle_irq_entry`

```text
Name and stability: fn handle_irq_entry(entry: &P1EntryContext) (P6-
  internal; the single dispatch target the P1 IRQ vector entry calls;
  installed by the [workflow](05-implementation-workflow.md) integration
  step)
Purpose and caller: the bounded Host IRQ lifecycle for one vector entry;
  called by the P1 exception path only
Inputs / outputs: P1 entry context (read-only) → outcomes recorded;
  returns to the P1 exit path
Preconditions / postconditions:
  - executing on a LocalReady pCPU, interface enabled per W02 posture
  - on return: every acknowledged assigned ID completed exactly once
    (INV-B of [02](02-architecture-and-state.md) §5); counters updated;
    no consumer left mid-invocation
State and ownership change: hardware active/priority state advances per
  acknowledge/complete pairs; stats records increment
Concurrency/allocation context: IRQ context; no allocation, no blocking,
  no lock; bounded iterations (DISPATCH_BOUND, design-fixed small value,
  rationale: keep worst-case entry time determinate; remainder is
  re-delivered by hardware because it stays pending)
Errors and failure guarantee: no exceptional exit; all outcomes are the
  named rows of [03](03-code-contracts-identification.md) §4
Security/authorization checks: identification validates before any
  registry access; consumer invocation passes immutable context only
Validation: W03-DV03/DV04/DV05/DV07
```

### Pseudocode

```text
depth = 0
loop:
  if depth == DISPATCH_BOUND:
      stats.bound_exit(pcpu, depth); emit irq.bound_exit   # INV-C exit
      return                                               # between iters
  classification = classify_acknowledge(supported)         # §3 of 03
  match classification:
    NoPending     -> stats.spurious(pcpu); return          # only no-EOI exit
    ReservedBand(v)-> stats.reserved(pcpu, v); emit rate-limited; continue
    OutOfSupported(v)-> stats.unknown(pcpu, v); emit rate-limited
                       write EOI(v)                        # complete as data
                       continue
    Assigned(id)  ->
      slot = registry.peek(id)                            # acquire read
      match slot:
        Some(consumer) ->
          stats.acked(pcpu, id)
          consumer.invoke(&HostIrqContext { pcpu, entry, id, depth })
          # consumer returns → completion obligation below (INV-B)
        None ->
          stats.consumerless(pcpu, id); emit rate-limited
      write EOI(same acknowledged value)                  # once, here only
      stats.completed(pcpu, id)
      depth += 1
```

Post-condition note: the EOI write is the *only* ICC_EOIR write site in
P6 (single completion owner; W04 SGIs and W05 PPIs complete through this
same loop because their consumers run inside it).

## 2. Consumer contract

```text
Name and stability: ConsumerCallback = fn(&HostIrqContext) (P6-internal;
  the minimal stage-local surface per [README decision 5](README.md))
Purpose and caller: the subsystem's in-context reaction; called only by
  handle_irq_entry
Inputs / outputs: immutable context → none (no return protocol in P6;
  the subsystem's own state carries its results)
Obligations (documented as part of the registration API; violations are
  reviewed defects, not runtime-checked):
  - bounded work; no allocation; no blocking; no long busy-waits
  - no enabling/disabling other interrupts except through the registry
    API from a context where that is defined (own disable from own
    handler is the supported pattern)
  - no assumption of nesting or re-entrancy for its own ID (flat
    handling; the same ID cannot re-enter before completion)
  - may re-arm hardware sources (e.g. timer compare) as its design owns
Validation: consumer correctness is owned by the consumer's package
  (W05/W08 register their own review evidence); W03 validates only the
  invocation mechanics
```

## 3. Registry contracts

### 3.1 `register_consumer`

```text
Name and stability: fn register_consumer(id: PhysicalIntId, owner:
  OwnerTag, cb: ConsumerCallback) -> Result<(), RegisterError> (P6-
  internal; called by owning subsystems during their initialization,
  never from IRQ context)
Preconditions / postconditions:
  - slot for id empty (double registration → Err(AlreadyRegistered));
    W02's ledger shows the calling pCPU LocalReady for PPIs/SGIs; for
    SPIs the distributor is DistributorReady
  - success: slot published (release) while the ID is still disabled;
    the caller enables afterwards via enable_id (registration-before-
    enable, [README decision 4](README.md))
Errors: AlreadyRegistered, IdUnsupported, NotReady (ledger)
Concurrency: registration lock (P3 irq-save class) around slot write;
  slot immutable after publication; deregistration intentionally absent
  (Reserved) — the API's absence is the design
Security/authorization checks: owner tag recorded for diagnostics; P6 has
  no capability layer on physical registration (Host-internal surface;
  Guest-facing authorization is P5/W07 scope)
Validation: W03-DV04; registration ordering test (slot visible before
  enable observed by a fake delivery)
```

### 3.2 `enable_id` / `disable_id`

```text
Name and stability: fn enable_id(id) / fn disable_id(id) -> Result<(),
  EnableError> (P6-internal; the only runtime enable/disable path)
Purpose and caller: consumers arm/disarm their own sources (W05 timer
  PPIs, W08 maintenance PPI, W04 SGI test consumers, W11 scenarios)
Mechanics by class (all through the W02 access surface):
  SGI/PPI: local GICR ISENABLER0/ICENABLER0 bit for id, on the calling
           pCPU (must own the ID; cross-pCPU PPI enable is an error)
  SPI:     GICD ISENABLER/ICENABLER word under the distributor-scoped
           lock; RMW preserving other bits; supported-range check
Postconditions: enable after slot published; disable leaves pending
  latched state alone (pending observation stays W02/W12 diagnostic
  business, not silent clearing — runtime clear is Reserved)
Errors: IdUnsupported, NotOwning, RegisterError propagation
Validation: W03-DV04; masking behavior via W11 scenarios
```

### 3.3 Repeated and simultaneous events (documented policy)

Repeated same-ID arrival while active: hardware latches
(active-and-pending); after the EOI the interface re-signals; the next
loop iteration or entry delivers it. No software latching, no loss, no
special-casing. Simultaneous different-ID arrival: GIC priority order
(identical priorities: unspecified); W03 counts and delivers whatever the
GIC presents, per iteration. These two paragraphs are the contract W11's
VG-IRQ repeated scenarios exercise.

## 4. Consumerless / unknown outcome record

```text
Name and stability: fn record_unowned(kind, pcpu, raw_or_typed) (P6-
  internal; irq-stats path used by the loop's None/OutOfSupported rows)
Purpose and caller: make every unconsumed or anomalous acknowledge
  diagnosable and bounded (task book P6-V20)
Postconditions: counters increment; event emitted rate-limited (first
  occurrence per ID per window; the window/threshold constants are
  design-fixed and recorded in the implementation record)
Rationale: a consumerless enabled interrupt is a subsystem bug or
  firmware residue; it must not wedge (it completes) and must not flood
  (it rate-limits). W12 builds its isolation evidence on these outcomes.
```

## 5. Storm-containment bounds

```text
Name and stability: DISPATCH_BOUND (const), rate-limit window and
  threshold constants (P6-internal; values fixed at implementation from
  this design's rationale: entry work must stay bounded and small; the
  exact bound is recorded with its measurement in the implementation
  record, and W13's latency baseline (P6-V25) measures its effect)
Behavior: bound exit leaves pending work to hardware re-delivery (INV-C);
  no masking, no priority change, no deferral queue exists in P6
Non-goal (explicit): production IRQ-DoS resistance; W12's storm rows are
  smoke evidence only (task book P6-V22 wording)
Validation: W03-DV07 (bounded synthetic storm on the fake backend),
  W12-DV rows at integration
```

## 6. Stats contracts

```text
Name and stability: IrqStatsRecord (per pCPU, in P3 per-CPU storage) with
  monotone counters and once-per-crossing threshold events
Purpose and caller: per-pCPU diagnosability; W13 correlation; W12
  robustness baseline
Concurrency: owned by the counting pCPU; no cross-CPU writes; sampling by
  W13 reads without mutation
Validation: counter-accounting unit test (every classification outcome
  increments exactly its counter; loop invariants hold under synthetic
  sequences)
```
