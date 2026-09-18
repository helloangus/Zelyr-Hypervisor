# P2-W08 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W08 detailed design](README.md).

## 1. Package outcome

A regression suite whose single run produces, per scenario row, a recorded
outcome (passed / failed / blocked / not run) with input, command, seed
where applicable, environment, and timestamp — sufficient for P2-V10's
acceptance: "reproducible host-side evidence" for bounded bad-input
handling, conflict/overflow treatment, exhaustion/reuse, protected-page
safety, stress consistency, and deterministic repeated discovery. The suite
is evidence-producing machinery; this design defines it and prescribes
nothing about results.

## 2. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W08 relies on | Failure boundary if delivered differently |
|---|---|---|---|---|
| A1 | W01 diagnostic taxonomy and host-runnable intake over byte fixtures | [W01 intake boundary §3](../p2-w01-boot-platform-description-intake/01-intake-boundary.md) | S1xx expected observables | Divergent or missing classes: matrix rows updated only via a recorded revision; never by loosening |
| A2 | W02 five-state fact model, fatal set, determinism property | [W02 foundations §6–§7](../p2-w02-platform-discovery-normalization/01-scope-and-foundations.md) | S5xx comparisons and fatal-row expectations | Same rule |
| A3 | W03 conflict policy, `MapFatal` variants, seal validation, `MapSummary` equation | [W03 contracts §2, §6](../p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md) | S2xx expected observables | Same rule |
| A4 | W04 typed allocate/free errors, `AllocationStats` conservation, per-op audit hook | [W04 contracts §5–§7](../p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md) | S3xx expected observables and soak audits | If no audit hook exists at host strength, raise a W04 contract conflict; do not inspect internals from the harness |
| A5 | W05 typed heap errors, `HeapStats`, `audit()` | [W05 contracts §3–§5](../p2-w05-dynamic-small-allocation/03-code-contracts-heap.md) | S4xx heap scenarios | Same rule |
| A6 | W07 fixture format (`check` entry, manifest, class mapping) | [W07 fixtures](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md) | S1xx input construction and S5xx fixture reuse | W08 adds mutation fixtures in-format ([W07 §6](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md)) |
| A7 | W06 render determinism and `ConsistencyBroken` check IDs | [W06 contracts §1, §5](../p2-w06-platform-memory-inspection/03-code-contracts-inspection.md) | S5xx render comparison and divergence-injection rows | Same rule |
| A8 | Host test entry, execution, and gate integration from the P0 baseline | P0-W08 (planned) | Where/how the suite executes and how gates invoke it | If absent at implementation time, blocked upstream defect — build the harness code, record execution blocked |

Pseudo-random generation uses the P0 baseline's test-support generator if
provided; otherwise a dependency-free seeded generator implemented inside
the test-support code (no new dependency; P0-W18 governance).

## 3. Harness placement and ownership

- Harness and scenario code live with the host-test areas of the modules
  they exercise (physical placement pending P0-W03's workspace); W08's own
  artifacts are the matrices, the ID scheme, the evidence format, and the
  suite entry that orders scenario groups S1→S5.
- W08 owns no production module. Where a scenario needs to construct
  internal states (e.g. a divergent stats pair for W06-DV05-style
  injection), construction goes through the owning package's existing test
  constructors or, absent those, a raised sibling-design conflict.
- Scenario failures are reported verbatim: scenario ID, expected observable,
  actual observable, and the owning package ID. The suite never retries a
  failed scenario to "flakiness-filter" it — a safety suite either
  reproduces or is broken.

## 4. Proof boundary (binding on every matrix row)

Host-side results prove logic properties of the implemented contracts over
the exercised inputs. They do not prove:

- QEMU integration, real firmware blobs, or the A2/A3 P1 window behavior
  (W09's evidence);
- real-hardware semantics — caches, TLB, DMA, timing, SMP (no such
  mechanism exists in P2);
- completeness of the input space (S1xx/S2xx are structured enumerations,
  S3xx/S4xx are seeded samples — a passing soak bounds, not exhausts);
- performance, latency, or memory footprints (never measured here).

Every matrix row in [02](02-matrices-input-and-map.md) and
[03](03-matrices-allocator-stress-determinism.md) restates its own slice of
this boundary in the "Proves / does not prove" column.

## 5. Scenario taxonomy and numbering

```text
W08-S1nn  malformed DTB (P2-J01)          — intake boundary
W08-S2nn  map conflicts/overflow (P2-J02) — W03 builder + seal
W08-S3nn  allocator lifecycle/safety (P2-J03) — W04 (+ W05 release semantics)
W08-S4nn  allocation stress (P2-J04)      — W04 + W05 combined soak
W08-S5nn  determinism (P2-J05)            — W02/W03/W06 repeated runs
```

`nn` is two digits, assigned in the matrices; IDs are stable — retiring or
reordering a scenario is a recorded revision, and evidence from a retired
ID stays valid for what it tested.
