# P5-W05 Scope, Foundations, and Design Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W05 detailed design](README.md).

## 1. Scope classification in detail

### 1.1 Required

| Item | Statement |
|---|---|
| Authority record | {subject: CallerId (VM identity), target: slot + generation + class, rights: RightsSet, state: Active \| Revoked} in a dedicated fixed-capacity table |
| Rights vocabulary | Named classes Observe, Control, Modify, Destroy, Delegate; u32 bitset; Delegate defined, unimplemented in v0 |
| Bootstrap grant | Internal `grant` API, Hypervisor/test-bootstrap context only; returns an internal GrantHandle |
| Authorization check | `authorize(caller, validated target, required: RightsSet)` — mandatory parameter; two denial classes {NoAuthority, InsufficientRights} with internal causes |
| Revocation | One-way Active → Revoked tombstone via internal `revoke(GrantHandle)`; tombstone reclaimed on target-destroy event |
| Target interplay | Records bind to target slot+generation; target destruction/recreation neuters all matching records |
| No-shortcut guards | No default rights; no identity-valued logic; no record construction outside `grant`; unknown/absent requirement denies |
| Lock-order rule | W04 and W05 locks never held simultaneously; sequencing is W06's obligation |
| Host-testable seam | Grant/check/revoke exercisable with synthetic identities and test referents |

### 1.2 Reserved

| Item | Trigger for activating |
|---|---|
| Delegation, attenuation, derived capabilities, revocation trees | A future approved design under ADR-013's evolution clause; v0 proves only grant/check/revoke |
| Guest-initiated grant/revoke calls | P10+ management plane with its own ABI design |
| Per-vCPU authority refinement | First mechanism needing per-vCPU trust separation |
| Additional rights classes | The owning operation design's need, reviewed into the vocabulary |
| Quotas on grants per subject | First policy-bearing mechanism that needs them |

### 1.3 Out of Scope

Dispatch ordering and the minimal operation (W06); handle lifecycle
(W04); Guest markers and two-context scenario definitions (W07); fuzz and
stress harnesses (W08); telemetry transport and redaction (W09);
authentication/RBAC/Control Domain/policy persistence (task book §1); any
completion or evidence claim.

## 2. Prerequisite assumed contracts

Form per the W01 ledger §3
([../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)):

```text
AC-05.1  The W02 error boundary defines NO_AUTHORITY (class 7) and
         INSUFFICIENT_RIGHTS (class 8) as Guest-visible classes, with
         internal causes never Guest-visible, and RESOURCE_EXHAUSTED
         (class 10) for capacity conditions.
Source:  ../p5-w02-hypercall-abi-error-boundary/README.md (sibling design)
Failure: Contract Conflict between designs on class meaning or numbering —
         resolved at design review, never by local re-mapping.

AC-05.2  The W04 identity boundary provides validated target handles
         (slot/generation/class), a destroyed-event hook, and generation
         bumping on destroy/reuse.
Source:  ../p5-w04-handle-lifecycle-type-safety/README.md (sibling design)
Failure: Contract Conflict if hook semantics or generation discipline
         change; the target-generation binding of decision 6 depends on
         both.

AC-05.3  The caller's VM identity is available from the execution context at
         dispatch time; identity is not bound to a pCPU and is not
         authority by itself.
Source:  ../../../p4/plans/p4-w04-vcpu-entry-exit.md;
         ../../../p3/plans/p3-w14-p4-smp-handoff.md;
         ../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md
         (caller vocabulary, FD-4)
Failure: Blocked Prerequisite for guest-path evidence; host-seam evidence
         uses synthetic CallerIds. A pCPU-bound identity would be a
         Contract Conflict against P3-W14's rule and this design.

AC-05.4  Synchronization primitives and lock-order discipline exist for
         registry-class shared state.
Source:  ../../../p3/plans/p3-w06-concurrency-synchronization.md
Failure: Blocked Prerequisite at implementation entry.

AC-05.5  Host-side unit/integration test gates exist and run Rust host tests.
Source:  ../../../p0/plans/p0-w07-development-quality-gates.md
Failure: Blocked Prerequisite for execution evidence; tests still written.
```

## 3. Resolved design decisions and their authority

### Decision 1 — Explicit records with implicit presentation

Authority is a record created only by `grant`. A caller exercising authority
presents only the object handle; the check searches for an Active record
matching (caller subject, target binding) with the required rights. No
Guest-visible capability token exists in v0.

Rationale: caller association (P5-T07) becomes structural — authority that
is keyed to the subject cannot travel (the P5-V12 two-context property:
another VM holding the same handle value has no record). Explicit tokens
would require forgery-proof encoding (a harder problem) for no P5 need.
Authority: P5-T07; ADR-013; task book P5-V12.

### Decision 2 — Caller identity is VM identity; vCPUs share authority

`CallerId` is the VM identity of the execution context. Every vCPU of one VM
holds the same authority.

Rationale: P5's isolation scenarios are two *VM* contexts; per-vCPU
authority has no P5 consumer and would entangle the scheduler (P7) which
may move vCPUs. The identity remains non-authoritative (it only selects
records). Authority: task book P5-V12 ("security contexts"); P3-W14
no-pCPU-binding rule; design-owned granularity.

### Decision 3 — Rights vocabulary and mandatory requirement

`RightsSet` is a u32 bitset: Observe (bit 0), Control (bit 1), Modify
(bit 2), Destroy (bit 3), Delegate (bit 4, defined, unimplemented in v0).
Every `authorize` call takes an explicit `required: RightsSet` — there is
no default, no single-right convenience overload that skips the parameter,
and an empty required set denies (an operation that requires nothing is not
authorized to exist in v0; its design must declare at least one bit).

Rationale: operation-level rights (P5-T08) need a stable vocabulary the
check cannot silently relax; a mandatory parameter plus deny-on-empty makes
"forgot to require" unrepresentable. Class semantics: Observe = read
state/inspect; Control = lifecycle transitions; Modify = mutate object
state/data; Destroy = destroy the object; Delegate = transfer authority
(future). Authority: P5-T08; design-owned encoding under task book §8.

### Decision 4 — Bootstrap-only grant; internal grant handle

`grant` is an internal API invoked from Hypervisor initialization or test
bootstrap only; it returns an internal `GrantHandle` (an identifier into
the record table) used solely for `revoke`. The grant handle is never
Guest-visible and never enters the W04 object table in v0.

Rationale: P5's grant source is the Hypervisor/test bootstrap (P5-V07); a
Guest-visible grant object would pre-design delegation semantics (Reserved)
and create a forgeable-target problem. ADR-038's bootstrap principle is
honored at test scale. Authority: P5-T09; P5-V07; ADR-038 analogy.

### Decision 5 — One-way revocation with tombstones

`revoke(GrantHandle)` moves the record Active → Revoked (one-way; re-grant
creates a new record). Tombstones are retained — distinguishable internally
from never-granted — until reclaimed by the target-destroy event or table
reclamation. Uses against a Revoked record are denied; the Guest observes
the same `NO_AUTHORITY` class as never-granted; the internal cause and the
revoked-use event preserve the P5-V08 observation loop.

Rationale: tombstones give revocation evidence without changing the
Guest-visible vocabulary (W02 uniformity rule); one-way state avoids
resurrection bugs; bounded reclamation keeps the table fixed-capacity.
Authority: P5-T12; P5-V08; design-owned mechanics.

### Decision 6 — Target-generation binding

A record binds to the target's (slot, generation, class) at grant time. Any
destroy or destroy-and-reuse of the target slot changes the generation, so
no pre-existing record can match again; the W04 destroyed-event hook
additionally reclaims matching tombstones promptly.

Rationale: authority must not outlive or transfer across object recreation
(a stale-generation grant would otherwise reattach to a new object — the
capability analogue of the stale-handle bug ADR-013's generations exist to
prevent). Authority: ADR-013 (generation in the capability model); P5-V08.

### Decision 7 — Structural no-identity-shortcut guards

The check contains no VM-ID-valued logic, no first-VM or role concept, no
default-allow branch, and no conversion from identity to rights. Records
are constructed only inside `grant`. The only input that can produce a
success is an Active record with sufficient rights.

Rationale: ADR-051 and ADR §19 make identity-derived privilege an
architecture violation; the guards make it unrepresentable rather than
forbidden. Authority: ADR-051, ADR §19; P5-T11; P5-V07.

### Decision 8 — Separate lock; never nested with W04's

The record table has its own lock. W04 and W05 locks are never held
simultaneously; W06's sequence (identity check under W04's lock, released;
then authorize under W05's lock, released; then execute) is the composition
rule. A destroyed target discovered after authorize surfaces as an invalid
handle at execution and is contained by W06.

Rationale: avoids lock-order analysis across packages and keeps each
boundary's critical sections small; the ordering rule is testable at W06.
Authority: P3-W06 discipline; design-owned rule, W06-enforced.

### Decision 9 — Fixed record capacity; exhaustion semantics

The record table is fixed-capacity (value fixed at implementation with
recorded rationale). Exhaustion is `RESOURCE_EXHAUSTED` (W02 class 10) at
the granting layer — never a denial that misrepresents the caller's
authority, and never silent tombstone eviction of Active records.

Rationale: capacity exhaustion is a Hypervisor condition; blaming the
caller's authority would corrupt the denial vocabulary. Tombstone
reclamation (decision 5) keeps the pressure bounded. Authority: W02 class
semantics; design-owned.

## 4. Relationship to W06 and W07

- W06 declares each call's `required` set, sequences validate → authorize →
  execute, and owns containment; W05 provides the check and denial causes
  only.
- W07 drives the observable loop: bootstrap grants two contexts, exercises
  valid/insufficient/revoked/cross-VM cases, and asserts Guest-visible
  uniformity; W05 supplies the bootstrap hooks and the internal-cause event
  vocabulary that makes P5-V08 observable without changing Guest-visible
  codes.

## 5. Open questions and labels

| Item | Classification | Handling |
|---|---|---|
| Whether any P5 minimal operation needs Control or Destroy rights | W06's design freedom | W06 declares the requirement; W05's vocabulary already carries it |
| Grant handle placement if a future design makes grants Guest-visible objects | Reserved | Would extend the W04 class set and revisit decision 4; new design required |
| Rights-encoding exposure to Guests (e.g., for a future delegation call) | Reserved | Not in v0; new ABI design required |
| Any request to let VM identity, first-VM status, or role authorize | `ADR Required` | Contradicts ADR-051/§19; stop and record; never a local choice |
