# P4-W07 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W07 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-H01–H03, P4-I01–I02)

- The repeat driver: sequential Guest episodes with two teardown depths, per
  W03/W04 sequencing, with per-episode construction of all Guest-owned state.
- Same-session restart evidence machinery: accounting restoration checks and
  cross-episode equality of the declared determinism surfaces.
- Cold-boot consistency expectations: declared repeat minimums, stability
  criteria, and the run-record fields that make consistency checkable by W08.
- Telemetry ledger: per-domain counters over the routed event set
  (`s2.*`, `gm.*`, `vcpu.*`, `diag.*`), per-episode count vectors, and a run
  summary.
- Fault-correlation records: episode, vCPU, scenario, Guest PC, faulting IPA,
  diagnosis verdict, stop cause — assembled from W06/W04 data at stop time.
- Run-record grammar P4-RR v1 and per-scenario expected count patterns.

### Reserved (must not be precluded; not implemented in P4)

- A persistent event log, ring buffer, or telemetry transport (re-entry: the
  P0-W12/W13 evolution; the ledger's read surface keeps it pluggable).
- Multi-VM or multi-vCPU episode interleaving and per-pCPU counter
  aggregation (re-entry: P6+/P7; correlation fields already carry the vCPU
  identity slot).
- Cross-pCPU Stage-2 consistency proof (re-entry: the W02 invalidation seam's
  Reserved extension).
- Host-side CI orchestration beyond the QEMU environment (re-entry: W08's
  automation and the P0 CI package).
- Counters beyond the P4 event set (scheduler, IRQ latency, lock contention)
  (re-entry: their owning stages add categories per the P0 namespace rules).

### Out of Scope

A production metrics backend or telemetry API, scheduler statistics (P7),
fault-tolerant VM orchestration, final VM create/destroy lifecycle semantics,
a performance-measurement methodology or KPI, Guest-side instrumentation (the
marker stream is W05 data), Linux Guest regression (P8+), and any
hardware-behavior claim from repeat evidence.

## 2. Assumed upstream contracts and failure boundaries

Per the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review; divergence becomes a recorded conflict (W01 §4) and the affected W07
step stops.

| ID | Assumed contract | Source (plan/design path) | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | Typed addresses/IDs; failure-classification split (Guest-caused vs invariant) | [P0-W15](../../../p0/plans/p0-w15-address-identifier-type-safety.md), [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md); W01 R19 | correlation fields are typed; episode setup failures are VM-facing values | absent → W07 blocks (Coding-Guidelines shape) |
| M2 | W03 sequencing contract (construction order §4), deterministic initialization, `GuestRam::init_zeroed` re-init support, accounting marks and release | [P4-W03](../p4-w03-guest-memory-image/02-architecture-and-state.md) §3–§5; [03-code-contracts-guest-memory.md](../p4-w03-guest-memory-image/03-code-contracts-guest-memory.md) §3.2, §3.4 | the driver sequences episodes through these exact operations; re-init returns RAM to the deterministic base; release clears accounting | if `Loaded → init_zeroed` or accounting restoration is not delivered, `RamReuse` is dropped to Reserved and W07 records the gap — the required `FullRebuild` form never depends on it |
| M3 | W04 stop path and teardown sequencing (`vcpu.destroy` → `space.destroy` → `GuestRam::release`), stop retention of the last frame/diagnostic, budget constant | [P4-W04](../p4-w04-vcpu-entry-exit/04-code-contracts-vcpu-run.md) §2.2, §4 | `FullRebuild` executes the full sequence; `RamReuse` truncates deliberately after `space.destroy`; stop results feed correlation | sequencing changes → joint design note; W07 never reorders |
| M4 | W06 `ExitDiagnostic`/`MatchVerdict` values and `diag.*` event set; containment-first ordering | [P4-W06](../p4-w06-fault-isolation-diagnostics/06-validation-and-handoff.md) §4; [02-architecture-and-state.md](../p4-w06-fault-isolation-diagnostics/02-architecture-and-state.md) §6–§7 | correlation records cite W06 fields verbatim; the driver never re-renders or re-judges a diagnosis | field changes → joint review; W07 consumes, never re-derives |
| M5 | P0 logging/trace baseline + namespace with levels, trim rules, and build identity; P0-W09 runner + P1-W10 declared environment conventions | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md), [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md), [P1-W10](../../../p1/plans/p1-w10-qemu-boot-regression.md); W01 R06/R20/A8 | events and the run record emit through the baseline; cold boots run in the declared environment | absent → degrade to the P0 logging fallback; the run record's emission degrades with it and W08 is notified via the grammar's version field |
| M6 | P2 allocator accounting exposed for restoration checks (allocated == freed after teardown) | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md), [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md); W01 R09/R12 | the residue check reads accounting totals per episode | absent → the check degrades to the W03-side accounting marks and records the gap; the P4-V10 evidence then cites the degraded basis truthfully |

Entry-order boundary: driver logic, ledger, and grammar with host-side unit
tests proceed against assumed signatures; P4-V10–V12-class on-target evidence
requires M2–M5 delivered; the [workflow](05-implementation-workflow.md) marks
the gates.

## 3. Authority analysis for contested areas

- **Repeat form authority.** W03 §4 explicitly delegates the choice ("which
  repeat form the repeatability design selects is W07's"). This design takes
  it: `FullRebuild` is required, `RamReuse` is the second form. The
  `RamReuse` truncation point (teardown stops after `space.destroy`, before
  `GuestRam::release`) is a deliberate, recorded intermediate state — not a
  "half-torn Guest": W04 §4's prohibition targets *unplanned* partial
  teardown, and W03's lifecycle explicitly supports re-init from `Loaded`.
  If W03's implementation record shows the re-init path undeliverable,
  `RamReuse` drops to Reserved with the gap recorded (M2 boundary).
- **Telemetry internals vs "without prescribing its internals".** The plan's
  work sequence item 2 says the detailed design defines repeat scenarios,
  state-reset expectations, and telemetry evidence "without prescribing its
  internals" — that sentence addresses the *plan* not prescribing them; the
  detailed design must. This design therefore fixes P4-local internals
  (ledger, counters, run record) while explicitly marking them temporary and
  unfrozen (task book §1 Out of scope: final telemetry API), recorded as
  facts by W09.
- **Where automation ends and repeatability begins.** W07 owns the *what*
  (repeat minimums, stability criteria, expected count patterns, run-record
  fields); W08 owns the *how* (invocation, verdict rules, evidence files).
  The seam is the P4-RR v1 grammar: W07 freezes its shape for both sides;
  W08 parses it. No conflict exists; both plans name the seam.
- **P2-ACR-01 visibility.** W07's residue checks read the P2 allocator's
  accounting surface (M6) and the W03 accounting marks; it asserts nothing
  about ADR-level `MemoryObject`/`MemoryRegion` semantics. P2-ACR-01 remains
  `ADR Required`/unresolved and travels to W09's unresolved-conflict list.
- **Requirement-wording provenance.** Fine-grained H/I wording of the
  superseded root task book is untracked (W01 A9 pattern); meaning is fixed
  from the tracked task book §5 rows and the W07 plan scope, and the
  implementation record cites this basis.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | Episode model: one episode = construct (space, RAM, vCPU) → run → stop → teardown; the driver is the sole setup/teardown coordinator; strictly sequential episodes; exactly one live episode | W04/W03 both delegate coordination to "the P4 minimal VM setup / W07 repeat driver"; a single owner prevents sequencing drift | W03 §4; W04 §2.1, §4; Plan Agent guardrail (one owner per transition) |
| D2 | Two teardown depths: `FullRebuild` (all objects rebuilt; required) and `RamReuse` (truncate after `space.destroy`; `GuestRam` retained, re-`init_zeroed`, reloaded) | `FullRebuild` is the strongest no-residue proof; `RamReuse` isolates cleanup/initialization determinism from allocation variance (P4-H03); W03 explicitly supports both | W03 §4 ("both forms supportable"); task book P4-H01/H03 |
| D3 | Determinism is measured: five declared surfaces compared for exact equality across episodes and cold boots — post-load RAM digest, initial `VcpuContext`, Guest marker stream, exit-class/stop-cause sequence, per-episode count vector (counts compared modulo the monotonic episode index) | P4-V10/V11 require "consistent/deterministic" results; consistency needs a named comparison set, else it is unfalsifiable | task book P4-V10–V12; plan P4-H02/H03 |
| D4 | Digest = P4-local FNV-1a-64 over the post-load RAM content; no new dependency; equality-detection only, documented as non-cryptographic | determinism checking must not force a dependency decision (P0-W18 governance, assumed); inequality detection needs no cryptographic strength | task book §8 Implementation Choice; ADR-006 dependency restraint |
| D5 | Telemetry = in-memory ledger + run-record emission through the P0 baseline; no transport/backend/persistence; counters derive only from routed sibling events (no second emission point) | task book Out of scope (production backend); ADR-048 structure; one-emission-source rule prevents divergence from W02–W06 events | ADR-048; W01 A8; task book §1 |
| D6 | Run-record grammar P4-RR v1: versioned, line-oriented, machine-readable, emitted at episode boundaries and run end; fields fixed in [04 §4](04-code-contracts-telemetry.md); grammar changes bump the version and require W08 agreement | W08 consumes markers/records; a stable agreed surface is the W07→W08 handoff; versioning keeps it honest without freezing an API | W07 plan handoff; W05 §6 marker-change precedent |
| D7 | Declared repeat minimums: same-session ≥ 3 episodes with ≥ 1 `RamReuse`; cold-boot ≥ 5 iterations of the same declared build; stability = all determinism surfaces equal + all episode verdicts determinate; W08's manifest may raise, never lower | minimums must exist before automation to be objective; kept small enough to stay in-scope for P4 while detecting residue/order dependence | task book P4-V10/V11; W07 plan items 3–4 |
| D8 | Residue checks: per-episode accounting restoration (allocated == freed, via M6/M2 surfaces), re-init equivalence (digest after re-init == digest after first init), and a no-static-Guest-state review rule (all Guest state lives in per-episode objects) | "does not rely on prior Guest RAM or global CPU residue" needs mechanical checks plus a structural rule, not only equality | task book P4-V10; plan P4-H01 |
| D9 | W07 adds no `unsafe`; the driver and ledger run in setup context (not IRQ, not exit path); any discovered need is a recorded deviation with SAFETY justification and inventory entry | the driver coordinates already-unsafe seams (W04's switch); it needs none of its own | ADR-006; P0-W10 governance (assumed, W01 R20); Coding Guidelines |

## 5. Open items recorded by this design

| ID | Item | Owner / path | Handling |
|---|---|---|---|
| O1 | P4-RR v1 field list must be acknowledged by W08 before either side implements (marker-grammar precedent). | [P4-W08](../p4-w08-qemu-integration-regression/README.md) joint review at implementation | recorded as a review row in the [workflow](05-implementation-workflow.md) step 4; divergence fails review |
| O2 | Count-vector equality across cold boots assumes a fixed scenario sequence and fixed budget constant; if the exit budget or any count-affecting constant changes between builds, the stability criterion applies per-build-identity, not across builds. | W07 records; W08 manifest carries build identity | handled by carrying build identity in the run record (D6); no cross-build comparison is claimed |
