# P5-W03 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Complete the Coding-Guidelines preflight. Confirm the W01 entry review and
re-check the assumed contracts of
[01 §2](01-scope-and-foundations.md) at implementation entry: AC-03.1
(Stage-2 query), AC-03.2 (protected-range exclusion; P2-ACR-01 visible),
AC-03.3 (Host Stage-1 coverage), AC-03.4 (newtype vocabulary), AC-03.5
(host gates), AC-03.6 (granule).

Stop and obtain direction instead of guessing when any of the following
occurs:

- the implemented P4 query cannot express permission or memory type —
  apply the AC-03.1 `Documentation Gap` handling; never fabricate a check;
- a query returns a protected-range translation — do not "fix" it in W03;
  raise the invariant, record a `Contract Conflict` against the P4/P2
  ownership contract, and stop the affected step;
- the granule or Host-Stage-1 coverage differs from the assumed facts —
  revise this design through review; do not special-case locally;
- implementing a consuming call (W06) seems necessary to test the boundary —
  it is not; the fake-space seam exists for exactly that, and reaching into
  W06 scope is a violation;
- a reviewer asks for shared-memory, mapping, or grant-table features —
  Out of Scope; route to the future owning design.

## 2. Ordered implementation steps

### Step 1 — range vocabulary and checked construction

Target: G1 types ([03 §1–§3](03-code-contracts-guest-range.md)).

Work: implement `GuestIpa`, `GuestLen`, `GuestAccessKind`, and
`GuestRange::new` with the limit parameter. Keep arithmetic checked; no
wrapping paths.

Suggested observation: host build with clippy clean; overflow-debug tests.

**Acceptance:** construction is total and checked; zero length constructs;
no newtype-free integer parameters exist on the boundary's public surface.  
**Failure/blocker:** any unchecked arithmetic is a review stop (Coding
Guidelines; decision 1 of [01](01-scope-and-foundations.md)).

### Step 2 — fault domain and the fake Stage-2 space

Target: G5 types; test-only `FakeStage2Space`.

Work: implement `GuestDataFault` with telemetry payloads and the W02
conversion; implement the fake port in the test module with page-granular
control over presence, permissions, type, and a protected-range predicate.

**Acceptance:** conversion is the only constructor path to the
`GUEST_MEMORY_FAULT` boundary error; the fake space can express every
[03 §5](03-code-contracts-guest-range.md) test case.  
**Failure/blocker:** a telemetry payload containing a Host address is a
review stop (04 §4).

### Step 3 — Stage-2 query port binding

Target: G2 trait plus the Arch implementation over P4's query capability.

Work: define the trait per [03 §4](03-code-contracts-guest-range.md); bind
it to the P4-implemented query; map P4's permission/type facts into the
boundary vocabulary; attach the Specification-Investigation note (decision 9
of [01](01-scope-and-foundations.md)). Any `unsafe` in the binding follows
P0-W10 governance.

**Acceptance:** the binding adds no Stage-2 mechanism and mutates nothing;
context rules (no unbounded blocking) are documented at the trait.  
**Failure/blocker:** AC-03.1 not evidenced — complete the trait and fake
work, mark Arch binding acceptance blocked, record the dependency.

### Step 4 — validator

Target: G3 `validate_guest_range`
([03 §5](03-code-contracts-guest-range.md)).

Work: implement the page walk exactly per the pseudocode: limit already
checked at construction; per-page query; direction and type checks;
protected-range guard; segment tiling; all-or-nothing.

**Acceptance:** every required negative case yields its designed cause; Ok
results tile exactly; no memory access occurs during validation.  
**Failure/blocker:** a temptation to validate lazily (per-page during copy)
is a design violation (decision 2 of [01](01-scope-and-foundations.md)) —
stop.

### Step 5 — accessors

Target: G4 `copy_from_guest` / `copy_to_guest`
([04 §2–§3](04-code-contracts-guest-accessor.md)).

Work: implement segment-driven volatile byte access; length-equality
precondition checks escalate as invariants; no coalescing; document the
performance-reserved inner loop.

**Acceptance:** byte-exact round trips on discontiguous fake mappings; no
access outside segments; no allocation in the copy path.  
**Failure/blocker:** post-validation Host faulting observed in tests
contradicts AC-03.3 — record a `Contract Conflict` against the P1 host
Stage-1 contract; do not add per-access fault recovery locally.

### Step 6 — host validation suite

Target: host test module (P0-W07/W08 assumed gates).

Work: implement every unit case named in
[03 §5](03-code-contracts-guest-range.md) and
[04 §5](04-code-contracts-guest-accessor.md), plus the property harness
surface W08 consumes (totality, all-or-nothing, tiling, boundedness).

**Acceptance:** all cases designed in this package have executing tests or
explicitly recorded blocked status; no test depends on a real Guest.  
**Failure/blocker:** host gates unavailable (AC-03.5) — tests written,
execution recorded blocked.

### Step 7 — closure review

Work: run the validation matrix in
[06](06-validation-and-handoff.md) §2, confirm the handoff checklist, and
record evidence in
`../../verification/p5-w03-guest-data-safety-verification.md`; record
decisions (including the fixed limit values and their rationale) and
deviations in `../p5-w03-guest-data-safety-record.md`. Completion is claimed
only in those records, only for what ran.

## 3. Evidence destinations

| Artifact | Path |
|---|---|
| Implementation record (decisions, limit values, deviations, changed files, `unsafe` delta) | `../p5-w03-guest-data-safety-record.md` |
| Verification record (commands, environments, results, run/not-run) | `../../verification/p5-w03-guest-data-safety-verification.md` |
| Fault-cause vocabulary consumed by | [P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md) via W03's event vocabulary in [04 §4](04-code-contracts-guest-accessor.md) |
