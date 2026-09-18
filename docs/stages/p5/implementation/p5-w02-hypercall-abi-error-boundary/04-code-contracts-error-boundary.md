# P5-W02 Code Contracts — Error Boundary

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W02 detailed design](README.md). Template per the
implementation-design checklist §3. Names are internal Rust API; only the
numeric codes are guest-visible ABI.

## 1. Status taxonomy (the closed guest-visible outcome set)

| Code | Name | Meaning (Guest-visible) | Decided by |
|---|---|---|---|
| 0 | `OK` | Request completed; X1–X3 carry defined results | operation contract |
| 1 | `UNKNOWN_CALL` | Call number is not defined by this ABI | decode/route (W02) |
| 2 | `VERSION_MISMATCH` | Requested version is not compatible with this boundary | compatibility check (W02) |
| 3 | `UNSUPPORTED_FEATURE` | Call is defined but its feature is absent from this build | route (W02) |
| 4 | `MALFORMED_REQUEST` | Envelope violation: wrong immediate, junk reserved halves, nonzero reserved register, invalid version word, nonzero unknown flag/reserved bits in a call's defined inputs | decode (W02); call contracts may extend to their own reserved fields |
| 5 | `INVALID_ARGUMENT` | A defined input violates the call's stated semantics (not memory, not a handle, not state, not capacity) | call contract (W06) / W03 for range legality outside memory faults |
| 6 | `INVALID_HANDLE` | Object reference does not decode, does not exist, is stale, or has the wrong type | handle lookup (W04) |
| 7 | `NO_AUTHORITY` | The caller holds no active authority for this object (never granted, or revoked) | authority check (W05) |
| 8 | `INSUFFICIENT_RIGHTS` | Authority exists but lacks the operation's required right | rights check (W05) |
| 9 | `BAD_STATE` | Object and authority are valid, but the operation is illegal in the object's current lifecycle state | lifecycle/op state check (W04/W06) |
| 10 | `RESOURCE_EXHAUSTED` | A Hypervisor-side capacity condition prevented the operation | allocating operation (W04/W05/W06) |
| 11 | `GUEST_MEMORY_FAULT` | The Guest-data boundary rejected an address/range: unmapped, permission, memory-type, or range illegality | Guest-data validation (W03) |

Rules:

- The set is closed. Adding a code is an ABI compatibility event per
  [06](06-validation-and-handoff.md) §3.
- Rejection codes are identical for every caller in the same situation;
  nothing in the mapping may consult identity (ADR-013/ADR-051) — authority
  *outcomes* are codes 7/8, authority *inputs* are W05's records.
- Sub-causes (stale vs wrong-type vs absent; never-granted vs revoked) are
  internal telemetry detail only, never Guest-visible (§4).

## 2. `HypercallStatus` (type)

```text
Name and stability: HypercallStatus — closed enum; guest-visible mapping
  fixed by §1.
Purpose and caller: the only guest-visible outcome vocabulary; produced by
  the dispatch flow, consumed by compose_result.
Inputs / outputs: to_guest_code(&self) -> u32 (total); From<GuestBoundaryError>
  (the single conversion, §3).
Preconditions / postconditions: to_guest_code is bijective with §1 codes;
  OK is constructible only as an operation result, never from an error.
State and ownership: value type; no state.
Concurrency/allocation: pure; allocation-free; Send/Sync by construction.
Errors and failure guarantee: no fallible operations.
Security/authorization checks: the enum carries no payload that could leak
  Host state; diagnostic detail lives in the separate telemetry event, not
  in the status.
Logic: enum + match-based mapping; no Display into Guest-visible channels.
Validation: unit test — table §1 equals the enum mapping, entry by entry
  (single-source review); fuzz invariant: every produced status is in the
  table (W08).
```

## 3. `GuestBoundaryError` and the single conversion

```text
Name and stability: GuestBoundaryError — internal enum of Guest-caused
  failure kinds; internal stability.
Purpose and caller: produced by boundary modules (W03/W04/W05 handlers and
  W06 call logic) instead of numeric codes, so rejection sites stay
  type-checked.
Variants (illustrative, non-exhaustive names): UnknownCall, VersionMismatch,
  UnsupportedFeature, MalformedEnvelope, InvalidArgument, InvalidHandle
  { cause: HandleInvalidCause }, NoAuthority { cause: AuthorityAbsenceCause },
  InsufficientRights { required, held }, BadState { expected, actual },
  ResourceExhausted { pool }, GuestMemoryFault { cause: GuestDataFaultCause }.
Conversion: impl From<GuestBoundaryError> for HypercallStatus — the only
  site where internal failure becomes a guest-visible code. Mapping:
  variant ↔ code per §1; cause payloads are dropped at the conversion
  (they feed telemetry separately, §4).
Preconditions / postconditions: conversion is total and lossless at class
  granularity, lossy at cause granularity by design.
State and ownership: per-request value.
Concurrency/allocation: fixed-size; allocation-free.
Errors and failure guarantee: none (it is the error type).
Security/authorization checks: cause payloads never cross the boundary;
  authority payloads name right classes, not caller identities of other VMs.
Logic: match arm per variant returning the §1 code.
Validation: unit test per variant→code; review that no other
  status-construction site exists (review gate in 05 workflow step 6).
```

## 4. Distinguishability rules and telemetry vocabulary

Class-boundary rules resolving the plan's required distinctions:

- **Unknown vs unsupported:** absent from the registry → `UNKNOWN_CALL`;
  present but feature-gated → `UNSUPPORTED_FEATURE`. A future call must
  enter the registry with a feature bit to be "known".
- **Malformed vs invalid argument:** envelope-level structure (immediate,
  reserved halves/registers, version word, reserved bits inside defined
  inputs) → `MALFORMED_REQUEST`; a well-formed input that violates stated
  call semantics → `INVALID_ARGUMENT`.
- **Invalid handle vs bad state:** the reference fails decode/existence/
  generation/type → `INVALID_HANDLE`; the reference is valid but the
  operation is illegal for the object's current lifecycle state →
  `BAD_STATE`. Ordering (handle before authority) is W06's integration rule;
  this table fixes only the class meanings.
- **No authority vs insufficient rights:** absence (including revocation) →
  `NO_AUTHORITY`; presence without the needed right → `INSUFFICIENT_RIGHTS`.
  Revoked authority is therefore Guest-visible as `NO_AUTHORITY`, with
  "revoked" retained as an internal cause for telemetry and the V08 loop.
- **Resource vs argument:** `RESOURCE_EXHAUSTED` reports Hypervisor-side
  capacity (table full, pool empty), never a Guest input error.

Telemetry event vocabulary emitted by the boundary (transport and redaction
are [P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md)):

```text
hypercall.result { caller_vm, call_number, outcome: HypercallStatus,
                   cause: Option<internal cause enum>, duration_class }
hypercall.envelope_reject { caller_vm, reason: envelope check id }
```

Constraints: no Guest buffer contents, no Host addresses, no object-table
locations, no raw register dumps in events; `caller_vm` is the W01 caller
identity, attributed per VM as P5-V15 requires.

## 5. `InvariantViolation` and escalation

```text
Name and stability: InvariantViolation — distinct type; internal.
Purpose and caller: represents a broken Hypervisor invariant detected during
  any stage; raised by any module (M1–M6 or a called operation).
Inputs / outputs: carries an internal diagnostic descriptor (stage, check
  name, opaque context id) — never Guest-visible values.
Preconditions / postconditions: no conversion into HypercallStatus or
  GuestBoundaryError exists (compile-time separation, Decision 7).
  escalate_invariant(v) -> ! routes to the fatal-classification path
  (AC-02.3) and does not resume the Guest.
State and ownership: per-request; consumed by escalation.
Concurrency/allocation: construction allocation-free; escalation does not
  return.
Errors and failure guarantee: escalation is the terminal state of the
  request; no partial Guest-visible effects were performed (validation-
  before-effect rule) — any observed Guest-visible effect of an escalated
  request is itself an invariant failure to diagnose.
Security/authorization checks: not applicable (this is the non-Guest class
  by definition).
Logic: raise → unwind to the dispatch entry → escalate; never log through
  Guest-visible channels before the fatal path has captured diagnostics.
Validation: type-level review (no conversion impls exist); host test that a
  forced invariant in a stub handler terminates without composing a status;
  the QEMU-level fatal-path behavior is owned by P0-W14/P1-W07 evidence.
```

## 6. Security review anchors for this boundary

- Result granularity leaks: the twelve classes are the entire Guest-visible
  vocabulary; any richer Guest-visible distinction (for example distinguishing
  "stale" from "wrong type") violates the design and is a review stop.
- Uniform behavior: identical inputs from different callers yield identical
  codes; there is no identity input to the mapping (structural, §2).
- No Host information: composition (03 §7) and the telemetry constraints
  (§4) are the two value paths out of the boundary; both are reviewed to
  carry no Host pointer, offset, or object-table location.
- Timing: status-class timing side channels are a recorded limitation
  ([01](01-scope-and-foundations.md) §5), not a P5 deliverable.
