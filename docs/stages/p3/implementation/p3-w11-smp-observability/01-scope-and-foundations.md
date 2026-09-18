# P3-W11 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W11 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"SMP lifecycle, events, contention, and timing have governed observable
outputs" requires five concrete artifacts:

1. **Storage with meaning**: the W04-reserved per-CPU
   `TelemetryCounters` block given a catalog — counter names, slot
   assignment, update rule, and capacity fit —
   [03-code-contracts-counter-block.md](03-code-contracts-counter-block.md).
2. **Event inventory under governance**: the SMP event list registered in
   the P0-W13 namespace with per-event context, channel class (P0-W12),
   and trim class —
   [04-code-contracts-event-catalog.md](04-code-contracts-event-catalog.md).
3. **Attribution**: the typed identity triple and target-side fields that
   make every observation CPU-meaningful —
   [02-architecture-and-state.md](02-architecture-and-state.md) §4.
4. **Seams**: the named observation points inside W02/W03/W05/W06/W07/W08
   that make their activity reportable, agreed as requirements on those
   designs ([04](04-code-contracts-event-catalog.md) §6).
5. **Read surfaces**: snapshot, aggregation, and dump contracts so W12/W13
   can assert instead of scraping logs
   ([03](03-code-contracts-counter-block.md) §5–§6).

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Trace-event namespace and review rules | [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) | Domains covering boot/cpu/…, naming, version/compat, new-event review; P0-V09 evidence | If the approved P0-W13 contract classifies SMP events differently or fixes a naming scheme, adapt the catalog's ids to it; a blocking divergence (e.g. no domain admits host-CPU events) is an `ADR Required`/`Architecture Change Request` against P0-W13, not a local workaround |
| Diagnostic channels, levels, trimming semantics | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md) | Channel semantics (human log, structured trace, metrics, crash dump), release-trimming rules, version-identity association | If the channel contract cannot carry an exception-context-safe structured event, events classified exception-safe here become a recorded conflict with P0-W12; W11 does not invent a side channel |
| Per-CPU counter storage | [P3-W04](../p3-w04-per-cpu-runtime/README.md) slot contract §5.4 | u64-aligned slots with W04-owned capacity, placement, alignment, and lifetime; per-CPU private; read aggregation W11's | If the catalog cannot fit W04's recorded capacity, the layout change is a W04 design change (its §4 rule); record it, never silently truncate the catalog |
| Lifecycle event sources | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) | Transition operations emit their transition event (W03 hands the content contract to W11's catalog); registry state authoritative | If W03's design lands without an emission point, W11's lifecycle coverage is blocked — record as a seam conflict; W11 must not synthesize lifecycle events from polls |
| Notification observation points | [P3-W07](../p3-w07-cross-cpu-notification/README.md) (parallel, unread) | Sent/received reporting at W07's send and reception paths via W11's increment/emission API | If W07's protocol cannot report source or target identity, the affected event field is dropped from the catalog with the gap recorded — coverage that cannot exist is not fabricated |
| TLB-transport observation points | [P3-W08](../p3-w08-tlb-shootdown-transport/README.md) (parallel, unread) | Request/completion reporting at W08's request and acknowledgement paths | Same treatment as W07; no Stage-2 semantics are asserted by any W11 event |
| Contention observation points | [P3-W06](../p3-w06-concurrency-synchronization/README.md) (parallel, unread) | W06 calls the contention-observation API at its designated contention sites with a site identifier | If W06's lock semantics offer no bounded observation point, the contention rows of the catalog are recorded as blocked pending W06 — the task-book requirement stays visible, not silently dropped |
| Boot phase transition points | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) | Phase-transition and rendezvous events emitted from W05's transitions; raw-counter reads at the same points | W05's §6 already reserves event content to the P0 governance with W11 owning the catalog; a divergence is resolved between the two designs |
| Shared-state classification taxonomy | [P3-W10](../p3-w10-smp-safety-audit/README.md) (parallel, unread) | The audit's state classes (immutable-after-boot / CPU-local / atomic / lock-protected / boot-only) as the vocabulary this design's state summary and classification reviews cite | If W10's approved taxonomy differs, the classification wording in the architecture file is aligned to it — the observability design classifies its own states using the audit's terms, never inventing a rival taxonomy |
| Identity and access types | [P3-W01](../p3-w01-cpu-topology-inputs/README.md), [P3-W04](../p3-w04-per-cpu-runtime/README.md) | `LogicalCpuId`, `HardwareCpuId`, `current()`, read-only lookup table, `OnlineSet` from [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) | Shape deviations are contract conflicts resolved between designs; W11 adds no second accessor path |
| Architectural counter read seam | Layering rules (ADR-041/ADR-043); [P3-W05](../p3-w05-smp-boot-synchronization/README.md) informative-timing note | An arch-module-provided raw counter read usable at EL2 on the P1 baseline; Core sees an opaque raw value only | If the P1 baseline disallows or reserves the counter read, the boot-timing record is blocked and recorded as such — timing claims remain impossible either way |

### 1.3 Why no hidden essential deliverable remains

- Plan step 1 ("inspect lifecycle, event, transport, audit, and P0
  telemetry governance") is realized by §1.2: each inspected contract is
  either consumed as an assumed contract with a boundary, or named as a
  seam requirement on a parallel design.
- Plan step 3 ("integrate observability with stress, QEMU regression, P4
  handoff, and evidence locations") is realized by the read surfaces
  (W12/W13 consumers), the handoff summary obligations in
  [06-validation-and-handoff.md](06-validation-and-handoff.md) §3, and the
  verification destinations fixed in the validation matrix.
- Plan step 6 ("record available observations, limits, and consumers") is
  bounded to what W11's scope produces: the catalog, the non-guarantees,
  and the consumer list; actual observations belong to W12/W13 execution
  records, not to W11.

## 2. Scope classification

### 2.1 Required

- Counter catalog: lifecycle, notification, transport, and contention
  counters with slot assignment, update rules, and capacity fit.
- SMP event inventory: lifecycle, notification, transport, contention,
  and boot-synchronization groups with per-event context, channel class,
  and trim class, registered under the P0-W13 namespace.
- `CpuAttribution` model and its sourcing rules.
- Snapshot, aggregation, and dump read surfaces with their context rules.
- Named observation-point requirements on W02/W03/W05/W06/W07/W08, with
  each seam's acceptance status recorded.
- Host-side tests for counter semantics (reentrancy, wrap, capacity,
  aggregation) and the P3-V11 capture/review evidence.

### 2.2 Reserved (must not block a future design; not implemented now)

- Runtime event filtering; trigger: the P0-W12 filter contract naming the
  SMP domain.
- Counter-wrap alarms / saturation monitoring; trigger: declared stress
  rates making the u64 bound approachable.
- Cross-boot persistence or aggregation of counters; trigger: a later
  telemetry-design decision.
- Per-counter cache-line placement changes; trigger: W12 contention
  evidence justifying a layout change (layout is W04's).
- Timing-derived criteria of any kind; trigger: the P6 timer baseline.

### 2.3 Out of Scope

- Telemetry transport, ring buffer, logging subsystem, filter engine
  (P0-W12's withheld scope).
- Notification/TLB protocol semantics (W07/W08); lock semantics and wait
  policy (W06); lifecycle transitions (W03); rendezvous (W05).
- Guest-visible telemetry, guest events, vCPU/scheduler events (P4+/P7).
- Performance claims, latency guarantees, or timing-based pass criteria.
- Board/SoC/QEMU-specific output content (ADR-043/ADR-052).
