# P7-W03 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W03 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `cpuset` | bounded pCPU-set type and its operations | none (value type) | ids | set values / errors | id allocation, registry state |
| `placement` | spec validation, resolved placement, frozen attachment | none (pure over inputs) | `PlacementSpec`, registry snapshot, ledger snapshot | `ResolvedPlacement` / typed error | runtime scheduling decisions |
| `ledger` | authoritative vCPU→placement map; exclusivity index; configuration audit trail | the map + exclusivity counts | accepted `ResolvedPlacement` from validation | lookups, `is_exclusively_held(pcpu)`, snapshots | runqueue membership, lifecycle state |
| `eligibility` | the pure predicate consumed by gate (W02) and picker (W05) | none | `ResolvedPlacement`, pcpu, online query | bool | pick order, migration |
| `config-telemetry` | semantic fields for applied/rejected configuration | none (emits semantics) | validation outcomes | event fields (rendering W09) | encoding, rates |

Dependency direction (normative): `eligibility` → `placement`/`cpuset` +
registry query; `ledger` → `placement`; consumers (W02 gate, W05 picker,
W07 flows, W08 idle) depend on `eligibility` + read-only ledger snapshots;
nothing depends back on consumers. No architecture-register, timer, or GIC
access exists anywhere in these modules.

## 2. Placement lifecycle

```text
 vCPU configured (Offline)                 (frozen for the rest of P7)
        |
        v
 authorize (P5 hook) -> validate_placement --Err--> typed rejection,
        |                   |  ^                        vCPU stays Offline,
        Ok                  |  |                        rejection event
        v                   +--(revalidate on retry)        |
 ResolvedPlacement frozen into vCPU + ledger entry            |
        |                                                     |
        v                                                     v
 placement_applied event                          placement_rejected event
```

Properties: attach happens exactly once per vCPU in P7 (single
configuration; reconfiguration attempts are explicitly rejected — decision
3 of the README); `ResolvedPlacement` is immutable after attachment; ledger
removal happens only at vCPU teardown, which is outside P7 scope (Reserved),
so the ledger is append-only within the stage. Pause/resume (W07) and
scheduling (W05/W08) never mutate placement; they only read it.

## 3. pCPU-registry mapping (P3-W03)

| P3 state | `validate_placement` (config time) | `is_eligible` (run time) |
|---|---|---|
| `Online` | accepted as target / member | eligible |
| `Present`, `Starting`, `Initializing` | rejected (`PcpuNotOnline`) | not eligible (harmless: could not have been configured) |
| `Failed` | rejected (`PcpuNotOnline`) | not eligible — dynamically, without placement rewrite |
| absent from registry | rejected (`UnknownPcpu`) | not eligible |

The two-column split is the design's answer to "what if a pCPU fails after
configuration": configuration-time validation is strict (no deferred
errors), run-time eligibility re-checks online status so a failed pCPU
drops out safely while its vCPUs' remaining eligibility is preserved. What
then happens to a pinned vCPU whose only pCPU failed (it becomes
undispatchable) is a containment concern owned by W07/W08; this design
guarantees only that eligibility is false and the fact is observable.

## 4. Core objects and ownership

| Object | Owner | Mutability | Writers |
|---|---|---|---|
| `ResolvedPlacement` (inside vCPU) | the vCPU object (P4); this module owns its interior | frozen after attach | validation path, once |
| placement ledger | this module's `ledger` | append-only in P7 | validation path, under ledger lock |
| exclusivity index | `ledger` interior | updated with ledger entry | same |
| `CpuSet` values | value semantics | immutable once built | builders |

Single-writer rule: only the validation path (holding both the vCPU's
configuration authority and the ledger lock) creates placements; the gate
and picker are pure readers. Two concurrent validations racing for the same
exclusive pCPU are serialized by the ledger lock; the loser observes the
winner and returns `ExclusiveConflict` — never a partial state.

## 5. Authorization boundary

The configuration entry point is the only path that reaches
`validate_placement`. Its contract requires, before validation, a capability
receipt produced by the P5 rights check whose rights include the
scheduling-policy control right (concrete right identifier owned by P5,
P7-IN-06). The receipt reference is recorded in the ledger entry and in the
`placement_applied` event, giving the configuration audit trail a subject.
The placement modules perform no capability lookup themselves (ADR-013
layering, mirroring the W02 engine rule). A call without a receipt is
rejected `NotAuthorized` and emits `placement_rejected` — configuration
attempts are visible even when refused.

## 6. Concurrency model

- Validation runs in a schedulable (non-IRQ) configuration context; it may
  allocate through the P2 allocator contract.
- The ledger lock is held only for the map/exclusivity update (O(1)–O(set
  word count)); registry queries happen outside the ledger lock (registry
  is P3-owned, separately synchronized).
- `is_eligible` is lock-free: it reads the frozen placement (immutable),
  the caller-provided online query, and the exclusivity answer — the last
  via an atomic snapshot read of the ledger index (updates are rare,
  configuration-time only; readers tolerate the append-only growth with
  acquire semantics per the P3 atomic-ordering baseline).
- No placement code runs in IRQ context; wakeup-time eligibility checks
  (W06/W08) call the same pure predicate from their own bounded contexts.

## 7. Failure model

| Condition | Classification | Behavior |
|---|---|---|
| Invalid spec (empty set, unknown/offline pCPU, capacity, conflicts) | Guest-adjacent control error (recoverable, `InvalidInput` class) | typed rejection; nothing attached; `placement_rejected` emitted |
| Reconfiguration attempt on non-`Offline` vCPU | recoverable, explicit | `ReconfigurationNotSupported`; event emitted; no silent accept |
| Missing/insufficient capability | authorization rejection (recoverable) | `NotAuthorized`; event emitted with absent-receipt marker |
| pCPU fails after configuration | runtime condition, not a configuration error | eligibility false at pick time; handling owned by W07/W08; observable via ledger + registry |
| Prerequisite mismatch (no suitable P5 right, no capacity constant) | blocked prerequisite | W01 §5 procedure; package step stops |

No placement failure ever panics the hypervisor and none mutates scheduling
state as a side effect.
