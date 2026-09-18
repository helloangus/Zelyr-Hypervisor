# P6-W03 Architecture, State Authority, and Concurrency

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W03 detailed design](README.md).

## 1. Logical modules

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `irq-identify` | Read and decode the acknowledge value into a typed classification; own the special-band rules | none (pure over the read value + supported range) | IAR value, W02 range facts | `IrqClassification` | Does not dispatch, complete, or consult the registry |
| `irq-registry` | Consumer slots (one per supported ID), registration-before-enable, enable/disable mechanics per ID class | Slot table; per-ID enabled bookkeeping | registration requests from owning subsystems | invocation targets; enable/disable operations | Does not run handlers; does not own hardware sequence correctness beyond the enable write |
| `irq-dispatch` | The per-entry loop: acknowledge → classify → dispatch outcome → complete → repeat/exit | per-entry bound state (local) | P1 entry context | dispatch outcome records; EOI writes | Does not register consumers; no policy on consumer work |
| `irq-stats` | Per-pCPU and per-ID counters, threshold events | per-CPU counter records (P3 storage); global threshold bookkeeping | outcomes from `irq-dispatch` | counters, events | Not a logging framework; no latency computation (W13's) |

Placement follows the established crate layering (architecture-layer GIC
mechanics; stats records in P3 per-CPU storage); map by layer if the tree
differs.

## 2. State authority table

Every interrupt-related state has exactly one owner; the table is the
review reference for "one owner per transition".

| State | Authority | Notes |
|---|---|---|
| Hardware pending/active bits | The GIC (hardware) | Software observes via ISPENDR/ISACTIVER-class reads; W02 clears residuals only; W03 never writes pending bits at runtime |
| Enabled/disabled per ID | `irq-registry` via its enable/disable operations (SGI/PPI: local GICR registers on the owning pCPU; SPI: GICD registers under the distributor-scoped lock) | Registration publishes the slot before the enabling write |
| Consumer slot content | The owning subsystem, through `irq-registry`, exactly once, before enable | No runtime replacement in P6 |
| Acknowledge | The acknowledging pCPU (IAR read) | Only in `irq-dispatch` |
| Completion (EOI) | The acknowledging pCPU, exactly once per acknowledged ID | Only in `irq-dispatch`; combined drop+deactivate per posture |
| Priority configuration | W02 baseline; W10 semantics later | W03 writes no priority at runtime |
| Routing | W02 initial; W04 changes | W03 writes no routing |
| Counters | `irq-stats` on the counting pCPU | Monotone increments; cleared only by W13-approved diagnostics if ever |

## 3. Core objects

### 3.1 `PhysicalIntId` family

`SgiId` (0..=15), `PpiId` (16..=31), `SpiId` (32..=supported max), unified
as `PhysicalIntId` enum; constructors are total-with-reason (validated),
never `From<u32>` unchecked. Extends the P0 newtype baseline
([p0-w15](../../../p0/plans/p0-w15-address-identifier-type-safety.md)
assumed contract). Contracts in
[03](03-code-contracts-identification.md) §2.

### 3.2 `ConsumerSlot`

One immutable registration per supported ID: callback function pointer,
owner tag (subsystem identity for diagnostics), registration-order token.
Published release before the enabling write; read acquire in dispatch.
No mutable consumer state exists in the registry.

### 3.3 `HostIrqContext`

The per-invocation immutable view handed to a consumer: owning pCPU id,
reference to the P1 entry context, the typed ID, and the dispatch depth
(iteration within the entry). No allocation; lifetime bounded by the
dispatch call.

### 3.4 `IrqStatsRecord` (per pCPU)

Counters: `acked`, `completed`, `spurious`, `unknown`, `consumerless`,
`bound_exits`, per-class (SGI/PPI/SPI) acked counts, per-ID
consumer-invocation counts (fixed-size array over the supported range).
Monotone; wrap-around tolerated (counts are diagnostics, not
authorities); threshold events fire once per crossing.

## 4. Per-IRQ lifecycle (software view)

```text
Disabled —register(consumer)—> RegisteredDisabled
RegisteredDisabled —enable (slot already published)—> EnabledQuiet
EnabledQuiet —hardware asserts—> DeliveredToCpu (hardware pending/active)
DeliveredToCpu —ack by some pCPU—> InHandler (active at that pCPU)
InHandler —consumer returns—> Completed (EOI: drop+deactivate)
EnabledQuiet —disable—> RegisteredDisabled (re-disable while active is
    Reserved: P6 consumers disable only from their own handler context
    or before they ever enable; documented limitation)
```

Repeated arrival (active-and-pending): the GIC latches a further assert
while active; after completion the CPU interface re-signals; W03 adds no
software latching — the next IAR read in the same or a later entry
delivers it. This is the documented repeated-event policy
([04](04-code-contracts-dispatch-completion.md) §3.3).

Simultaneous events: the GIC orders by priority (identical priorities:
unspecified order); W03 applies no ordering policy and counts what
arrives. Both properties satisfy the plan's repeated/simultaneous
acceptance without new state.

## 5. Loop invariants (Required)

- INV-A every classification outcome is one of the five named outcomes;
  no sixth path exists (no fall-through).
- INV-B every assigned acknowledged ID reaches exactly one EOI before the
  loop continues or exits; the EOI value is the exact acknowledged value.
- INV-C the per-entry bound is checked before each additional
  acknowledge; a bound exit happens only between iterations, never with
  an acknowledged-but-uncompleted ID outstanding.
- INV-D consumer invocation receives only a validated typed ID and an
  immutable context; no registry-internal state is exposed.
- INV-E counters increment once per outcome; threshold events are
  non-reentrant (fire at most once per crossing).

## 6. Concurrency model

- Dispatch runs in IRQ context on the acknowledging pCPU: no allocation,
  no blocking, no lock acquisition that a consumer could hold
  (registration lock is only taken at registration time, which is never
  from a consumer callback; P3 lock-order baseline: registry lock below
  the distributor lock, never held across consumer invocation).
- Registry writes occur with interrupts disabled at the registering CPU
  and complete (slot published, then enable) before returning; readers
  need no lock (release/acquire publication; slots immutable after
  publication).
- Per-CPU counters are per-CPU-storage local; no cross-CPU aggregation in
  IRQ context (W13 aggregates at sampling time).
- The distributor-scoped lock (W02's global class) is taken only in the
  SPI enable/disable mechanics, never in the dispatch loop.

## 7. Telemetry

Events under the P0 namespace: `irq.dispatch_outcome` (pcpu, outcome
class, typed ID where assigned — sampled under high rate), `irq.bound_exit`
(pcpu, depth), `irq.threshold` (counter kind, threshold), `irq.registration`
(id, owner tag). High-rate outcomes aggregate into counters with periodic
sampled events rather than per-event streams; W13 defines the correlation
(P6-V24).
