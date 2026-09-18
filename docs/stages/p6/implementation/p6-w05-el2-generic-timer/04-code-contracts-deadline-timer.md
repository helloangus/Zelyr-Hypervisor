# P6-W05 Code Contracts — Per-pCPU Deadline Timer and IRQ Receipt

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W05 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (decisions D1–D8),
[02](02-architecture-and-state.md) (ownership, state machine),
[03](03-code-contracts-time-core.md) (time types).

## 1. Data structure: `DeadlineTimer` (per pCPU)

```text
Name and stability: DeadlineTimer — one instance per online pCPU, created at
  per-pCPU bring-up. Internal to W05; consumers hold only a registration
  handle. Stable within P6.
Fields (all owner-pCPU-private; never shared across pCPUs):
  state:           Idle | Armed | Fired            (§4.1 of [02])
  deadline:        Option<TimerDeadline>
  event_id:        Option<ConsumerEventId>          (opaque consumer tag)
  arm_generation:  u32 (saturating)                 (race discriminator)
  expiry:          Option<ExpiryRecord>             (single-slot handoff)
  consumer:        Option<ConsumerSlot>
  diagnostics:     saturating counters per [02] §7
Purpose and caller: sole mutable authority for the pCPU's EL2 deadline
  behavior; called by bring-up (init/intake), consumers (arm/cancel/rearm),
  the W03 IRQ dispatch (expiry_service), and mainline dispatch
  (take_expiry_record).
Preconditions: constructed only after P3 per-pCPU lifecycle and W02 local
  readiness; timer registers left disabled at construction.
Postconditions: after init, state == Idle with registers disabled.
Concurrency: owner-pCPU-only (D7); IRQ vs. mainline mutual exclusion per
  the P3 irq-save lock discipline; critical sections O(1) (no allocation,
  no loops).
Errors: operation-level, listed per function below; a detected impossible
  state forces Idle with registers disabled and is counted (never a panic
  from IRQ context; escalation per [06] §2).
Validation: W05-DV04–DV09.
```

`ExpiryRecord`:

```text
Name and stability: ExpiryRecord — bounded, copy-able record:
  event_id: ConsumerEventId
  armed_deadline: TimerDeadline      (the deadline the record is for)
  service_counter: TimerCounter      (observed at expiry service)
  generation: u32                    (matching arm_generation)
  late_by: Option<TimerDelta>        (service_counter past deadline, when
                                      beyond the declared late threshold)
One slot per timer; overwritten (counted) if the consumer does not keep up.
```

## 2. Intake

### 2.1 `intake_and_init()`

```text
Name and stability: fn intake_and_init(pcpu: PcpuId) ->
  Result<(), TimerIntakeError> — called once per pCPU at bring-up.
Purpose and caller: validate the platform timer basis (D1/D6) and construct
  the pCPU's DeadlineTimer in the disabled state.
Inputs / outputs: pCPU identity -> readiness or a precise rejection.
Preconditions: P3 pCPU online; W02 local GIC readiness acknowledged;
  P1 timer-access baseline established (all assumed contracts,
  [01] §5).
Postconditions: on success, the timer exists, Idle, registers disabled,
  diagnostics zeroed; on failure, no timer is usable on that pCPU and the
  failing check is identifiable.
Concurrency: bring-up context; no concurrent users exist yet.
Errors and failure guarantee (named cases):
  FrequencyUnavailable   — CNTPCT/CNTFRQ access failed or frequency is zero
                           /implausible -> intake rejected.
  FrequencyMismatch      — this pCPU's frequency differs from the validated
                           system value -> that pCPU's timer blocked (stage
                           blocker recorded, other pCPUs unaffected).
  IntidNotProvided       — W02/W03 intake did not bind the EL2 timer PPI
                           (D6 violated) -> rejected; Core must not guess.
  TrapPostureUnsupported — the delivered P1 baseline does not leave the D1
                           register surface usable -> rejected with
                           diagnosis; D1 amendment required, not a local
                           workaround.
Security/authorization checks: platform facts are untrusted until checked.
Logic (pseudocode):
    freq_raw = read CNTFRQ_EL0                     // ordered read
    freq = TimerFrequency::from_intake(freq_raw)?
    if freq != validated_system_frequency() { return FrequencyMismatch }
    intid = timer_ppi_intid_from_intake()?         // W02/W03 data, not a
                                                   // Core constant
    verify trap posture per the P1 baseline record?
    construct DeadlineTimer { state: Idle, registers disabled }
Validation: W05-DV01.
```

## 3. Consumer operations

### 3.1 `arm()`

```text
Name and stability: fn arm(&mut self, deadline: TimerDeadline,
  event: ConsumerEventId) -> Result<ArmToken, TimerError>
Purpose and caller: program a one-shot expiry for the owning pCPU; called
  by consumers (W06 deferred-expiry assists; future P7 tick) in mainline
  context on the owning pCPU.
Inputs / outputs: absolute deadline + opaque event tag -> token carrying
  arm_generation (for later cancel/rearm correlation).
Preconditions: intake passed; state is Idle or Fired; deadline is a valid
  TimerDeadline (already checked arithmetic — arm performs no arithmetic).
Postconditions: on success, compare-value register == deadline, timer
  enabled, state == Armed, generation incremented, arm_count incremented.
Concurrency: mainline, owner pCPU, O(1); may be preempted by the expiry
  service between the register write and the state store — the generation
  discipline below makes that benign.
Errors: TimerError::NotIntakePassed; TimerError::DeadlineInPast (deadline
  <= an ordered current-counter read: legal to detect here, fires
  immediately if armed — callers choose); TimerError::StateConflict
  (impossible-state path).
Security/authorization checks: caller is trusted Host code; the consumer
  slot model (one consumer per pCPU timer) is the authorization boundary.
Logic (pseudocode; barrier placement per §5):
    acquire owner-pCPU timer lock (irq-save)
    disable_timer_registers()            // Enable=0 first: a re-arm never
                                         // inherits a live pending signal
    instruction_barrier()                // ISB: order the disable before
                                         // reprogramming ([03] §3.1 rule)
    write_compare_value(deadline)        // CNTHP_CVAL_EL2
    arm_generation += 1 (saturating)
    state = Armed; deadline = Some(deadline); event_id = Some(event)
    enable_timer()                       // CNTHP_CTL_EL2: Enable=1,
                                         // IMASK=0; ISTATUS ignored here
    arm_count += 1
    release lock
Validation: W05-DV04, DV07 (one-shot attribution).
```

### 3.2 `cancel()`

```text
Name and stability: fn cancel(&mut self, token: ArmToken) ->
  Result<CancelOutcome, TimerError>
Purpose and caller: withdraw an armed expiry; consumers in mainline context
  on the owning pCPU.
Inputs / outputs: token (generation correlation) -> outcome:
  Cancelled | AlreadyFired(record pending) | StaleGeneration.
Preconditions: intake passed.
Postconditions: on Cancelled, timer disabled, state Idle, deadline cleared,
  cancel_count incremented; on AlreadyFired, state Fired and the expiry
  record remains for dispatch; on StaleGeneration, nothing changed (the
  caller's view of the arm is outdated — counted diagnostic).
Concurrency: mainline, owner pCPU; race with expiry service resolved by the
  generation check inside the same irq-save lock as arm() — the lock makes
  arm/cancel/service mutually exclusive on the owning pCPU; the "in-flight
  PPI after disable" case is handled at service time (below), not by
  spinning.
Errors: TimerError::NotIntakePassed; impossible-state path as in [02] §4.1.
Security/authorization checks: as arm().
Logic (pseudocode):
    lock(irq-save)
    if token.generation != arm_generation { StaleGeneration; unlock; return }
    disable_timer()                      // deasserts level-signaled PPI
    instruction_barrier()
    if state == Armed { state = Idle; deadline = None; cancel_count += 1;
                        outcome = Cancelled }
    else if state == Fired { outcome = AlreadyFired }
    unlock
Validation: W05-DV07 (cancel-before-fire yields no expiry record).
```

### 3.3 `expiry_service()` — W03 dispatch entry (IRQ context)

```text
Name and stability: fn expiry_service(&mut self) -> ServiceClassification
  — called by the W03-classified timer PPI handler, in IRQ context, on the
  owning pCPU. This function is the entire W05 work performed in IRQ
  context; it is bounded and allocation-free (D4).
Purpose and caller: convert a classified timer PPI into an expiry record
  (or a classified non-expiry outcome) and leave completion responsibility
  with W03.
Inputs / outputs: none -> classification for W03 bookkeeping:
  Expired | CancelledFire | StaleFire | UnexpectedFire.
Preconditions: called only from the W03 handler bound to the intake INTID;
  postconditions: state is Fired (Expired) or Idle (other outcomes); the
  register condition for a stale/cancelled fire is cleared (Enable already
  0 from cancel, or explicitly disabled here for UnexpectedFire).
Concurrency: IRQ context; takes the same owner-pCPU irq-save lock; O(1).
Errors: none returned; impossible states are forced to Idle, counted, and
  diagnosed.
Security/authorization checks: none beyond the W03 classification contract
  (an unattributed PPI reaching this function is UnexpectedFire by
  definition and is diagnosed, never ignored).
Logic (pseudocode):
    lock(irq-save)
    observed = ordered counter read (ISB + CNTPCT_EL0)
    match state {
      Armed if arm_generation == current => {
          late = observed past deadline by > threshold ? Some(delta) : None
          if expiry.is_some() { consumer_absent_count += 1 }   // overwrite
          expiry = Some(ExpiryRecord { event_id, armed_deadline: deadline,
                                       service_counter: observed,
                                       generation, late_by: late })
          state = Fired; fire_count += 1;
          if late.is_some() { late_fire_count += 1 }
          classification = Expired
      }
      Armed => { // generation moved: a cancel/rearm interleaved before we
                 // ran; this fire is stale
                 state = Idle; disable_timer(); StaleFire (counted) }
      Idle  => { CancelledFire (counted: cancel won the race; W03
                 completes a condition-cleared delivery) }
      _     => { disable_timer(); state = Idle; UnexpectedFire (counted) }
    }
    unlock
Validation: W05-DV06 (attributability, race outcomes), W05-DV07.
```

### 3.4 `rearm()`

```text
Name and stability: fn rearm(&mut self, mode: RearmMode) ->
  Result<ArmToken, TimerError> — called by the consumer after observing an
  expiry record, in mainline context on the owning pCPU.
Purpose and caller: repeated-event behavior (D3). Modes:
  RearmMode::AfterNow(TimerDelta)      — next deadline = ordered
                                         current-counter read + delta;
  RearmMode::Anchored(TimerDelta)      — next deadline = previous armed
                                         deadline + delta, with bounded
                                         catch-up: at most
                                         MAX_CATCHUP_STEPS (declared
                                         constant) anchor steps are
                                         applied; if the anchor would
                                         still be in the past after the
                                         cap, it falls back to AfterNow
                                         and counts catch_up_capped.
Inputs / outputs: mode -> new token.
Preconditions: state is Fired or Idle; a previous deadline exists for
  Anchored; postconditions as arm().
Concurrency / errors / security: as arm().
Logic: deadline = per mode (checked arithmetic via deadline_after);
  then the arm() sequence.
Validation: W05-DV05, DV08 (stability, bounded catch-up).
```

### 3.5 `take_expiry_record()` and consumer registration

```text
Name and stability:
  fn take_expiry_record(&mut self) -> Option<ExpiryRecord>
  fn register_consumer(&mut self, slot: ConsumerSlot) -> Result<(), TimerError>
  fn clear_consumer(&mut self)
Purpose and caller: mainline dispatch (bounded): read-and-clear the single
  record; bind at most one consumer per pCPU timer.
Preconditions: owner pCPU; mainline context.
Postconditions: record cleared after read; register_consumer rejects a
  second consumer (ConsumerAlreadyBound) — one consumer per timer is a
  design invariant, not a registry.
Concurrency: irq-save lock; O(1).
Errors: ConsumerAlreadyBound; NotIntakePassed.
Validation: W05-DV06 (record handoff), reviewed against D4 (no callback in
  IRQ context — dispatch happens in mainline after take_expiry_record()).
```

## 4. W03 integration boundary

```text
Name and stability: timer PPI handler registration — the W03 classified-
  dispatch integration point. W05 supplies a handler for the intake INTID;
  W03 supplies classification, acknowledgement, and completion.
Boundary rules (assumed W03 contract, [01] §5):
  - the handler runs in the W03 IRQ context with bounded stack/context;
  - W05 never acknowledges or completes (EOI/deactivate) the PPI itself —
    it returns the ServiceClassification and W03 owns the lifecycle;
  - a CancelledFire must be classifiable by W03 as a safe condition-
    cleared delivery; if the W03 design cannot express it, that is a
    cross-design conflict to raise (Architecture Change Request against
    the W03 lifecycle), not a W05-local workaround.
Validation: W05-DV06; joint scenario with W03 evidence at integration.
```

## 5. Register sequence and barrier contract

All accesses are Arch-domain `unsafe` with `SAFETY` comments; exact
instruction spellings are verified against the architecture revision locked
at implementation (task book §8). Required ordering properties:

| Sequence point | Required ordering | Rationale |
|---|---|---|
| Disable (`CNTHP_CTL_EL2` Enable=0) before compare-value write | program order, then ISB before dependent decisions | a re-arm must not inherit a live pending condition; the ISB orders subsequent counter reads after the disable |
| Compare-value write before Enable=1 | program order (system registers) | arming with a stale compare value would fire immediately at the wrong time |
| Enable=1 before ERET into a Guest or before returning from bring-up | DSB SY | the armed state must be globally visible before Host execution continues into a context that assumes it |
| ISB before every `CNTPCT_EL0` read used for a decision | ISB | counter reads may be speculatively performed; decisions must use post-event values ([03] §3.1) |
| Disable on any failure path | immediately, before diagnostics | a failure must never leave an unsupervised deadline armed ([02] §6) |

Volatile/masking rules: control/status register writes construct the full
register value with reserved bits written as observed-at-read (read-modify-
write with reserved-bit preservation); no read of `CNTHP_TVAL_EL2` on the
arm path (D2); counter reads never infer timer state — `Fired` is entered
only via the PPI service.
