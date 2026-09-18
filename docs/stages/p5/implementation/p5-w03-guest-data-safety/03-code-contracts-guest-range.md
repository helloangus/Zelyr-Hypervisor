# P5-W03 Code Contracts — Range Vocabulary and Validator

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W03 detailed design](README.md). Checklist §3 template.
Names are internal Rust API; nothing here is Guest-visible ABI.

## 1. `GuestIpa`, `GuestLen` (types)

```text
Name and stability: GuestIpa(u64), GuestLen(u64) newtypes; internal;
  reconcile spelling with the P0-W15 vocabulary at review (AC-03.4).
Purpose and caller: the only integer carriers for Guest-supplied address and
  length values entering this boundary; used by G1/G3 and by consuming call
  handlers (W06).
Inputs / outputs: constructors from raw u64 that record provenance as
  untrusted; accessors return the raw value where needed for the walk.
Preconditions / postconditions: no range/overflow semantics in the newtypes
  themselves; arithmetic happens only in checked G1 constructors.
State and ownership: value types; Copy-safe; no InteriorState.
Concurrency/allocation: trivially shareable; allocation-free.
Errors and failure guarantee: none.
Security/authorization checks: type system prevents Guest values from
  binding to Host pointer types (decision 1 of 01).
Logic: transparent wrappers; no Display into Guest-visible channels.
Validation: unit tests for construction and accessor round-trip; review that
  no `usize`/`u64` parameter of this boundary lacks the newtype.
```

## 2. `GuestRange::new` (function)

```text
Name and stability: GuestRange::new(base: GuestIpa, len: GuestLen, limit:
  GuestLen) -> Result<GuestRange, GuestDataFault>; internal; total.
Purpose and caller: the checked-construction gate of V1; called by consuming
  handlers before any Stage-2 query.
Inputs / outputs: in — base, length, the call's declared limit (decision 6 of
  01); out — range or fault with cause RangeInvalid (arithmetic) /
  LengthExceeded (limit).
Preconditions / postconditions: post — on Ok, base + len ≤ u64::MAX and len
  ≤ limit; len == 0 is Ok (vacuous, decision 3 of 01).
State and ownership: none.
Concurrency/allocation: pure; allocation-free.
Errors and failure guarantee: total; exactly one cause on Err.
Security/authorization checks: all inputs untrusted; checked arithmetic
  only (Coding Guidelines).
Logic:
  function new(base, len, limit):
    if len > limit:            return Err(LengthExceeded)
    end = checked_add(base, len) or return Err(RangeInvalid)
    return Ok(GuestRange { base, len })   // end held internally for the walk
Validation: unit tests — zero, limit boundary, limit+1, u64::MAX overflow
  cases, base near u64::MAX; property: construction never wraps (W08).
```

## 3. `GuestAccessKind` (type)

```text
Name and stability: enum { Read, Write }; internal.
Purpose and caller: direction tag for validation and access; set by the
  consuming call contract (W06), never by the Guest.
Inputs / outputs: —
Preconditions / postconditions: —
State and ownership: —
Concurrency/allocation: —
Errors and failure guarantee: —
Security/authorization checks: direction is Hypervisor-side intent; a Guest
  cannot request a direction (it is implied by the call).
Logic: —
Validation: compiler-exhaustive matches.
```

## 4. `Stage2QueryPort` (trait) and `PageTranslation` (type)

```text
Name and stability: Stage2QueryPort::query_page(space, page_base) ->
  Result<PageTranslation, QueryUnavailable>; Arch-implemented over the
  P4-established query capability; internal.
Purpose and caller: G3's only source of truth about Stage-2 state; host
  tests supply a fake implementation (the seam).
Inputs / outputs: in — the active Guest address space reference and one
  granule-aligned IPA; out — PageTranslation { hpa_base, readable, writable,
  memory_type: NormalCacheable | Other } or QueryUnavailable.
Preconditions / postconditions: page_base is granule-aligned (G3 aligns);
  the result describes exactly that granule at query time.
State and ownership: none retained; P4 owns the space.
Concurrency/allocation: must be callable in exception context without
  unbounded blocking (AC-03.1 context rule); allocation-free.
Errors and failure guarantee: QueryUnavailable means the P4 capability could
  not answer — an invariant-class condition for this boundary (it cannot be
  attributed to the Guest), escalated per the W02 invariant model.
Security/authorization checks: returns physical targets only to the
  boundary; never to callers or logs.
Logic: thin binding to the P4 query; mapping of P4 permission/type facts
  into this vocabulary; no new Stage-2 mechanisms.
Validation: Arch review that the binding matches P4's implemented query;
  fake-port parity tests; Specification-Investigation note for the
  permission/type mapping (decision 9 of 01).
```

## 5. `validate_guest_range` (function)

```text
Name and stability: validate_guest_range(space, range, access, port) ->
  Result<ValidatedGuestRange, GuestDataFault>; internal; total.
Purpose and caller: phase V2; called by consuming handlers (W06) after V1;
  host tests and property harnesses call it directly.
Inputs / outputs: in — space reference, validated GuestRange, access kind,
  query port; out — segment list tiling the range, or fault with exactly one
  cause {Unmapped, PermissionDenied, MemoryTypeRejected}.
Preconditions / postconditions: range came from GuestRange::new (limit and
  overflow already handled). Post — on Ok, segments are ordered by range
  offset, tile the range exactly (sum of lengths == range.len; gaps
  impossible), each segment's page passed query with direction-consistent
  permission and NormalCacheable type; on Err, zero memory accesses have
  occurred anywhere.
State and ownership: none retained beyond the returned value.
Concurrency/allocation: allocation bounded by limit/granule + 1 entries
  (02 §4); no locks of its own; callable in exception context.
Errors and failure guarantee: total; all-or-nothing (decision 2 of 01).
Security/authorization checks: every page is queried — there is no trusted
  fast path, no "already validated" cache, and no identity input.
Logic:
  function validate_guest_range(space, range, access, port):
    if range.len == 0: return Ok(ValidatedGuestRange { segments: [] })
    segments = []
    for page in pages(range, GRANULE):           // first/last pages partial
      t = port.query_page(space, page.base)?
      if !t.present:                 return Err(Unmapped)
      match access:
        Read  -> if !t.readable:     return Err(PermissionDenied)
        Write -> if !t.writable:     return Err(PermissionDenied)
      if t.memory_type != NormalCacheable:
                                   return Err(MemoryTypeRejected)
      if protected_range_contains(t.hpa_base):
                                   return Err(InvariantViolation)   // fail closed
      seg_hpa = t.hpa_base + page.offset
      segments.push(Segment { hpa: seg_hpa, offset: page.range_offset,
                              len: page.bytes_in_this_page })
    return Ok(ValidatedGuestRange { segments })
Validation: unit tests per cause, per boundary shape: zero length, single
  byte, exact page, page+1, page-boundary start/end, unmapped first/middle/
  last page, RO range with Write (and RW with Read), device-typed page,
  protected-range translation (guard), QueryUnavailable escalation; property
  tests on random ranges over synthetic spaces (W08 seam).
```

Note: `protected_range_contains` is the belt-and-braces guard (AC-03.2);
its predicate comes from the P2-established protected-range vocabulary, not
from a W03-local list.
