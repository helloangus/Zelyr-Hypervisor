# P5-W03 Code Contracts — Accessor and Fault Domain

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W03 detailed design](README.md). Checklist §3 template.
Names are internal Rust API; nothing here is Guest-visible ABI.

## 1. `ValidatedGuestRange` (type)

```text
Name and stability: ValidatedGuestRange { segments: [GuestSegment] }; value;
  internal.
Purpose and caller: the only authorized description of Guest memory a
  handler may act on; produced by validate_guest_range, consumed by the
  accessors.
Inputs / outputs: —
Preconditions / postconditions: construction is private to G3 — no other
  module can synthesize one; segments tile their range exactly; empty iff
  the range length was 0.
State and ownership: per-operation; dropped after access.
Concurrency/allocation: fixed capacity bounded by the limit; no sharing
  across operations.
Errors and failure guarantee: —
Security/authorization checks: the type is the INV-P5-02 enforcement point:
  Guest-supplied numbers cannot become memory access without passing through
  it.
Logic: opaque list; accessors expose length and iteration for G4 only.
Validation: type-level review (no public constructor); parity tests with
  the validator.
```

## 2. `copy_from_guest` (function)

```text
Name and stability: copy_from_guest(dst: &mut [u8], src:
  &ValidatedGuestRange) -> Result<(), InvariantViolation>; internal.
Purpose and caller: Guest → Host read of a validated buffer; called by
  consuming handlers (W06) at execution time.
Inputs / outputs: in — Host-owned destination buffer and validated segments;
  out — Ok with dst filled, or InvariantViolation.
Preconditions / postconditions: pre — dst.len() == src.total_len() (checked;
  a mismatch is a caller bug = invariant, not a Guest outcome). Post — on
  Ok, every byte of dst corresponds to exactly one validated segment byte;
  on Err, dst content is undefined and the operation escalates (never
  resumes the Guest with a result).
State and ownership: mutates only dst (Host-owned); Guest memory untouched
  (read-only operation).
Concurrency/allocation: exception context; allocation-free; volatile reads;
  segment order preserved; no coalescing across segments.
Errors and failure guarantee: the only error class is the invariant one
  (post-validation Host faulting contradicts AC-03.3) — Guest-visible
  failure was settled in validation.
Security/authorization checks: no address in the copy path originates from
  Guest input; every address came from a Stage-2 query inside the validator.
Logic:
  function copy_from_guest(dst, src):
    debug_assert(dst.len() == src.total_len())
    pos = 0
    for seg in src.segments:
      for i in 0..seg.len:
        dst[pos + i] = volatile_read_u8(seg.hpa + i)
      pos += seg.len
    return Ok(())
Validation: unit tests over a fake space with aliased/discontiguous pages
  (per-page mapping verified byte-exact); review that no raw-pointer path
  bypasses segments; volatile semantics review (Coding Guidelines).
```

Note: byte-granular volatile access is the correctness-first v0 shape;
performance-oriented (word-wide, cache-managed) copies are Reserved and may
only replace the inner loop with an approved design that preserves the
segment discipline and cache-maintenance duties.

## 3. `copy_to_guest` (function)

```text
Name and stability: copy_to_guest(dst: &ValidatedGuestRange, src: &[u8]) ->
  Result<(), InvariantViolation>; internal.
Purpose and caller: Host → Guest write of a validated buffer; called by
  consuming handlers (W06).
Inputs / outputs: mirror of §2; mutates only the validated Guest pages.
Preconditions / postconditions: pre — src.len() == dst.total_len(). Post —
  on Ok, every validated Guest byte equals the corresponding src byte; all-
  or-nothing holds because validation preceded; on Err, escalate (invariant).
State and ownership: mutates Guest memory strictly within segments.
Concurrency/allocation: as §2.
Errors and failure guarantee: as §2.
Security/authorization checks: as §2; additionally, writes never extend
  beyond segment boundaries even if src claims more bytes (length equality
  is a precondition, and the loop is segment-driven, not src-driven).
Logic: mirror of §2 with volatile_write_u8.
Validation: mirror of §2 plus a read-after-write check on fake spaces;
  boundary cases at segment ends.
```

## 4. `GuestDataFault` and W02 conversion (types/functions)

```text
Name and stability: enum GuestDataFault { Unmapped, PermissionDenied,
  MemoryTypeRejected, RangeInvalid, LengthExceeded } with cause payloads
  (page base, ipa, access kind) for telemetry; internal.
Purpose and caller: G5's cause vocabulary; converted at the call boundary
  into GuestBoundaryError::GuestMemoryFault (W02) so the Guest observes the
  single GUEST_MEMORY_FAULT class regardless of cause.
Inputs / outputs: impl From<GuestDataFault> for GuestBoundaryError — total,
  cause-preserving internally (telemetry), class-fixed Guest-visibly.
Preconditions / postconditions: —
State and ownership: per failure.
Concurrency/allocation: fixed-size.
Errors and failure guarantee: —
Security/authorization checks: payloads never reach Guest-visible values or
  logs unredacted (W09 consumes the redaction rule); payloads contain no
  Host addresses — only Guest-supplied values and fault classes.
Logic:
  fn from(f: GuestDataFault) -> GuestBoundaryError:
    GuestBoundaryError::GuestMemoryFault { cause: f.cause_class() }
Validation: unit test the mapping; review that no other construction of
  GuestMemoryFault exists.
```

Relationship to `RESOURCE_EXHAUSTED`: segment-storage exhaustion during
validation (if dynamically allocated) is a Hypervisor capacity condition and
maps to the W02 class 10, not to `GuestDataFault` — it does not accuse the
Guest's range.

## 5. Host-testable seam summary (for W08)

The following pure functions accept synthetic inputs and require no VM,
QEMU, or trap state:

- `GuestRange::new` — arbitrary u64 base/len/limit.
- `validate_guest_range` — arbitrary ranges against a `FakeStage2Space`
  (test-only implementation of `Stage2QueryPort`) that can encode mapped/
  unmapped/RO/RW/device/protected pages arbitrarily.
- `GuestDataFault` conversion.

Invariants fuzzing must preserve: totality (no panic on any input);
all-or-nothing (Err ⇒ no segments produced); segment tiling (no gaps/overlaps
on Ok); bounded walk length; construction never wraps. Stress property:
repeated validate/access cycles over mutating fake spaces leak no state.
