# P5-W04 Code Contracts — Lifecycle Operations

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W04 detailed design](README.md). Checklist §3 template.
All operations hold the table lock for their whole body unless stated.

## 1. `register` (function — internal creation)

```text
Name and stability: register(&self, class: ObjectClass, ref: ObjectRef) ->
  Result<ObjectHandle, TableFull>; internal (not Guest-reachable).
Purpose and caller: the only creation path; used by Hypervisor/bootstrap/
  test setup (decision 5 of 01). Consumed by W05 bootstrap grants and W07
  two-context setup through their own designs.
Inputs / outputs: class + referent identity → minted handle or capacity
  failure.
Preconditions / postconditions: post — slot occupied with generation = old
  generation + 1 (or the slot's initial generation for a never-used slot);
  the returned handle decodes to exactly (slot, generation, class).
State and ownership: one slot transitions Free → Occupied; the referent is
  recorded, never owned.
Concurrency/allocation: lock held; allocation-free (static backing);
  lock-free fast path deliberately absent (Reserved).
Errors and failure guarantee: TableFull when no allocatable slot exists —
  a Hypervisor capacity condition (W02 class 10 at the call layer), not a
  Guest fault. No partial state on failure.
Security/authorization checks: caller authorization for register is by
  construction (not Guest-reachable); future exposure would be a new
  design.
Logic:
  function register(class, ref):
    lock()
    slot = free_list.pop() or { unlock(); return Err(TableFull) }
    if slot.generation == GEN_MAX: mark_retired(slot); retry once
    slot.generation += 1
    slot.state = Occupied { generation, class, ref }
    handle = encode(slot.index, slot.generation, class)
    unlock()
    emit lifecycle event (outside lock): registered { class }
    return Ok(handle)
Validation: unit tests — capacity exhaustion, retired-slot skip, minted
  handle decodes identically; repeated register/destroy cycles keep
  invariants (harness).
```

## 2. `lookup` (function — identity validation)

```text
Name and stability: decode_and_validate(&self, raw: u64) ->
  Result<DecodedHandle, HandleInvalidCause>; internal; total.
Purpose and caller: the front half of every use; called by with_object and
  destroy; host tests call it directly (forge/stale corpora).
Inputs / outputs: raw Guest-supplied u64 → decoded fields or cause.
Preconditions / postconditions: post — on Ok the slot is Occupied with
  matching generation and class. Err carries exactly one of BadEncoding /
  FreeSlot / GenerationMismatch / ClassMismatch.
State and ownership: read-only on the table; no state change.
Concurrency/allocation: lock held for the table check; allocation-free.
Errors and failure guarantee: total; causes exclusive (decision 8 of 01:
  Guest-visible outcome is uniformly INVALID_HANDLE via W02 conversion).
Security/authorization checks: no identity of the *presenting caller* is an
  input — validity is caller-independent; authority is not consulted.
Logic:
  function decode_and_validate(raw):
    h = decode(raw)?                        // BadEncoding
    lock()
    s = slots[h.slot]
    err = if s is Free          -> if never_used(s) FreeSlot else GenerationMismatch
          elif s.generation != h.generation -> GenerationMismatch
          elif s.class != h.class           -> ClassMismatch
          else                              -> Ok
    unlock()
    return err
Validation: unit corpora — random u64, zero, max, valid-then-destroy,
  destroy-reuse, cross-class presentation, generation off-by-one; property:
  every Err maps to INVALID_HANDLE with one internal cause (W08).
```

Note: distinguishing "never used" from "freed" in FreeSlot vs
GenerationMismatch is for telemetry only; both are Guest-visible as
INVALID_HANDLE and never reveal slot history beyond that.

## 3. `with_object` (function — synchronized access)

```text
Name and stability: with_object<T>(&self, raw: u64, expected: ObjectClass,
  f: impl FnOnce(&ObjectRef) -> T) -> Result<T, HandleInvalidCause>;
  internal.
Purpose and caller: the only way to *use* a valid handle; called by W06
  handlers so object access is serialized against destruction.
Inputs / outputs: raw handle + required class + closure → closure result or
  invalid cause.
Preconditions / postconditions: closure runs only if decode_and_validate
  succeeded with the expected class; the table lock is held for the
  closure's duration; the closure receives the identity, not the slot.
State and ownership: none beyond the validated read; the closure must not
  re-enter the table (re-entrancy = InvariantViolation).
Concurrency/allocation: lock held across the closure — the closure's cost
  discipline is the caller's contract (02 §4: no Guest-memory copy under
  lock; W06 sequences its operations accordingly).
Errors and failure guarantee: exactly the lookup causes; closure panics are
  not caught (a panic is an invariant-class event per P0-W14 policy).
Security/authorization checks: expected class is Hypervisor-side intent
  from the call contract; the Guest cannot request a type check bypass.
Logic:
  function with_object(raw, expected, f):
    h = decode_and_validate(raw)?          // includes class check
    lock()
    result = f(slots[h.slot].ref)
    unlock()
    return Ok(result)
Validation: host tests — closure sees the right referent; concurrent
  destroy during closure is impossible (lock); class mismatch rejected
  before closure runs.
```

## 4. `destroy` and cascade (functions — lifecycle mutation)

```text
Name and stability: destroy(&self, raw: u64) -> Result<Destroyed,
  HandleInvalidCause>; destroy_vm_cascade(&self, vm_handle: u64) ->
  Result<DestroySummary, HandleInvalidCause>; internal.
Purpose and caller: lifecycle teardown; invoked by Hypervisor lifecycle /
  bootstrap teardown (not Guest-reachable in P5); the cascade is the hook
  W05 consults (via the destroyed event) to retire derived authority, and
  the point AC-04.5's stop-path integration attaches.
Inputs / outputs: raw handle → Destroyed, or cause; cascade → summary
  { vcpu_slots_freed, vm_slot_freed }.
Preconditions / postconditions: destroy — handle valid (existence +
  generation; class check is against the slot's own class). Post — slot
  Free with generation + 1; every prior handle to it stale. Cascade —
  transactional: all contained Vcpu slots freed first (each generation
  bumped), then the Vm slot; no interleaved observation of a half-cascaded
  state is possible (lock).
State and ownership: slot transitions only; referents are untouched (P4
  owns stopping them — the caller of cascade coordinates with P4's stop
  path).
Concurrency/allocation: lock held across the whole cascade; allocation-
  free; emits events after unlock.
Errors and failure guarantee: repeated destroy of the same handle →
  GenerationMismatch or FreeSlot; cascade on an invalid Vm handle → cause;
  no partial teardown on error paths.
Security/authorization checks: destroy authority is Hypervisor-internal in
  P5 (decision 5 of 01); should a Guest-facing destroy ever be designed, it
  requires W05 DESTROY rights and is a new design.
Logic:
  function destroy(raw):
    h = decode_and_validate(raw)?          // generation-stale check first
    lock(); s = slots[h.slot]
    previous_generation = s.generation
    s.generation += 1; s.state = Free { generation }
    push_free_list(s); unlock()
    emit destroyed { class, slot: h.slot, generation: previous_generation }
                                           // W05 reclaim / W09 hook point
    return Ok(Destroyed)

  function destroy_vm_cascade(vm_raw):
    h = decode_and_validate(vm_raw)?       // must decode to class Vm
    lock()
    freed = []
    for slot in occupied slots where ref is Vcpu{vm: h.vm_id}:
      freed.append((slot, slot.generation))
      slot.generation += 1; slot.state = Free; push_free_list(slot)
    vm_slot = slots[h.slot]
    freed.append((vm_slot, vm_slot.generation))
    vm_slot.generation += 1; vm_slot.state = Free; push_free_list(vm_slot)
    unlock()
    for (slot, gen) in freed:              // per-slot events after the fact
      emit destroyed { class, slot, generation: gen }   // W05 reclaim hook
    return Ok(summary)
Validation: unit tests — repeated destroy, destroy-after-reuse, cascade
  ordering (vcpu slots before vm slot, all in one critical section), stale
  vcpu handle rejection immediately after cascade; stress cycles (W08
  create/lookup/destroy/recreate).
```

## 5. Conversion to the W02 boundary (function)

```text
Name and stability: impl From<HandleInvalidCause> for GuestBoundaryError —
  every variant → GuestBoundaryError::InvalidHandle { cause }; the only
  conversion; TableFull → ResourceExhausted.
Purpose and caller: W06 handlers converting table outcomes at the call
  boundary; Guest sees INVALID_HANDLE (class 6) or RESOURCE_EXHAUSTED
  (class 10) uniformly.
Validation: mapping test; review that no other construction site of
  InvalidHandle exists.
```
