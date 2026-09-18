# P6-W04 Architecture, Accounting, and Concurrency

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W04 detailed design](README.md).

## 1. Logical modules

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `sgi-target` | `SgiTarget` forms, eligibility evaluation, decomposition into SGI1R encodings | none (pure over the sets) | PcpuId values, ledger views, affinity map | encodings + validated target lists | Does not emit; no policy on who may send |
| `sgi-send` | Emission and send accounting | per-sender send counters | validated targets | SGI1R writes, accounting entries | Does not validate (done upstream); no consumer semantics |
| `sgi-accounting` | Correlation of send and receipt records; drift report | global per-SGI send totals; receipt mirrors of W03 counters | send records; W03 stats | accounting views, drift events | No enforcement, no timeout, no rebalancing |
| `spi-routing` | Route-change protocol, route registry, route events | route table (per supported SPI: current affinity, enabled state) | change requests | IROUTER writes under lock; events | No routing policy; no trigger-type changes; no Guest routes |

Placement follows the established crate layering; map by layer if the tree
differs.

## 2. Core objects

### 2.1 `SgiTarget`

```text
enum SgiTarget {
  One(PcpuId),
  ExplicitList { members: bounded set of PcpuId },   // validated, any size
  AllOnlineExcludingSelf,                            // IRM form; declared,
                                                     // not used by P6
                                                     // internal consumers
} (P6-internal)
```

`ExplicitList` decomposition: the architecture encodes one affinity level
plus a 16-target list per SGI1R write; targets sharing (aff3,aff2,aff1)
group into one write with a TargetList; others form additional writes. The
decomposition is deterministic (fixed ordering by affinity value) so a
send's write sequence is reproducible — a property the W04-DV05 accounting
review relies on.

### 2.2 `SgiPartition`

The static table of [README decision 4](README.md): rows carry the SGI ID,
the owning consumer kind (P3 transport / P6 generic event / W11 validation
/ unassigned), and the registered consumer expectation. The partition is a
constant of this design; a row's consumer registers through the W03
surface at its own init. The partition is *not* an ABI: P7/P8 receive the
table as a documented mechanism fact, not a frozen contract.

### 2.3 `SendRecord` and accounting views

Per sending pCPU: monotone counters per SGI ID and per decomposition
write. Global view (sampled, lock-free): per-SGI send totals vs summed
receipt counters from W03's stats. Drift report: emitted as an event on
sampling divergence beyond zero where the target set's readiness
guarantees convergence (failed-target divergence is expected and labeled).
No timeouts, no retransmission — SGIs are not messages; the accounting
exists to make P6-V04/V05 evidence determinate, not to add reliability
semantics the hardware does not need.

### 2.4 `SpiRouteEntry` and route state machine

```text
per supported SPI (immutable after boot except via change_route):
  InitialRoute(boot affinity)          # W02 state, imported at init
  -> Rerouted(target affinity)         # via change_route only
  failure: unchanged (Rejection returned)
```

`change_route` internal sequence (the only state-changing path):

```text
check: spi in supported range; route table holds InitialRoute|Rerouted
lock:  distributor-scoped lock (W02 class)
check: enable state (read GICD ISENABLER bit) == disabled, else
       Err(RouteEnabled)                 # caller disables first via W03
check: pending/active bits (GICD ISPENDR/ISACTIVER) == 0, else
       Err(RouteBusy)                    # determinate: no in-flight move
write: IROUTER<spi> = affinity(target) (64-bit write, reserved zero),
       IRM=0
dsb()                                   # route visible before any later
                                        # enable by the caller
unlock; emit route event with old/new affinity
```

The check-lock-check ordering under the lock closes the TOCTOU window on
pending/active: delivery of this SPI to the old target cannot start after
the checks (it is disabled) and cannot be mid-flight (it is inactive);
post-change asserts route to the new affinity. This is the determinacy
argument P6-V06 requires.

### 2.5 Affinity mapping

`PcpuId → GIC affinity fields` derives from the P2/P3 identity facts
(MPIDR affinity per pCPU). The mapping is read-only platform data; no
board constant appears in W04. Aff3 handling follows the pinned GIC
revision's SGI1R/IROUTER field layout (Specification Investigation
checkpoint, W01 step-1 record).

## 3. Concurrency model

- Senders: any pCPU, any thread context where interrupts are not
  concurrently reconfigured; send is a single system-register write per
  decomposition — no lock (the write is atomic at the architectural
  level). Send counters are per-sender (no cross-CPU writes).
- Eligibility evaluation: acquire reads of P3 and W02 ledger views; no
  lock; the validation-then-emit order substitutes for locking
  ([01](01-scope-and-foundations.md) §4 states the residual race and its
  harmless consequence).
- Route changes: serialized by the distributor-scoped lock (same class as
  W02 post-bring-up global mutation and W03's SPI enable/disable);
  lock order per P3 baseline: base locks → distributor lock; never held
  across a consumer callback or an SGI send.
- Accounting sampling: W13-context reads only; no IRQ-context aggregation.

## 4. Failure and degradation behavior

| Failure | Outcome |
|---|---|
| Ineligible target in set | Whole send rejected (`TargetIneligible`); zero writes; counted |
| Empty target set after decomposition | Rejected (`EmptyTargetSet`); counted |
| SGI1R write on a not-yet-group-1-enabled local interface | Unreachable: sender is by definition `LocalReady` (interface enabled) |
| Route change on enabled or busy SPI | Rejected (`RouteEnabled` / `RouteBusy`); state unchanged |
| Send/receipt drift (non-failed target) | Drift event + report entry; investigated as a hardware/anomaly diagnostic; never auto-corrected |
| P3 handoff partition conflict | Recorded conflict per [01 §1.3](01-scope-and-foundations.md); no local renumbering |

No W04 failure panics; all are named results or reported anomalies
consistent with the P3/W03 failure vocabularies.

## 5. Telemetry

Events under the P0 namespace: `sgi.send` (sender, sgi id, target form,
write count — rate-limited or sampled), `sgi.receipt_sample` (via W03
stats surface; W04 adds none in IRQ context), `sgi.drift` (sgi id, send
total, receipt total, labeled expected/unexpected), `spi.route_change`
(spi, old affinity, new affinity, result). W13 correlates these for
P6-V24–V26.
