# P7-W02 Lifecycle and Admission Code Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W02 detailed design](README.md).

Contracts follow the implementation-design checklist §3 template. All names
are internal, stage-local design names (see the parent README's excluded-
interfaces rule): mechanical renaming is allowed; semantic change is a new
design decision. Pseudocode is design logic, not runnable production code;
allocation, locking, and IRQ rules follow
[architecture and state](02-architecture-and-state.md) §5.

## §1 Types

### Contract 1.1 — `VcpuRunState`

```text
Name and stability: enum VcpuRunState { Offline, Runnable, Running, Blocked,
Paused, Stopped, Faulted }; internal to the lifecycle module; not public API.
Purpose and caller: the single authoritative representation of where a vCPU
  sits in the ADR §4.1 lifecycle; read by every scheduler consumer, written
  only by the engine (Contract 1.4).
Inputs / outputs: n/a (type). Derives equality and copy semantics; carries no
  payload (payload belongs to auxiliary, owner-declared state — W06/W07).
Preconditions / postconditions: value is always one of the seven variants;
  no other encoding (no booleans, sentinels, or optionals standing in for a
  state) is permitted anywhere in P7.
State and ownership change: stored inside the vCPU lifecycle cell; owner is
  the lifecycle engine.
Concurrency/allocation context: plain data; reads without the cell lock are
  permitted only for audit snapshots that tolerate staleness and re-verify
  under the lock (see Contract 1.10).
Errors and failure guarantee: n/a.
Security/authorization checks: n/a; carrying Guest-influenced data in this
  type is prohibited.
Logic: none (data).
Validation: exhaustive-match and no-other-encoding checks in host tests
  (W02-DV03).
```

### Contract 1.2 — `LifecycleEvent`

```text
Name and stability: enum LifecycleEvent { MakeRunnable, Dispatch, Deschedule,
  Block, Wake, Pause, Resume, Stop, Fault }; internal.
Purpose and caller: names the requested lifecycle change. Producers by
  contract: MakeRunnable — configuration/admission path (W03 consumers);
  Dispatch — admission gate; Deschedule — preemption/deschedule paths (W04,
  W05); Block — block path (W06); Wake — wakeup path (W06, from P6 events);
  Pause/Resume/Stop — pause/stop paths (W07) carrying authority context;
  Fault — fault path (P4 exit + P5 classification).
Inputs / outputs: n/a. `Pause`, `Resume`, `Stop` are "control events" and
  require a populated TransitionContext (Contract 1.5).
Preconditions / postconditions: legality per the Contract 1.3 table.
State and ownership change: n/a.
Concurrency/allocation context: plain data.
Errors and failure guarantee: n/a.
Security/authorization checks: control events without context are rejected
  by the engine, never defaulted.
Logic: none (data).
Validation: producer-path reviews assert each producer emits only its
  contracted events (W02-DV02).
```

### Contract 1.3 — Transition legality table (`successor`)

```text
Name and stability: fn successor(state: VcpuRunState, event: LifecycleEvent)
  -> Option<VcpuRunState>; pure; internal; also used host-side for table
  tests and by the audit checker.
Purpose and caller: the total legality function; the engine applies it, the
  checker re-derives it, host tests enumerate it.
Inputs / outputs: (state, event) -> Some(next) iff legal, None otherwise.
Preconditions / postconditions: total over the 7x9 input space; deterministic.
State and ownership change: none (pure).
Concurrency/allocation context: none; must be allocation-free and branch-
  light (it runs under the cell lock and in IRQ context).
Errors and failure guarantee: None is data, not an error.
Security/authorization checks: none (legality is authority-independent; the
  authority check is separate, Contract 1.4).
Logic:

  table (rows = event, columns = from):
               Offline   Runnable  Running   Blocked   Paused  Stopped/Faulted
  MakeRunnable Runnable  None      None      None      None    None
  Dispatch     None      Running   None      None      None    None
  Deschedule   None      None      Runnable  None      None    None
  Block        None      None      Blocked   None      None    None
  Wake         None      None      None      Runnable  None    None
  Pause        None      Paused    Paused*   Paused    None    None
  Resume       None      None      None      None      Runnable None
  Stop         Stopped   Stopped   Stopped*  Stopped   Stopped None
  Fault        None      None      Faulted   None      None    None

  * Pause/Stop from Running: the caller must already have brought the vCPU
    out of Guest execution on its owning pCPU (pCPU-owner sequencing, W07);
    the engine cannot verify this and asserts it via the context (below).

Validation: exhaustive host-side table test enumerating all 63 pairs against
  this matrix (W02-DV03); this test is the P7-V03 reference.
```

### Contract 1.4 — `try_transition` (engine)

```text
Name and stability: fn try_transition(cell: &VcpuLifecycleCell,
  event: LifecycleEvent, ctx: &TransitionContext)
  -> Result<VcpuRunState, TransitionError>; internal.
Purpose and caller: the only writer of run state. Called by admission gate
  (Dispatch), deschedule paths (W04/W05), block/wakeup paths (W06), pause/
  stop paths (W07), fault path, configuration path (MakeRunnable).
Inputs / outputs: cell + event + context -> new state or typed error.
Preconditions: caller does NOT hold the cell lock (engine takes it); for
  control events, ctx carries caller role + capability-receipt reference
  produced upstream by the P5 rights check; for Pause/Stop from Running,
  ctx carries the pCPU-owner attestation (Contract 1.6).
Postconditions: on Ok, cell state == returned value and a transition record
  (from, event, to, ctx role, timestamp source) replaced the previous one;
  on Err, cell unchanged except last-rejection bookkeeping; a trace-
  semantics event is marked for emission in both cases.
State and ownership change: exactly the cell interior; nothing else.
Concurrency/allocation context: takes the cell leaf-lock; critical section
  is O(table lookup); no allocation; not for IRQ context (use 1.7).
Errors and failure guarantee: Illegal { from, event } (table says None);
  ControlNotAuthorized (control event, absent/invalid context);
  InvalidSequencing (Running-source control event without owner
  attestation). Failure guarantee: rejected request never mutates run state
  and never panics.
Security/authorization checks: context presence and role validity only;
  actual capability verification stays in P5 upstream (ADR-013 layering).
Logic:

  lock(cell)
  from = cell.state
  guard control_event(event) implies ctx.has_authority() else
      reject(ControlNotAuthorized)
  guard (from == Running and event in {Pause, Stop}) implies
      ctx.has_owner_attestation() else reject(InvalidSequencing)
  match successor(from, event):
      None      -> record_rejection(cell, from, event); reject(Illegal)
      Some(to)  -> cell.state = to
                   cell.last = TransitionRecord { from, event, to, ctx }
                   mark_trace(TransitionAccepted { vcpu, from, event, to })
                   release; return Ok(to)
Validation: host unit tests per error case; property test "any interleaving
  of engine calls leaves the cell in a table-reachable state" (W02-DV03/04).
```

### Contract 1.5 — `TransitionContext`

```text
Name and stability: struct TransitionContext { role: CallerRole,
  authority: Option<CapabilityReceiptRef>, owner_attestation: Option<PcpuId> };
  internal.
Purpose and caller: carries who is asking and on what authority; built by
  each producer path, consumed by the engine.
Inputs / outputs: CallerRole in { ConfigPath, SchedulerCore, BlockPath,
  WakeupPath, PausePath, StopPath, FaultPath, AuditPath }.
Preconditions / postconditions: control events require authority =
  Some(receipt) where receipt was produced by the P5 rights check upstream;
  the engine never dereferences the receipt (it records the reference).
State and ownership change: none.
Concurrency/allocation context: plain data, stack-borne; no allocation to
  build in IRQ context (receipt refs are handles, not copies).
Errors and failure guarantee: n/a.
Security/authorization checks: this type is the seam that keeps P5
  capability authority upstream of the policy-free engine; forging a role
  from Guest-reachable code is structurally impossible because no Guest-
  input path constructs a context (asserted in review, W02-DV02).
Logic: none (data).
Validation: review; negative tests that control events without context are
  rejected (W02-DV03).
```

### Contract 1.6 — pCPU-owner attestation

```text
Name and stability: field owner_attestation: Option<PcpuId> in
  TransitionContext, set only by code executing on that pCPU's scheduling
  context (the per-pCPU control loop or its direct callee).
Purpose and caller: makes "the vCPU is out of Guest execution before
  Pause/Stop is applied from Running" checkable: only the owning pCPU's
  path can truthfully set it.
Inputs / outputs: the PcpuId of the executing pCPU, cross-checked by the
  engine against the gate's bookkeeping (pcpu.current_vcpu == this vcpu).
Preconditions / postconditions: engine accepts Running->Pause/Stop only
  when attestation matches the pCPU that currently owns the vCPU.
Concurrency/allocation context: n/a.
Errors and failure guarantee: mismatch -> InvalidSequencing; state unchanged.
Security/authorization checks: prevents a remote CPU from declaring a
  running vCPU paused; remote flows (W07/W08) must request, and the owner
  applies.
Logic: comparison inside the engine critical section.
Validation: host test simulating a mismatched remote request (W02-DV03).
```

### Contract 1.7 — `try_transition_from_irq`

```text
Name and stability: fn try_transition_from_irq(cell: &VcpuLifecycleCell,
  event: LifecycleEvent) -> Result<VcpuRunState, TransitionError>; internal;
  callable from IRQ/timer callback context (P6 events).
Purpose and caller: bounded engine entry for IRQ-raised events (in P7,
  effectively Wake-class events sourced by P6; W06 owns the surrounding
  wakeup protocol).
Inputs / outputs: as 1.4 minus context (IRQ paths raise no control events).
Preconditions: event is non-control and legal from IRQ (table rows
  Wake, Deschedule-mark only); caller is in IRQ context on the pCPU that
  owns the event source.
Postconditions: as 1.4; additionally, any follow-up work (queue mutation,
  reschedule kick) is NOT done here — the contract returns and the bounded
  post-IRQ step (W06) performs it.
State and ownership change: cell interior only.
Concurrency/allocation context: no allocation; O(1); no cross-CPU spin;
  uses the IRQ-safe locking discipline from the P3-W06 baseline.
Errors and failure guarantee: same rejections as 1.4; never panics.
Security/authorization checks: control events are structurally rejected
  (no context parameter exists).
Logic: as 1.4, minus context handling, plus early-out for control events.
Validation: host tests with simulated IRQ interleavings (W02-DV03/04);
  QEMU: timer-raised wake leads to a legal state (P7-V02/V03 support).
```

## §2 Gate and exit boundary

### Contract 2.1 — `admit_for_entry` (admission gate)

```text
Name and stability: fn admit_for_entry(pcpu: PcpuId, vcpu: VcpuRef,
  eligibility: impl Fn(PcpuId) -> bool) -> Result<EntryPermit,
  AdmissionError>; internal to the admission module; called by the per-pCPU
  control loop (W05) immediately before invoking the P4 entry mechanism.
Purpose and caller: the single normal-entry boundary. Enforces: scheduler
  active; placement eligibility (predicate supplied by W03's resolved
  placement); dispatch transition; sole current-vCPU slot.
Inputs / outputs: pCPU, vCPU reference, W03 eligibility predicate ->
  EntryPermit (proof the gate passed; consumed by the entry call) or error.
Preconditions: caller executes on `pcpu`'s scheduling context with IRQs in
  the state the P3 baseline prescribes for scheduler critical sections;
  scheduler mode is Active (else BypassRequired error — see 2.3);
  `eligibility` is derived from the vCPU's frozen resolved placement, not
  from policy state.
Postconditions: on Ok — vcpu state == Running (Dispatch accepted);
  pcpu.current_vcpu == Some(vcpu); no other pCPU's current slot changed.
  On Err — nothing changed (state re-verified as Runnable and left so).
State and ownership change: vCPU cell (via engine), pCPU current slot (this
  module, sole writer).
Concurrency/allocation context: gate sequence runs with IRQ masking per P3
  rules, holds no queue lock (the caller popped the candidate and released
  the queue before gating); O(1) checks; no allocation.
Errors and failure guarantee: NotActive (scheduler mode Inactive);
  NotDispatchable (state != Runnable at gate time — lost race, caller
  requeues); PlacementIneligible (predicate false); SlotOccupied (the pCPU
  still holds a current vCPU — invariant-violation-grade, also recorded via
  the audit path). Failure guarantee: all-or-nothing; a failed gate leaves
  both cell and slot unchanged.
Security/authorization checks: eligibility predicate encapsulates the W03
  placement validation; the gate takes no Guest-influenced input.
Logic:

  assert scheduler_mode == Active else return Err(NotActive)
  assert pcpu.current_vcpu.is_none() else return Err(SlotOccupied)  # audit-grade
  if !is_dispatchable(vcpu.state) { return Err(NotDispatchable) }
  if !eligibility(pcpu) { return Err(PlacementIneligible) }
  match try_transition(cell, Dispatch, ctx(SchedulerCore)):
      Ok(Running) -> pcpu.current_vcpu = Some(vcpu)
                     mark_trace(GateResult { pcpu, vcpu, Admitted })
                     return Ok(EntryPermit)
      Err(e)      -> mark_trace(GateResult { pcpu, vcpu, Rejected(e) })
                     return Err(e.into())
Validation: host tests for each rejection and the success interleaving
  (W02-DV04); QEMU P7-V02 (entry occurs only through the gate).
```

### Contract 2.2 — `complete_exit` (exit boundary)

```text
Name and stability: fn complete_exit(pcpu: PcpuId, vcpu: VcpuRef,
  outcome: ExitOutcome) -> PostExitControl; internal; called by the exit
  path after the P4 mechanism has regained EL2 and saved Guest context,
  before any scheduling decision.
Purpose and caller: the single normal-return boundary; guarantees no exit
  path re-enters the Guest directly and that the pCPU slot is released
  consistently with the vCPU's next state.
Inputs / outputs: ExitOutcome in { Descheduling(DescheduleReason),
  Blocking, FaultDetected(FaultClassified), ControlRequested(Pause|Stop) }
  — produced by the exit-classification seam (P4 boundary + W06/W07
  consumers) — -> PostExitControl in { ReconsiderCurrent, Yielded }
  telling the control loop whether the same vCPU may be re-picked.
Preconditions: pcpu.current_vcpu == Some(vcpu); vcpu state == Running;
  P4 exit already saved arch context (asserted, not performed, here).
Postconditions: on Descheduling — state == Runnable, slot cleared;
  on Blocking — state == Blocked, slot cleared; on FaultDetected — state ==
  Faulted, slot cleared; on ControlRequested — the control event was
  applied (Pause/Stop paths from W07 completed the transition); slot
  cleared in all cases. deadline bookkeeping cancellation is the caller's
  next step (W04 contract), not this function's.
State and ownership change: cell (via engine) + pCPU slot (this module).
Concurrency/allocation context: pCPU-local, IRQ-masked per P3 rules; O(1);
  no allocation.
Errors and failure guarantee: precondition mismatch is an invariant
  violation (fatal per P0 policy), not a recoverable error — this function
  has no Err branch by design; wrong-slot state indicates memory-safety or
  sequencing corruption upstream.
Security/authorization checks: ControlRequested carries the W07-obtained
  authority inside its payload and is applied through the engine's control-
  event checks.
Logic:

  assert pcpu.current_vcpu == Some(vcpu)          # fatal otherwise
  assert vcpu.state == Running                    # fatal otherwise
  match outcome:
      Descheduling(r) -> try_transition(cell, Deschedule, ctx(SchedulerCore))
      Blocking        -> try_transition(cell, Block, ctx(BlockPath))
      FaultDetected(c)-> try_transition(cell, Fault, ctx(FaultPath))
      ControlRequested(k) -> apply_control_via_owner(k)   # engine call per W07 flow
  pcpu.current_vcpu = None
  mark_trace(ExitCompleted { pcpu, vcpu, outcome })
  return if vcpu.state == Runnable { ReconsiderCurrent } else { Yielded }
Validation: host state-machine tests per outcome (W02-DV03/04); QEMU P7-V02
  (returns after exit reach the scheduler), P7-V09 support (slot/state
  agreement across switches).
```

### Contract 2.3 — `activate_scheduler` and the bypass register

```text
Name and stability: fn activate_scheduler() -> Result<(), ActivationError>;
  SchedulerMode { Inactive, Active } with fn scheduler_mode() -> SchedulerMode;
  static BYPASS_PATHS: reviewable table of { id, class: BypassClass, scope }
  where BypassClass in { EarlyBoot(E1), BringUp(E2), EmergencyDebug(E3) }.
Purpose and caller: activation is called once by the boot sequence
  (P1/P3 integration point) when every online pCPU is ready to run the
  control loop. The bypass register statically enumerates every code path
  that performs Guest entry or lifecycle mutation outside the gate, each
  tagged with its W01 E1–E3 class.
Inputs / outputs: none -> activation result.
Preconditions: all online pCPUs (P3 registry) have their scheduler state
  initialized; E2-registered bring-up paths have completed or been retired.
Postconditions: mode == Active for the rest of execution (no deactivation
  contract exists; E3 remains by definition outside normal scheduling);
  E1/E2 bypass instances become illegal — any use afterwards is an
  invariant violation recorded through the audit/fatal path.
State and ownership change: the one-time mode flag.
Concurrency/allocation context: called at boot, before Guest work exists;
  cross-CPU visibility of the flag must use release/acquire semantics per
  the P3 atomic-ordering baseline.
Errors and failure guarantee: AlreadyActive (double activation); 
  PcpusNotReady (registry disagreement) — both leave mode unchanged.
Security/authorization checks: n/a (boot-authority path).
Logic: registry consistency check; store mode with release; mark_trace(
  SchedulerActivated).
Validation: review that BYPASS_PATHS covers exactly the W01 §E classes
  (W02-DV02); QEMU: post-activation Guest entry only via gate (P7-V02).
```

## §3 Audit

### Contract 3.1 — `check_scheduler_invariants`

```text
Name and stability: fn check_scheduler_invariants(snapshot: &SchedulerSnapshot)
  -> InvariantReport; pure; internal; runnable host-side (property tests)
  and as a QEMU assertion hook at safe sampling points.
Purpose and caller: the mechanical form of the plan's "checkable"
  invariants; called by host tests (continuous), QEMU hooks (sampled),
  W11 stress harness, and reviewers.
Inputs / outputs: SchedulerSnapshot { per-vcpu (id, state), per-pcpu
  current_vcpu, queue membership summaries } -> InvariantReport with one
  verdict + evidence per invariant.
Preconditions / postconditions: snapshot consistency (taken under the
  documented sampling discipline: cells read with staleness tolerance,
  re-verified under lock on suspicion — audit never mutates).
State and ownership change: none.
Concurrency/allocation context: no locks taken; allocation allowed (host/
  diagnostic context only — never called from IRQ context).
Errors and failure guarantee: findings are data; converting a finding into
  a fatal condition is the caller's policy decision (P0 panic policy for
  genuine invariant violations, report-only for audit sampling races that
  re-verify clean).
Security/authorization checks: n/a.
Logic:

  INV-1 no-double-run: count over pcpus of (current == v) <= 1 for all v
  INV-2 agreement: v.state == Running  <=>  exactly one pcpu has
        current == v   (both directions; Offline..Paused imply no slot)
  INV-3 non-runnable exclusion: GateResult/EntryPermit records reference
        only vCPUs whose state was Running at entry; snapshot flags any
        Guest-entry evidence for state != Running
  INV-4 engine-only mutation: cell.last records form a chain rooted at
        Offline with legal edges (table re-derivation)
  INV-5 monotonicity: for each vCPU, cumulative dispatch count never
        decreases across snapshot sequence; rejections never decrement
        anything (measurement rendering is W09's; the invariant here is
        that audit-visible counters do not regress)
  report each invariant: Pass | Fail(evidence)
Validation: host property tests generate random legal event sequences and
  assert all-Pass (W02-DV04); QEMU sampled checks (P7-V04).
```

## §4 Supporting data contracts

```text
EntryPermit: zero-payload proof type returned by the gate; realized in Rust
  as a borrow/marker tying the entry call to a successful gate result. The
  realization must make "enter without a permit" unrepresentable where the
  type system allows; where it cannot (assembly boundary), the entry call
  asserts it. Stability: internal.

ExitOutcome / DescheduleReason: ExitOutcome per Contract 2.2;
  DescheduleReason { SliceExpired, Voluntary, RescheduleRequest } —
  mechanism-side semantics fixed here; W09 owns rendering; W04/W05 produce.

TransitionRecord { from, event, to, role, stamp }: per-cell audit trail of
  the last transition only (full history is trace's job, W09).

AdmissionError { NotActive, NotDispatchable, PlacementIneligible,
  SlotOccupied, BypassRequired }: maps onto the P7 error classes
  (InvalidInput-class control errors; SlotOccupied escalates to audit).
```

All contracts above are subject to the W02 constraint set: no policy
parameters, no Guest-influenced input, no allocation or long work in IRQ
context, leaf-lock discipline, and engine-only state mutation.
