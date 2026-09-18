# P3-W11 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W11 detailed design](README.md).

## 1. Logical modules

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Counter catalog | Define the named per-CPU counters, their slot assignment in the W04 block, and their update rule | The catalog definition (compiled into the slot map) | W04 slot capacity/alignment; plan scope | increment API consumed by owning CPUs; slot map used by readers | Counter placement (W04); semantic authority for the counted activity (W03/W07/W08/W06) |
| Event catalog | Define the SMP event inventory, ids under the P0-W13 namespace, context fields, channel and trim classes | The catalog definition and registration | P0-W13/W12 contracts; plan scope | emission API consumed by P3 packages; reviewable registration | Channel mechanics (P0-W12); the observed mechanisms themselves |
| Attribution | Define and source the per-event CPU identity triple and target fields | None (value types) | `current()` (W04), W03 records, caller-supplied typed ids | `CpuAttribution` values embedded in events and dumps | Identity assignment (W01); lifecycle state answers (W03) |
| Read/aggregation | Turn per-CPU counters into snapshots, sums, and rendered dumps | None (derived values) | W04 lookup table, W03 `OnlineSet`, slot map | `SmpCounterSnapshot`, sums, dump lines | Any authority over the underlying activity; global coherence (impossible without stop-the-world) |
| Boot-timing record | Capture raw counter readings at W05 phase transitions and emit deltas informatively | One fixed-capacity boot-only record, boot-CPU-written | arch counter-read seam; W05 transition points | informative timing diagnostics | Any latency claim, pass criterion, or timer dependency |

These are logical modules; concrete crate/module placement is reserved to
the workspace-owning approved design, as in the sibling designs.

## 2. Counter model

Every counter is one `u64` atomic slot inside the owning CPU's
`TelemetryCounters` block ([P3-W04](../p3-w04-per-cpu-runtime/README.md)
slot contract §5.4). Properties, all normative:

- **Single writer by construction**: only the CPU that owns the block
  updates its counters. There is no cross-CPU counter write path; a
  foreign-CPU increment API would be a design violation.
- **Update = one atomic fetch-add, Relaxed.** Same-CPU reentrancy
  (exception interrupting interrupted code) is the only same-slot
  contention; the atomic RMW makes the update lost-update-free. Relaxed
  ordering is sufficient because each slot is single-CPU monotonic and no
  cross-slot relationship is assumed by any update.
- **No allocation, no lock, no call-out**: updates are legal from
  exception/interrupt context. This is the IRQ-safety obligation the plan
  attaches to observability; the [catalog contract](03-code-contracts-counter-block.md)
  keeps every increment on this path.
- **Zero-initialized at area allocation** (W04 already zero/fills slots);
  counters are reset only by boot. Wrap is the defined overflow behavior
  of the atomic add (two's complement); it is unreachable at declared P3
  rates (bound analysis in [03](03-code-contracts-counter-block.md) §2)
  and wrap-alarm monitoring is Reserved.
- **Counters are evidence, not authority**: decision 3 of the entry
  README. A mismatch between a counter and its owning subsystem's state is
  a diagnosable defect charged to the seam, not a new truth.

The catalog is fixed and small (see
[03](03-code-contracts-counter-block.md) §2); it must fit W04's recorded
capacity. Slot assignment order equals catalog order, stable per W04's
`layout_version`; adding a counter is a W11 design change recorded through
the namespace review, never a silent slot grab.

## 3. Event model

Each event in the catalog ([04](04-code-contracts-event-catalog.md) §2)
carries:

- **Id**: `domain.event` under the P0-W13 namespace, with the domain
  assignment recorded per event (`cpu.*` for host physical-CPU activity,
  `boot.*` for boot-ordering activity — final ids follow the approved
  P0-W13 contract; see the boundary in [01](01-scope-and-foundations.md)
  §1.2).
- **Context fields**: `CpuAttribution` (emitter) plus optional typed
  target/source identities and small enumerated payloads (from/to states,
  outcome, phase). No free-form strings, no platform names, no addresses
  except where a dump contract explicitly renders them (dumps render
  W03/W04 diagnostics, not events).
- **Channel class**: which P0-W12 channel the event targets (structured
  trace for most; the SMP-ready dump and fatal-path consumption follow
  the crash-dump channel's rules). W11 classifies; the channel mechanics
  are P0-W12's.
- **Trim class**: `Always` (lifecycle transitions, start outcomes,
  rendezvous result, SMP-ready — the boot-visibility minimum) or
  `DebugOnly` (per-occurrence notification/transport storms, contention
  observations, raw timing deltas — the high-rate classes ADR-048 allows
  trimming). Counters have no trim class: they are always-on by design
  decision 7 of the entry README.
- **Context safety**: whether the emission point may run in
  exception/interrupt context (transport-received events may; boot-phase
  events may not). Exception-safe events must satisfy the same
  no-allocation/no-lock rule as counter updates; if the approved P0-W12
  channel contract cannot accept a no-allocation structured event, that
  event is a recorded conflict, not a silent reclassification.

## 4. Attribution model

```text
CpuAttribution { logical: LogicalCpuId, hardware: HardwareCpuId, role: BootRole }
BootRole ∈ { Boot, Secondary }
```

- Sourcing: an emitting CPU sources its own attribution from its installed
  per-CPU header (W04 `current()`), never from a parameter a caller could
  forge; the coordinator-attributed events (start-requested, rendezvous
  result) source the *target* CPU from the typed argument the owning
  package already carries (W02's start request, W05's attempted set).
- Rendering: decimal logical id, typed hardware-id rendering (W01's
  `Display`), `boot`/`secondary` role — the exact output format of the
  P0-W12 human-log channel; W11 fixes content, the channel fixes form.
- The triple is mandatory: an event without it fails the catalog review
  (W11-DV03). Counter rows are attributed by *which CPU's block they live
  in*; a dump therefore renders the triple per row from the area header,
  not from the counter.

## 5. Read and aggregation model

- **When**: diagnostic time only — boot diagnostics after the rendezvous
  (the SMP-ready dump point W05 designates), stress harness checkpoints
  (W12), regression assertions (W13), and the fatal path if W09's contract
  consumes it. Not from exception context; not on the hot paths of the
  observed mechanisms.
- **Which CPUs**: the snapshot iterates W03's `OnlineSet` through W04's
  read-only lookup table. Non-online CPUs have blocks but are excluded
  from sums; their inclusion is a dump option (quarantined/failed CPU
  counters can be evidence of what a failed CPU did before failing).
- **Coherence**: a snapshot reads each included CPU's counters once, in
  fixed slot order, with acquire loads. Per-slot values are atomic; the
  multi-counter set per CPU is not a consistent instant, and the sum
  across CPUs is not a consistent instant. Consumers needing exact
  accounting (W12 sent == received) rely on quiescence: the stress
  scenario quiesces senders before reading, making the best-effort
  snapshot exact in the cases the matrix needs. This obligation is stated
  in the scenario contract W12 owns, not re-defined here.
- **Output**: `SmpCounterSnapshot` (per-CPU rows, each with attribution
  and slot values) plus derived sums; `render_counter_dump` renders the
  P0-W12-governed lines used at SMP-ready and on demand.

## 6. Boot-timing record

One fixed-capacity record, owned by the observability module, written only
by the boot CPU at W05's designated transition points, never written after
`SmpReady`, read-only afterwards (boot-only state in the P3-V10 taxonomy;
single writer, no lock needed — publication rides W05's SMP-ready fence).

Each entry: phase marker (from W05's `BootPhase` vocabulary), raw counter
reading (opaque value from the arch seam), and monotonic sequence number.
The delta view (entry[N] − entry[N−1]) is *informative*; no consumer may
convert it to time units or a criterion. This is the entire P3 answer to
"boot-synchronization timing" — ordering proof comes from W05's phase
events; the record adds raw, uninterpreted magnitude.

## 7. Concurrency and state summary

| State | Owner | Writers | Readers | Protection |
|---|---|---|---|---|
| Per-CPU counter slots | owning CPU | owning CPU (exception-safe RMW) | snapshot/dump (diagnostic time) | per-slot atomicity; single-writer rule |
| Event stream | P0-W12 channel | emitting CPUs via emission API | channel consumers | channel contract; W11 adds no buffering |
| Boot-timing record | observability module | boot CPU only, pre-SMP-ready | dumps, W12/W13 evidence | boot-only; published by W05's fence; read-only after |
| Catalogs (counter/event) | this design | compile time | review, slot map | immutable after build |

W11 introduces no lock, no allocator use after boot, no `static mut`
(behind-the-scenes statics follow the workspace design's sanctioned
patterns), and no cross-CPU write path.
