# P5-W04 Code Contracts — Handle Codec and Object Table

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W04 detailed design](README.md). Checklist §3 template.
Names are internal Rust API; handle bit patterns are opaque to Guests and
carry no stability promise at major 0.

## 1. `ObjectHandle` (type + codec)

```text
Name and stability: ObjectHandle(u64); internal.
Purpose and caller: the opaque reference token; minted by the table,
  presented by Guests through W02 argument registers, decoded by the table
  on every use.
Inputs / outputs: ENCODE(slot: u32, generation: u32, class: ObjectClass) ->
  ObjectHandle (internal only); DECODE(value: u64) -> Result<DecodedHandle,
  HandleInvalidCause> where DecodedHandle { slot: usize, generation: u32,
  class: ObjectClass }.
Preconditions / postconditions: decode is total over all u64; Ok implies
  class tag is a defined ObjectClass, slot < CAPACITY, and value != 0.
Layout: bits[63:40] slot, bits[39:16] generation (24 bits), bits[15:0] class
  tag (decision 1 of 01); tags start at 1 so zero is never valid.
State and ownership: value type; no interior state.
Concurrency/allocation: pure; allocation-free.
Errors and failure guarantee: Err(BadEncoding) exactly for: zero value,
  undefined class tag, slot ≥ CAPACITY, or reserved-bit violations within
  fields. No other cause can come from decode.
Security/authorization checks: decode asserts nothing about existence,
  generation truth, or rights — those are the table's checks; nothing about
  Host structure is derivable from a handle.
Logic:
  function decode(v):
    if v == 0:                    return Err(BadEncoding)
    class = ObjectClass::from_tag(v & 0xFFFF) or return Err(BadEncoding)
    slot  = (v >> 40) & 0xFFFFFF
    gen   = (v >> 16) & 0xFFFFFF
    if slot >= CAPACITY:          return Err(BadEncoding)
    return Ok(DecodedHandle { slot, generation: gen, class })
Validation: exhaustive decode tests over the boundary values (0, 1, max
  u64, slot = CAPACITY-1 vs CAPACITY, every defined tag, undefined tags);
  property: decode never panics and Err is always BadEncoding (W08).
```

## 2. `ObjectClass` (type)

```text
Name and stability: enum ObjectClass { Vm = 1, Vcpu = 2 } with
  from_tag(u16) -> Option<ObjectClass>; internal; extension Reserved.
Purpose and caller: type-safety tag in handles and slots; checked on every
  decode and every lookup.
Inputs / outputs: —
Preconditions / postconditions: tags nonzero, unique, densely documented;
  from_tag is total.
State and ownership: —
Concurrency/allocation: —
Errors and failure guarantee: from_tag None ⇒ decode-level BadEncoding
  upstream.
Security/authorization checks: class mismatch rejection is decided by the
  table (with_object), never by the referent — a Vcpu handle presented
  where a Vm is required is rejected before any P4 state is touched.
Logic: —
Validation: tag/value table review; exhaustive from_tag tests.
```

## 3. `ObjectTable` (type)

```text
Name and stability: ObjectTable<const CAPACITY: usize> { slots:
  [Slot; CAPACITY], free_list, lock }; internal; sole existence authority.
Purpose and caller: H2/H3 operations; constructed once by the Hypervisor
  core at initialization; host tests construct private instances (seam).
Inputs / outputs: per operation (§4 of 04-code-contracts-lifecycle-ops.md).
Preconditions / postconditions: invariants after every operation: each slot
  is exactly one of Free{generation} / Occupied{generation, class, ref};
  free_list is a permutation of free, non-retired slots; generation of a
  slot strictly increases across its state changes; sum of occupied ≤
  CAPACITY.
State and ownership: all slot state; the lock.
Concurrency/allocation: table-wide spinlock per P3-W06 registry rules; all
  mutating operations and with_object hold it; static backing (fixed array)
  so no allocation on the lifecycle path; a dynamically backed variant may
  use the P2 small-allocation foundation at construction time only.
Errors and failure guarantee: operations are total; internal invariant
  breaches (free_list corruption, generation regression) are
  InvariantViolation.
Security/authorization checks: no API on this type exposes rights, caller
  identity, or Host addresses; ref contents (VmId/VcpuId) never leave the
  boundary except through with_object's closure scope.
Logic: straightforward slot array; the interesting contracts are the
  operations (companion file §2–§4).
Validation: invariant-checking test harness (walks all slots and the free
  list after every operation in tests); stress loop support (register/
  destroy cycles) for W08.
```

## 4. `ObjectRef` (type)

```text
Name and stability: enum ObjectRef { Vm(VmId), Vcpu { vm: VmId, vcpu:
  VcpuId } }; internal.
Purpose and caller: identity of the P4-established referent stored in an
  occupied slot and yielded to with_object closures.
Inputs / outputs: —
Preconditions / postconditions: identifiers are the P4-established identity
  newtypes (AC-04.1); W04 never dereferences them outside H4's binding.
State and ownership: value; copied out to closures under lock.
Concurrency/allocation: value semantics.
Errors and failure guarantee: —
Security/authorization checks: an ObjectRef is not authority (decision 7 of
  01); it must not be logged or composed into Guest-visible values.
Logic: —
Validation: review that no code path persists an ObjectRef beyond a
  with_object closure.
```
