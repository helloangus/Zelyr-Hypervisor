# P6-W04 Scope, Prerequisites, and Mechanism Foundations

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W04 detailed design](README.md).

## 1. Prerequisite and assumed-contract inventory

### 1.1 W02 readiness and initial routing

Assumed source: [P6-W02](../p6-w02-physical-gic-bring-up/README.md). W04
consumes: the readiness ledger (`LocalReady` per pCPU, `DistributorReady`),
the initial SPI routing state (every supported SPI disabled, routed
IRM=0 to the boot-pCPU affinity), the distributor-scoped lock class, and
the register-access surface. Failure boundary: if the initial routing
state or lock class differs from the W02 design's record, the route-change
protocol's determinacy argument is unsound; stop and record against the
W02 design.

### 1.2 W03 dispatch and consumer surface

Assumed source: [P6-W03](../p6-w03-physical-interrupt-lifecycle/README.md).
W04 consumes: SGI classification and consumer registration
(`register_consumer` with an `SgiId`), the single acknowledge/complete
path (SGI consumers run in the dispatch loop), and the per-pCPU receipt
counters. Failure boundary: if the consumer surface shape differs, W04's
receipt side has no owner; stop and record against the W03 design.

### 1.3 P3 lifecycle, notification, TLB-transport, synchronization

Assumed sources:
[p3-w03](../../../p3/plans/p3-w03-physical-cpu-lifecycle.md) (online set
and failed/excluded vocabulary),
[p3-w07](../../../p3/plans/p3-w07-cross-cpu-notification.md) (minimal
notification primitive; its delivery/accounting semantics are P3's),
[p3-w08](../../../p3/plans/p3-w08-tlb-shootdown-transport.md) (W04 defines
*no* TLB semantics; the transport boundary stays P3's),
[p3-w06](../../../p3/plans/p3-w06-concurrency-synchronization.md) (lock
classes and atomic ordering). Also the affinity facts: P2/P3 CPU identity
maps `PcpuId` ↔ MPIDR affinity values; W04 derives GIC affinity fields
from that mapping, never from a board table.

Failure boundary: if P3's notification primitive turns out to consume a
different SGI range than this design's partition assumes, the partition
row of the [README decision 4](README.md) is a recorded conflict — the P3
handoff contract governs; W04 must not renumber P3's range locally. If P3
provides no SGI consumption at all (a polling primitive), rows SGI 0..=7
return to the unassigned/reserved pool by a recorded revision of this
design, not by silent local choice.

### 1.4 Entry gating

Per the plans index, W04 progresses after W02 and W03 evidence exist; the
task book §2 entry review (P6-ENTRY-03/04 for P3 contracts) applies. W11
multi-vCPU scenarios additionally require the evidenced upstream
multi-vCPU contract before they can run (W11's condition, not W04's, but
W04's multi-target evidence is one of its inputs).

## 2. The mechanism boundary

W04 owns exactly:

```text
send side:   compose typed targets → validate eligibility (atomic view)
             → emit SGI1R write(s) → publish send accounting
receipt side: W03 dispatch classifies the SGI → W04-registered consumer
             (per partition row) runs → W03 completes → W04 receipt
             counters increment (via the consumer or the stats surface)
route side:  quiesce check → lock → IROUTER write → (re-enable if
             previously enabled) → unlock → route event
```

Not owned: what any consumer does with the event (P3 transport, P7 policy,
W11 scenarios); TLB invalidation; when a pCPU should be kicked; any
Guest-visible SGI.

## 3. Scope classification detail

**Required:** `SgiTarget` forms and decomposition; eligibility predicate;
ICC_SGI1R emission with barrier/sequencing rules; send/receipt accounting
and drift reporting; the SGI ID partition; the validation-scenario SGI; SPI
route-change protocol with quiesce checks and IRM=0-only targets; route
change events; failure outcomes (ineligible target, quiesce conflict).

**Reserved (with triggers):** additional P6 Host-SGI message kinds
(trigger: an approved consumer design with a protocol need); 1-of-N (IRM)
SPI routing and any "any-CPU" delivery (trigger: an approved design that
accepts indeterminate target attribution); affinity re-balancing or
routing policy (trigger: P7 or an approved device design); SGI priority
differentiation (trigger: P6-W10 semantics); broadcast self-excluding
sends as a P6-internal consumer path (the form is declared; P6 consumers
use explicit targets).

**Out of Scope:** scheduler policy and reschedule semantics (P7);
TLB-shootdown semantics (P3-W08 boundary); Guest SGIs and virtual
interrupts (W07/W08, P8); load balancing; RPC/message layers; GICv2 SGI
mechanics; Orange Pi hardware tier (P15).

## 4. Eligibility and failure rules (Required)

- Eligible target = P3 online ∧ W02 `LocalReady` ∧ not
  `LocalFailed`. The predicate is evaluated over a consistent view of both
  sets at send time (ledger reads are acquire; the pair is read
  target-by-target before any emission, so the validated set is fixed
  before the first write — the validation-then-emit order is what makes a
  rejected send truly non-partial).
- Ineligible target in a requested set → the whole send is rejected with
  `TargetIneligible(pcpu)`; no SGI1R write occurs; the caller decides the
  recovery (P3 vocabulary owns the consequence of an offline target).
- Self-target is eligible if the sending pCPU is itself ready (it is,
  since it is executing).
- A target that becomes ineligible *after* validation but before hardware
  consumption cannot cause loss: its GICR holds the SGI pending while
  disabled (W02 baseline), and a `LocalFailed` pCPU is by definition
  excluded from later consumption — the accounting drift report makes
  this visible rather than silent (drift is expected only in the
  failed-target case and is reported, not treated as corruption).

## 5. Non-policy commitments (Required review points)

- The generic Host event SGI carries zero payload and zero priority
  meaning; nothing in W04 may branch on "why" a send happened.
- No W04 path inspects or influences vCPU/run-queue state; pCPU is the
  only target namespace.
- No W04 path is Guest-reachable; the Validation Guest observes Host SGI
  effects only through declared W11 scenarios.
- Route changes serve test/consumer designs only; W04 implements no
  balancing heuristic and no default re-routing.
These commitments are the review targets for W04-DV08.
