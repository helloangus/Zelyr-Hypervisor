# P5-W05 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

Complete the Coding-Guidelines preflight. Confirm the W01 entry review and
re-check the assumed contracts of
[01 §2](01-scope-and-foundations.md): AC-05.1 (W02 classes), AC-05.2 (W04
validation + destroyed hook), AC-05.3 (caller identity), AC-05.4 (sync
discipline), AC-05.5 (host gates). Confirm the sibling W04 design's hook and
shape contracts are compatible with
[03 §5](03-code-contracts-rights-and-grant.md).

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W04 sibling's destroyed-hook or validated-shape contract differs from
  what [03 §5](03-code-contracts-rights-and-grant.md) assumes —
  `Contract Conflict` between sibling designs; resolve at design review; do
  not adapt silently;
- a use case asks for delegation, attenuation, or guest-initiated grants —
  Reserved / Out of Scope; route to the future owning design; any pressure
  to make identity authorize is `ADR Required`;
- caller identity cannot be obtained from the execution context without
  pCPU binding — `Contract Conflict` against P3-W14's rule; stop the
  guest-path step, keep host-seam work going;
- lock-order analysis wants a shared W04/W05 lock — reject (decision 8 of
  [01](01-scope-and-foundations.md)); the never-nested rule plus W06
  sequencing is the design;
- a reviewer asks for performance shortcuts past the exact-binding match —
  reject; matching shortcuts (indexes, caching) are Reserved with a
  W08-contention-evidence trigger.

## 2. Ordered implementation steps

### Step 1 — rights vocabulary

Target: C1 ([03 §1–§2](03-code-contracts-rights-and-grant.md)).

Work: implement `RightsSet` with the five named classes, checked
construction (no undefined bits, no empty required set), superset algebra,
and the v0 DELEGATE-not-grantable rule.

Suggested observation: host build; construction accept/reject tests.

**Acceptance:** the type cannot express "no requirement" or undefined
rights; class semantics documented as in [03 §1](03-code-contracts-rights-and-grant.md).  
**Failure/blocker:** a need for a sixth class or different semantics is a
vocabulary change — route through design review, do not redefine locally.

### Step 2 — record table and grant path

Target: C2/C3 ([03 §3–§4](03-code-contracts-rights-and-grant.md)).

Work: implement the fixed-capacity record table with private record
construction, `grant` (bootstrap-only), `GrantHandle` minting, capacity
handling, and the granted-event emission point (outside the lock). Fix the
capacity value and record its rationale in the implementation record.

**Acceptance:** no code path outside `grant` constructs a record; DELEGATE
grants rejected; exhaustion yields TableFull without evicting Active
records.  
**Failure/blocker:** any Guest-reachable path reaching grant is a design
violation — stop and remove it.

### Step 3 — authorization check

Target: C4 ([04 §1–§2](04-code-contracts-check-revoke.md)).

Work: implement `authorize` exactly per pseudocode: exact binding (slot,
generation, class), subject equality, Active/Revoked evaluation, rights
superset, cause exclusivity, and the single cause→class conversion site.

**Acceptance:** the unit matrix (valid, never-granted, revoked,
cross-subject, insufficient, destroyed-recreated target, identity probes)
passes; the check is a pure function of (table, caller, target, required).  
**Failure/blocker:** any branch keyed on a CallerId *value*, or any
fallback allow, is a review stop (decision 7 of
[01](01-scope-and-foundations.md)).

### Step 4 — revocation and reclaim hook

Target: C4/C5 ([04 §3–§4](04-code-contracts-check-revoke.md)).

Work: implement `revoke` with the one-way state and AlreadyRevoked
outcome, and `on_target_destroyed` reclamation wired to W04's destroyed
hook (per AC-05.2's sibling contract).

**Acceptance:** grant → use → revoke → denied → revoke-again
(AlreadyRevoked) loop passes; reclamation removes only matching records and
never resurrects authority.  
**Failure/blocker:** W04 hook unavailable — implement the reclaim function
and mark the wiring blocked; correctness is unaffected (generation
binding), only tombstone hygiene waits.

### Step 5 — host validation suite and stress seam

Target: host test module (P0-W07/W08 assumed gates).

Work: implement the matrix from
[04 §1](04-code-contracts-check-revoke.md) plus the seam summary of
[04 §5](04-code-contracts-check-revoke.md): interleaved
grant/revoke/authorize/destroy stress with the stated invariants, and the
two-context exercise shape (two synthetic subjects, shared handle value)
that [P5-W07](../p5-w07-validation-guest-isolation-suite/README.md) will
mirror on QEMU.

**Acceptance:** no unauthorized success anywhere in the matrix or stress
runs; purity and totality hold.  
**Failure/blocker:** host gates unavailable (AC-05.5) — tests written,
execution recorded blocked.

### Step 6 — integration points (blocked-ready)

Target: W06 sequencing contract; caller-identity binding.

Work: write the sequencing contract for
[P5-W06](../p5-w06-dispatch-permission-containment/README.md) into the
implementation record (validate under W04 → release lock → authorize →
release → execute; the call's required-rights declaration duty), and mark
the guest-path caller-identity binding blocked on AC-05.3 evidence if P4
context facts are not yet available.

**Acceptance:** W06 can state its full sequence from the record without
asking W05 for values; blocked items are explicit.  
**Failure/blocker:** any temptation to fix W06's internal ordering here is
scope leak — record as a handoff note.

### Step 7 — closure review

Work: run the validation matrix in
[06](06-validation-and-handoff.md) §2, confirm the handoff checklist, and
record evidence in
`../../verification/p5-w05-capability-rights-bootstrap-revocation-verification.md`;
record decisions (capacity value, any vocabulary notes) and deviations in
`../p5-w05-capability-rights-bootstrap-revocation-record.md`. Completion is
claimed only in those records, only for what ran.

## 3. Evidence destinations

| Artifact | Path |
|---|---|
| Implementation record (capacity, vocabulary notes, deviations, changed files, `unsafe` delta) | `../p5-w05-capability-rights-bootstrap-revocation-record.md` |
| Verification record (commands, environments, results, run/not-run) | `../../verification/p5-w05-capability-rights-bootstrap-revocation-verification.md` |
| Authority-event vocabulary consumed by | [P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md) |
