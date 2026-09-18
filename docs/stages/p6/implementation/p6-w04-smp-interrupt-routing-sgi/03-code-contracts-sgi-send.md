# P6-W04 Code Contracts — SGI Targets, Send, and Accounting

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W04 detailed design](README.md).  
**Convention:** checklist §3 contract template. System-register access
follows the W02 surface rules
([W02 03](../p6-w02-physical-gic-bring-up/03-code-contracts-register-access.md) §5:
write-and-verify posture already established; SGI1R is a fire-and-forget
write with no read-back). Pseudocode is design logic.

## 1. `SgiTarget` construction and decomposition

```text
Name and stability: SgiTarget { One, ExplicitList, AllOnlineExcludingSelf }
  with constructors validating against the P3 online ∩ W02 LocalReady
  predicate (pure constructors return Result; no fallthrough forms)
Purpose and caller: the only way to name SGI destinations; callers are P6
  consumers (validation scenarios, event kick) and, via W11, test paths
Inputs / outputs: PcpuId values / set → validated target
Preconditions / postconditions: One(p) requires p eligible; ExplicitList
  requires every member eligible and the set non-empty; the IRM form
  requires ≥ 1 other online-ready pCPU to exist (a lone-CPU system
  rejects the form with NoOtherTarget)
Errors: TargetIneligible(pcpu), EmptyTargetSet, NoOtherTarget
Validation: unit tests per form against synthetic ledger views
```

```text
Name and stability: fn decompose(target) -> BoundedVec<Sgi1rEncoding>
  (P6-internal; deterministic affinity-ordered decomposition per
  [02](02-architecture-and-state.md) §2.1)
Purpose and caller: turn a validated target into the exact SGI1R write
  sequence; used by send_sgi
Postconditions: encodings are non-overlapping and cover exactly the
  validated set (IRM form: exactly one encoding with the IRM bit);
  bound: the decomposition of any supported pCPU count fits a fixed
  capacity derived from the declared CPU count
Validation: decomposition unit tests incl. affinity-boundary cases
```

## 2. `send_sgi`

```text
Name and stability: fn send_sgi(target: &SgiTarget, sgi: SgiId,
  sender: PcpuId) -> Result<SendReceipt, SendError> (P6-internal; the
  only SGI emission path in P6)
Purpose and caller: send a Host SGI with target attribution; callers are
  the P6 event kick, W11 scenarios, and the P3 transport's SGI row if P3
  routes its primitive through this surface (its choice; the transport
  semantics remain P3's)
Inputs / outputs: validated target + SGI ID + sender identity →
  SendReceipt { writes: count, encoding summary } | SendError
Preconditions / postconditions:
  - executing at EL2 on `sender`, which is LocalReady (Group 1 enabled)
  - `sgi` is an assigned partition row (unassigned rows are rejected —
    UnassignedSgi — so accidental sends on reserved IDs are impossible)
  - success: every encoding written to ICC_SGI1R_EL1 in decomposition
    order; send counter incremented per write; no read-back required
    (architectural fire-and-forget)
  - failure: zero writes occurred (validation precedes emission)
State and ownership change: hardware SGIs become pending at the targeted
  GICRs (or delivered to the sending pCPU for self-target, looping
  through the same path per [README decision 2](README.md))
Concurrency/allocation context: no lock; single volatile sysreg write per
  encoding; fixed-capacity encoding buffer (no allocation); callable from
  thread and (documented) IRQ contexts of consumers that the W03
  obligations permit
Errors: TargetIneligible, EmptyTargetSet, NoOtherTarget, UnassignedSgi
Security/authorization checks: partition-row check only; this is a
  Host-internal mechanism (no Guest reachability; Guest SGI authorization
  is P5/W07 scope)
Logic:
  encodings = decompose(target)
  for e in encodings: write ICC_SGI1R_EL1 = e.value   # volatile, ordered
                                                      # by program order;
                                                      # no barrier required
                                                      # between encodings
                                                      # for correctness of
                                                      # targeting (each is
                                                      # independently
                                                      # routed); a final
                                                      # dsb() if the caller
                                                      # needs send-before-
                                                      # flag publication
  send_counters[sender][sgi] += encodings.len()
  Ok(SendReceipt { writes: encodings.len() })
Validation: W04-DV04 (CPU0→CPU1), DV05 (reverse + multi-target), DV02
  (form/decomposition review)
```

## 3. Receipt side

```text
Name and stability: SgiPartition consumer registration (per
  [02](02-architecture-and-state.md) §2.2) via the W03
  register_consumer surface with the row's SgiId
Purpose and caller: bind each assigned row to its owning consumer at
  that consumer's init (P3 transport row, P6 generic-event row, W11
  validation row)
Contract notes:
  - exactly one consumer per row (W03 enforces); unassigned rows must
    never be registered or enabled
  - the consumer runs under W03's obligations (bounded, no blocking);
    the generic-event row's consumer is the per-CPU event-state check
    whose content belongs to its owning design (W05/W07 consumers and
    later P7), not to W04
  - completion is W03's combined EOI; W04 adds no completion logic
Validation: registration evidence in W04-DV04; unassigned-row rejection
  unit test
```

## 4. Accounting contracts

```text
Name and stability: SendCounters (per sending pCPU, per SGI ID; monotone)
  fn accounting_view() -> DriftReport (P6-internal; sampled, non-IRQ
  context; reads send totals and W03 per-pCPU receipt counters)
Purpose and caller: make P6-V04/V05 target attribution determinate and
  inspectable; W13 consumes the view for evidence
Preconditions / postconditions: view reflects a quiescent sample (no
  concurrent-send fence is attempted; sampling semantics documented as
  eventually-consistent across repeated samples)
Errors: none; divergence is data (DriftReport rows), not failure
Logic:
  for each assigned sgi row:
    sent    = Σ sender counters (per decomposition write)
    receipts= Σ pCPU W03 acked counters for that SGI ID
    expected_drift = Σ over failed-target events recorded since boot
    if sent != receipts + expected_drift -> DriftRow { labeled unexpected }
Validation: W04-DV05 accounting review; synthetic drift unit test
```

Rationale (recorded): no timeout or retransmission is defined. An SGI to
a ready target is delivered by the GIC or the platform is broken; a broken
platform is a diagnostic, not a retry policy. This keeps W04 free of
message-layer semantics the plan excludes.

## 5. Explicitly not authorized here

No SGI priority programming, no Group-0 SGI, no Guest-targeted send, no
GICD_SGIR (v2) path, no direct GICR SGI pending manipulation to fake
delivery (test injection uses real sends through this API), and no
scheduler-named API (no `reschedule()`, no `wake()` — consumers wrap the
mechanism under their own designs).
