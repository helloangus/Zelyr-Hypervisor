# P7-W03 Placement Code Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W03 detailed design](README.md).

Contracts follow the implementation-design checklist §3 template. Names are
internal, stage-local design names; mechanical renaming is allowed, semantic
change is a new design decision. `PcpuId` and `VcpuId` denote the logical
identity newtypes delivered by the P3/P4 contracts (P7-IN-04/05) — this
design consumes them and does not redefine them.

## §1 Data structures

### Contract 1.1 — `CpuSet`

```text
Name and stability: type CpuSet — fixed-capacity bitset over the logical
  pCPU id space; internal; `Copy` where the capacity permits.
Purpose and caller: the only representation of multi-pCPU eligibility;
  used by PlacementMode, the ledger index, validation, and the picker seam.
Inputs / outputs: construction from a bounded id iterator / single id;
  operations: contains(id), insert(id) -> Result<(), CapacityExceeded>,
  remove(id), is_empty(), count(), intersects(&CpuSet) -> bool,
  union/intersection builders, iteration in ascending id order.
Preconditions / postconditions: all ids < CAPACITY; CAPACITY is a
  compile-time constant derived from the P2/P3 topology facts (P7-IN-03/04)
  and fixed per build configuration; empty set is a valid value but never a
  valid placement (validation rejects it).
State and ownership change: value semantics; no heap.
Concurrency/allocation context: allocation-free; safe to read concurrently
  (immutable after construction).
Errors and failure guarantee: CapacityExceeded on out-of-space insert — a
  configuration error surfaced to validation, never a panic or truncation.
Security/authorization checks: ids reaching a CpuSet must have passed
  registry validation; the type does not defend against nonsense ids by
  policy, but out-of-range ids are arithmetic-checked (checked shifts /
  word index computation) and yield CapacityExceeded rather than UB.
Logic: word = id / BITS_PER_WORD, bit = id % BITS_PER_WORD over a
  [u64; WORDS] array; all arithmetic checked by construction
  (CAPACITY <= WORDS * BITS_PER_WORD is a compile-time assertion).
Validation: host unit tests per operation incl. boundary ids
  (0, CAPACITY-1, CAPACITY) and round-trips (W03-DV02).
```

### Contract 1.2 — `PlacementSpec` / `PlacementMode` / `ResolvedPlacement`

```text
Name and stability:
  enum PlacementMode { Pinned(PcpuId), Shared(CpuSet) }
  struct PlacementSpec { mode: PlacementMode }          # external input
  struct ResolvedPlacement { set: CpuSet,               # eligibility set
                             exclusive: Option<PcpuId> }# pinned target
  internal.
Purpose and caller: Spec is what a configurator supplies; Resolved is what
  the system freezes and every consumer reads.
Inputs / outputs: as above.
Preconditions / postconditions: a valid ResolvedPlacement always satisfies
  (exclusive == Some(p)) => set == {p}; Shared specs never produce an
  exclusive field; Resolved is immutable after attachment.
State and ownership change: Resolved is cloned/frozen into the vCPU object
  and referenced by the ledger.
Concurrency/allocation context: value types; no allocation beyond CpuSet's
  fixed array.
Errors and failure guarantee: n/a (data); invalid combinations cannot be
  constructed through the validation path (Contract 2.1).
Security/authorization checks: Spec is untrusted management input; it
  reaches validation only through the authorized entry point (Contract 2.3)
  and every field is validated — no field is copied forward unvalidated.
Logic: none (data).
Validation: construction round-trip and invariant tests (W03-DV02).
```

## §2 Validation

### Contract 2.1 — `validate_placement`

```text
Name and stability: fn validate_placement(spec: &PlacementSpec,
  vcpu: VcpuId, vcpu_state: VcpuRunState, registry: &PcpuRegistrySnapshot,
  ledger: &PlacementLedger) -> Result<ResolvedPlacement, PlacementError>;
  pure over its inputs; internal.
Purpose and caller: the single legality authority for placement
  configuration; called by the authorized configuration entry point
  (Contract 2.3) before any attachment.
Inputs / outputs: spec + current facts -> complete resolved placement or a
  typed error; never a partial/defaulted placement.
Preconditions: caller holds configuration authority for `vcpu` (P5 receipt
  checked upstream per Contract 2.3); `registry` is a consistent snapshot;
  the ledger lock is taken by the entry point around validate+attach (see
  2.2) — validation itself takes no locks.
Postconditions: on Ok, attaching the returned placement to the ledger
  cannot fail (all conflicts were resolved under the same snapshot); on
  Err, nothing anywhere changed.
State and ownership change: none (purity is what makes attach-all-or-
  nothing possible).
Concurrency/allocation context: schedulable context; allocation allowed
  (fixed-size only in practice); not callable from IRQ context.
Errors and failure guarantee — closed set:
  EmptyAffinity            Shared spec with empty set
  UnknownPcpu(id)          id not in the registry snapshot
  PcpuNotOnline(id)        registry state != Online (P3-W03 mapping)
  CapacityExceeded         spec exceeds CpuSet capacity
  DuplicateConfiguration   vcpu already has a ledger entry (reconfiguration)
  ReconfigurationNotSupported vcpu_state != Offline (explicit Reserved
                           rejection; never silently accepted)
  ExclusiveConflict(pcpu)  target/member pCPU exclusively held by another
                           pinned vCPU, or a Shared set contains another
                           vCPU's pinned pCPU
  (NotAuthorized is returned by the entry point before validation.)
Failure guarantee: error leaves ledger, vCPU, and registry untouched.
Security/authorization checks: input validation only; authority was
  checked upstream; no Guest-reachable path constructs a spec.
Logic:

  guard vcpu_state == Offline else return Err(ReconfigurationNotSupported)
  guard ledger.get(vcpu).is_none() else return Err(DuplicateConfiguration)
  match spec.mode:
    Pinned(p) ->
      guard registry.contains(p)          else Err(UnknownPcpu(p))
      guard registry.is_online(p)         else Err(PcpuNotOnline(p))
      guard !ledger.is_exclusively_held(p) else Err(ExclusiveConflict(p))
      # exclusivity is bidirectional: any existing Shared set containing p
      # must also be rejected
      guard !ledger.any_shared_set_contains(p) else Err(ExclusiveConflict(p))
      return Ok(ResolvedPlacement { set: CpuSet::of(p), exclusive: Some(p) })
    Shared(s) ->
      guard !s.is_empty()                 else Err(EmptyAffinity)
      for p in s:  # ascending iteration, bounded by CAPACITY
        guard registry.contains(p)        else Err(UnknownPcpu(p))
        guard registry.is_online(p)       else Err(PcpuNotOnline(p))
        guard !ledger.is_exclusively_held(p) else Err(ExclusiveConflict(p))
      return Ok(ResolvedPlacement { set: s, exclusive: None })

Validation: exhaustive host negative tests, one per error variant, plus a
  matrix test over valid combinations (W03-DV03/DV05); this test set is the
  P7-V07 reference at unit level.
```

### Contract 2.2 — ledger entry (`PlacementLedger::attach`)

```text
Name and stability: fn attach(&self, vcpu: VcpuId, rp: ResolvedPlacement,
  receipt: CapabilityReceiptRef) -> Result<(), LedgerError>; internal;
  called by the configuration entry point immediately after Contract 2.1
  returns Ok, while still holding the ledger lock.
Purpose and caller: makes the validated placement authoritative: the map
  entry, the exclusivity index, and the audit fields land atomically.
Inputs / outputs: as above; LedgerError::AlreadyPresent (defensive — can
  only fire on a caller bug, since 2.1 checked).
Preconditions: validation just returned Ok for the same vcpu/ledger under
  this lock hold; vcpu_state still Offline (checked again here — the
  lifecycle authority owns the state; a race with a lifecycle change
  re-reads and aborts).
Postconditions: ledger maps vcpu -> rp; exclusivity index updated if
  rp.exclusive.is_some(); entry records the receipt reference and a
  monotonic configuration sequence number.
State and ownership change: ledger interior only.
Concurrency/allocation context: ledger lock (short, bounded); allocation
  through the P2 allocator contract.
Errors and failure guarantee: AlreadyPresent or lifecycle race -> abort
  with nothing attached; attach is atomic.
Security/authorization checks: stores the receipt reference for audit;
  performs no capability logic.
Logic: map insert; if exclusive, set index bit; record (receipt, seq);
  mark_trace(PlacementApplied { vcpu, mode, set, seq }).
Validation: concurrency test — N threads validating conflicting pinned
  specs; exactly one attaches, others get ExclusiveConflict (W03-DV03).
```

### Contract 2.3 — authorized configuration entry point

```text
Name and stability: fn configure_placement(authority: &ConfigAuthority,
  vcpu: VcpuId, spec: PlacementSpec) -> Result<(), PlacementError>;
  internal; the only caller of 2.1/2.2.
Purpose and caller: enforces the authorization seam and the
  validate+attach atomicity; called from the bring-up/configuration path
  (management flows are later-stage; P7's caller is stage bring-up).
Inputs / outputs: ConfigAuthority carries the P5 capability receipt whose
  rights include the scheduling-policy control right (identifier owned by
  the P5 rights model, P7-IN-06).
Preconditions: caller is in the configuration context; vCPU exists (P4
  object) and its state is readable.
Postconditions: on Ok — placement frozen and ledgered, event emitted; on
  Err — nothing changed, rejection event emitted with the reason.
State and ownership change: as 2.2.
Concurrency/allocation context: takes the ledger lock only between
  validation and attach; registry reads outside the lock.
Errors and failure guarantee: NotAuthorized (missing/insufficient receipt)
  is checked first and short-circuits; all other errors per 2.1/2.2; no
  path panics.
Security/authorization checks: the P5 rights check happens upstream of this
  function and its receipt is carried in; this module never interprets
  rights bits itself (ADR-013 layering).
Logic:

  guard rights_include_scheduling_policy(authority.receipt)
    else { emit(PlacementRejected { vcpu, NotAuthorized }); return Err }
  lock(ledger)
  rp = validate_placement(spec, vcpu, vcpu.run_state(), registry, ledger)?
  attach(ledger, vcpu, rp, authority.receipt)?;
  unlock(ledger)
  return Ok(())
Validation: negative tests for the unauthorized path; review that no other
  code path reaches validate/attach (W03-DV02/DV05).
```

## §3 Runtime queries

### Contract 3.1 — `is_eligible`

```text
Name and stability: fn is_eligible(rp: &ResolvedPlacement, pcpu: PcpuId,
  online: impl Fn(PcpuId) -> bool) -> bool; pure; internal.
Purpose and caller: the single eligibility rule consumed by the W02
  admission gate and the W05 enqueue/pick paths.
Inputs / outputs: frozen placement + candidate pCPU + registry online
  query -> bool.
Preconditions: rp is the vCPU's frozen placement; `online` reflects the P3
  registry (the caller supplies the snapshot discipline).
Postconditions: none (pure).
State and ownership change: none.
Concurrency/allocation context: lock-free, allocation-free; callable from
  bounded wakeup contexts (W06/W08) and IRQ-adjacent paths subject to
  their own bounds.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a (scheduler-internal inputs only).
Logic:

  rp.set.contains(pcpu) && online(pcpu)
  # exclusivity needs no check at run time: validation guaranteed that a
  # pinned pCPU appears in exactly one placement set system-wide.

Validation: property tests over random valid placements/registries
  (W03-DV03); matrix review that gate and picker call this same function
  (W03-DV04) — a second eligibility rule anywhere is a review failure.
```

### Contract 3.2 — ledger queries

```text
Name and stability: fn placement_of(&self, vcpu: VcpuId)
  -> Option<&ResolvedPlacement>; fn is_exclusively_held(&self, pcpu: PcpuId)
  -> bool; fn configuration_snapshot(&self)
  -> impl Iterator<Item = (VcpuId, &ResolvedPlacement)>; internal.
Purpose and caller: read-side API for audit, telemetry (W09), diagnostics,
  and the W11/W12 placement-matrix checks.
Inputs / outputs: as above.
Preconditions / postconditions: snapshot iteration reflects the append-only
  ledger at some point at-or-after call time (acquire semantics per P3
  baseline); `is_exclusively_held` is exact.
State and ownership change: none.
Concurrency/allocation context: lock-free readers over the append-only
  structure; iteration may allocate (diagnostic context only).
Errors and failure guarantee: n/a.
Security/authorization checks: snapshots may contain placement data only;
  no Guest data is reachable through them.
Logic: direct map/index reads.
Validation: consistency tests — every attached vCPU visible in snapshots;
  exclusivity index agrees with map contents (W03-DV03).
```

## §4 Static pinned equivalence statement

For a vCPU configured `Pinned(p)` where `p` is `Online`, the P7 scheduler
must exhibit exactly the P4-era static-binding behavior: the vCPU is
dispatched only on `p`; no other vCPU is ever dispatched on `p`
(bidirectional exclusivity); the only behavioral difference permitted is
the W02 admission gate's bookkeeping (state transitions and slot updates),
which P4's baseline did not perform but which is invisible to the Guest.
This statement is the comparison basis for P7-V05: the declared static
baseline (P4-W09 asset running under its direct path pre-activation, E2
class) and the pinned vCPU under the activated scheduler must show
identical pCPU residency and progress characteristics within the declared
test tolerances. Anything else — silent widening, fallback to shared, or
differing residency — is a P7-V05 failure, not a configuration choice.

## §5 Telemetry semantic fields

```text
placement_applied   { vcpu, mode, set | pinned-target, seq, receipt-ref }
placement_rejected  { vcpu, spec-summary, reason: PlacementError, seq }
placement_snapshot  (query, for W09/W11/W12: id -> mode + set)
```

Field names are semantic; encoding, filtering, and rate control are W09
([P7-W09](../p7-w09-accounting-diagnostics/README.md)). Every rejection is
emitted — including `NotAuthorized` — so invalid controls are visible even
when refused ("non-silent" includes the authorization boundary).

## §6 Default placement rule (stage policy, recorded)

A vCPU configured with no explicit placement receives
`Shared(all online pCPUs at configuration time)` — the maximal shared set —
preserving P4-era "any CPU" behavior as the default while keeping the
frozen-set semantics. Rationale: minimal surprise against the pre-P7
baseline; explicit masks remain available. This default is P7 stage policy
owned by this design and revisitable by a later approved design.
