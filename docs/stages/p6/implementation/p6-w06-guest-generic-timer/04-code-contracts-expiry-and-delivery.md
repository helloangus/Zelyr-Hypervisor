# P6-W06 Code Contracts — Expiry Evaluation, Deferred Delivery, PPI Conversion

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W06 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (D1–D9),
[02](02-architecture-and-state.md) (state machine, ownership),
[03](03-code-contracts-vcpu-timer-state.md) (typed domains, sequences).

## 1. Event identity intake

### 1.1 `TIMER_EVENT_INTID` resolution

```text
Name and stability: TIMER_EVENT_INTID — an intake-resolved value (not a
  Core constant): the INTID W06 uses for the Guest virtual-timer event,
  expected to be the architecture-assigned EL1 virtual-timer PPI INTID
  (PPI 14 -> INTID 30). Internal to W06; stable within P6.
Purpose and caller: fixes the identity used for W07 injection and for the
  W03 PPI-dispatch binding; consumed by [2] and [3].
Preconditions: intake validates (a) the value against the W01-reconciled
  platform capability data and the W02/W03 INTID intake, and (b) that the
  P4 Validation Guest's interrupt layout exposes the same number for its
  virtual timer (identity mapping, D5).
Postconditions: resolution happens once per system bring-up; failure
  leaves Guest timer delivery disabled with a precise diagnosis.
Concurrency: bring-up only.
Errors and failure guarantee: IntakeMismatch -> Guest timer delivery
  disabled (attempts produce a diagnosable disabled-feature outcome, not
  silent misrouting); Guest timer state save/restore continues (harmless).
Security/authorization checks: platform + Validation Guest layout are
  untrusted inputs until cross-checked (identity mapping is a P6
  assumption that must be evidenced, not assumed silently).
Logic: fetch both sources, compare, record the resolved value and its
  evidence paths in the implementation record.
Validation: W06-DV01.
```

## 2. Pure condition evaluation

### 2.1 `evaluate()`

```text
Name and stability: fn evaluate(ctl: GuestTimerCtl,
  cval: Option<VirtDeadline>, now: VirtCounter) -> ConditionVerdict
  where ConditionVerdict is one of:
    Disabled            (enable == 0)
    MaskedExpired       (condition met, imask == 1)
    Pending             (condition not yet met, enabled, unmasked)
    ExpiredUnmasked     (enabled, unmasked, met)
  Pure, Core-domain, hostable. Stable within P6.
Purpose and caller: the single definition of the architectural condition
  used by exit evaluation, PPI conversion confirmation, and tests.
Preconditions: ctl is a validated image ([03] §1.2).
Postconditions: verdict is total (every input maps to exactly one
  variant); `now` is only compared when enabled and a deadline exists.
Concurrency/allocation context: none (pure, O(1)).
Errors: none.
Security/authorization checks: guest-controlled inputs; evaluation is
  range-safe by construction (typed comparisons, no arithmetic).
Logic (pseudocode):
    if !ctl.enable { return Disabled }
    let Some(cval) = cval else { return Pending }   // never programmed
    let met = now >= cval                            // typed compare
    match (met, ctl.imask) {
      (true, true)  => MaskedExpired,
      (true, false) => ExpiredUnmasked,
      (false, _)    => Pending,
    }
Validation: host-side unit tests over the full verdict matrix (W06-DV02);
  this function is the reference for DV02/DV04 expectations.
```

Note the deliberate mapping to D7: `MaskedExpired` never produces an
injection; the restored hardware reproduces the state at the next run.

## 3. Deferred delivery (absent-vCPU expiry, D4)

### 3.1 `request_deferred_timer_event()`

```text
Name and stability: fn request_deferred_timer_event(vcpu) -> InjectOutcome
  — thin, typed wrapper over the W07 injection contract, called from the
  exit sequence ([03] §3.2) and available to future assist paths only
  through a design amendment ([03] §4).
Purpose and caller: queue the timer event for a currently-absent vCPU so
  presentation occurs at or before its next entry.
Inputs / outputs: target vCPU -> W07 outcome (Accepted | Coalesced |
  Rejected{reason}).
Preconditions: the vCPU is absent or transitioning (exit exclusivity
  held); TIMER_EVENT_INTID resolved; W07 lifecycle live.
Postconditions: on Accepted/Coalesced, W07 holds the pending event (W07
  owns dedupe and counts); on Rejected, nothing queued and the reason is
  counted and diagnosable (e.g. vCPU destroyed — a destroy race, not a
  Host fault).
Concurrency: exit context; W07's synchronization applies beyond this
  call; O(1), no allocation in exit path.
Errors: Rejected cases are named and counted; W06 never retries in a
  loop.
Security/authorization checks: the source is Host mechanism code (W06),
  not a Guest request, so no capability check applies; the Guest cannot
  influence the target (P6 injects only to the record's own vCPU).
Logic: delegate to W07 inject(vcpu, TIMER_EVENT_INTID, priority = default
  priority carried by W06, source = TimerExpiry); map outcome to
  counters (`timer_deferred_inject_count`, `timer_coalesced_count`).
Validation: W06-DV04 (expired-at-exit sub-case), W06-DV06.
```

## 4. Physical-PPI conversion (D6)

### 4.1 `vtimer_ppi_handler()` — W03 dispatch entry

```text
Name and stability: fn vtimer_ppi_handler(vcpu, record) ->
  PpiServiceClassification — bound at intake to TIMER_EVENT_INTID through
  the W03 classified dispatch; runs in IRQ context on the vCPU's hosting
  pCPU. Arch-domain wrapper around Core decisions. Stable within P6.
Purpose and caller: convert a fired Guest virtual-timer PPI into the W07
  software event while honoring the single-outstanding rule.
Inputs / outputs: the loaded (or just-exited) vCPU's record ->
  classification for W03 bookkeeping (Converted | Coalesced |
  Unexpected | Inconsistent).
Preconditions: called only via W03 dispatch for the intake INTID; record
  lock discipline applies; bounded O(1) work, no allocation (Coding
  Guidelines IRQ rule).
Postconditions:
  Converted/Coalesced: `physical_outstanding == true`; exactly one
  physical INTID 30 held; W07 has the event (new or coalesced); the
  physical interrupt is NOT deactivated here (held until completion hook
  or exit release).
  Unexpected: fired while the vCPU is absent or the timer was silenced —
  W06 requests release through the physical lifecycle operation, counts,
  and does not inject (an injection would have no consumer).
  Inconsistent: dispatch while already held — invariant violation path:
  count `timer_impossible_state_count`, request release, do not inject;
  recovery leaves the machine consistent ([02] §4.2).
Concurrency: takes the record irq-save lock; interacts with W07 inside it
  (W07's own locking applies); no loops.
Errors: none returned; classifications + counters carry the result.
Security/authorization checks: none beyond dispatch binding (Host
  mechanism path).
Logic (pseudocode):
    lock(record)
    now = virt_counter_read()          // ISB + CNTVCT
    verdict = evaluate(live-ctl-image, live-cval, now)
    match (verdict, record.physical_outstanding) {
      (ExpiredUnmasked, false) => {
          outcome = virq_inject(vcpu, TIMER_EVENT_INTID, TimerExpiry)
          record.physical_outstanding = true     // hold, do not deactivate
          count convert/coalesce per outcome; classification = Converted
      }
      (_, true) => { count impossible; request release; classification =
                     Inconsistent }            // do not double-hold
      (MaskedExpired|Pending, false) => {      // raced with Guest fix or
          request release; classification = Unexpected } // mask change
      (Disabled, false) => { request release; classification = Unexpected }
    }
    unlock(record)
Validation: W06-DV04/DV06; storm-behavior expectations inform W12 but are
  evidenced there.
```

The hold-then-release split is the storm-prevention core of D6: a
level-signaled PPI that is deactivated while its condition persists
re-pends immediately; holding it active until the Guest completes the
software event (or exit silences the condition source) makes re-delivery
exact rather than repetitive.

### 4.2 `on_guest_completion()` — W09 hook

```text
Name and stability: fn on_guest_completion(vcpu, record, vintid) -> ()
  — called by W09's maintenance processing when the Guest visibly
  completes the timer event (assumed W09 contract, [01] §5). Stable
  within P6.
Purpose and caller: release the held physical interrupt so a persisting
  condition re-fires with correct level semantics (D6).
Preconditions: `vintid == TIMER_EVENT_INTID` (other INTIDs are ignored —
  this hook is W06's, reached through W09's completion correlation);
  bounded O(1), maintenance context.
Postconditions: `physical_outstanding == false`; physical interrupt
  released through the physical-lifecycle operation; if the Guest
  meanwhile reprogrammed the timer so the condition is false, the release
  is quiet (no re-fire) — both cases are correct.
Concurrency: record irq-save lock; no allocation.
Errors: none; an absent hold on a completion report is counted as a
  diagnostic anomaly (not fatal; W09's report may be racing exit release).
Security/authorization checks: none (Host-internal correlation).
Logic: lock; if flag set: release, clear, count; unlock.
Validation: W06-DV06 (release-on-completion), DV04 (level re-fire case).
```

## 5. Trapped Guest access (Guest-caused error path)

### 5.1 `handle_trapped_timer_access()`

```text
Name and stability: fn handle_trapped_timer_access(exception_context) ->
  GuestFaultDisposition — called from the EL2 synchronous-exception path
  (P1 entry baseline) when the syndrome identifies an EL1 physical-timer
  register access trapped per D1. Stable within P6.
Purpose and caller: answer trapped access with one controlled, VM-local
  Guest fault through the P5 error boundary — no emulation, no Host state
  change beyond counters.
Inputs / outputs: captured context -> disposition (advance past the
  instruction with a fault delivered to the vCPU per the P5/P4 fault
  mechanism).
Preconditions: syndrome classification performed by the P1/P4 exception
  path; the vCPU context is the faulting one.
Postconditions: the Guest observes a defined fault (or defined
  disabled-access behavior per the P5 boundary); Host state unchanged;
  `timer_trapped_access_count` incremented.
Concurrency: VM-exit context; O(1); no allocation.
Errors: none; a second-level anomaly (fault delivery itself failing) is a
  P4/P5 boundary concern, escalated per their contracts.
Security/authorization checks: the Guest chose to make the access; the
  response is containment, so no untrusted value is parsed beyond the
  syndrome the P1 path already classified.
Logic: count; build the Guest-fault disposition per the P5 error
  taxonomy; return it to the exception path. EL1 virtual-timer accesses
  must never reach this function (they are untrapped, D1); if one does,
  that is a trap-posture intake failure — count and raise the P1 boundary
  conflict.
Validation: W06-DV05.
```

## 6. Intake checks (W06-local)

`vtimer_intake()` runs at bring-up (after P1/W01/W02 inputs exist): verifies
the D1 trap posture against the P1 baseline record (virtual-timer access
untrapped, physical-timer access trapped); resolves TIMER_EVENT_INTID
([1.1]); verifies identity mapping against the Validation Guest layout;
records all outcomes. Failure disables Guest timer **delivery** with
diagnosis while save/restore remains safe. Validation: W06-DV01.
