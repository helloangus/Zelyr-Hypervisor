# P6-W02 Architecture, Objects, State Machines, and Concurrency

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W02 detailed design](README.md).

## 1. Logical modules

Placement follows the crate layering the P0–P5 designs establish: register
access and GIC sequences are architecture/platform-layer knowledge; the
readiness ledger type is Core-consumable but GIC-specific facts live behind
the module boundary. If the established crate tree differs, map by layer;
the module contracts are the design, not file paths.

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `gic-regaccess` | The single audited `unsafe` boundary: mapped-frame handles, volatile register read/write/read-modify-write, field masks, barrier helpers | Frame mappings (created once from PlatformInfo) | mapped frame descriptors | typed register operations | Does not know GIC semantics; no sequencing policy |
| `gic-distributor` | Phase-A global bring-up: identity confirmation, quiesce, configure, initial SPI state, enable | Global distributor lifecycle state (single instance) | W01 decision + expected identity; mapped GICD frame | `DistributorReadiness` | Does not touch GICR or ICC registers; no routing policy (initial determinate routing only) |
| `gic-local` | Phase-B per-pCPU bring-up: GICR identity + wake, local SGI/PPI baseline, ICC sequencing, local readiness publication | Local context per pCPU (in P3 per-CPU storage) | own pCPU identity; expected identity; mapped GICR frame | `LocalGicReadiness` publication | Does not mutate the Distributor; does not enable consumer-owned interrupts |
| `gic-readiness` | The readiness/failure ledger type and queries | Per-pCPU atomic readiness slots + global distributor slot | publications from `gic-distributor` / `gic-local` | queries for W03/W04/W05 | Does not perform bring-up; cannot modify slots except through the owning path |
| `gic-telemetry` (thin) | W02 event emission under the P0 namespace contract | none | readiness outcomes, residuals, failures | events | Not a logging framework |

## 2. Core objects

### 2.1 `GicFrames`

The mapped GICD frame and the GICR frame set, created once from confirmed
platform facts at Phase-A time. GICR frames are mapped lazily per pCPU by
that pCPU (or mapped boot-time if the established early-mapping contract
already covers the region — either is acceptable; the *access* rule INV-3
does not change). Each mapping is an immutable handle; no runtime
map/unmap of GIC frames exists in P6.

### 2.2 `DistributorState` (global, single instance)

```text
Unconfigured -> Confirmed -> Quiesced -> Configured -> Enabled
      |              |            |            |
      +--------------+------------+------------+--> DistributorFailed(reason)
```

- `Unconfigured`: pre-probe. No write has occurred.
- `Confirmed`: probe results match `ExpectedGicIdentity`.
- `Quiesced`: GICD disabled and completion observed; residuals surveyed.
- `Configured`: SPI range disabled + determinately routed; priorities
  initialized; group configuration set.
- `Enabled`: GICD enabled, completion observed; global bring-up done.
- `DistributorFailed`: fatal for interrupt functionality — the
  initialization owner stops and the P1 crash/diagnostic path owns the
  outcome (a hypervisor whose distributor cannot initialize has no
  recoverable interrupt service; this is an invariant failure, not a guest
  fault).

Transition owner: the Phase-A sequence, boot pCPU, once. No other code path
may write the distributor lifecycle state. The state object lives in the
global initialization area established by P1/P3 (no new global-singleton
mechanism is invented here).

### 2.3 `LocalGicContext` (per pCPU, in P3 per-CPU storage)

Fields: mapped GICR frame handle; probe record (affinity match, processor
number, Last-flag observation); wake record (timed handshake outcome);
local baseline record (residual SGI/PPI counts); published readiness slot
written exactly once. The context exists only for the owning pCPU; the boot
pCPU's context is created by the same Phase-B sequence run on the boot CPU
— no BSP-local shortcut or reuse path exists (plan requirement).

### 2.4 `ReadinessLedger` (in `gic-readiness`)

Per-pCPU slot: `NotStarted | LocalReady | LocalFailed(reason)`, atomic,
written once release-ordered by the owning pCPU. Global slot:
`DistributorPending | DistributorReady | DistributorFailed`, written once
by Phase A. Queries: `distributor_ready()`, `local_ready(pcpu)`,
`ready_set()` (for W04 targeting). The ledger is the *only* consumer-visible
view of bring-up; consumers must not read `gic-local` internals.

## 3. Lifecycle and state machines

### 3.1 Phase A — global (boot pCPU, single-threaded by boot rendezvous)

```text
probe GICD identity (read-only)             [W01 DistributorIdentityProbe]
  mismatch -> DistributorFailed + capability fold-back + stop
verify coverage: enumerated GICR frames match the possible-pCPU set
  (reads of GICR_TYPER only, pre-wake; miss -> DistributorFailed
   with Incomplete finding naming the affinity)
disable GICD; poll until disabled (bounded)
survey+clear residuals: supported-SPI pending/active read -> log -> clear
configure: SPI range disabled, priority defaults, group config,
  determinate routing to boot-pCPU affinity (IRM=0)
enable GICD (ARE + Group1 NS); poll until enabled (bounded); DSB
publish DistributorReady (release)
```

### 3.2 Phase B — local (each pCPU at its P3 local-init point)

```text
probe own GICR frame identity (read-only)   [W01 RedistributorIdentityProbe]
  mismatch -> LocalFailed(identity) + ledger publication
wake GICR: ProcessorSleep=1; poll ChildrenAsleep (bounded timeout)
  timeout -> LocalFailed(wake-timeout)
configure local baseline: SGI/PPI all disabled, defaults, residuals
  surveyed+cleared+logged
program CPU interface: SRE write-and-verify -> CTLR (EOImode=0, defaults)
  -> PMR allow-all -> ISB; (group enable is last, §4)
enable Group1 at the interface; ISB
publish LocalReady (release); emit telemetry
```

The boot pCPU runs Phase B after Phase A completes; secondary pCPUs run
Phase B after the P3 boot rendezvous releases them and after the
Distributor is `Enabled` (a pCPU whose local init starts before global
enable would publish readiness that W04 could act on inconsistently —
ordering invariant ORD-1: `DistributorReady` publication happens-before any
`LocalReady` publication except the boot pCPU's, which happens after its own
Phase A).

### 3.3 Failure behavior

Local failure publishes `LocalFailed(reason)` and returns; P3 lifecycle
owns the consequence (CPU excluded from the online/eligible set). Global
failure stops the boot sequence through the established initialization
error path. No W02 path panics on a pCPU-local failure; no W02 path
continues serving interrupts after a global failure.

## 4. Concurrency model

- Phase A runs before secondary release (P3 rendezvous), so distributor
  state has one writer without locking during bring-up. Post-bring-up,
  global GICD mutation (W04 routing) must take the distributor-scoped
  lock; lock order: P3 base locks → distributor lock (W04 documents its
  position; W02 only defines the lock object's existence and class per the
  P3 synchronization contract).
- Local state: one writer (the owning pCPU). No lock. Cross-CPU visibility
  only through the ledger's release/acquire publications.
- The ledger slots are the only shared mutable W02 state; they use
  acquire/release atomics per the P3 atomic-ordering contract. No
  read-modify-write on slots (write-once discipline).
- IRQ context: bring-up runs with interrupts disabled at the CPU (pre-GIC
  state) — the P3 local-init context; W02 does not manage interrupt
  masking itself beyond the architectural requirement that Phase A/B are
  not interruptible mid-sequence (documented in contracts).

## 5. Security model

- Platform-provided frame descriptors were validated by W01/P2; W02
  re-checks mapping bounds at map time (the register-access surface
  rejects out-of-frame offsets).
- The Secure world is never written: only Non-secure-view registers are
  touched; a dual-security-state indication is a stop condition per the P6
  posture ([01 §2](01-scope-and-foundations.md)).
- No Guest input exists in W02; Guest GIC state is not designed here
  (W07/W08 boundary).
- Residual firmware configuration is treated as untrusted: surveyed,
  logged (bounded), and cleared rather than adopted.

## 6. Telemetry events

Under the P0 trace-event namespace (assumed contract; W13 correlates):
`gic.distributor_phase` (state entered, duration), `gic.local_phase`
(pcpu, state entered), `gic.residual` (scope, kind, count, sampled IDs —
bounded), `gic.local_failure` (pcpu, reason), `gic.probe_result` (scope,
match/mismatch). Payloads carry affinity/processor numbers, never platform
names. High-rate residuals are aggregated (counts), not per-event
streams.
