# P6-W11 Guest Scenario Matrix

**Status:** Proposed detailed design; implementation and validation not
claimed.  
**Parent:** [P6-W11 detailed design](README.md).  
**Audience:** load before writing, changing, or running any scenario. The
asserted semantics are the W10 contract §7
(`../p6-w10-interrupt-semantics/01-interrupt-semantics-contract.md`); timer
ownership and deferred expiry are the W06 contract
(`../p6-w06-guest-generic-timer/README.md`); presentation and maintenance are
the W08/W09 contracts.

## 1. Temporary Guest interrupt layout (stage-local, non-frozen)

The scenarios use the following P6-temporary identifiers. They are owned by
this design under the task book §1 Reserved clause, exist only inside the P6
Validation Guest environment, and define no machine ABI, DTB binding, or P8
layout (see §4).

| Element | Temporary selection | Notes |
|---|---|---|
| Timer | the vCPU's architectural virtual generic timer (EL1 access), per the W06 contract | no virtual device timer is introduced |
| Test vIRQ identifiers | INTIDs 32–63 (SPI-class range) within the P6 virtual-IRQ namespace, injected by the Host per the W07/W08 contracts | values are temporary; scenarios reference them only through Guest-visible symbolic names resolved by the Host-side test configuration |
| Guest observation form | architectural acknowledge/EOI of the virtual CPU interface exactly as presented by the W08 contract, plus PSTATE masking | exact access form follows the reconciled specification and the W08 contract; no vGIC MMIO access |
| Scenario control | the P5-authorized Guest test/control path (HVC) for triggers that require Host cooperation | no new Guest-callable interface is added by W11 |

## 2. Scenario matrix

Legend: **Trigger** = how the scenario starts; **Expected** = the
Guest-observable the suite asserts; **Pass** = the objective pass condition;
**Repeat** = execution-count semantics (counts declared at run time and
recorded in evidence); **Inject** = failure/negative variation, if any;
**Evidence** = destination (all rows: the W11 verification record, which links
raw artifacts per the P0-W09 conventions).

### VG-TIMER-01 — Guest virtual timer programs and fires → P6-V09

- **Precondition:** W06 timer contract implemented and evidenced; Guest at
  EL1 with exception vectors installed.
- **Trigger:** Guest reads the virtual-timer counter, programs a near-future
  deadline, enables the timer.
- **Expected:** one virtual-timer exception delivered to the Guest's EL1
  vector; Guest handler records expiry, masks/disables the timer, and emits
  its result record.
- **Pass:** exactly the programmed expiry is observed with a handled
  exception return; counter read is monotonically consistent with the
  programmed deadline.
- **Proves / does not prove:** Guest-observable timer firing in QEMU; not
  Host timer accuracy on real hardware, not scheduler behavior.
- **Repeat:** N ≥ 3 expiries with distinct deadlines. **Inject:** program an
  already-past deadline — expiry must still be observed exactly once
  (documented behavior, not silently dropped).
- **Evidence:** Guest result record + Host timer telemetry correlation.

### VG-TIMER-02 — timer state survives HVC → P6-V10

- **Precondition:** VG-TIMER-01 passes; P5 HVC control path available.
- **Trigger:** program a deadline; make a controlled HVC call; return.
- **Expected:** timer state (counter relation, enabled status, pending
  expiry if due during the call) preserved per the W06 exit/entry contract;
  expiry observed at most once and not lost.
- **Pass:** no duplicate expiry, no lost expiry, no Host-state contamination
  observable in Guest counter values.
- **Proves / does not prove:** HVC-boundary preservation in QEMU; not
  arbitrary-Guest-exit coverage beyond the declared exit classes.
- **Repeat:** N ≥ 5 crossings. **Inject:** expire during the HVC — deferred
  presentation per W06 must be observed.
- **Evidence:** Guest result record + Host exit telemetry correlation.

### VG-TIMER-03 — timer state across controlled exit/re-entry → P6-V10

- **Precondition:** VG-TIMER-02 passes; controlled Guest exit/re-entry
  available per the P4/W06 contracts.
- **Trigger:** program a deadline beyond the exit window; request controlled
  exit; re-enter.
- **Expected:** timer behavior unchanged across the exit; expiry observed
  after re-entry per the W06 contract.
- **Pass:** same as VG-TIMER-02 for the exit/re-entry class.
- **Proves / does not prove:** exit-class preservation; not P7
  context-switch semantics (not exercised here).
- **Repeat:** N ≥ 5. **Inject:** none beyond VG-TIMER-02's.
- **Evidence:** Guest result record + Host exit telemetry correlation.

### VG-TIMER-04 — deferred expiry while the vCPU is absent → P6-V09/V10

- **Precondition:** W06 deferred-expiry contract implemented and evidenced.
- **Trigger:** program a short deadline; request the Host to keep the vCPU
  absent past expiry (declared test control); re-enter.
- **Expected:** the expiry event is presented at or after the next entry
  exactly once (W06 deferred-delivery contract).
- **Pass:** exactly-once deferred delivery; no loss; no cross-vCPU
  attribution (single-vCPU variant).
- **Proves / does not prove:** deferred-expiry eventual delivery in QEMU;
  not multi-vCPU isolation (VG-SMP-01) and not absence durations beyond the
  declared control.
- **Repeat:** N ≥ 3. **Inject:** expire twice while absent — Guest observes
  per the W06 coalescing/deferral record, and the observed count must match
  the W06 contract's documented policy.
- **Evidence:** Guest result record + Host timer telemetry correlation.

### VG-IRQ-01 — single vIRQ presented, acknowledged, completed → P6-V11

- **Precondition:** W07/W08 injection/presentation contracts implemented and
  evidenced; Guest vectors handle the interrupt class.
- **Trigger:** Host injects one authorized vIRQ to the Guest while it runs.
- **Expected:** exception at the EL1 vector; Guest acknowledges via the
  presented interface, observes the expected identifier, EOIs, emits record.
- **Pass:** exactly-once delivery of the intended event, acknowledged and
  completed; no other event observed.
- **Proves / does not prove:** end-to-end single-vIRQ presentation in QEMU;
  not Host SGI routing attribution (VG-SGI-01/W04) and not real-hardware
  latency.
- **Repeat:** N ≥ 10 across distinct test IDs in 32–63. **Inject:** none
  (negative input cases are W12's).
- **Evidence:** Guest result record + Host vIRQ lifecycle telemetry.

### VG-IRQ-02 — multiple distinct pending vIRQs → P6-V12

- **Precondition:** VG-IRQ-01 passes.
- **Trigger:** inject M distinct vIRQs (M > presentable slots when the
  W08 LR count permits) while masked or before the Guest acks.
- **Expected:** all M eventually presented and completed exactly once each;
  survival of deferred presentation (S-PD3).
- **Pass:** determinate completion for every distinct event; no loss, no
  duplication, no merge.
- **Proves / does not prove:** distinct-event independence under deferral;
  not unbounded M (M declared from the W07 capacity contract) and not
  ordering guarantees beyond the W10/W07 contracts.
- **Repeat:** N ≥ 3 rounds. **Inject:** none (over-capacity pressure is
  shared with W09-DV01; W11 records the Guest side only).
- **Evidence:** Guest result records (per-event) + Host telemetry.

### VG-IRQ-03 — mask / pending / unmask → P6-V13

- **Precondition:** W10 SEM-MASK semantics contract approved; L3 control
  available through the P5-authorized path; L1 always available.
- **Trigger:** mask (L1; plus L3 in a second variant), inject vIRQ, observe
  masked interval, unmask.
- **Expected:** no event during masking; event observable after unmask;
  exactly one completion (S-M1).
- **Pass:** pending preserved under mask; observable after unmask; no false
  completion.
- **Proves / does not prove:** the W10 masking rules in QEMU; not L2
  priority masking unless the W08 contract presents it (record unavailable
  otherwise); not hardware behavior.
- **Repeat:** N ≥ 5 per masking layer actually available. **Inject:** unmask
  with no pending event — Guest must observe nothing (no spurious event).
- **Evidence:** Guest result record + Host bookkeeping correlation.

### VG-IRQ-04 — repeated same-vIRQ arrival → P6-V14

- **Precondition:** VG-IRQ-01 passes; W10 SEM-REPEAT semantics approved.
- **Trigger:** inject the same vIRQ repeatedly: (a) while pending, (b) while
  presented/active.
- **Expected:** behavior follows the documented S-PD1/S-PD2 policy —
  collapsed re-pend at most once while active; no state corruption.
- **Pass:** final delivery count matches the documented policy exactly; no
  corruption indicators (state machine review + Guest observation).
- **Proves / does not prove:** the documented repeat policy; not that the
  policy equals hardware edge/level semantics in general.
- **Repeat:** N ≥ 5 repeats per phase. **Inject:** rapid repeat burst within
  one presentation — bounded by the same policy; any deviation is FAIL.
- **Evidence:** Guest result record + Host duplicate-completion counters.

### VG-IRQ-05 — concurrent timer plus vIRQ → P6-V15

- **Precondition:** VG-TIMER-01 and VG-IRQ-01 pass; W10 SEM-PRIO semantics
  approved.
- **Trigger:** make a band-A (timer) and a band-B (vIRQ) event ready at the
  same opportunity; also run the staggered variant (band-A pending behind a
  presented band-B).
- **Expected:** both classes progress in the entry; simultaneous case
  presents band A first (S-P1/S-P2); staggered case preserves the relation
  without loss.
- **Pass:** both events observed complete; the documented band relation
  holds in every round.
- **Proves / does not prove:** the documented priority relation; not full
  GIC priority/preemption coverage and not deterministic sub-band ordering.
- **Repeat:** N ≥ 10 rounds. **Inject:** none.
- **Evidence:** Guest result records with per-event order markers + Host
  telemetry.

### VG-IRQ-06 — List-Register pressure and maintenance progression → P6-V16, P6-V17

- **Precondition:** W08 presentation and W09 maintenance contracts
  implemented and evidenced.
- **Trigger:** inject more distinct pending vIRQs than presentable slots;
  Guest completes presented events progressively.
- **Expected:** no loss or overwrite under pressure (W09 I1); all events
  eventually presented and completed exactly once (W09 I2/I3); Guest observes
  continuous progression — every event completes within the declared bound.
- **Pass:** determinate completion of all injected events; no duplicate
  completion observable at the Guest.
- **Proves / does not prove:** maintenance-driven progression as
  Guest-observable in QEMU; not fairness, not maintenance timing on real
  hardware, not Linux vGIC behavior (W09 boundary).
- **Repeat:** N ≥ 3 pressure rounds. **Inject:** none (W12 owns induced
  fault cases; Guest-side negative variation is not in scope here).
- **Evidence:** Guest per-event completion records + Host maintenance
  telemetry (W09-DV02/DV03 cross-reference).

### VG-SGI-01 — Host-SGI-driven chain → P6-V11 (supports, does not replace, P6-V04–V06)

- **Precondition:** W04 Host SGI/routing contract implemented and evidenced;
  W07 injection path available.
- **Trigger:** Host cross-pCPU SGI triggers a vIRQ injection to the running
  Guest (declared test control path).
- **Expected:** Guest observes the injected event exactly once, end to end.
- **Pass:** exactly-once Guest-observable delivery correlated to the Host
  SGI trigger in Host telemetry.
- **Proves / does not prove:** the Guest-visible tail of the SGI chain in
  QEMU; **not** SGI target attribution, multi-target accounting, or routing
  changes — those remain W04's P6-V04–P6-V06 evidence.
- **Repeat:** N ≥ 5. **Inject:** none.
- **Evidence:** Guest result record + Host SGI and vIRQ telemetry.

### VG-SMP-01 — multi-vCPU timer isolation (conditional) → P6-V19

- **Precondition:** evidenced multi-vCPU contract
  ([P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md)) and P4 vCPU
  isolation confirmed in workflow step 1; otherwise BLOCKED mode.
- **Trigger:** two vCPUs program independent deadlines; Host exercises them
  concurrently within declared bounds.
- **Expected:** each vCPU observes only its own expiries with independent
  state.
- **Pass:** zero cross-vCPU expiry observation; independent completion
  counts.
- **Proves / does not prove:** multi-vCPU timer isolation in the declared
  QEMU configuration; not scheduling (P7), not real-hardware SMP behavior.
- **Repeat:** N ≥ 3. **Inject:** none.
- **Evidence:** per-vCPU Guest records + Host telemetry.

### VG-SMP-02 — multi-vCPU vIRQ isolation (conditional) → P6-V18

- **Precondition:** as VG-SMP-01; otherwise BLOCKED mode.
- **Trigger:** Host injects events directed to vCPU 0 only, while vCPU 1
  runs and records everything it observes.
- **Expected:** vCPU 1 observes none of vCPU 0's events and vice versa.
- **Pass:** zero cross-vCPU delivery; directed events complete on their
  target only.
- **Proves / does not prove:** vIRQ isolation across vCPUs in the declared
  QEMU configuration; not P8 Guest-SMP semantics and not hostile-target
  cases (W12 owns those).
- **Repeat:** N ≥ 3. **Inject:** none.
- **Evidence:** per-vCPU Guest records + Host telemetry.

### Exception-vector coverage (cross-cutting, prerequisite assertion)

Every scenario implicitly exercises the Guest's EL1 exception vectors; the
suite additionally records, for each run, that unexpected-vector entries did
not occur (an entry to an unexpected vector is a FAIL with the syndrome
recorded). This covers the plan's "EL1 exception-vector handling" scope
without a separate Host mechanism claim; deep vector diagnostic behavior
remains the P1/P4 contracts.

## 3. Result records and classification

Each scenario execution emits one structured Guest result record over the P4
Validation Guest debug/result channel (assumed contract; failure boundary in
[02-harness-and-workflow.md](02-harness-and-workflow.md) §1). Logical fields:

```text
scenario_id            declared ID (§2)
run_id                 unique per harness invocation
environment            declared environment class and configuration reference
started / ended        Guest-visible timestamps where available; ordering markers otherwise
result                 PASS | FAIL | BLOCKED | NOT-RUN
observed               scenario-specific values (expiry counts, IDs seen, completion counts, order markers)
expected_ref           pointer to the asserted contract rule (W10 §7 / W06 / W09 row)
host_correlation       Host run reference for telemetry correlation
notes                  FAIL/BLOCKED reason class; never free-form Host log dumps
```

Classification rules:

- **PASS** — every §2 Pass condition observed in the Guest record within the
  declared bounds, with Host correlation consistent.
- **FAIL** — a Pass condition contradicted (wrong count, cross-vCPU
  observation, duplicate, loss, unexpected vector entry, timeout). FAIL rows
  name the violated condition; they are evidence, not shame — never retried
  silently.
- **BLOCKED** — a prerequisite is absent or unevidenced (multi-vCPU
  contract, L2 unavailability, missing upstream evidence). Recorded as a
  stage block with the missing-prerequisite name; neither failure nor pass;
  required by P6-V18/P6-V19 wording.
- **NOT-RUN** — declared but not executed in this invocation, with the reason
  (ordering, environment limitation). Distinct from BLOCKED: the prerequisite
  exists; the run did not happen.

## 4. Non-freeze statement

The §1 layout, the scenario set, the record schema, and the repetition bounds
are P6-stage validation assets. They freeze no machine ABI, no Guest DTB
binding, no vCPU count requirement, and no P7/P8 test contract. P7/P8
regression users extend the maintained asset under their own approved designs
and may reshape suite internals without versioning.
