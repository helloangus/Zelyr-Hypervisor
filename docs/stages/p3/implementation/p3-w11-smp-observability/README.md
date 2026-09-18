# P3-W11 SMP Observability — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The governed observable outputs for physical-CPU state and
cross-CPU SMP activity required by
[P3-W11](../../plans/p3-w11-smp-observability.md).  
**Owner/change context:** P3-W11 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W11. It defines the per-CPU
telemetry counter block (the meaning of the storage
[P3-W04](../p3-w04-per-cpu-runtime/README.md) reserved), the SMP event
catalog under the P0 trace-event namespace, the CPU-attribution model, the
read/aggregation surface that turns counters into assertable evidence, and
the integration seams that [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md),
[P3-W07](../p3-w07-cross-cpu-notification/README.md),
[P3-W08](../p3-w08-tlb-shootdown-transport/README.md), and
[P3-W05](../p3-w05-smp-boot-synchronization/README.md) use to make their
activity observable. It deliberately does **not** design the telemetry
transport or logging subsystem itself (the P0-W12 plan explicitly withholds
it), does not define notification or TLB-transport semantics (their designs
own those; this design only names the observation points they must call),
does not add guest telemetry, and does not make any performance claim.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, prerequisite failure boundaries, resolved decisions |
| [02-architecture-and-state.md](02-architecture-and-state.md) | counter-block model, attribution model, event/channel split, read/aggregation model, trimming rules |
| [03-code-contracts-counter-block.md](03-code-contracts-counter-block.md) | counter storage, increment, snapshot, aggregation, and dump contracts |
| [04-code-contracts-event-catalog.md](04-code-contracts-event-catalog.md) | event inventory, emission contract, contention and boot-timing observation contracts |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim, and no observation described here has been collected.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W11 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-048 makes structured tracing/metrics a first-class capability and
  requires that trace events be compilable-trimmable with runtime filtering
  and that high-overhead events be disablable for production/benchmark; ADR
  principle 10 states observability is not an afterthought. These are the
  quality bars this design designs to — it does not implement the
  telemetry mechanism.
- The P0 diagnostics governance
  ([P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)) owns
  log levels, channel semantics (human log, structured trace, metrics,
  crash dump), release trimming, and build/version metadata association.
  The P0 trace-event namespace
  ([P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md))
  owns event-domain classification, naming, versioning, and the new-event
  review rule. This design cites those contracts and works inside them; it
  does not restate or redefine them (failure boundaries in
  [01](01-scope-and-foundations.md) §1.2 — both are planned packages, not
  delivered implementations).
- [P3-W04](../p3-w04-per-cpu-runtime/README.md) (slot contract §5.4)
  reserved the per-CPU `TelemetryCounters` block and named W11 as its
  content owner: "catalog, rates, and event ids owned by W11 under the
  P0-W13 namespace … the block is per-CPU private (read aggregation is
  W11's design)". W04 fixes placement, alignment, capacity, and lifetime;
  W11 fixes meaning.
- The P3 task book requires lock-contention and IPI-latency trace events
  and makes CPU lifecycle and cross-CPU activity observable; P3-V11
  requires meaningful CPU attribution for lifecycle, notification,
  shootdown-transport, contention, and boot-synchronization observations.
  Attribution here means the typed identity triple of
  [P3-W01](../p3-w01-cpu-topology-inputs/README.md) plus the boot role —
  never a platform or board name (ADR-043/ADR-052).
- No timer exists in the P1–P3 baseline (P6 owns the timer baseline). The
  plan's "boot-synchronization timing" observability is therefore designed
  as raw architectural-counter deltas recorded as informative diagnostics
  (see [04](04-code-contracts-event-catalog.md) §5); P3 asserts ordering
  and attribution, never latency values.

Classification:

- **Required** for W11 closure: the per-CPU counter catalog and its
  increment/snapshot/aggregation contracts, the SMP event inventory with
  per-event context and channel classification, the attribution model, the
  read-only dump/snapshot surfaces consumed by W12/W13, the trimming
  classification seam, and P3-V11 review evidence.
- **Reserved** with recorded triggers: runtime filtering of SMP events
  (trigger: the P0-W12 filter contract naming the SMP domain);
  counter-wrap alarms and saturation monitoring (trigger: declared stress
  rates approaching the u64 bound — unreachable at declared limits);
  cross-boot counter persistence or aggregation into a permanent store
  (trigger: a later telemetry-design decision); per-counter cache-line
  layout changes (trigger: W12 contention evidence; layout is W04's);
  timing-based pass criteria (trigger: the P6 timer baseline owning real
  timing).
- **Out of Scope:** the telemetry transport, ring buffer, or logging
  subsystem (P0-W12 explicitly withholds them); notification and TLB
  protocol semantics (W07/W08); lock semantics and their wait policies
  (W06); lifecycle transitions (W03 — W11 observes, never mutates);
  guest-visible telemetry and guest events (P4+); scheduler/vCPU events
  (P7); performance certification; any pass/fail criterion expressed as a
  timing value.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| CPU discovered / start-requested / entered-EL2 / online / failed observable | [event catalog](04-code-contracts-event-catalog.md) §2 (lifecycle group), [counter block](03-code-contracts-counter-block.md) §2 | P3-V11 (W11-DV01) |
| Notification sent/received observable | [event catalog](04-code-contracts-event-catalog.md) §2 (notification group); observation points named as W07 requirements | P3-V11 (W11-DV01, DV04) |
| TLB transport request/completion observable | [event catalog](04-code-contracts-event-catalog.md) §2 (transport group); observation points named as W08 requirements | P3-V11 (W11-DV01, DV04) |
| Synchronization contention observable | [event catalog](04-code-contracts-event-catalog.md) §4; contention counters in [counter block](03-code-contracts-counter-block.md) §2 | P3-V11 (W11-DV01) |
| Boot-synchronization timing observable (informative, no timer) | [event catalog](04-code-contracts-event-catalog.md) §5; boot-record contract in [architecture](02-architecture-and-state.md) §6 | P3-V11 (W11-DV01) |
| State output includes hardware identity, logical identity, boot/secondary role | attribution model in [architecture](02-architecture-and-state.md) §4; contract in [counter block](03-code-contracts-counter-block.md) §3 | P3-V11 (W11-DV03) |
| Governed naming and output semantics; no board leakage | [event catalog](04-code-contracts-event-catalog.md) §3; review in [workflow](05-implementation-workflow.md) step 5 | P3-V11 (W11-DV01, DV03) |
| Integrate with stress, QEMU regression, P4 handoff, evidence locations | [handoff](06-validation-and-handoff.md) §3 | W11 closure review (W11-DV06/DV07) |
| Telemetry/log review evidence under the stated test environment | [validation](06-validation-and-handoff.md) matrix item W11-DV05 | P3-V11 (W11-DV05); matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P0 documentation scaffold only — no workspace, no Rust
sources, no runtime state, and no telemetry implementation of any kind.
Only P0-W01/P0-W02 have implementation records; the P0-W12/W13 diagnostics
and namespace contracts exist as plans, not as delivered artifacts. Sibling
P3 designs W01–W05 are present as proposed designs in this worktree;
W06–W10 are being prepared in parallel and are referenced by path and
P3-Wxx ID without assuming their content. Each ledger row below states the
missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| SMP lifecycle, events, contention, timing have governed observable outputs | No counter, event, or channel exists anywhere in the tree | The W11 counter catalog in the W04-reserved block plus the SMP event inventory emitted through the P0-W12 channel seam | "Governed" means declared under the P0 namespace/channel contracts; nothing observable can exist until the catalog and emission seam are defined | W11 (this design); storage W04; governance P0-W12/W13 | W11-DV01 catalog review; DV02 tests |
| CPU attribution is meaningful | No attribution value exists | `CpuAttribution` (logical, hardware, boot role) sourced from W04 `current()` / W03 records for every event and every counter row | P3-V11's core wording; an event without identity is not SMP observability | W11 (shape); W01 types; W04 accessor | W11-DV03 attribution review |
| Notification and transport activity is observable | No observation points exist; W07/W08 designs are parallel and unread | Named observation points (call requirements) on W07/W08, with counters/events W11 owns | W11 cannot observe activity that no upstream design is required to report; the seams must be agreed, not assumed | W11 (points named); W07/W08 (call them) | W11-DV04 seam-acceptance review |
| Contention is observable | No lock exists yet (W06 parallel) | Contention counters plus an observation API W06 calls at its designated contention sites | The task book requires lock-contention events; without a designed seam W06 would improvise ad-hoc output — exactly what the plan forbids | W11 (counters/API); W06 (call sites) | W11-DV04 |
| Boot-synchronization timing is observable | No timer exists in P1–P3; W05 recorded counter reads as informative-only | The boot-timing record (raw architectural-counter deltas) with the informative-only classification and the arch-side read seam | P3-V11 names boot-synchronization timing; the only P3-legal form is a raw counter delta explicitly barred from timing claims | W11 (record); arch seam owner per layering rules | W11-DV01 classification review |
| Counter storage exists with defined meaning | W04 reserved untyped u64-aligned slots with capacity it owns | The catalog mapped onto that capacity, with slot assignment stable per W04 `layout_version` | Meaning without storage is prose; storage without meaning is a dumping ground — W04's slot contract requires this split | W04 capacity; W11 assignment | W11-DV02 capacity-fit tests |
| Evidence consumers (W12/W13) can assert on outputs | No dump/snapshot surface exists | The snapshot/aggregation/dump contracts and their call points at SMP-ready and on the fatal path | W12's accounting checks and W13's matrix assertions need a designed read surface, not log scraping | W11; consumers call | W11-DV05/DV06 |

No ledger row requires this design to invent a crate name, a transport, or a
timing policy owned elsewhere; the open prerequisite risks are tabulated in
[01](01-scope-and-foundations.md) §1.2.

## Resolved design decisions and their authority

1. **Two observation forms, one owner.** Counters (per-CPU `u64` slots in
   the W04 block — always-on, allocation-free, exception-safe) and events
   (catalogued, channel-classified, trim-classified) are separate forms;
   counters answer "how many", events answer "what happened, to whom, in
   what order". The counter catalog and the event inventory are both owned
   by this design, registered under the P0-W13 namespace, and mapped onto
   the P0-W12 channel semantics. Rationale: P3-V11 needs both totals
   (stress accounting for W12) and ordered per-occurrence attribution
   (boot-order review); collapsing them would lose one or the other.
2. **Counters are per-CPU-private, atomic, lock-free, allocation-free, and
   exception-safe.** Each counter is updated only by its owning CPU with a
   single atomic fetch-add (Relaxed); no counter update takes a lock,
   allocates, or calls into another subsystem, so updates are legal from
   exception/interrupt context per the slot-owner rules W04 fixed.
   Relaxed ordering suffices because each counter is single-CPU monotonic;
   cross-CPU visibility is supplied only at read time by the aggregation
   contract. Rationale: re-entrancy (exception interrupting interrupted
   code on the same CPU) is the only same-address contention a per-CPU
   counter faces, and an atomic RMW is the minimal sound answer; Coding
   Guidelines forbid heavier interrupt-context work.
3. **Authority stays with the owning packages; observability is derived.**
   A counter or event never *is* authoritative state: lifecycle truth is
   W03's registry, rendezvous truth is W05's phase word, notification and
   transport truth is W07/W08. Discrepancies between an emitted observation
   and an owner's state are a diagnosable defect, resolved in favor of the
   owner. Rationale: one fact, one owner (the same principle W05 applied to
   readiness vs fate); telemetry-as-authority would create a second
   lifecycle authority, which the guardrails forbid.
4. **Attribution is a typed triple plus target-side context, never a
   string.** Every event carries the emitting CPU's `CpuAttribution`
   (logical id, hardware id, boot role); directed activity (start request,
   notification, TLB request) additionally carries the target CPU's
   identity as a typed field. No event field may carry a platform, board,
   or SoC name (ADR-043/ADR-052). Rationale: the plan's "hardware identity,
   logical identity, and boot/secondary role" wording, made structural;
   strings would be unenforceable at review.
5. **Aggregation is diagnostic-time-only and documented as non-coherent.**
   The snapshot/aggregation surface reads per-CPU blocks through W04's
   read-only lookup table restricted to W03's `OnlineSet`, is callable only
   outside exception context, and reports per-CPU snapshots taken at
   nominally different instants; sums are best-effort evidence, with
   per-counter atomicity per CPU. Rationale: a globally coherent
   multi-counter snapshot needs stop-the-world machinery P3 does not have
   (and P16 owns); claiming coherence would be a false guarantee.
6. **Boot-synchronization "timing" is a raw-counter delta record, not a
   latency measurement.** The boot-timing record stores raw architectural
   generic-counter readings taken at W05's phase transitions via an
   architecture-side read seam; deltas are emitted as informative
   diagnostics with the explicit property that no pass/fail criterion may
   depend on them. Real timing semantics are Reserved to the P6 timer
   baseline. Rationale: no timer exists in P1–P3 (W05 README decision 5
   already constrained its poll this way); this keeps P3-V11's "timing"
   observable without inventing a timing authority P3 does not own.
7. **Trim classification is a designed field, not an afterthought.** Every
   event carries a trim class (`Always`, `DebugOnly`) chosen per ADR-048's
   compile-time-trimming requirement; counters are always-on (their cost
   bound is stated in the architecture file). Runtime filtering is
   Reserved until P0-W12's filter contract names the SMP domain. Rationale:
   the ADR requires trimmability to be structural; retrofitting it would
   reclassify half the catalog later.
8. **Seam compatibility with parallel designs.** Where this design names
   observation points inside W06/W07/W08 scope, the points are requirements
   *on* those packages, not implementations of them: W11 defines what is
   reported; the owning design defines where its own logic calls the
   report. A point W07/W08/W06 cannot accept is recorded as a conflict
   between designs per the sibling-conflict rule — W11 does not relocate
   their logic to make observation convenient. Rationale: preserving one
   owner per mechanism while still closing P3-V11's coverage.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, the assumed-prerequisite failure boundaries (P0-W12/W13 are
   planned, not delivered; W07/W08 are parallel), and the decisions above.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for
   the counter/event/attribution model, the channel and trim
   classification, and the read/aggregation rules.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   the counter block with
   [03](03-code-contracts-counter-block.md) (steps 1–3), the event catalog
   and emission seams with [04](04-code-contracts-event-catalog.md)
   (steps 4–6).
4. Record implementation decisions in
   `../p3-w11-smp-observability-record.md` and evidence in
   `../../verification/p3-w11-smp-observability-verification.md` only when
   the work is performed. Validation conditions and the handoff checklist
   are in [06-validation-and-handoff.md](06-validation-and-handoff.md).
   Nothing in this design or any record may claim P3-V11 satisfied before
   real review evidence exists.

## Explicitly excluded interfaces

No telemetry transport, ring buffer, log formatter, or filter engine is
designed or authorized by W11; the emission contract targets the P0-W12
channel seam and nothing else. No notification send/receive primitive, TLB
request format, lock type, wait policy, lifecycle transition, or rendezvous
step is designed here — W11 names where their owners call the observation
API and nothing more. No guest-visible counter, event, ABI, or persistent
wire format is authorized: counter slot layout is internal representation
(never serialized; Coding Guidelines rule on raw struct memory), and the
event catalog is a versioned P0-W13 registration, not a stable external
ABI. A "telemetry subsystem" appearing in W11 code, or an event emitted
without `CpuAttribution`, is a scope violation to stop at review.

## Downstream handoff

- **W12** receives the counter snapshot/aggregation surface and the event
  stream as the accounting basis for stress pass conditions (sent == received,
  allocator accounting, contention bounds) and the per-CPU counter dump at
  SMP-ready as the before/after evidence frame.
- **W13** receives the SMP-ready dump, phase/rendezvous events, and
  attribution rules as per-row assertions for the 1/2/4/8 regression matrix;
  matrix execution evidence lands in W13's verification record, not W11's.
- **W14/P4** receives the observability contract (catalog summary, counter
  semantics, attribution model, non-guarantees — no timing proof, no
  cross-CPU coherence) as part of the P4 handoff; P4 consumes it through
  [P3-W14](../p3-w14-p4-smp-handoff/README.md) and must not treat event ids
  as a frozen external ABI without a versioning decision.
- **W09/P1-W07** (fatal diagnostics) may consume the counter dump on the
  fatal path through W09's contract; W11 only guarantees the dump is
  renderable from any CPU after build, not the fatal path's sequencing.
- **P7** (scheduler accounting) and **P6** (timer/IPI-latency semantics)
  find the contention and raw-timing seams recorded as extension points
  with their triggers; neither is implemented here.
