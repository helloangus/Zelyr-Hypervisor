# P7-W09 Code Contracts — Trace Events, Switch Reasons, Diagnostics

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W09 detailed design](README.md).  
Scope: trace-event set, switch-reason vocabulary, diagnostic snapshot and
failure report. Naming authority as stated in
[02-code-contracts-accounting.md](02-code-contracts-accounting.md).

---

## C-1. `SwitchReason` vocabulary

```text
Name and stability: enum SwitchReason { InitialDispatch, TimesliceExpiry,
                    PreemptedReconsideration, GuestBlocked, PausedControl,
                    StoppedControl, FaultedGuest, GuestHalted }; internal;
                    stable within P7; extended only through this design.
Purpose and caller: make every stop-running event distinguishable (P7-V20);
                    used by A-2, trace events, and diagnostics.
Mapping to producers:
  InitialDispatch           — first dispatch of a vCPU after Runnable.
  TimesliceExpiry           — deadline-driven extraction (W04 machinery).
  PreemptedReconsideration  — extraction after a reconsideration request
                              without a deadline expiry (policy result owned
                              by W05 selection; the mechanism reason is here).
  GuestBlocked              — WFI/WFE block commit (W06 B-1).
  PausedControl             — authorized pause commit (W07 P-2).
  StoppedControl            — authorized stop commit (W07 F-1/F-2 path).
  FaultedGuest              — guest-fault commit (W07 F-2).
  GuestHalted               — a classified guest stop/termination result
                              delivered by the P4-W04 stop path.
Guarantee: the vocabulary is mechanism-neutral (no priorities, no fairness
           classes; ADR-057/017 remain unprejudged); every disposition in
           F-4 maps to exactly one reason.
Security/authorization checks: none (enumeration).
Validation: P7-V20 distinguishability; exhaustiveness against F-4's
            ExitDisposition.
```

## C-2. Scheduler trace-event set

```text
Name and stability: the following event declarations, registered under the
                    P0-W13 scheduler domain naming and version rules (T-2);
                    final spelled names are fixed at implementation-time
                    registration, not invented ad hoc. Fields are semantic
                    content; encoding belongs to the governed telemetry
                    implementation.
Events and fields (identity fields omitted where obvious):
  vcpu_switch   { vm, vcpu, pcpu, out_reason?: SwitchReason, in_entity?,
                  monotonic_time }                     — from A-2
  vcpu_block    { vm, vcpu, pcpu, exit_hint, time }   — from A-3
  vcpu_wake     { vm, vcpu, source, outcome, requester_pcpu, home_pcpu, time }
  control_commit{ vm, vcpu, action: pause|resume|stop, disposition, time }
  guest_fault   { vm, vcpu, fault_class, time }       — class only; address
                    context stays in the P4-W06 diagnostic boundary (T-5)
  pcpu_idle     { pcpu, dir: enter|exit, source?: IdleWakeSource, time }
  pcpu_reconsider { target, result, iterations?, time }
  vm_pause_gen  { vm, generation, phase: requested|complete, time }
  diag_failure  { pcpu, vm?, vcpu?, class, time }     — the report marker (C-4)
Trimmability classes per P0-W12 (decision 8): vcpu_wake, pcpu_reconsider,
  vcpu_switch are trimmable high-frequency; vcpu_block, control_commit,
  guest_fault, pcpu_idle, vm_pause_gen, diag_failure are the retained
  minimum set.
Preconditions: registration through the P0-W13 governance review before first
               use; no event may ship as an unregistered string.
Postconditions: each emission associates the required context (identity,
               reason, monotonic time) so P7-V20's correlation holds.
Concurrency/allocation context: emissions from the hook paths are bounded and
               allocation-free under the P0-W12 channel contract (T-1); if the
               governed implementation cannot promise that in IRQ context,
               emissions defer to a safe point — the *record* update (A-x)
               never defers.
Errors and failure guarantee: a dropped or trimmed event is declared by the
               channel per P0-W12; records remain the source of truth, so
               counters never depend on trace delivery.
Security/authorization checks: no Guest-controlled bytes in any field
               (decision 7).
Validation: P7-V20; governance-compatibility review (workflow step 4).
```

## C-3. `SchedulerDiagnosticSnapshot`

```text
Name and stability: per-vCPU fixed-capacity recent-transition record
                    (capacity: a small reviewed constant, e.g. 8 entries);
                    entry = { SwitchReason or control/fault tag, pcpu,
                    monotonic_time }; plus per-pCPU last-idle/last-refusal
                    markers. Internal; stable within P7.
Purpose and caller: bounded memory of what recently happened to a vCPU, so a
                    failure report can show history without a trace buffer;
                    written by the same hooks that feed A-2..A-5.
Inputs / outputs: written at hook points; read by C-4's renderer.
Preconditions: none beyond record existence (A-1).
Postconditions: capacity-bounded ring; overwrite of the oldest entry; no
               allocation; never a crash-dump substitute.
State and ownership change: the vCPU's snapshot record only; written by the
               owning pCPU.
Concurrency/allocation context: writer-pCPU-only; bounded; IRQ-safe.
Errors and failure guarantee: none.
Security/authorization checks: entries are identifiers/enumerations only.
Validation: P7-V21 — recent transitions appear in failure reports in order;
            capacity behavior under long histories.
```

## C-4. `report_scheduler_failure`

```text
Name and stability: report_scheduler_failure(context: FailureContext) -> void;
                    internal; stable within P7; invoked by invariant-violation
                    sites (stalled intent, impossible state, invariant breach
                    surfaced per P0-W14) and by W11's invariant checks.
Purpose and caller: emit the P7-V21 diagnostic: current pCPU (hardware +
                    logical identity per T-4), VM/vCPU identifiers, lifecycle
                    state, placement eligibility summary, stop/disposition
                    reason, recent transitions (C-3), pending wake-event set,
                    pause/stop markers, intent/idle state, and reconsideration
                    counters — through the P0-W12 human channel with a
                    structured diag_failure event (C-2).
Inputs / outputs: FailureContext — the identifiers and enumerations the
                  caller already holds; nothing is re-derived from Guest
                  memory.
Preconditions: the calling path has already classified the situation as a
               scheduler failure or invariant breach; Guest-caused,
               VM-facing faults do NOT come here (they are normal contained
               events per W07 F-2 and appear only in records/trace).
Postconditions: one deterministic report: fixed field order, stable names,
               no timestamps formatting variance beyond the monotonic value;
               the hypervisor's subsequent behavior follows the P0-W14
               classification of the breach — this report never decides it.
State and ownership change: none (read-render).
Concurrency/allocation context: failure path: bounded, allocation-free
               rendering; no locks beyond the channel's own contract (T-1).
Errors and failure guarantee: the report must not fail; if the channel is
               unavailable, the renderer completes silently (records still
               hold the facts) — never a secondary panic.
Security/authorization checks: no Guest-controlled bytes (decision 7); no
               Host pointer disclosure; identifiers are project-typed values.
Logic: gather → order fields → emit human + structured.
Validation: P7-V21 — all required elements present and correctly ordered;
            "no crash-dump claim" review; safe-logging consistency with T-6.
```

## C-5. Governance registration review obligations

```text
Name and stability: a review obligation, not a function.
Content: before W09 code lands, the event set of C-2 is registered through
         the P0-W13 new-event review with: domain placement (scheduler),
         names, fields, version/compat behavior, and trimmability classes;
         and the C-4 report shape is checked against the P0-W12 minimum
         panic/failure information and version-identity association. The
         implementation record links the registration outcome.
Failure/blocker: if the P0-W13 governance mechanism is not yet implementable
         (T-2 unevidenced), the events are still declared here but emission
         is blocked; record the blocked prerequisite — do not emit ad-hoc
         strings as a workaround.
Validation: workflow step 4 review; P7-V20/V21 depend on it.
```
