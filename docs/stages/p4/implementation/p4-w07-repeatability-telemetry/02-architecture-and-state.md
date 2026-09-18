# P4-W07 Architecture, Objects, and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W07 detailed design](README.md).

## 1. Logical modules

All modules are Core-side (no architecture registers appear anywhere); the
driver coordinates Arch seams through their published contracts only. Crate
and file placement follows the P0 workspace design (assumed contract, W02
foundations M8); this design fixes logical modules and boundaries, not file
paths.

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `run-episode` | Episode lifecycle: plan → construct → run → stop → teardown (two depths); sequencing only, via W02/W03/W04 contracts | none persistent beyond one episode's objects while it is live | `EpisodePlan`, sibling contracts | episode result + correlation data | fault judgment (W06), run decisions (W04), mapping/memory mechanics |
| `run-driver` | The repeat loop: iterate the episode list, hold the ledger, compare determinism surfaces, emit the run record | driver state: episode index, accumulated comparisons, ledger reference | run plan (episode list), episode results | run result, P4-RR v1 record | QEMU invocation (W08), verdict naming for automation (W08) |
| `determinism` | Surface capture and equality checking (digest, context equality, sequence equality) | none (pure checks over caller data) | post-load RAM, `VcpuContext`, marker stream, episode outcomes | equality results + digests | fixing what the surfaces contain (owner: this design's D3, applied here) |
| `telemetry-ledger` | Per-domain counters, fault-correlation records, count vectors, summary assembly | counters, correlation list (per run; in-memory) | routed events (`s2.*`, `gm.*`, `vcpu.*`, `diag.*`), episode boundaries | counts, correlation records, summary values | event emission (siblings own it), transport (none), policy (none) |
| `run-record` | P4-RR v1 formatting into bounded buffers through the logging baseline | none (writes caller buffers) | driver/ledger values, build identity | run-record lines | verdict decisions (reports them, does not make them) |

Layering check: no module names registers, descriptors, boards, SoCs, or
QEMU; the driver touches W02/W03/W04 objects only through their published
contracts. Guest-owned memory is read for the digest through the same
W03-provided host view used by the loader (M2), never through a new path.

## 2. Core objects and ownership

### 2.1 `RunDriver` (module `run-driver`)

- **Owned state:** the episode list (from the run plan), current episode
  index, the determinism-comparison accumulator, and a reference to the
  telemetry ledger.
- **Immutable after construction:** the run plan (scenario sequence, modes,
  repeat minimums).
- **Not owned:** any Guest object (episodes own theirs while live), the
  address space, the vCPU, Guest RAM (all created and consumed inside
  episodes), the ledger's semantics (it owns its own contract).
- **Lifecycle:** constructed per cold boot for the declared run; consumed at
  run end after the record is emitted. One driver per environment at a time
  (structural: it is created by setup, and a second concurrent driver is a
  host invariant violation like W04's double entry).

### 2.2 `Episode` (module `run-episode`)

- A per-episode composite of the W02/W03/W04 objects, created and torn down
  inside `run_episode` per the W03 §4 construction order and W04 §4 teardown
  sequencing. The episode owns its objects exactly for the episode's
  duration; nothing escapes except result values (stop cause, diagnostic
  reference, marker echo, digests).
- **Two teardown depths** (D2): `FullRebuild` — `vcpu.destroy` →
  `space.destroy` → `GuestRam::release`; `RamReuse` — `vcpu.destroy` →
  `space.destroy`, then retain the `GuestRam`, re-`init_zeroed`, regenerate
  grants, re-load. The retained `GuestRam` is owned by the *driver* between
  `RamReuse` episodes (the one stateful exception to "nothing escapes"; it is
  released by the driver's `FullRebuild` episodes or final teardown).
- The truncation point is recorded in the episode result so no consumer can
  mistake a `RamReuse` episode for a fully torn-down one.

### 2.3 `TelemetryLedger` (module `telemetry-ledger`)

- **Owned state:** per-domain counters (fixed category set from the sibling
  event namespaces), per-episode count vectors, the fault-correlation record
  list, and the run summary values.
- **Not owned:** event emission (siblings emit; the ledger subscribes at
  their published points), any Guest object, the run record's grammar (that
  is `run-record`'s, versioned).
- Reset semantics: counters are monotonic within a run; per-episode vectors
  are snapshotted at episode end for the equality check (counts compared
  modulo the episode index, D3).

### 2.4 `DeterminismCheck` values (module `determinism`)

- Pure value results: `SurfaceDigest(u64)`, `ContextEqual(bool)`,
  `SequenceEqual(bool)`, `CountsEqual(bool)`. No state, no owner beyond the
  driver's accumulator.

## 3. Driver state machine

```text
[absent] --construct(run plan)--> Idle --next episode--> Ep(n) construct
   ^                                                        |
   |                                                        v
   +------------------ all episodes done ---- RunEnd <- stop/teardown
                                                     |        (record + verdicts)
                                        more episodes-+
```

Per episode (both depths share the front path; W03 §4 order normative):

```text
Ep(n):
  1. create address space            (W02)          [FullRebuild]
     or reuse retained GuestRam      (driver)       [RamReuse only]
  2. allocate + zero Guest RAM       (W03)          [FullRebuild]
     or init_zeroed retained RAM     (W03 re-init)  [RamReuse]
  3. mapping grants -> map           (W03 -> W02)
  4. validate image plan             (W03)
  5. write boot-info + copy image    (W03)          [deterministic surface:
                                                     digest captured here]
  6. construct vCPU                  (W04)          [deterministic surface:
                                                     context captured here]
  7. run: activate; enter; exits; stop   (W02/W04)  [outcomes + W06 diagnosis
                                                     captured here]
  8. teardown per depth              (D2)           [accounting check here]
  9. equality vs episode 0           (determinism)  [per run: vs first episode;
                                                     per cold boot: vs declared
                                                     first-boot values]
```

Rules:

- A failing step aborts the run with a determinate failure result; the driver
  never continues to the next episode after a teardown failure (W04 §4's
  "never leaves a half-torn Guest for the next iteration" duty).
- Episode 0 of a run is the reference for equality; the P4-RR record carries
  the reference digests so a cold boot N can be compared against boot 1's
  recorded values by W08.
- `RamReuse` episodes must produce the same post-load digest as the
  `FullRebuild` reference: re-init equivalence is itself a checked property
  (D8), not an assumption.

## 4. Ownership of "residue"

Three mechanisms, one per residue class:

1. **Guest RAM residue:** `FullRebuild` re-allocates; `RamReuse` re-zeros via
   W03's re-init; the digest equality proves the resulting bytes identical.
2. **Allocator/accounting residue:** the M6/M2 accounting totals are read
   before and after each teardown; an imbalance fails the episode (leak
   reporting per W03 §3.4 — surfaced, never swallowed).
3. **Global/CPU residue:** structural rule — W07 creates no static or global
   Guest state, and per W04's documented retention, the only cross-episode
   survivors are the pCPU-owned last exit frame and retained diagnostic
   (explicitly documented, overwritten by the next episode; they are
   diagnostics, not behavior inputs). The review rule (DV08) audits that no
   other survivor exists.

## 5. Determinism surfaces (normative list)

| # | Surface | Captured at | Equality rule |
|---|---|---|---|
| S1 | Post-load Guest RAM content | after W03 step 5 | FNV-1a-64 digest equality over the whole region |
| S2 | Initial vCPU context | after W04 step 6 | byte equality of `VcpuContext` (typed field comparison, not raw memory) |
| S3 | Guest marker stream | per episode, from console data as observed by the environment and echoed in outcomes | exact sequence equality of `VG-<id>:<EVENT>` lines for the scenario |
| S4 | Exit/stop sequence | per episode, from W04 outcomes | exact sequence equality of (class, stop cause) pairs |
| S5 | Count vector | episode end, from the ledger | equality modulo the episode index |

Notes: S3 in fault scenarios is a *prefix* of the ideal stream (the Guest
stops mid-scenario by design at the marked trigger); equality is defined as
equality of the declared expected prefix for that scenario (from the W05
table), not of full length. S5 excludes counters that legitimately vary with
episode index; the modulo rule is: compare the episode-local deltas, not
cumulative totals.

## 6. Concurrency model

- The driver runs in setup context on the boot/management pCPU, never in IRQ
  context and never inside the Guest run segment; episode steps 1–6 and 8–9
  are sequential setup work.
- Exactly one episode is live; the single-Guest P4 model (W01 A5) is
  preserved without a global singleton — the driver is an object created by
  setup, and W02/W03/W04 objects are per-episode.
- The ledger is written from setup context only (events are counted at
  episode boundaries by reading sibling-observable outcomes; no counter is
  incremented from the exit path, keeping W06's containment-first ordering
  intact).
- Cross-CPU operations: none (P4 scope; the Reserved W02 seam is untouched).
  The design does not preclude multi-pCPU: nothing here assumes the driver
  must be on the Guest's pCPU.

## 7. Telemetry points (W07-scope)

`run.episode.begin` (index, scenario, mode), `run.episode.end` (result,
depth), `run.determinism` (surface, result), `run.summary` (run end). These
names are stage-local, routed through the P0 baseline (W01 A8), and recorded
as implemented facts by W09. The P4-RR v1 run record ([04 §4](04-code-contracts-telemetry.md))
is the machine-readable emission; W08 parses it and nothing else for
verdicts.
