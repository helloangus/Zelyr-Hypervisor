# P5-W04 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Complete the Coding-Guidelines preflight. Confirm the W01 entry review and
re-check the assumed contracts of
[01 §2](01-scope-and-foundations.md): AC-04.1 (P4 identity), AC-04.2
(sync/allocation), AC-04.3 (W02 sibling classes), AC-04.4 (host gates),
AC-04.5 (P4 stop-path hook).

Stop and obtain direction instead of guessing when any of the following
occurs:

- P4 identity shapes are unavailable — build the table against test
  referents; mark integration acceptance blocked; do not invent VM/vCPU
  lifecycle;
- the W02 sibling design's class set changes incompatibly —
  `Contract Conflict` between designs; resolve at design review; do not
  adapt silently;
- a use case wants rights, quotas, or delegation in the table — Out of
  Scope; route to W05 or the future owning design;
- a use case wants a Guest-visible create/destroy call — Out of Scope
  (P10+); route to the management-plane design;
- lock-free or refcounted strategies are requested — Reserved with a
  W08-contention-evidence trigger; do not implement opportunistically.

## 2. Ordered implementation steps

### Step 1 — handle codec

Target: H1 ([03 §1–§2](03-code-contracts-handle-table.md)).

Work: implement `ObjectHandle` layout constants, `encode`, total `decode`
with the documented field checks, and `ObjectClass` tags.

Suggested observation: host build; exhaustive boundary-value tests.

**Acceptance:** decode is total; zero invalid; undefined tags and
out-of-range slots are `BadEncoding`; no panic on any u64.  
**Failure/blocker:** any decode path that can panic or infer Host state is
a review stop.

### Step 2 — object table core

Target: H2 ([03 §3](03-code-contracts-handle-table.md)).

Work: implement `ObjectTable` with fixed capacity, slot array, free list,
generation fields, retire-on-wrap, and the invariant-checking test harness
(walk slots + free list after every operation, test builds only).

**Acceptance:** table invariants hold across arbitrary operation sequences
in the harness; capacity and retirement rules behave as specified.  
**Failure/blocker:** a free-list shortcut that could hand out a retired
slot is a design violation (decision 3 of [01](01-scope-and-foundations.md))
— stop.

### Step 3 — lifecycle operations

Target: H3 ([04 §1–§4](04-code-contracts-lifecycle-ops.md)).

Work: implement `register`, `decode_and_validate`, `with_object`,
`destroy`, and `destroy_vm_cascade` exactly per contracts, including the
cause classification and the event emission points (outside the lock).

**Acceptance:** every lifecycle case in
[04](04-code-contracts-lifecycle-ops.md) validation notes has an executing
test; cascade is transactional under the lock; repeated destroy is stale.  
**Failure/blocker:** any path that leaves a half-cascaded state observable
is a review stop (decision 6 of [01](01-scope-and-foundations.md)).

### Step 4 — boundary conversion and event vocabulary

Target: cause → `GuestBoundaryError` conversion
([04 §5](04-code-contracts-lifecycle-ops.md)); lifecycle event constants
for the W09 vocabulary.

Work: implement the single conversion; define `registered` / `destroyed` /
`vm_destroyed` event payloads (class, counts — no referent identities, no
handle values unless W09's policy says otherwise).

**Acceptance:** Guest-visible outcomes are uniformly `INVALID_HANDLE`
(cause dropped) or `RESOURCE_EXHAUSTED` (capacity); no Host address or
referent id can reach events.  
**Failure/blocker:** a second construction site of `InvalidHandle` is a
review stop.

### Step 5 — host validation suite and stress seam

Target: host test module (P0-W07/W08 assumed gates).

Work: corpora tests (random/zero/max forged handles, stale, reused,
cross-class, generation off-by-one), lifecycle case tests, cascade tests,
and the create/lookup/destroy/recreate stress loop with invariant checking
that [P5-W08](../p5-w08-host-fuzz-stress-smp-baseline/README.md) consumes.
Add the declared two-pCPU exercise shape (concurrent with_object vs
destroy on distinct handles) if the P0 host environment supports threads;
otherwise document the seam and mark execution blocked.

**Acceptance:** no stale handle is ever accepted across the full corpus and
stress runs; invariants hold after every operation.  
**Failure/blocker:** host gates or thread support unavailable (AC-04.4) —
record blocked; do not substitute weaker evidence.

### Step 6 — integration points (blocked-ready)

Target: H4 referent binding; W05 destroy-event hook; P4 stop-path hook.

Work: wire the `ObjectRef` bindings to P4 identities where P4 is
implemented; otherwise define the binding surface and mark it blocked
(AC-04.1). Expose the destroyed-event hook W05 requires. Document the
cascade-trigger duty of the P4 stop path (AC-04.5) as the integration
point; do not implement P4's lifecycle.

**Acceptance:** the package compiles and tests green with test referents;
integration points are explicit and traceable to their ACs.  
**Failure/blocker:** missing upstream facts stay blocked; nothing is
stubbed to look integrated.

### Step 7 — closure review

Work: run the validation matrix in
[06](06-validation-and-handoff.md) §2, confirm the handoff checklist, and
record evidence in
`../../verification/p5-w04-handle-lifecycle-type-safety-verification.md`;
record decisions (capacity value and rationale, layout constants) and
deviations in
`../p5-w04-handle-lifecycle-type-safety-record.md`. Completion is claimed
only in those records, only for what ran.

## 3. Evidence destinations

| Artifact | Path |
|---|---|
| Implementation record (capacity, layout, deviations, changed files, `unsafe` delta) | `../p5-w04-handle-lifecycle-type-safety-record.md` |
| Verification record (commands, environments, results, run/not-run) | `../../verification/p5-w04-handle-lifecycle-type-safety-verification.md` |
| Lifecycle event vocabulary consumed by | [P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md) |
