# P5-W05 Code Contracts — Check, Revocation, and Denial Classes

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W05 detailed design](README.md). Checklist §3 template.

## 1. `authorize` (function — the permission check)

```text
Name and stability: authorize(&self, caller: CallerId, target:
  &ValidatedTarget, required: RightsSet) -> Result<AuthorizedView,
  DenyCause>; internal; total.
Purpose and caller: THE authorization gate (P5-T10/T11); called by W06 in
  its sequence after W04 identity validation and before execution.
Inputs / outputs: in — caller VM identity, W04-validated target shape,
  mandatory required set; out — AuthorizedView { } (v0: presence is the
  authorization; no elevated capabilities escape) or exactly one cause of
  {NeverGranted, Revoked, InsufficientRights{required, held}}.
Preconditions / postconditions: pre — required is non-empty (type rule);
  target shape came from W04 validation. Post — no state change; the scan
  considered every record.
State and ownership: read-only over the record table.
Concurrency/allocation: record-table lock held for the scan; allocation-
  free; bounded by capacity; callable in dispatch context.
Errors and failure guarantee: total; denial causes exclusive.
Security/authorization checks (the no-shortcut core, P5-T11):
  - matching requires subject equality AND exact target slot AND exact
    target generation AND class equality;
  - there is no default allow, no fallback rule, no identity-valued branch;
  - an unmatched request is denied even when the caller is the first VM,
    VM "0", or holds the handle of a destroyed-then-recreated object.
Logic:
  function authorize(caller, target, required):
    lock()
    result = Err(NeverGranted)
    for record in records where state is Active or Revoked:
      if record.target_slot == target.slot
         and record.target_generation == target.generation
         and record.target_class == target.class:
        if record.subject != caller:
          continue                        // another subject's grant: no leak
        if record.state == Revoked:
          result = Err(Revoked); break
        if record.rights.contains(required):
          result = Ok(AuthorizedView)
        else:
          result = Err(InsufficientRights { required, held: record.rights })
        break
    unlock()
    return result
Validation: unit matrix — valid use; no record; wrong subject (cross-VM
  with identical handle value); revoked; insufficient single/multiple
  rights; destroyed-then-recreated target (generation mismatch → denied);
  first-VM and VM-ID-0 negative probes (W05-DV05). Property: for fixed
  table state, authorize is a pure function of (caller, target, required).
```

Denial-cause to W02 class conversion (single site):

```text
NeverGranted, Revoked      -> GuestBoundaryError::NoAuthority (class 7)
InsufficientRights         -> GuestBoundaryError::InsufficientRights (class 8)
```

The Guest cannot distinguish NeverGranted from Revoked; telemetry events
retain the cause (P5-V08 observation loop), redacted by W09's policy.

## 2. Denial-class semantics (P5-T10 coverage)

| Case | Cause | Guest-visible |
|---|---|---|
| No record for (subject, target) | NeverGranted | NO_AUTHORITY |
| Record revoked | Revoked | NO_AUTHORITY |
| Record exists for another subject (cross-VM handle reuse) | (no match →) NeverGranted | NO_AUTHORITY |
| Record Active, rights short | InsufficientRights | INSUFFICIENT_RIGHTS |
| Target invalid/stale/wrong-type | (W04 cause, upstream in W06's order) | INVALID_HANDLE |
| Object capacity/quota on grant | TableFull | RESOURCE_EXHAUSTED |

Rules: wrong-type and invalid-object rejection happen in W04's check before
`authorize` runs — W05 never reclassifies them. `BAD_STATE` (object in an
illegal lifecycle state) is the operation's own check downstream (W04/W06
contracts); it is not an authority outcome.

## 3. `revoke` (function — basic revocation, P5-T12)

```text
Name and stability: revoke(&self, handle: GrantHandle) -> Result<Revoked,
  GrantHandleInvalid>; internal; NOT Guest-reachable in v0.
Purpose and caller: the revoke half of the grant/use/revoke/reject loop;
  called by the bootstrap/grantor context (Hypervisor/test harness).
Inputs / outputs: in — GrantHandle; out — Revoked confirmation or
  handle-invalid (already revoked is Idempotent? — no: revoking a Revoked
  record returns GrantHandleInvalid::AlreadyRevoked so the caller observes
  the one-way state; revoking a reclaimed handle escalates as invariant).
Preconditions / postconditions: post — record state is Revoked (one-way);
  subsequent authorize attempts on the same binding are denied with Revoked
  cause; the record is retained as a tombstone until reclaimed.
State and ownership: one record transitions Active → Revoked.
Concurrency/allocation: lock held; allocation-free.
Errors and failure guarantee: no partial state; AlreadyRevoked is a normal
  outcome, not an error condition for the table.
Security/authorization checks: revoke authority is the grantor context's
  (bootstrap) by construction; a Guest-facing revoke path would require the
  Delegate/Destroy right design reserved for later stages.
Logic:
  function revoke(handle):
    lock()
    record = records.get(handle) or { unlock(); return InvariantViolation }
    if record.state == Revoked: { unlock(); return Err(AlreadyRevoked) }
    record.state = Revoked
    unlock()
    emit revoked { }                         // outside lock
    return Ok(Revoked)
Validation: unit loop — grant → authorize ok → revoke → authorize denied →
  revoke again → AlreadyRevoked; cross-check tombstone presence; reclaim on
  destroy (§4).
```

## 4. `on_target_destroyed` (function — reclaim hook)

```text
Name and stability: on_target_destroyed(&self, slot: usize, old_generation:
  u32) -> usize (reclaimed count); internal; wired to W04's destroyed-event
  hook (AC-05.2).
Purpose and caller: prompt tombstone/record reclamation for a destroyed
  target slot; correctness does not depend on it (generation binding already
  neuters matching) — this is bounded hygiene.
Inputs / outputs: in — the destroyed slot and its pre-destroy generation;
  out — number of records removed.
Preconditions / postconditions: post — no record targeting (slot,
  old_generation) remains; Active records are never evicted (they were
  already neutered by the generation change, and their removal here is
  correct: their authority target no longer exists in that generation).
State and ownership: removes matching records only.
Concurrency/allocation: takes only the record-table lock (never nested with
  W04's — decision 8 of 01); allocation-free.
Errors and failure guarantee: total; a missed event has no correctness
  effect, only retained tombstones (telemetry note).
Security/authorization checks: none needed (reclamation cannot create
  authority).
Logic:
  function on_target_destroyed(slot, old_generation):
    lock()
    n = remove all records where target_slot == slot
             and target_generation == old_generation
    unlock()
    return n
Validation: unit tests — cascade of a VM with granted vcpu targets reclaims
  all; late event after slot reuse removes nothing (generation mismatch).
```

## 5. Host-testable seam summary (for W07/W08)

Pure-ish surface over synthetic identities and test referents:

- `grant` / `revoke` / `authorize` / `on_target_destroyed` over a private
  record table with synthetic CallerIds and synthetic target shapes — no
  VM, no QEMU, no trap state.
- Invariants tests and fuzzing must preserve: no unauthorized success
  (authorize Ok ⇒ an Active record with the required rights matched subject
  and exact target binding); one-way revocation; tombstone persistence
  until reclaim; generation-change neutering; capacity exhaustion never
  evicts Active records; authorize is a pure function of
  (table state, caller, target, required).
- Stress property: interleaved grant/revoke/authorize/destroy cycles never
  produce an Ok after revoke of the matching record, and never panic.
