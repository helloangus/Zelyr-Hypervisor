# P6-W05 Validation, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W05 design entry](README.md). This file closes the workflow in
[05-implementation-workflow.md](05-implementation-workflow.md).

## 1. Validation matrix

Every row defines planned evidence with an objective passing condition. None
claims a result. QEMU (virt, declared configuration) is the reference
environment for every runtime row; **QEMU timer emulation succeeding does not
prove real-hardware timer correctness** — real-hardware validation belongs to
the Orange Pi stage per the ADR and the P6 task book.

| ID | Requirement (plan/task book) | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W05-DV01 | Timer capability intake (P6-E) | intake review + exercise | run intake on the declared QEMU configuration; corrupt one input in a test harness to exercise each rejection path | checks pass on the reference platform; each rejection (frequency unavailable/mismatch, INTID missing, trap posture) produces its named diagnosis and leaves the timer disabled | the intake boundary rejects bad platform facts; not that every real platform's facts are correct |
| W05-DV02 | Frequency/time conversion boundary (P6-E) | host-side unit tests | conversion property tests against a reference table; overflow and horizon-boundary cases; zero-frequency rejection | all conversions checked, no naked u64 arithmetic at call sites, boundaries exact | conversion correctness; not hardware counter behavior |
| W05-DV03 | Per-pCPU monotonic-time ownership (P6-V08 basis) | code review + QEMU read check | review ordering discipline at every `clock_read()` call site; QEMU: repeated ordered reads never regress | every decision-grade read is ISB-ordered; observed reads monotonic on every exercised pCPU | the read discipline and observed monotonicity in the declared environment; not architectural monotonicity of all real counters |
| W05-DV04 | Arm/cancel/rearm behavior (P6-V07) | QEMU one-shot scenario | arm a future deadline, observe exactly one attributable expiry record with matching event id and generation | exactly one record per arm; attributable to the armed pCPU and event | one-shot semantics; not latency performance |
| W05-DV05 | Repeated events and bounded catch-up (P6-V08) | QEMU repeated scenario | anchored rearm over many expiries; overdue-consumer case with capped catch-up; AfterNow mode | repeated expiries remain stable; catch-up never exceeds the declared cap; catch_up_capped counted when triggered | defined repeated behavior; not tick policy or scheduling fairness |
| W05-DV06 | IRQ receipt integration and diagnostics (P6-V07/V08 basis) | QEMU integration with W03 + review | expiry through the W03 classified dispatch; forced race cases (cancel in flight); consumer-absent overwrite | expiry service classifications match [04] §3.3; CancelledFire classified safely by W03; counters reflect each outcome | the W03 integration contract; not device IRQ behavior or W03 correctness itself |
| W05-DV07 | Cancellation semantics (P6-V07 "controlled rearm/cancel result") | QEMU negative scenario | cancel-before-fire yields no record and Cancelled; cancel-after-fire yields AlreadyFired; stale-token cancel yields StaleGeneration | each named outcome observed exactly as specified; no lost or duplicated records | controlled cancel results; not absolute timing guarantees |
| W05-DV08 | Entry/exit monotonicity and multi-pCPU stability (P6-V08) | QEMU multi-pCPU scenario | sample `clock_read()` across Guest entry/exit windows (with the P4 Guest path) on multiple online pCPUs while repeated timers run; run repeat iterations | no observed regression across windows; repeated per-pCPU events stable across iterations; pCPU A's timer never attributes to pCPU B | observed stability in the declared environment and iterations; not wall-clock correctness or real-hardware timing |
| W05-DV09 | Diagnostics and telemetry surface (P6-M input) | review + counters observed in DV04–DV08 evidence | counters and trace events present and correlated with scenario outcomes (P6-W13 consumes) | arm/cancel/fire/late/consumer-absent/intake counters match scenario expectations | observability basis; not the latency baseline itself (P6-W13) |
| W05-DV10 | Mechanism-not-policy + handoff readiness (closure) | design-conformance review | review public surface against D5; handoff checklist §3 | no scheduling/tick/preemption semantics in W05's surface; consumers can act on the contracts without inventing policy | scope conformance; not downstream designs' correctness |

Planned, run, blocked, and failed are distinct evidence states; each is
recorded with command, input, environment, and timestamp in the verification
record. No W05 validation proves P6-V09–P6-V06 (other packages), scheduler
behavior, or real-hardware correctness, and none may be reported as doing so.

## 2. Error, security, and observability model

**Error model.** Named error cases and their guarantees:

- Intake failures (`TimerIntakeError` cases, [04](04-code-contracts-deadline-timer.md)
  §2.1): pCPU timer unusable, diagnosis recorded, no partial enablement.
- Conversion/arithmetic failures (`ConversionError`, `DeadlineOverflowError`):
  inputs unchanged, caller decides; overflow is never a wrap.
- Operation failures (`TimerError::NotIntakePassed`, `DeadlineInPast`,
  `StateConflict`, `ConsumerAlreadyBound`, `StaleGeneration` outcomes): the
  timer's state machine remains valid after every error.
- Impossible state: forced to Idle with registers disabled, counted, and
  surfaced as an invariant diagnostic; Guest activity can never cause it
  (W05 has no Guest-controlled inputs), so an impossible state is a Host
  invariant failure, not a Guest fault — the panic-policy distinction of the
  ADR applies.
- Failure ordering rule: on any failure path the timer is disabled before
  diagnostics are emitted ([04] §5), so no failure leaves an armed deadline.

**Security model.** W05 has no Guest-facing surface; its trust boundaries are
platform facts (validated at intake) and consumer code (bounded by the
single-consumer slot and the O(1) IRQ-context contract). The `unsafe` surface
is limited to the Arch-domain register accessors, each inventoried with a
`SAFETY` justification; the barrier table ([04] §5) is part of the audited
boundary.

**Observability model.** The counters and trace events of
[02](02-architecture-and-state.md) §7 are the designed telemetry surface;
P6-W13 owns collection, correlation, and any latency baseline built on them.
The late-fire counter is a mechanism diagnostic with a declared threshold —
it measures, it does not correct, and it makes no performance claim.

## 3. Handoff checklist

Before handing W05 to a reviewer, provide:

- the exact changed-file list and the crate/module placement chosen for the
  logical modules;
- the W05 `unsafe` inventory delta (register accessors) with `SAFETY`
  justifications, and confirmation no other `unsafe` was added;
- W05-DV01–DV10 evidence paths and run status, including explicit not-run
  entries (e.g. multi-pCPU rows blocked by an unmet P3/P4 prerequisite, or
  real-hardware rows — always not run in P6);
- the validated frequency, the intake INTID binding, and the declared
  constants (`MAX_ARM_HORIZON_TICKS`, late-fire threshold,
  `MAX_CATCHUP_STEPS`) as actually implemented;
- confirmation that no scheduler/tick/preemption policy, Guest-visible timer
  surface, wall-clock conversion, crate dependency, or board/QEMU constant
  was introduced;
- open items for consumers, without resolving their contracts here:
  - **P6-W06** (`../p6-w06-guest-generic-timer/README.md`): expiry-record
    mechanism and typed domains are ready; W06 owns virtual-domain
    conversion and Guest timer state;
  - **P6-W10 / P6-W11**: one-shot/repeated/cancel semantics and evidence
    paths are available for the timer-plus-vIRQ and Validation Guest
    scenarios;
  - **P6-W13**: telemetry counters and thresholds are in place; the latency
    baseline remains W13's deliverable;
  - **P7**: the mechanism and its limits are evidenced; every scheduling
    policy on top is P7-owned;
  - cross-design: the `CancelledFire` classification expectation placed on
    W03 ([04] §4) must be reconciled with W03's delivered lifecycle; if it
    cannot, that is an Architecture Change Request, not a W05 change.

No completion claim may be made anywhere in this design; completion evidence
belongs only in `../../verification/p6-w05-el2-generic-timer-verification.md`.
