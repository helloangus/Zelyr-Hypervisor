# P5-W03 Architecture, State, and Concurrency

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W03 detailed design](README.md).

## 1. Logical modules

| Module | Layer | Responsibility | Owned state / artifacts | Inputs | Outputs | Non-responsibility | Failure boundary |
|---|---|---|---|---|---|---|---|
| G1 Range vocabulary | Core | Typed Guest IPA/length values; checked `GuestRange` construction; access-kind enum | The newtype definitions | Raw Guest-supplied integers | Validated-shape ranges or `RangeInvalid` | Semantics of what a call *means* by a range; Stage-2 facts | Construction is total and checked; overflow is a construction error, never wraparound |
| G2 Stage-2 query port | Core trait / Arch impl | Expose the P4-established per-page query as a consumable port: present/absent, HPA base, R/W permissions, memory type | The trait definition; Arch binding to P4 state | Space reference + one page-aligned IPA | One page query result | Mutation of Stage-2 (P4 owns map/unmap/protect); page-table walking internals | Query cannot lie about presence; a query failure is an `InvariantViolation`, not a Guest fault |
| G3 Range validator | Core | Full-range, page-stepped validation; permission/type/limit checks; segment production | None (pure given the port) | Space ref, `GuestRange`, `GuestAccessKind`, limit | `ValidatedGuestRange` (segments) or `GuestDataFault` | Copying; policy of the calling hypercall | Validation is total; every failure carries exactly one cause |
| G4 Page-bounded accessor | Core | Directional copy between Host buffers and validated segments; volatile access; no contiguity assumptions | None | Host buffer + `ValidatedGuestRange` | `Ok(())` or `InvariantViolation` | Re-validation; Stage-2 state changes | Post-validation access failure is an invariant (AC-03.3), never a Guest outcome |
| G5 Fault domain | Core | Cause enum; conversion to the W02 boundary error; telemetry cause vocabulary | The cause types | Validator results | `GuestBoundaryError::GuestMemoryFault` + internal cause | Guest-visible code selection (W02 owns the class) | Causes never leak to Guest-visible values |

Layering: G1, G3, G4, G5 are architecture-independent; only G2's
implementation touches Stage-2 state, and only through P4-established
queries. No module branches on board/SoC/QEMU identity.

## 2. Core objects and ownership

| Object | Kind | Owner | Lifetime | Invariants |
|---|---|---|---|---|
| `GuestIpa` / `GuestLen` | Newtypes | G1 | Value | Constructed only from checked operations; no naked integer crosses the boundary |
| `GuestRange` | Value | G1 | Per request | `base + len` does not overflow; `len ≤ limit` is checked per call before the walk |
| `PageTranslation` | Value (from port) | G2 | Per query | Represents exactly one granule's Stage-2 state at query time |
| `GuestSegment` | Value | G3 | Per validated range | `hpa` came from a G2 query of the page containing `range_offset`; `len > 0`; segments tile the range exactly |
| `ValidatedGuestRange` | Value | G3 | One operation | Sum of segment lengths equals range length; empty iff length 0; produced only by successful validation |
| `GuestDataFault` | Value | G3/G5 | Per failure | Exactly one cause; converts only into the W02 `GUEST_MEMORY_FAULT` class |

Ownership rule: W03 introduces no long-lived mutable state. Every value is
per-operation. The Stage-2 address space itself remains owned by P4's
contracts; W03 holds a reference through the query port only.

## 3. Operation lifecycle

```text
Caller (W06 handler) holds: space ref, raw base, raw len, direction, limit
  |
  v
[V1 Construct]   G1: checked GuestRange (overflow/limit checks; zero ok)
  |                 RangeInvalid / LengthExceeded -> fault (no Stage-2 touch)
  v
[V2 Validate]    G3: for each page in range: G2 query ->
  |                 absent            -> Unmapped
  |                 !read  & Read     -> PermissionDenied
  |                 !write & Write    -> PermissionDenied
  |                 type not normal   -> MemoryTypeRejected
  |                 hpa in protected  -> InvariantViolation (AC-03.2 guard)
  |                 else append segment
  |                 any fault -> FAIL: no access has occurred (all-or-nothing)
  v
[V3 Access]      G4: walk segments in order; per segment:
  |                 volatile read/write of segment.len bytes at segment.hpa
  |                 host fault -> InvariantViolation (AC-03.3)
  v
[V4 Done]        Result to caller; segments dropped; no retained state
```

Rules:

- The Guest is stopped for the whole operation (it issued the hypercall);
  decision 7 of [01](01-scope-and-foundations.md) fixes capture-then-use.
- V2 performs no Guest- or Host-visible side effect; V3 is the only memory
  access in the package.
- Either phase may be executed by host tests with a fake G2 port.

## 4. Concurrency and allocation model

- **Thread context:** the boundary runs in hypercall (VM-exit/synchronous-
  exception) context inherited from the dispatch flow; it adds no locking of
  its own. If the P4 query port takes locks internally, its P4-defined
  context rules govern (AC-03.1); the W03 contract assumes the query is
  callable in exception context without unbounded blocking, and a query that
  cannot honor that is a `Contract Conflict` against P4-W02 to record.
- **Shared state:** none created by W03. The space is P4-owned; concurrent
  Stage-2 mutation is out of the P5 proven scope (decision 8 of
  [01](01-scope-and-foundations.md)).
- **Allocation:** the segment list is bounded by the per-call limit
  (`limit / granule + 1` entries); implementations may use a fixed-capacity
  stack structure or the P2 small-allocation foundation (AC via FD-2);
  allocation failure is `ResourceExhausted` at the *call* level (W02 class
  10), not a Guest-data fault — it says nothing about the Guest's range.
- **Boundedness:** walk trip count ≤ limit/granule + 1; per-page work is
  constant; no unbounded loops exist on Guest-controlled inputs.

## 5. Security model

- Untrusted inputs: base, length, direction, and the space selection are all
  Guest-influenced; only G3's checks make them usable.
- No Host disclosure: segments are internal; nothing about Host physical
  layout is returned to callers, logged, or composed into Guest results
  (W09 redaction consumes the same rule).
- Fail-closed forbidden boundary: the protected-range guard converts a
  boundary breach into the W02 invariant escalation — the Guest observes
  containment, and the Hypervisor treats it as its own bug to diagnose.
- Side-effect hygiene: only V3 touches memory; a denied request has zero
  memory effect; device memory is unreachable by policy (decision 5 of
  [01](01-scope-and-foundations.md)).
- Uniformity: identical ranges from different callers validate identically;
  no identity participates in G1–G5.
