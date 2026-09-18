# P6-W05 Architecture, Ownership, and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W05 design entry](README.md).

## 1. Position in the P6 event chain

W05 is the Host-side time foundation of the P6 asynchronous-event chain:

```text
W05 deadline expires on a pCPU
  -> EL2 physical-timer PPI raised
  -> P6-W03 physical IRQ lifecycle classifies and dispatches
  -> W05 expiry service records a bounded expiry record (this design)
  -> W05 consumer (W06 deferred-expiry evaluation; later P7 tick policy)
       observes the record in mainline context
```

W05 does not deliver Guest-visible events and does not touch virtual
interrupts. Guest-timer expiry reaching a vCPU is P6-W06 behavior built on
W05's expiry record; hardware presentation is P6-W08. Layer placement: the
time-value types ([03](03-code-contracts-time-core.md)) are
architecture-mechanism types; the counter/deadline-register access is
Arch-domain code; no Core-layer module may contain the system-register
sequences defined here.

## 2. Logical modules

Physical crate/file placement is decided by the workspace-owning packages and
recorded in the implementation record; this design fixes logical modules,
responsibilities, and boundaries only.

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `time-core` (time values and clock) | Typed counter/delta/deadline values; checked conversion from real-time units via the validated frequency; ordered counter reads | None (stateless value space; a validated frequency constant per intake) | frequency value from intake; requested deltas | typed instants, deadlines, conversion results | timer programming; wall clock; cross-CPU synchronization of time |
| `deadline-timer` (per pCPU) | Arm/cancel/rearm of one deadline event; expiry record production; register sequencing and barriers | the pCPU's timer object: state machine, armed deadline, arm generation, pending expiry record, diagnostics counters | consumer arm/rearm/cancel calls on the owning pCPU; classified timer PPI from W03 | bounded expiry records; readiness/intake results; diagnostics | consumer policy; IRQ classification; completion (EOI/deactivate) responsibility, which stays with W03 |
| `timer-consumer-dispatch` (registration) | Binds at most one consumer to a pCPU timer; hands expiry records to the consumer in bounded mainline context | consumer registration slot | consumer registration; expiry records | consumer callback in mainline context | the consumer's own semantics (W06 expiry evaluation, future P7 tick) |

There is exactly one `deadline-timer` object per online pCPU, created during
per-pCPU bring-up (after P3 lifecycle and W02 local GIC readiness) and
destroyed only with pCPU shutdown, which P3 owns. A consumer slot that is
empty is valid: the timer still arms, fires, and records; the record is
retained until read or overwritten by the next expiry (overwrites are counted
as `consumer_absent` diagnostics, not silently dropped — see
[04](04-code-contracts-deadline-timer.md) §3).

## 3. Core objects and ownership

| Object | Type basis | Sole owner | Mutated by | Lifetime |
|---|---|---|---|---|
| `TimerCounter(u64)` / `TimerDelta(u64)` / `TimerDeadline(u64)` | newtype wrappers per D2 | value semantics (no shared state) | construction/conversion functions | value lifetime |
| pCPU timer object (`DeadlineTimer`) | state machine + registers | the owning pCPU (D7) | owner-pCPU execution contexts only (arm/rearm/cancel calls, IRQ service, mainline dispatch) | pCPU online period (P3 lifecycle) |
| Expiry record (`ExpiryRecord`) | small typed record | the pCPU timer object until handed to the consumer; then the consumer | expiry service (write), consumer (read-and-clear) | until consumed or overwritten (counted) |
| Frequency fact (`TimerFrequency`) | validated per-pCPU-identical value | W05 intake (created once, read-only afterwards) | intake check only | system lifetime |
| Timer PPI INTID binding | platform/arch intake data (D6) | W02/W03 intake; read-only for W05 | intake | system lifetime |

Ownership rules with no exceptions: no static mutable global timer state; no
cross-pCPU pointer to another pCPU's timer object; no aliasing of the expiry
record between IRQ and mainline contexts other than the handoff defined in
[04](04-code-contracts-deadline-timer.md) §3 (single-writer, single-reader,
generation-guarded).

## 4. Lifecycle and state machines

### 4.1 DeadlineTimer state machine (per pCPU)

```text
            arm(deadline, event)            cancel()
  +------+ --------------------> +-------+ ----------> +------+
  | Idle |                       | Armed | <-----------+ ......  (rearm from
  +------+ <-------------------- +-------+  cancel() wins     service below)
     ^         timer IRQ fires    |   |
     |         (W03 dispatch)     |   | rearm() from expiry service
     |                            |   v
     |                        +-------+
     +----------------------- | Fired |  expiry record stored; consumer
       rearm -> Idle/Armed    +-------+  dispatch pending
```

- **Idle:** compare-value register disabled (`CNTHP_CTL_EL2`.Enable = 0). No
  deadline is trusted.
- **Armed:** deadline programmed, timer enabled, `arm_generation` incremented.
  `arm_generation` is a saturating counter that tags the armed interval; the
  expiry service and cancel paths use it to detect a stale fire (a fire whose
  service-time generation does not match the current generation).
- **Fired:** the PPI was received and the expiry record stored. If the
  consumer requested anchored repetition, the service may rearm (transition
  Armed, deadline-anchored, bounded catch-up per D3); otherwise Idle.

Legal transitions only; any observed state outside this machine is an
impossible-state diagnostic (counted, timer forced to Idle with the register
disabled, and surfaced per the failure model in
[06](06-validation-and-handoff.md) §2).

### 4.2 Cancel/fire race

Cancel and expiry can race because the PPI can already be in flight when
`cancel()` executes. The resolution, in order:

1. `cancel()` disables the timer first (deasserting the level-signaled PPI),
   then invalidates the arm generation, then transitions Idle.
2. A PPI that arrives after the disable is a condition-cleared delivery: the
   W05 handler observes the disabled/changed state and reports a
   *cancelled-fire* classification result to the W03 lifecycle, which owns
   acknowledgement and completion. No expiry record is produced.
3. A PPI that arrives before the disable with a matching generation produces a
   normal expiry record; the subsequent `cancel()` observes `Fired` and
   transitions Idle, returning a "already fired" result so the consumer learns
   that the event it cancelled was delivered.

This division keeps completion responsibility entirely inside the W03
lifecycle (assumed contract; failure boundary in
[01](01-scope-and-foundations.md) §5). If W03's delivered lifecycle cannot
express a condition-cleared delivery as a safe classified outcome, this is a
boundary conflict to raise against the W03 design — not something W05 works
around locally.

### 4.3 Intake lifecycle

```text
No intake -> frequency + INTID + trap-posture checks -> IntakePassed
          -> any check fails -> IntakeRejected (timer stays unused; precise
                                diagnosis recorded; no partial enablement)
```

Intake is re-checked at per-pCPU bring-up: `CNTFRQ_EL0` must match the
validated system frequency on every online pCPU; a mismatch blocks that pCPU's
timer and is a stage-blocker diagnosis (P6-V07/P6-V08 rows must record it).

## 5. Concurrency model

- **Owner-pCPU-only (D7):** every mutating operation — arm, cancel, rearm,
  expiry service, record read-and-clear — executes on the owning pCPU. The P3
  per-CPU ownership contract is the authority that makes this sound; P3
  guarantees no other pCPU touches the object.
- **IRQ vs. mainline on the same pCPU:** the two contexts that touch the
  timer object are the W03 IRQ dispatch context (expiry service) and normal
  EL2 mainline context (arm/cancel/rearm, consumer dispatch). Their mutual
  exclusion follows the P3 synchronization contract
  (`../../../p3/plans/p3-w06-concurrency-synchronization.md` — irq-save local
  locking discipline, declared lock order). The critical sections are O(1):
  state field updates, one record store, counter increments. No allocation,
  no loops over unbounded sets, no consumer callback inside the lock (D4).
- **Arm generation:** a saturating per-timer counter makes the
  fire/cancel/rearm interleavings decidable without a second lock
  ([04](04-code-contracts-deadline-timer.md) §3 pseudocode).
- **Expiry record handoff:** single-producer (expiry service), single-consumer
  (mainline dispatch), guarded by the arm generation and the
  record-present flag; the handoff never shares a mutable reference across
  contexts.
- **Register access:** all `CNTHP_*`/`CNTPCT_EL0` accesses are `unsafe`
  system-register operations confined to the Arch-domain module, each with a
  `SAFETY` justification and the barrier placement of
  [04](04-code-contracts-deadline-timer.md) §5. Nothing outside that module
  may contain these accesses.

## 6. Security and isolation model

W05 has no Guest-controlled inputs. Its trust obligations are: (a) platform
facts (frequency, INTID) are untrusted until intake validates them;
(b) consumer code is trusted Host code, but the record-handoff interface is
still typed and bounded so a misbehaving consumer cannot grow IRQ-context
work; (c) the timer must be disabled by default at bring-up and on every
failure path, so a failure never leaves an unsupervised deadline armed.

## 7. Telemetry surface (names fixed by this design)

Per-pCPU saturating counters and trace events, consumed later by P6-W13:

| Counter / event | Meaning |
|---|---|
| `arm_count`, `cancel_count`, `rearm_count` | lifecycle operation totals |
| `fire_count` | expiries stored as records |
| `cancelled_fire_count` | condition-cleared deliveries (race path) |
| `late_fire_count` | service-time counter already past the armed deadline by more than a declared threshold — measured, never corrected |
| `consumer_absent_count` | expiries recorded with no registered consumer (overwritten records) |
| `intake_failure_count` | intake rejections, with the failing check recorded |
| trace: `timer_armed`, `timer_expired`, `timer_cancelled` | per declared P0-W13 trace-namespace encoding; W05 fixes event identity, W13 owns collection |

The late-fire threshold is a declared constant of this design (mechanism
diagnostic, not a latency KPI); P6-W13 owns any latency measurement and
reporting built on top of it.
