# P5-W05 Architecture, State, and Concurrency

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W05 detailed design](README.md).

## 1. Logical modules

| Module | Layer | Responsibility | Owned state / artifacts | Inputs | Outputs | Non-responsibility | Failure boundary |
|---|---|---|---|---|---|---|---|
| C1 Rights vocabulary | Core | `RightsSet` bitset; named classes; checked union/contains; Display for diagnostics only | The type and constants | Required/held sets | Set algebra results | Deciding which call needs which right (call designs/W06) | An undefined bit cannot be constructed; empty required set is a construction error |
| C2 Authority records | Core | Fixed-capacity record table; sole truth about grants; Active/Revoked/tombstone lifecycle | The table and its lock | grant/revoke/authorize/reclaim requests | Records, grant handles, outcomes | Handle validity (W04); object state (P4); policy (none in P5) | Table invariants hold after every operation; breach is `InvariantViolation` |
| C3 Bootstrap grant path | Core | The only record-creation path; bootstrap context assertion; GrantHandle minting | None beyond C2 | subject, target (W04-validated), rights | GrantHandle or capacity error | Any guest-reachable path (none exists) | Guest-reachability of grant would be a design violation, structurally impossible via internal-only API |
| C4 Authorization check | Core | Subject+target lookup; state and rights evaluation; denial causes | None (reads C2) | CallerId, validated target, required set | `Authorized` view or cause {NeverGranted, Revoked, InsufficientRights} | Sequencing (W06); execution | No match semantics other than exact binding; no default allow |
| C5 Reclaim hook | Core | Consumes W04 destroyed events; neuters/reclaims records bound to the destroyed slot generation | None beyond C2 | Destroyed event (slot, old generation) | Reclaimed count | P4 lifecycle itself | A missed event is masked by generation mismatch (correctness) but leaves tombstones (bounded) — a telemetry note, not a breach |

Layering: all modules are architecture-independent and host-testable;
identity types come from W04/P4 vocabularies. No module branches on
board/SoC/QEMU identity.

## 2. Core objects and ownership

| Object | Kind | Owner | Lifetime | Invariants |
|---|---|---|---|---|
| `CallerId` | VM identity newtype | C2 records (values supplied by dispatch context, AC-05.3) | Per record | Identifies exactly one VM context; not pCPU-bound; never implies rights |
| `RightsSet` | u32 bitset | C1 | Value | Only defined bits set; Delegate never granted in v0 |
| `AuthorityRecord` | {subject, target_slot, target_generation, target_class, rights, state} | C2 | Until reclaimed | state ∈ {Active, Revoked}; constructed only by grant |
| `GrantHandle` | internal record identifier | C2 mints, C3 returns | Until reclaim | Names exactly one record; never Guest-visible; unusable after reclaim |
| `DenyCause` | Enum {NeverGranted, Revoked, InsufficientRights{required, held}} | C4 | Per denial | Converts only to W02 classes 7/8; never Guest-visible itself |
| Record-table lock | Table-wide spinlock | C2 | Table lifetime | Never held simultaneously with W04's table lock (decision 8 of 01) |

Ownership rule: C2 owns authority truth; W04 owns handle truth; neither
mirrors the other. The destroyed-event hook is the only coupling, and it is
advisory for reclamation (correctness is already guaranteed by the
generation binding — decision 6 of [01](01-scope-and-foundations.md)).

## 3. Record lifecycle state machine

```text
                    grant(subject, target(slot,gen,class), rights)
   [no record] --------------------------------------------->  [Active]
                                                                    |
                          revoke(grant_handle)                      | authorize(caller, target, required)
                                  |                                 |  - caller != subject        -> no match (as NeverGranted)
                                  v                                 |  - target slot/gen changed  -> no match (neutered)
                              [Revoked]                             |  - rights superset missing  -> InsufficientRights
                                  |                                 |  - all match, rights ok     -> Authorized
                    authorize -> denied (Revoked cause)             |
                                  |                                 |
        target-destroy event (slot, old gen): records for that slot
        are reclaimed (removed); generation binding already neutered them
```

Terminal rules:

- Revocation is one-way; there is no un-revoke. Re-grant after revoke
  creates a fresh Active record (a new GrantHandle).
- A record whose target slot generation changed is permanently unmatchable
  even before reclamation — correctness never depends on event delivery.
- Record capacity exhaustion at grant is `TableFull` → W02
  `RESOURCE_EXHAUSTED`; Active records are never evicted to make room.

## 4. Concurrency model

- **Locks:** one record-table spinlock (P3-W06 registry discipline). Lock
  ordering rule: the W04 table lock and the C2 lock are never held
  simultaneously — W06's sequence releases W04's lock before calling
  `authorize`. No other lock may be taken while holding C2's.
- **Hold-time bound:** no Guest-memory access, telemetry emission, or
  callback under the C2 lock; `authorize` is a pure table scan bounded by
  capacity.
- **Cross-CPU:** grant/revoke/authorize on distinct records proceed in
  parallel only as the single lock serializes them; W08 may rely on exactly
  this guarantee. Revoke-vs-authorize races linearize at the lock: a use
  either completes before revoke takes effect or is denied after.
- **Interrupt context:** bootstrap grants run in init/test context;
  hypercall-path `authorize` inherits dispatch context rules; reclaim runs
  in the context of W04's destroyed event delivery, outside both locks'
  nesting (it takes C2's lock alone).

## 5. Security model

- **Authority only from records:** the sole success path is an Active
  record with sufficient rights bound to the exact target generation;
  handles, VM IDs, first-VM status, roles, and CPU placement have no
  representable influence (decision 7 of [01](01-scope-and-foundations.md)).
- **Two-context isolation by construction:** authority is keyed to subject;
  a second VM presenting the same handle value has no matching subject —
  denial without any extra "isolation check".
- **Uniform Guest-visible denial:** NeverGranted, Revoked, and wrong-subject
  are indistinguishable to the Guest (`NO_AUTHORITY`); only
  InsufficientRights differs (`INSUFFICIENT_RIGHTS`), and it reveals only
  the required/held right classes, never record counts or other subjects'
  existence.
- **Revocation integrity:** one-way state; tombstones preserve the
  revoked-use evidence loop; reclamation cannot resurrect or evict Active
  records.
- **Fail-closed invariants:** a record invariant breach (Active record with
  undefined rights bit, grant handle naming a reclaimed record, state
  regression) is a Hypervisor bug → `InvariantViolation` escalation (W02
  model), never a Guest-visible oddity.
