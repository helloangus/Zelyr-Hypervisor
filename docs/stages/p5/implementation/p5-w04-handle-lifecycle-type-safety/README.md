# P5-W04 Handle Lifecycle and Type Safety — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The opaque object-reference boundary required by
[P5-W04](../../plans/p5-w04-handle-lifecycle-type-safety.md): handle
identity and opaqueness, the object table's lifecycle authority,
stale-reference protection across slot reuse, repeated-destroy and
owner-disappearance treatment, forged-value and generation/type-mismatch
rejection, and the explicit rule that a handle grants no authority.  
**Owner/change context:** P5-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W04. It converts the bounded
work-package plan into a typed object-reference mechanism with named
lifecycle operations. It deliberately does **not** freeze handle bit layouts
as a public contract, design garbage collection, define all future object
classes, design capability rights (P5-W05), or claim implementation evidence.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the supporting file its assigned step needs:

| Assigned step | Load |
|---|---|
| Scope classes, goal-to-baseline ledger, assumed contracts, decision rationale | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Modules, ownership, slot lifecycle state machine, concurrency model | [02-architecture-and-state.md](02-architecture-and-state.md) |
| Implement the handle codec and the object table | [03-code-contracts-handle-table.md](03-code-contracts-handle-table.md) |
| Implement allocation, lookup, destruction, and cascade operations | [04-code-contracts-lifecycle-ops.md](04-code-contracts-lifecycle-ops.md) |
| Execute the ordered workflow | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Judge acceptance, review security behavior, or hand off | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Before editing, the agent must also have read the repository `AGENTS.md`,
[documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P5 task book](../../task-book-v0.1.md), the
[P5-W04 plan](../../plans/p5-w04-handle-lifecycle-type-safety.md), the W01
entry boundary
([entry README](../p5-w01-entry-contract-reconciliation/README.md) and
[ledger](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)
§3), and the sibling boundaries: the
[W02 error boundary](../p5-w02-hypercall-abi-error-boundary/README.md) whose
`INVALID_HANDLE` class this design feeds, and
[P5-W05](../p5-w05-capability-rights-bootstrap-revocation/README.md), the
primary consumer of this identity boundary. This document is a proposed
design; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → P5-W04 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-013: Capability v0 is object handle + rights + generation; handles use
  generation against stale references. This design owns the handle and
  generation *mechanism*; W05 owns the rights and caller association built
  on it.
- ADR-051 and ADR §19: a handle identifies an object and grants no authority;
  no VM-ID, role, or first-VM logic may substitute for checks.
- ADR §12/§19: no Host pointer or object address is ever disclosed through a
  handle; lengths/indices use checked arithmetic.
- Task book §8: handle encoding, object-table representation, and lifetime
  mechanics are **Implementation Choice**, selected in approved detailed
  design — which this document is. The plans freeze none of these values.
- Plan out-of-scope honored: no garbage collection, no all-future-object
  design, no Rust type/API freezing beyond this design's own contracts, no
  rights semantics.

Classification:

- **Required** — the opaque handle value type and codec (decode/encode with
  field-range validation); the fixed-capacity object table as sole lifecycle
  authority; per-slot generation discipline including slot-reuse and
  retire-on-wrap; type-tag checking; destruction with cascade to owned
  objects; the invalid-cause vocabulary mapping to W02's `INVALID_HANDLE`
  class; the no-authority rule (no rights in any table API); the lock
  discipline and closure-based access form; host-testable lifecycle seam.
- **Reserved** — additional object classes (interrupt, virtual IRQ,
  Endpoint, Notification, SharedRegion, device, memory, service objects —
  task book §1 Reserved); guest-visible object creation (P10+ management
  work; v0 registration is internal/bootstrap only); reference counting or
  lock-free table strategies; handle-pattern stability across the stage
  boundary; quota policy.
- **Out of Scope** — capability/rights/grant/revoke semantics
  ([P5-W05](../p5-w05-capability-rights-bootstrap-revocation/README.md));
  dispatch ordering ([P5-W06](../p5-w06-dispatch-permission-containment/README.md));
  Guest-data ([P5-W03](../p5-w03-guest-data-safety/README.md)); Guest
  scenario markers ([P5-W07](../p5-w07-validation-guest-isolation-suite/README.md));
  fuzz generators and stress ([P5-W08](../p5-w08-host-fuzz-stress-smp-baseline/README.md));
  VM/vCPU lifecycle itself (P4 owns those objects); machine ABI (P8+).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Opaque identity; no Host-address disclosure (P5-T04) | [03 §1–§2](03-code-contracts-handle-table.md), [02 §5](02-architecture-and-state.md) | P5-V04 (W04-DV03) |
| Existence, destruction, stale-reference protection after reuse (P5-T05) | [04 §2–§3](04-code-contracts-lifecycle-ops.md), [02 §3](02-architecture-and-state.md) | P5-V04 (W04-DV01, DV02) |
| Repeated destroy; owner destruction; forged values; generation/type mismatch (P5-T05/T06) | [04 §3–§4](04-code-contracts-lifecycle-ops.md) | P5-V04/V05 (W04-DV01, DV02) |
| Handle does not grant authority (P5-T06) | [02 §5](02-architecture-and-state.md), [04 §5](04-code-contracts-lifecycle-ops.md) | review W04-DV04; behavioral proof with W05/W07 |
| Multi-pCPU readiness, telemetry needs, future-class extension | [02 §4](02-architecture-and-state.md), [03 §3](03-code-contracts-handle-table.md) | review W04-DV04, DV05 |
| Integration with W05 authority checks; identity ≠ permission | [README handoff](#downstream-handoff), [06 §4](06-validation-and-handoff.md) | W04 closure (W04-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18): no Rust sources exist; P4's VM/vCPU
objects, lifecycle, and repeatability packages are plans without
implementation or verification records; P3 synchronization and P2 allocation
packages are plans likewise; the W02 error boundary is a sibling design in
progress. Inputs below are **assumed contracts** under the W01 model, cited
by plan path, with explicit failure boundaries.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Valid handles identify objects; stale/forged/wrong-type are rejected (P5-V04/V05) | No handle or table code exists | Handle codec + object table + lifecycle operations (this design) | Rejection must be a property of the lookup discipline, not call-site care | W04 (this design) | W04-DV01–DV03 |
| Stale references stay invalid across slot reuse (P5-T05) | Nothing exists | Generation discipline with bump-on-free and retire-on-wrap | Reuse without generation bump is the classic UAF-equivalent; the rule must live in the free/alloc path | W04 | W04-DV01 |
| Handles reference P4-established objects without redesigning them (FD-3) | P4 object shapes are plans only | Assumed contract AC-04.1 (addressable VM/vCPU identities) with `Blocked Prerequisite` boundary | The table stores references; if no addressable identity exists, the table has nothing to reference | P4 owners; W04 consumes | W04-DV05 blocked until P4 evidence |
| A handle grants no authority (P5-T06, P5-V07 basis) | No authority mechanism exists (W05 sibling) | Structural rule: no table API carries rights; lookup yields identity only | If identity lookup returned privileges, W05's check would be bypassable by construction | W04 rule; W05 mechanism; W06 ordering | W04-DV04; behavioral proof arrives with W05/W07 |
| Repeated lifecycle stress preserves invariants (P5-V13 basis) | No code exists | Host-testable table seam + stated invariants (totality, no stale acceptance) | Stress evidence requires an exercisable seam, not a Guest | W04 seam; W08 fuzz/stress | W04-DV01; W08 evidence |
| Multi-pCPU readiness (task book §1 Reserved; P5-V14 basis) | P3 sync packages are plans | Lock discipline per AC-04.2 and the closure-based access form | Concurrent lookup/destroy races must be impossible by design, not by hope | W04; P3-W06 discipline | W04-DV04; W08 two-pCPU evidence |

No ledger row requires an architecture decision beyond the task book's
Implementation-Choice classification; each concrete selection is recorded
with rationale in [01-scope-and-foundations.md](01-scope-and-foundations.md)
§decisions.

## Resolved design decisions and their authority

Full statements and rationale in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §decisions.
Summary:

1. Guest-visible handle is one opaque u64; internal fields (type tag,
   generation, slot index) are not a Guest contract, and zero is invalid.
2. Fixed-capacity object table is the sole lifecycle authority; capacity is
   an implementation fact with recorded rationale.
3. Generation increments on every free; a slot whose generation would wrap
   is retired permanently.
4. Object classes form an extensible enum; v0 defines Vm and Vcpu only;
   class mismatch is a defined invalid cause.
5. Handles are created only by internal/bootstrap registration; P5 defines
   no Guest-visible object-creation call.
6. Destruction is table-mediated with an owner cascade (Vm destruction
   invalidates contained Vcpu slots) in one transaction, vCPU slots first.
7. Lookup returns identity, never rights; object access uses a
   closure-under-lock form; escaping an object reference into unsynchronized
   use is prohibited.
8. All invalid shapes — bad encoding, free slot, generation mismatch, class
   mismatch — map to W02's single `INVALID_HANDLE` class with internal
   causes; the Guest observes no sub-cause.
9. Table state is shared and lock-protected under the P3-W06 discipline;
   no lock is held during Guest memory access or telemetry emission.

## Work breakdown and loading order

1. Read [01](01-scope-and-foundations.md) for the ledger, assumed contracts
   AC-04.1–AC-04.5, and decisions 1–9.
2. Read [02](02-architecture-and-state.md) for modules, the slot lifecycle
   state machine, and the concurrency model.
3. Implement the codec and table per
   [03](03-code-contracts-handle-table.md), then the lifecycle operations per
   [04](04-code-contracts-lifecycle-ops.md), in the order of
   [05](05-implementation-workflow.md).
4. Record decisions and deviations in
   `../p5-w04-handle-lifecycle-type-safety-record.md` when implementation
   starts; record commands and results in
   `../../verification/p5-w04-handle-lifecycle-type-safety-verification.md`
   when evidence exists. Neither this design nor any record claims W04 or P5
   complete.

## Explicitly excluded interfaces

No capability, rights, grant, revoke, or delegation semantics (W05); no
dispatch ordering (W06); no Guest-data types (W03); no hypercall envelope
values (W02); no Guest markers (W07); no fuzz generators (W08); no telemetry
transport (W09). No Guest-visible object creation, destruction request, or
handle minting call is defined in P5 — registration and destruction
triggers are internal/bootstrap mechanisms. No VM/vCPU lifecycle semantics
are designed here; the table references P4-owned objects and never mutates
them. No garbage collector, finalizer, or persistent handle store exists. No
crate/file layout is fixed. Any implementation that leaks a Host address
through a handle, returns rights from a lookup, or lets an object reference
escape unsynchronized use violates the design and is a review stop.

## Downstream handoff

Per the [P5 plan index](../../plans/README.md) consumer map:

- **W05**
  ([capability, rights, bootstrap, revocation](../p5-w05-capability-rights-bootstrap-revocation/README.md))
  receives the identity boundary: valid-handle semantics, the class
  vocabulary, the invalid-cause list, and the rule that authority records
  key to (caller, target handle) without the table ever holding rights.
  W05's grant/revoke must compose with the table's destroy cascade
  (a destroyed target must make derived authority unusable — checked in
  W05's design, ordered by W06).
- **W06**
  ([dispatch, permission, containment](../p5-w06-dispatch-permission-containment/README.md))
  receives defined invalid-reference outcomes: every lookup failure is
  W02's `INVALID_HANDLE` with an internal cause, uniform for all callers,
  and lookup precedes authority in the ordering W06 assembles.
- **W07**
  ([validation guest suite](../p5-w07-validation-guest-isolation-suite/README.md))
  receives the externally observable reference semantics: valid, stale,
  forged, zero/max, wrong-type, and repeated-destroy behaviors all yield the
  same Guest-visible class, enabling determinate markers.
- **W08**
  ([host fuzz/stress/SMP baseline](../p5-w08-host-fuzz-stress-smp-baseline/README.md))
  receives the create/lookup/destroy/recreate stress seam, the stated
  invariants (no stale acceptance, totality, bounded capacity), and the
  declared two-pCPU scenario shape (closure-based access).
- **W09/W10** receive the handle-event vocabulary (lifecycle events, invalid
  causes) for redaction and regression; P6 receives the extensible class
  mechanism only through evidenced closeout — interrupt and virtual-IRQ
  object classes enter through their own P6 designs using the same table
  rules.
