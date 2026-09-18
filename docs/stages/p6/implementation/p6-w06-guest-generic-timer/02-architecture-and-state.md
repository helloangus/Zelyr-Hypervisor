# P6-W06 Architecture, Ownership, and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W06 design entry](README.md).

## 1. Position in the P6 event chain

W06 turns Guest-programmed timer state into virtual-interrupt work and keeps
that state stable across Guest transitions:

```text
Guest (EL1) programs CNTV_*                      (direct, untrapped — D1)
        |
        v
condition holds -> physical INTID 30 PPI -> W03 classified dispatch
        |                                          |
        v                                          v
W06 PPI conversion (D6): inject vINTID 30 ----> W07 pending state
        |                                          |
        | single-outstanding: physical             v
        | INTID held, not deactivated      W08 List-Register presentation
        |                                          |
        v                                          v
exit: save/disable/evaluate; deferred      Guest completes -> W09 reports
injection if expired-unmasked              completion -> W06 releases the
                                           held physical interrupt
```

W06 owns condition evaluation, vCPU timer state, the transition sequences,
and the physical-PPI conversion policy. W07 owns pending/active software
semantics, W08 owns List-Register presentation, W09 owns maintenance
processing, W03 owns physical acknowledge/completion responsibility. The
dependency direction is explicit: W06 consumes W07's injection contract and
W03's dispatch; it never manipulates List Registers and never performs
Guest-visible masking policy beyond honoring the architectural mask bit.

Layer placement: `vcpu-timer-state` and `vtimer-expiry` are Core-domain
(architecture-neutral semantics, hostable); `vtimer-entry-exit` and
`vtimer-ppi-handler` are Arch-domain (system registers, barriers). No
Core-layer module may contain register access.

## 2. Logical modules

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `vcpu-timer-state` | Per-vCPU timer record: saved Guest registers (typed), shadow condition state, outstanding-physical flag, diagnostics | the per-vCPU record fields | entry/exit sequences; PPI handler; completion hook | condition snapshots for evaluation; save/restore values | Guest register values' semantics (evaluator's job); W07 pending state |
| `vtimer-expiry` | Pure evaluation: given saved/register control state and a typed counter observation, decide the architectural condition (expired, unmasked, enabled) | none (pure) | control value, deadline value, counter observation, offset | condition verdict + diagnostic classification | injection, register access |
| `vtimer-entry-exit` | Arch-domain entry sequence (offset write, restore) and exit sequence (save, disable, evaluate, request deferred injection, release physical) | none (operates on the vCPU record) | vCPU record; P4 transition points | restored/quiet hardware; exit-time evaluation result | the transition machinery itself (P4); scheduling of the vCPU (P7) |
| `vtimer-ppi-handler` | W03-dispatched handler for the intake-validated INTID 30: convert physical PPI to W07 event; hold/release the physical interrupt per D6 | per-vCPU outstanding-physical flag (in the record) | classified PPI dispatch; W09 completion report | W07 injection requests; release operations through the W03 physical lifecycle | classification/completion responsibility (W03); maintenance logic (W09) |

One `vcpu-timer-state` record exists per Guest vCPU from vCPU creation to
destruction (P4/P5 lifecycle owns the record's existence; W06 owns its
contents and initializes them at vCPU creation: timer disabled, zero
deadline, no outstanding physical interrupt).

## 3. Core objects and ownership (one owner per datum)

| Datum | Sole owner | Written by | Read by |
|---|---|---|---|
| Saved `CNTV_CTL`/`CNTV_CVAL` (typed) | the vCPU record (`vcpu-timer-state`) | exit save; entry restore (idempotent); vCPU creation init | evaluator; diagnostics |
| Shadow condition snapshot (last evaluated verdict) | the vCPU record | exit evaluation; PPI-handler conversion | deferred-delivery accounting; telemetry |
| `physical_outstanding` flag (one held INTID 30 max) | the vCPU record, per D6 | PPI handler (set); completion hook and exit release (clear) | PPI handler; exit sequence; diagnostics |
| W07 pending/active state for vINTID 30 | W07 exclusively — W06 only queries outcomes of inject/completion calls | W07 (via W06's inject requests and W09's completion reports) | W06 (via return values only) |
| List-Register residency | W08 exclusively | W08 | never W06 |
| Hardware `CNTV_*`/`CNTVOFF_EL2`/`CNTHCTL_EL2` registers | the running pCPU while the vCPU is loaded; written only by `vtimer-entry-exit` (and the Guest itself through the architecture, untrapped) | entry/exit sequences; Guest EL1 code | evaluator (ordered reads) |
| `CNTVOFF` value | the constant `0` for P6, carried explicitly in conversions (D2/D9) | intake/init | all conversions |

No W06 datum is stored per-pCPU across runs (D3): the only per-pCPU aspect is
that the loaded hardware registers physically sit on the pCPU currently
running the vCPU, and the entry/exit sequences move authoritative state to
and from the vCPU record at every transition.

## 4. Lifecycle and state machines

### 4.1 vCPU timer record lifecycle

```text
vCPU created -> Initialized (timer disabled, zero deadline, flag clear)
  entry -> Loaded (registers restored; offset written; Guest may reprogram)
  exit  -> Persisted (registers saved; hardware quiet; condition evaluated;
           deferred event queued if expired-unmasked; physical released)
  ... entries/exits repeat with no pCPU affinity of the record ...
vCPU destroyed -> Destroyed (W07 must hold no pending timer event for this
           vCPU at destruction: destroy path queries W07 and a pending
           timer event at destroy time is a counted anomaly on the destroy
           path, not silent loss)
```

### 4.2 Guest timer delivery state machine (per vCPU; W06's view)

States are W06-owned verdicts; W07's pending state is the delivery
authority and its transitions below are outcomes W06 observes, not state
W06 stores.

```text
                 +---------------------------------------------+
                 v                                             |
  [Quiet: enabled=0 or not expired or masked-expired]          |
     | Guest enables/expires unmasked (running)                 |
     | physical INTID 30 fires                                  |
     v                                                          |
  [Held: physical_outstanding=true; W07 inject requested] ------+
     |                                |                         |
     | W09 reports Guest completion   | exit while held         |
     v                                v                         |
  release physical -> re-arm cycle    disable timer (source off),
  (condition may re-fire: correct     then release physical safely,
  level semantics)                    flag clear -> [Quiet]
```

- **Quiet:** no held physical interrupt. Either the condition is false, the
  Guest masked it, or the timer is disabled. Nothing is owed to the Guest.
- **Held:** exactly one physical INTID 30 is outstanding (acknowledged,
  completion deferred). The software event for this delivery has been
  requested through W07 (new or coalesced — both count as held; repeated
  fires cannot occur while held because the held physical interrupt stays
  active until release, which is the storm-prevention property of D6).
- Exit while Held: save first (the Guest may have reprogrammed during
  handling), disable the timer (condition source off), then release the
  held physical interrupt and clear the flag.
- Impossible-state rule: `physical_outstanding` observed true at entry, or a
  second INTID 30 dispatch while Held, is an invariant violation — counted,
  diagnosed, recovered by releasing the held interrupt; never silently
  ignored ([06](06-validation-and-handoff.md) §2).

### 4.3 Deferred-expiry sub-cases (D4)

1. **Expired at exit (unmasked):** exit evaluation queues the event through
   W07 before the vCPU runs again; W08 presents it at the next entry.
2. **Deadline in the future at exit:** nothing is queued; the restored
   hardware expires naturally during the next run and flows through the
   Held cycle. Both sub-cases together realize "eventual delivery when an
   expiry occurs while the vCPU is absent".
3. **Expired and masked at exit:** nothing is queued (D7); restoration
   reproduces the masked condition at the next run.

## 5. Concurrency model

- **Transition exclusivity:** entry and exit sequences run in the P4 vCPU
  transition context on the pCPU hosting the transition; P4 guarantees no
  concurrent entry/exit for the same vCPU. The PPI handler for INTID 30 can
  run on the hosting pCPU while the vCPU is loaded (during Guest execution
  the handler runs in the W03 IRQ context) or on the hosting pCPU after an
  exit (a PPI already in flight). It is never invoked on a pCPU that is not
  the vCPU's current host in P6's static binding; if a future stage relaxes
  binding, this handler must be re-derived (Reserved note for P7).
- **IRQ vs. mainline:** the PPI handler mutates the record
  (`physical_outstanding`, inject request) inside the per-vCPU irq-save
  lock discipline delivered by P3 synchronization
  (`../../../p3/plans/p3-w06-concurrency-synchronization.md`); the exit
  sequence takes the same lock for its evaluation-and-release section.
  Critical sections are O(1): no loops, no allocation (D4 of W05 applies
  equally here — Coding Guidelines IRQ rule).
- **Completion hook (from W09):** runs in maintenance-processing context
  (W09's design bounds it); W06's hook does only the release operation and
  counter updates, O(1).
- **Record handoff across transitions:** the vCPU record is memory owned by
  the VM/vCPU objects (P4); W06 never caches pointers to it in per-pCPU
  storage across a transition (D3).
- **W07 interaction:** inject and query calls are W07's locked operations;
  W06 performs no direct bit manipulation of W07 state (single-owner rule,
  §3).

## 6. Security model (untrusted Guest inputs)

- All Guest-controlled values that W06 stores (control register fields,
  deadline) are read from hardware registers, not from Guest memory, and are
  still treated as untrusted: the evaluator validates control-field encoding
  and uses checked arithmetic on typed deadlines; a Guest deadline in the
  far past/future is a legal value handled by the condition logic, never a
  Host fault.
- Trapped EL1 physical-timer access is Guest-caused: one controlled Guest
  fault via the P5 error boundary, VM-local, counted; no emulation, no Host
  state change, no panic.
- The Guest cannot cause: unbounded Host work (all paths O(1)), pending-
  state growth (W07 dedupes; W06 requests at most one outstanding event
  class), or cross-vCPU effects (state is per-vCPU; INTID is validated at
  intake; targets are never Guest-chosen in P6).
- An exit storm caused by Guest timer programming (rapid enable/disable) is
  bounded by the same O(1) per-transition work; pathological rates are
  W12's robustness concern and must only degrade diagnosably.

## 7. Telemetry surface (names fixed by this design)

| Counter (per vCPU unless noted) | Meaning |
|---|---|
| `timer_entry_restore_count` / `timer_exit_save_count` | transition activity |
| `timer_condition_met_exit_count` | expiries found at exit evaluation |
| `timer_deferred_inject_count` | events queued while absent (sub-case 1) |
| `timer_ppi_convert_count` | physical INTID 30 conversions while running |
| `timer_coalesced_count` | inject attempts W07 answered as already pending |
| `timer_completion_release_count` | physical releases on Guest completion |
| `timer_trapped_access_count` | trapped EL1 physical-timer accesses (Guest faults) |
| `timer_unexpected_ppi_count`, `timer_impossible_state_count` | invariant diagnostics (Host-attributed) |
| trace: `vtimer_deferred`, `vtimer_converted`, `vtimer_released` | per the P0-W13 trace namespace; W13 owns collection |

These feed P6-W13's factual records and P6-V24 correlation; W06 fixes
identity, not collection or presentation.
