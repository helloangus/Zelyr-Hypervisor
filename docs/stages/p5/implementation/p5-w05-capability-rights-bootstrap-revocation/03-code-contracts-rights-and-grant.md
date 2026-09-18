# P5-W05 Code Contracts — Rights Vocabulary and Grant Path

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W05 detailed design](README.md). Checklist §3 template.
Names are internal Rust API; rights encodings never cross the Guest
boundary in v0.

## 1. `RightsSet` (type)

```text
Name and stability: RightsSet(u32) bitset with named-class constants;
  internal.
Purpose and caller: the operation-level rights vocabulary (P5-T08); used by
  grant (held rights), by call designs/W06 (required rights), and by the
  check (superset evaluation).
Inputs / outputs: constants OBSERVE = 1<<0, CONTROL = 1<<1, MODIFY = 1<<2,
  DESTROY = 1<<3, DELEGATE = 1<<4; constructors that reject undefined bits;
  contains(required), union, intersection.
Preconditions / postconditions: only bits 0..=4 constructible; DELEGATE is
  never granted in v0 (grant rejects it — decision 3 of 01); a required set
  of zero cannot be constructed (an operation must declare at least one
  class).
State and ownership: value type.
Concurrency/allocation: trivially shareable; allocation-free.
Errors and failure guarantee: construction is the only fallible step.
Security/authorization checks: the type cannot express "any right" or
  "no requirement"; diagnostics may print class names, never Guest-visibly.
Logic: bitmask with checked construction; no arithmetic that could set
  undefined bits.
Validation: unit tests — construction accept/reject per bit, DELEGATE
  grant rejection, empty-required rejection; review that no constants
  beyond bit 4 exist.
```

Class semantics (fixed here so call designs and tests share one meaning):
Observe — read state/inspect the object; Control — trigger lifecycle
transitions; Modify — mutate object state or data; Destroy — destroy the
object; Delegate — transfer authority (future; unimplemented).

## 2. `CallerId` (type)

```text
Name and stability: CallerId — the VM identity of the execution context
  (value supplied by the dispatch context per AC-05.3); internal newtype.
Purpose and caller: the subject of every record; compared, never
  interpreted.
Inputs / outputs: obtained from the dispatch flow (W06) for guest paths;
  constructed from test identities in host seams.
Preconditions / postconditions: identifies exactly one VM context; no
  ordering or privileged values exist in this type's semantics.
State and ownership: value.
Concurrency/allocation: value semantics.
Errors and failure guarantee: —
Security/authorization checks: the type carries no privilege; any code
  branching on a CallerId *value* (rather than equality against a record
  subject) is a design violation — review gate.
Logic: transparent wrapper; no constants such as "the privileged VM".
Validation: review for identity-valued logic; property tests use multiple
  synthetic subjects.
```

## 3. `AuthorityRecord` and `GrantHandle` (types)

```text
Name and stability: AuthorityRecord { subject: CallerId, target_slot:
  usize, target_generation: u32, target_class: ObjectClass, rights:
  RightsSet, state: Active | Revoked }; GrantHandle(usize) — internal
  record identifier.
Purpose and caller: the record is the unit of authority; the handle is the
  grantor's only way to address it for revocation.
Inputs / outputs: —
Preconditions / postconditions: records are constructed only inside
  grant (private construction); a GrantHandle always names an unreclaimed
  record until reclaim invalidates it (use after reclaim = InvariantViolation).
State and ownership: owned by the record table.
Concurrency/allocation: protected by the table lock; fixed-size.
Errors and failure guarantee: —
Security/authorization checks: GrantHandle is never Guest-visible, never
  enters the W04 object table in v0, and never appears in logs (telemetry
  uses record presence/counts, not handles).
Logic: —
Validation: review that no public constructor exists for AuthorityRecord;
  test that a reclaimed GrantHandle use escalates as invariant.
```

## 4. `grant` (function — the only creation path)

```text
Name and stability: grant(&self, subject: CallerId, target:
  &ValidatedTarget, rights: RightsSet) -> Result<GrantHandle, TableFull>;
  internal; NOT Guest-reachable.
Purpose and caller: bootstrap/test authority creation (P5-T09); called by
  Hypervisor initialization and by W07's two-context setup through its own
  design.
Inputs / outputs: in — subject, a W04-validated target (slot, generation,
  class — the check consumes the W04-validated shape, AC-05.2), held
  rights; out — GrantHandle or capacity failure.
Preconditions / postconditions: pre — rights contains no DELEGATE (v0);
  target validated by W04. Post — exactly one Active record exists with
  those fields; the returned handle names it.
State and ownership: one record slot transitions Empty → Active.
Concurrency/allocation: table lock held for the insert; fixed capacity;
  no allocation beyond the table.
Errors and failure guarantee: TableFull → W02 RESOURCE_EXHAUSTED at the
  granting layer (decision 9 of 01); no partial state.
Security/authorization checks: caller authorization for grant is by
  construction (bootstrap context only); should grant ever become
  Guest-reachable, that is a new design with DELEGATE semantics — not this
  one.
Logic:
  function grant(subject, target, rights):
    assert rights has no DELEGATE            // v0 rule
    lock()
    if no free record slot: { unlock(); return Err(TableFull) }
    record = Active { subject, target.slot, target.generation,
                      target.class, rights }
    handle = insert(record)
    unlock()
    emit granted { }                         // outside lock; no subject values
    return Ok(handle)
Validation: unit tests — DELEGATE rejection, capacity exhaustion, record
  contents exactly as granted; two-context setup exercise (W07 seam).
```

## 5. Target-shape contract with W04

`ValidatedTarget` is the W04-validated (slot, generation, class) triple —
the output of W04's `decode_and_validate`, which W06 obtains before calling
`authorize` (sequencing obligation). W05 never re-decodes raw handle
values: raw validation belongs to W04 alone, and trusting a validated shape
is what makes the two locks never-nested rule workable.

Rationale and boundary: if W06 cannot supply a validated shape (for example,
a future call that authorizes before validation), that call's design must
sequence validation first or extend this contract explicitly; improvising a
second decode path in W05 is a design violation.
