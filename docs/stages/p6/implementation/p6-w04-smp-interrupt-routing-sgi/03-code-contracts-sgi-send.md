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
  SendReceipt { writes: count, intended_targets: bounded set,
    target_attempts: count, encoding summary } | SendError
Preconditions / postconditions:
  - executing at EL2 on `sender`, which is LocalReady (Group 1 enabled)
  - `sgi` is an assigned partition row (unassigned rows are rejected —
    UnassignedSgi — so accidental sends on reserved IDs are impossible)
  - success: every encoding written to ICC_SGI1R_EL1 in decomposition
    order; separately account one accepted call, encoding writes and intended
    target attempts; this receipt does not prove remote delivery or completion;
    no read-back required
    (architectural fire-and-forget)
  - failure: zero writes occurred (validation precedes emission)
State and ownership change: emit the validated SGI requests; remote pending,
  acknowledgement and completion are not certified by this return. Self-target
  uses the same emission path per [README decision 2](README.md)
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
  accepted_calls[sender][sgi] += 1
  register_writes[sender][sgi] += encodings.len()
  for t in validated_target_set: target_attempts[sender][sgi][t] += 1
  Ok(SendReceipt { writes: encodings.len(),
                  intended_targets: validated_target_set,
                  target_attempts: validated_target_set.len() })
  # Independent bounded atomic telemetry updates per W13; not a coherent snapshot.
  # Counter overflow is flagged, never wrapped into an exact-count assertion.
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
Name and stability: SgiAccountingView (internal diagnostic value), produced by
  accounting_view(); replaces the mixed-unit DriftReport proposal.
Purpose and caller: expose units and target attribution for W11/W13 and
  P6-V04/V05 without inventing message reliability.
Inputs / outputs: per-sender accepted_calls, register_writes, per-target
  target_attempts, validation_rejections, and W03 per-target acked/completed;
  output labels units, run/epoch, target set, sampling mode and overflow status.
Preconditions / postconditions: normal sampling is concurrent/approximate;
  it is not called quiescent without an external scenario handshake. No
  subtraction across unlike units, automatic loss count or coalescing estimate.
State and ownership: sender counters belong to W04; receiver ack/completion
  counters remain W03-owned. No second receipt mirror becomes authoritative.
Concurrency/allocation context: non-IRQ sampling over bounded tables; send-side
  accounting supports thread/IRQ preemption through the W13 atomic-counter
  contract. Per-pCPU placement alone does not prevent interrupted updates.
Errors and failure guarantee: rejected preflight publishes zero writes/attempts;
  concurrent sampling, overflow or failed targets make exact reconciliation
  unavailable, not automatic hardware-failure evidence. Never erase a discrepancy
  by adding an invented expected_drift allowance.
Security/authorization: diagnostics do not authorize retry, target removal or
  resource release; no Guest-visible or frozen ABI is added.
Logic: copy unit-tagged samples from the authoritative counters. By default
  report ObservedOnly. A separate controlled scenario may evaluate per-target
  target_attempt deltas against ack/completion deltas only under amendment 07's
  exclusive, serialized and quiesced conditions; otherwise ExactUnavailable.
Validation: W04-DV02/DV05 with unequal encoding/target counts, concurrent same-ID
  sends, sampling races, failed targets and counter exhaustion.
```

[Amendment 07](07-accounting-units-remediation.md) defines the counter units,
controlled comparison premises and source basis. An SGI is a signal, not one
queued message per write. A send return is neither remote acknowledgement nor
consumer-work completion. No timeout/retransmission is added to send_sgi;
validation harnesses use bounded observation and report failed/inconclusive
scenarios rather than asserting that any discrepancy proves broken hardware.

## 5. Explicitly not authorized here

No SGI priority programming, no Group-0 SGI, no Guest-targeted send, no
GICD_SGIR (v2) path, no direct GICR SGI pending manipulation to fake
delivery (test injection uses real sends through this API), and no
scheduler-named API (no `reschedule()`, no `wake()` — consumers wrap the
mechanism under their own designs).
