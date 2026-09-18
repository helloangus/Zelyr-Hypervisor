# P5-W04 Scope, Foundations, and Design Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W04 detailed design](README.md).

## 1. Scope classification in detail

### 1.1 Required

| Item | Statement |
|---|---|
| Handle value type | One opaque u64 carried through W02 argument registers; internal fields {type tag, generation, slot index}; never a Host address; zero invalid |
| Handle codec | Total decode validating field ranges, defined type tag, slot within capacity; no panic on any u64 |
| Object table | Fixed-capacity slot array; sole authority for existence, type, and lifecycle of referenced objects |
| Generation discipline | Generation stored per slot; incremented on every free; stale = any generation mismatch; retire-on-wrap |
| Type safety | Per-slot class; class tag in handle; mismatch is a defined invalid cause |
| Lifecycle operations | Internal registration (creation), lookup, destruction with owner cascade; repeated destroy defined |
| Invalid-cause domain | {BadEncoding, FreeSlot, GenerationMismatch, ClassMismatch} → W02 `INVALID_HANDLE`, causes internal-only |
| No-authority rule | No table API carries or returns rights; lookup yields identity only |
| Concurrency form | Table-wide lock (P3-W06 discipline); closure-based access; no unsynchronized reference escape |
| Host-testable seam | Table and codec exercisable without a VM or QEMU; stress loop support |

### 1.2 Reserved

| Item | Trigger for activating |
|---|---|
| New object classes (interrupt, virtual IRQ, Endpoint, Notification, SharedRegion, device, memory, service) | The owning future stage's approved design extends the class enum and its registration paths |
| Guest-visible object creation/destruction calls | P10+ management-plane work with its own ABI design |
| Reference counting or lock-free strategies | A future design with a demonstrated contention need and W08 evidence |
| Handle-pattern stability across stages | A future versioning decision; at major 0 handle bits are opaque and unstable |
| Quotas on objects per caller | First policy-bearing mechanism that needs them |

### 1.3 Out of Scope

Capability/rights/grant/revoke (W05); dispatch ordering (W06); Guest-data
(W03); envelope values (W02); Guest markers (W07); fuzz/stress harnesses
(W08); telemetry transport (W09); VM/vCPU lifecycle semantics (P4);
management plane (P10+); any completion or evidence claim.

## 2. Prerequisite assumed contracts

Form per the W01 ledger §3
([../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)):

```text
AC-04.1  P4-established VM and vCPU objects are addressable by stable
         identity (identifier newtypes) such that a table entry can
         reference one without owning or redesigning it.
Source:  ../../../p4/plans/p4-w04-vcpu-entry-exit.md;
         ../../../p4/plans/p4-w09-closeout-p5-handoff.md
Failure: Blocked Prerequisite — without addressable identities the table has
         no referents; W04's table logic and host seam can still be built
         and tested against test referents, but integration is blocked.

AC-04.2  Synchronization primitives and a lock-order discipline exist for
         shared registry-like state, with defined IRQ-context rules; small
         fixed-size allocation is available if the table is dynamically
         backed.
Source:  ../../../p3/plans/p3-w06-concurrency-synchronization.md;
         ../../../p2/plans/p2-w05-dynamic-small-allocation.md
Failure: Blocked Prerequisite at implementation; a static backing avoids the
         allocation dependency but never the locking one.

AC-04.3  The W02 error boundary defines INVALID_HANDLE (class 6) as the
         Guest-visible class for every reference failure, with internal
         causes never Guest-visible.
Source:  ../p5-w02-hypercall-abi-error-boundary/README.md (sibling design)
Failure: Contract Conflict between the two designs if W02's class set or
         cause rules change incompatibly — resolved at design review, not by
         local adaptation.

AC-04.4  Host-side unit/integration test gates exist and run Rust host tests.
Source:  ../../../p0/plans/p0-w07-development-quality-gates.md
Failure: Blocked Prerequisite for execution evidence; tests still written.

AC-04.5  P4's lifecycle facts include a defined stop/cleanup boundary a VM
         destruction cascade can hook.
Source:  ../../../p4/plans/p4-w07-repeatability-telemetry.md;
         ../../../p4/task-book-v0.1.md P4-V10
Failure: Documentation Gap — the cascade operation is still defined; its
         trigger integration is recorded as pending until P4's stop path is
         evidenced.
```

## 3. Resolved design decisions and their authority

### Decision 1 — Opaque u64 handle; fields are not contract

The Guest-visible handle is a single 64-bit value. Internal layout:
bits[63:40] slot index (24), bits[39:16] generation (24), bits[15:0] type
tag (16). All-zero is invalid by construction (type tags start at 1). The
Guest must treat the value as opaque; no field semantics, ordering, or
predictability is promised, and the layout may change between designs
without notice at major 0.

Rationale: opaqueness is the security property — nothing about Host
structure is derivable from a handle (ADR §12: no Host pointers via object
references). Encoding all three fields in the value makes validation purely
local (no lookup needed to reject a forged shape). Type tags start at 1 so
zero is never valid (required test case). Authority: ADR-013 (slot +
generation), ADR §12, task book §8 (encoding as Implementation Choice).

### Decision 2 — Fixed-capacity table as sole lifecycle authority

One object table owns existence: a handle is valid if and only if the table
says so. Capacity is fixed at implementation (proposed default: 64 slots)
with recorded rationale; no other structure may assert object existence.

Rationale: a single authority makes stale-reference protection provable;
parallel registries would create contradictory truth. Fixed capacity avoids
an allocation dependency in the lifecycle path (AC-04.2) and bounds every
operation. Authority: task book §8 (object-store mechanics as
Implementation Choice); design-owned.

### Decision 3 — Generation increments on every free; retire-on-wrap

Each slot carries a generation counter. Freeing increments it; allocation
stores the new generation into minted handles. A lookup whose handle
generation differs from the slot's is stale. A slot whose generation would
wrap is retired permanently (never reallocated).

Rationale: bump-on-free is the exact mechanism ADR-013 requires against
stale references after reuse; retire-on-wrap closes the theoretical 2^24
rebind window instead of ignoring it. Authority: ADR-013; ADR §12
(generation against stale references).

### Decision 4 — Extensible class enum; v0 defines Vm and Vcpu

`ObjectClass` is an enum with a stable-to-extend value space; v0 members are
Vm (tag 1) and Vcpu (tag 2). Future classes (interrupt, IPC primitives,
devices, memory, service objects) enter through their own designs. A handle
presented where another class is required is `ClassMismatch` — rejected
without interpretation of the referent.

Rationale: P5-V05 requires wrong-type rejection now; defining only P4-
backed classes avoids pre-designing P6+ semantics (stage boundary). The tag
inside the handle makes class rejection independent of table contents
(defense in depth). Authority: P5-T06; task book §1 Reserved.

### Decision 5 — No Guest-visible creation in P5; internal registration only

Handles enter the table only via an internal `register` API used by
Hypervisor/bootstrap/test setup (mirroring ADR-038's bootstrap-creates
principle at test scale). No hypercall mints or destroys object references
in P5; destruction triggers are internal lifecycle events (P4 stop path) or
bootstrap teardown.

Rationale: object creation policy is management-plane work (P10+); minting
handles by hypercall would design it early. P5's loops (grant/use/revoke)
need registration and cascade, not creation rights. Authority: task book
§1 Out of scope (dynamic VM configuration API); ADR-038 analogy.

### Decision 6 — Destruction with owner cascade in one transaction

Destroying a Vm-class object first frees every Vcpu slot referencing that
Vm (each with generation bump), then frees the Vm slot. The cascade is one
atomic operation under the table lock. Repeated destroy of the same handle
is `GenerationMismatch` (or `FreeSlot` if never reallocated).

Rationale: owner disappearance must not leave dangling contained objects
(P5-T05 explicitly lists owner destruction); one transaction avoids
partially-cascaded states observable by concurrent lookups. The vCPU-first
order gives destroy notifications a predictable direction. Authority:
P5-T05; design-owned mechanics.

### Decision 7 — Identity only; closure-based access

Lookup yields an identity value (class + referent identifiers). Operations
needing the live object run as a closure under the table lock
(`with_object`); returning a reference for unsynchronized use is not
offered. No table API parameter or result expresses rights or authority.

Rationale: keeps "a handle grants no authority" structural (P5-T06), and
serialization under the table lock removes lookup-vs-destroy races without
reference-count machinery (Reserved). Authority: P5-T06; ADR-013/ADR-051;
P3-W06 discipline.

### Decision 8 — Uniform Guest-visible rejection

Every invalid shape maps to W02's `INVALID_HANDLE` with an internal cause
({BadEncoding, FreeSlot, GenerationMismatch, ClassMismatch}). The Guest
cannot distinguish causes; telemetry can.

Rationale: sub-cause disclosure would hand attackers an oracle about table
state; internal causes preserve P5-V04's evidence granularity for tests.
Authority: W02 boundary design (status table rule); P5-V04.

### Decision 9 — Lock discipline

One table-wide lock (spinlock per P3-W06 rules) guards slot state; hold
times are bounded (no Guest memory access, no telemetry emission, no
callbacks under lock except the `with_object` closure whose cost discipline
is the caller's contract with W06).

Rationale: simple, correct, adequate at P5 scale; finer-grained schemes are
Reserved with a contention-evidence trigger. Authority: P3-W06 assumed
contract; task book §8.

## 4. Relationship to W05 and W06

- W05 keys authority records to (caller identity, target handle) and checks
  rights after W04 lookup confirms identity; the table never consults and
  never stores rights.
- When W04 destruction cascades, W05 must learn of target loss to retire
  derived authority; the composition rule (destroy ⇒ authority unusable) is
  a W05 design obligation and a W06 ordering obligation; W04 provides the
  destruction event point in its API for exactly that purpose
  ([04 §4](04-code-contracts-lifecycle-ops.md)).

## 5. Open questions and labels

| Item | Classification | Handling |
|---|---|---|
| Whether future classes need per-class subtables | Design freedom of the future class design | The mechanism (one table, class-tagged slots) is what P5 evidences; P6+ may revise with its own design |
| Handle values in telemetry | Redaction duty of W09 | W04 emits lifecycle events with handle *presence* and causes; W09 redacts values per its policy |
| Generation wrap in a 64-slot table | Closed by decision 3 | Retire-on-wrap; no further handling |
| Any request to let a Guest destroy objects via hypercall | Out of scope (P10+) | Route to management-plane design; label `ADR Required` only if it would alter the capability model |
