# P5-W03 Scope, Foundations, and Design Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W03 detailed design](README.md).

## 1. Scope classification in detail

### 1.1 Required

| Item | Statement |
|---|---|
| Guest range vocabulary | `GuestIpa`, `GuestLen`, `GuestRange {base, len}` newtypes with total, checked construction; `GuestAccessKind {Read, Write}` |
| Stage-2 query port | A Core-side port (Arch-implemented) expressing, per page: translation presence, target HPA base, read/write permissions, memory type — backed by the P4-W02 capability |
| Range validator | Full-range, page-stepped validation producing translated segments or a fault; no partial validity |
| Access rules | Direction-consistent permissions (Read needs Stage-2 read; Write needs Stage-2 write); normal-cacheable memory type only |
| Length rules | Checked `base + len` arithmetic; zero length valid and vacuous; per-call limit mechanism with bounded walk |
| Fault domain | Causes {Unmapped, PermissionDenied, MemoryTypeRejected, RangeInvalid, LengthExceeded} → W02 class `GUEST_MEMORY_FAULT`, causes internal-only |
| Forbidden-boundary guard | Any validated translation landing in a P2/P4-protected range is an `InvariantViolation`, not a Guest fault |
| Composition contract | All-or-nothing: no Guest-visible write occurs unless the entire range validated |
| Host-testable seam | Validator and accessor callable against a fake Stage-2 space with synthetic translations |

### 1.2 Reserved

| Item | Trigger for activating |
|---|---|
| Concrete per-call limit values | Implementation of each consuming call; recorded with rationale in the implementation record; not an ABI promise at major 0 |
| Cross-CPU concurrent-mutation safety | Integration with the P3-W14/P3-W08 TLB-shootdown transport in a future design; P4 proved current-path consistency only |
| Multi-granule walks (huge pages) | A future approved design; v0 walks the granule P4 proves |
| Performance-oriented copy strategies (caches, batching) | A future design with benchmark authority; no KPI in P5 |
| Shared-memory semantics (SharedRegion class) | ADR-034 future work; never implied by this boundary |

### 1.3 Out of Scope

Dispatch ordering (W06); handles/capabilities (W04/W05); envelope values
(W02); Guest markers (W07); fuzz generators (W08); telemetry transport
(W09); Stage-2 mutation APIs and page-table internals (P4); machine-memory
ABI (P8+); any completion or evidence claim.

## 2. Prerequisite assumed contracts

Form per the W01 ledger §3
([../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)):

```text
AC-03.1  The Guest's Stage-2 address space supports a diagnostic query that,
         for a Guest IPA, reports translation presence, the Host physical
         target, read/write permission state, and is sufficient to
         distinguish translation from permission faults.
Source:  ../../../p4/plans/p4-w02-stage2-address-space.md;
         ../../../p4/plans/p4-w06-fault-isolation-diagnostics.md
Failure: Blocked Prerequisite for implementation (design and host-seam work
         proceed). Documentation Gap if permission is queryable but memory
         type is not: v0 then rejects by policy only where type is
         expressible and records the limitation; it must not claim type
         checking it cannot perform.

AC-03.2  Stage-2 never maps a P2-protected (Hypervisor-owned) range for
         Guest access, and Guest RAM originates only from allocatable pages.
Source:  ../../../p2/plans/p2-w03-boot-memory-map-ownership.md;
         ../../../p2/plans/p2-w04-physical-page-allocation.md;
         ../../../p4/plans/p4-w03-guest-memory-image.md
Failure: Contract Conflict — if a query ever returns a protected-range
         translation, the accessor raises InvariantViolation (fail closed);
         the underlying ownership question stays with P2/P4 owners. Note
         P2-ACR-01 remains unresolved upstream and is carried visible; W03
         does not assume its resolution.

AC-03.3  Host Stage-1 maps all allocatable Host RAM, so access through a
         validated Stage-2-translated Host physical address cannot fault on
         the Host side.
Source:  ../../../p1/plans/p1-w08-host-stage1-address-space.md
Failure: Blocked Prerequisite — if Host-unmapped pages can back Guest RAM,
         the accessor's infallibility contract is false and this design must
         be revised; no per-access fault recovery is designed here.

AC-03.4  Semantic newtype conventions exist for addresses/identities
         (checked conversions, no naked usize).
Source:  ../../../p0/plans/p0-w15-address-identifier-type-safety.md
Failure: Documentation Gap — W03 defines its `GuestIpa`/`GuestLen` types in
         the project vocabulary and reconciles names at review if P0-W15
         lands with different spellings.

AC-03.5  Host-side unit/integration test gates exist and run Rust host tests.
Source:  ../../../p0/plans/p0-w07-development-quality-gates.md;
         ../../../p0/plans/p0-w08-host-side-testing-baseline.md
Failure: Blocked Prerequisite for execution evidence; tests are still written.

AC-03.6  The mapping granule P4 proves (expected 4 KiB) is the walk granule.
Source:  ../../../p4/plans/p4-w02-stage2-address-space.md (granule as
         recorded implementation fact)
Failure: Documentation Gap if multiple granules are proven; v0 then walks
         the smallest proven granule and records the limitation.
```

## 3. Resolved design decisions and their authority

### Decision 1 — IPA ranges, typed; Host pointers unreachable from Guest input

A Guest buffer argument is (base IPA, length). No Guest-supplied value is
ever cast to a Host pointer type; Host addresses exist only inside validated
segments produced from Stage-2 query results.

Rationale: makes INV-P5-01/02 structural. A call handler cannot form a Host
pointer from Guest input because no such conversion exists in the boundary's
types. Authority: ADR-007, ADR §19; P0-W15 vocabulary (AC-03.4).

### Decision 2 — Validate-then-access, two phases, all-or-nothing

Phase 1 (`validate_guest_range`) fully validates the range and produces
translated segments; phase 2 (access) performs the copy using only those
segments. If validation fails, no Guest-visible byte is read or written by
the operation.

Rationale: partial writes on a rejected request would make Guest-visible
state depend on where a fault occurred — harder to test, and a side effect on
a denied request (violating W02's validation-before-effect rule). All-or-
nothing costs a segment list per call, acceptable at P5 scale. Authority:
P5-V03/V10; W02 lifecycle rule S6.

### Decision 3 — Zero length is valid and vacuous

`GuestRange` with length 0 validates successfully and performs no access.

Rationale: a defined, deterministic outcome avoids a denial vector and
special cases at call sites; the alternative (rejecting zero) would make a
legal no-op a Guest fault. The rule is per-boundary so every consuming call
inherits it. Authority: design-owned (plan requires zero to be a *controlled*
case, not a specific verdict).

### Decision 4 — Per-page validation and segments; no contiguity assumption

The walk steps by the proven granule (AC-03.6); each page is individually
queried and yields a segment {HostPhysicalBase, range_offset, length}; a
range may begin and end mid-page; consecutive pages need not be physically
contiguous and are never coalesced into one access.

Rationale: Stage-2 mappings are per-page; contiguity assumptions are exactly
how unchecked-Host-access bugs are born. Segment-wise access keeps every
Host memory touch inside a queried translation. Authority: P4-W02 capability
shape; ADR §19.

### Decision 5 — Permission and memory-type rules

Read access requires Stage-2 read permission on every page; write requires
Stage-2 write. Execute permission is not consulted for data access. Guest
data buffers must be normal cacheable memory; device or otherwise non-normal
types are rejected (`MemoryTypeRejected`) because copy access to device
memory has architectural side effects (reads/writes with device semantics).

Rationale: direction-consistency implements "read-only write" as a controlled
case (P5-V03); the type rule prevents Guest-triggered device side effects
through a data boundary. Basis cited per Specification Investigation: AArch64
Stage-2 descriptor permission bits (S2AP) and attribute fields (MemAttr)
distinguish read/write permission and normal/device memory; the boundary's
vocabulary mirrors that distinction without importing encoding details.
Authority: task book §8; ADR-007.

### Decision 6 — Bounded length via a named limit mechanism, value at implementation

Every consuming call declares a `GUEST_DATA_LIMIT` (per call) from one named
constant family; requests with `len > limit` fail with
`LengthExceeded` before any walk; the walk's trip count is therefore bounded
by `limit / granule`. Values are fixed at implementation with recorded
rationale (per-call workload), reviewable, and are not ABI promises at
major 0.

Rationale: the plan removes "final maximum sizes" from W03's scope but
requires "maximum length" to be a controlled case; separating mechanism
(required now) from value (implementation fact) honors both. Unbounded walks
on Guest-controlled lengths are a denial vector. Authority: plan
out-of-scope note; P5-V03; design-owned mechanism.

### Decision 7 — Translations captured at validation, reused at access

Validation returns the segment list; access consumes exactly that list. No
re-walk between phases.

Rationale: re-walking would create a second decision point and more
TOCTOU surface within the operation; the captured list is the single
authority for what may be touched. Within one operation the Guest is stopped
(it issued the hypercall), so the capture-then-use chain is valid for the
operation's duration. Authority: design-owned; concurrency boundary in
decision 8.

### Decision 8 — Concurrency boundary: no concurrent Stage-2 mutation assumed

P4 proved current-path consistency only; cross-pCPU Stage-2 mutation during
a data operation is outside P5's proven scope. If a future design mutates
Stage-2 concurrently with data operations, it must integrate the P3
shootdown transport and revise this boundary; until then, such mutation is
an unhandled race and W08 scenarios must not construct it as a pass
condition.

Rationale: honesty about the proven foundation; the alternative — silently
assuming shootdown — would import an unproven mechanism. Authority: P4-W02
handoff boundary (FD-2); task book §1 Reserved.

### Decision 9 — Specification-investigation duty for architectural claims

Every permission/type/translation claim in this design cites its AArch64
Stage-2 basis (S2AP permissions, MemAttr attributes, translation-fault vs
permission-fault distinction in ESR ISS as diagnosis vocabulary). QEMU
observations appear only as test expectations, never as Core rules.

Rationale: task book §8 classifies Guest-data mapping/partial-access behavior
as Specification Investigation. Authority: task book §8; ADR-003 (QEMU is a
reference environment, not the architecture definition).

## 4. Fault-domain relationship to W02

`GUEST_MEMORY_FAULT` (W02 status class 11) is the only Guest-visible
vocabulary for every failure of this boundary. The internal causes
{Unmapped, PermissionDenied, MemoryTypeRejected, RangeInvalid, LengthExceeded}
exist for tests, telemetry, and diagnostics, and never change the Guest-visible
code. This split keeps P5-V03's required case coverage observable while
preserving W02's closed status table.

## 5. Open questions and labels

| Item | Classification | Handling |
|---|---|---|
| Stage-2 query cannot express memory type (if P4 lands so) | `Documentation Gap` | Reject by policy only where expressible; record limitation; no fabricated type checks |
| Whether limit values must be Guest-discoverable | Reserved | Not in v0; a future call contract may expose its own limit via W02's feature/call mechanism |
| Interaction of partial writes with Guest-visible state on *host-side* copy failure | `ADR Required` if it arises | The accessor treats post-validation Host faulting as an invariant (AC-03.3); a real need for recovery semantics would change the failure model — ADR process |
| Concurrent Stage-2 mutation tests in W08 | Boundary of scope | W08 may construct the *absence* of mutation; pass conditions requiring concurrent mutation are out of P5 evidence |
