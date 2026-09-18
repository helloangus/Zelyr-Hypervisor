# P7-W07 Code Contracts — Pause and Resume

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W07 detailed design](README.md).  
Scope: pause/resume and VM pause completion. Stop/fault contracts are in
[03-code-contracts-stop-fault.md](03-code-contracts-stop-fault.md). Names are
stage-local design freedom owned by this design (rationale: parent README
decisions 1–4); reconcile with the W02 module contract at implementation time
without changing W02-owned semantics.

---

## P-1. `request_vcpu_pause`

```text
Name and stability: request_vcpu_pause(target: VcpuHandleRef, caller: AuthorityRef)
                    -> PauseOutcome; internal to the scheduler control module;
                    stable within P7.
Purpose and caller: authorized pause of one vCPU from any lifecycle state.
                    Callers: the P5-dispatched control path (hypercall or
                    internal control plane) and request_vm_pause (P-3).
Inputs / outputs: target — P5-validated object reference for a vCPU (A-2);
                  caller — capability/authority reference (A-1).
                  PauseOutcome = PauseAccepted | PausedAlready |
                  ControlledDenied(denial) where denial ∈
                  { NoAuthority, WrongState, InvalidTarget } (classes per
                  P5-W05's controlled denial list).
Preconditions: caller context is a control context (never an IRQ handler);
               target handle resolved through the P5 object table.
Postconditions: NoAuthority/InvalidTarget → zero state change (authorization
                precedes every effect). PausedAlready → zero state change,
                explicit success. PauseAccepted → the vCPU is, or determinably
                becomes, Paused per the §3 decision table of
                [01-pause-stop-fault-architecture.md](01-pause-stop-fault-architecture.md);
                for Running targets the marker precedes reconsideration (S-3).
State and ownership change: pause-pending marker and (for non-Running states)
                the lifecycle transition via L-1; for Blocked targets, pending
                events retained (W6-1).
Concurrency/allocation context: bounded, allocation-free control path; takes
                the lifecycle lock inside L-1 only; no waiting on other pCPUs;
                reconsideration request is fire-and-record (S-3).
Errors and failure guarantee: denial classes carry no partial state; a failed
                authorization or wrong-state target leaves everything
                unchanged; reconsideration loss is impossible because the
                marker is stored (release) before the request is issued.
Security/authorization checks: A-1 rights check FIRST (check-then-act);
                A-2 handle validity; no role/VM-ID fallback (ADR-013/051);
                a Guest can pause only vCPUs its capabilities allow.
Logic: authorize → resolve state under L-1 → dispatch per decision table →
       record accounting/trace hook (P7-W07 handoff to W09).
Validation: P7-V15 pause matrix rows for every state; P5 denial reuse cases.
```

## P-2. `complete_pause_on_exit`

```text
Name and stability: complete_pause_on_exit(entity: VcpuRef) -> ExitDisposition;
                    internal; stable within P7.
Purpose and caller: consume the pause-pending marker at the scheduler exit
                    path — the defined point where a Running vCPU stops
                    executing. Called by the exit path (L-3 seam) before any
                    re-admission decision, for every vCPU that stops running.
Inputs / outputs: VcpuRef (the vCPU that just stopped executing, still
                  registered current on this pCPU). ExitDisposition =
                  ContinueScheduling | BecamePaused.
Preconditions: called only from the exit path with L-2 guaranteeing this is
               the vCPU's owning pCPU.
Postconditions: marker set → pre-pause context recorded (§4 of the
                architecture file), Running→Paused committed via L-1, current
                registration cleared, VM-pause completion detector updated
                (P-4); otherwise → no change.
State and ownership change: as above; no allocation.
Concurrency/allocation context: pCPU exit context: bounded, allocation-free,
                no cross-pCPU waits.
Errors and failure guarantee: L-1 rejection (a stop/fault raced in) → defer
                to that terminal path's outcome; pause is then moot and must
                not overwrite a terminal state.
Security/authorization checks: none — this consumes an already-authorized
                request; the marker cannot be set without P-1 authorization.
Logic: check marker → record context → transition → update completion
       detector → disposition.
Validation: P7-V15 running-pause rows; ordering review that marker
            consumption precedes re-admission.
```

## P-3. `request_vm_pause`

```text
Name and stability: request_vm_pause(vm: VmHandleRef, caller: AuthorityRef)
                    -> VmPauseOutcome; internal; stable within P7.
Purpose and caller: authorized pause of every member vCPU of a VM with the
                    §5 completion model. Callers: P5-dispatched control path.
Inputs / outputs: VmHandleRef (A-2); VmPauseOutcome = PausePending(generation)
                  | ControlledDenied(denial) | VmEmpty (a VM with no member
                  vCPUs completes trivially).
Preconditions: control context; VM object live under the P4/P5 lifecycle.
Postconditions: denial → zero change. Acceptance → generation set (release),
                every member dispatched per P-1, completion now detectable.
                Acceptance is NOT completion; see P-4.
State and ownership change: VM pause-pending generation; per-member effects
                via P-1.
Concurrency/allocation context: iteration is bounded by the member count
                (fixed at VM creation — P7 has no dynamic VM/vCPU creation,
                per task book Reserved list); no waiting.
Errors and failure guarantee: a member returning ControlledDenied (e.g.
                already Stopped) does not fail the VM pause; it is reported
                in the outcome's per-member summary so callers see the exact
                terminal composition.
Security/authorization checks: VM-level control right via A-1; the caller's
                authority over the VM is checked once, not re-derived per
                member.
Logic: authorize → set generation → iterate members via P-1 → return.
Validation: P7-V15 "VM pause leaves no Guest executing" case.
```

## P-4. `detect_vm_pause_completion`

```text
Name and stability: detect_vm_pause_completion(vm: VmRef) -> CompletionState;
                    internal; stable within P7.
Purpose and caller: evaluate the §5 completion predicate. Callers: the
                    scheduler exit path of the last departing member (primary),
                    and control-plane state queries (P5 result surface) from
                    non-VM-exit contexts.
Inputs / outputs: VmRef; CompletionState = Complete | Pending(running_members).
Preconditions: caller either owns a departing member's exit context or is a
               control-plane reader; predicate evaluation is read-only.
Postconditions: Complete → vm.observed_pause_generation ==
                vm.pause_generation; no member Running anywhere, verified
                under the lifecycle lock after the generation was set.
                Pending → names the still-running members for diagnostics.
State and ownership change: the observed-generation field on Complete.
Concurrency/allocation context: bounded member scan, allocation-free,
                lifecycle lock for the Running check only.
Errors and failure guarantee: cannot fail; Pending is a normal state.
Security/authorization checks: none (internal predicate); P5 decides what
               callers may observe it.
Logic: load generation (acquire); for each member verify not Running under
       L-1 serialization; on success record observed generation.
Validation: P7-V15; property test — after Complete, no admission admits any
            member until resume (P-5/P-6).
```

## P-5. `admission_pause_check`

```text
Name and stability: admission_pause_check(vm: VmRef) -> Admit | RefusePause;
                    internal; stable within P7; consumed at the L-3 pre-entry
                    point.
Purpose and caller: enforce the generation before every Guest entry so a VM
                    pause stops new execution immediately. Called by the
                    admission path for every candidate dispatch.
Inputs / outputs: the candidate's VM; Admit | RefusePause.
Preconditions: called under the admission serialization of L-3.
Postconditions: RefusePause → the candidate is not entered; the scheduler
                decision continues with other entities.
State and ownership change: none.
Concurrency/allocation context: bounded, allocation-free, on the hot entry
               path — single acquire-load of the generation; must be cheap.
Errors and failure guarantee: none.
Security/authorization checks: none (mechanism check, not authority).
Logic: compare member VM's pause generation against its observed value.
Validation: P7-V15; P7-W11 race stress (pause during entry).
```

## P-6. `request_vcpu_resume` / `request_vm_resume`

```text
Name and stability: request_vcpu_resume(target: VcpuHandleRef,
                    caller: AuthorityRef) -> ResumeOutcome; and
                    request_vm_resume(vm: VmHandleRef, caller: AuthorityRef)
                    -> VmResumeOutcome; internal; stable within P7.
Purpose and caller: authorized resume restoring eligibility per §4 of the
                    architecture file. Callers: P5-dispatched control path.
Inputs / outputs: as P-1/P-3. ResumeOutcome = ResumedToRunnable |
                  ResumedToBlocked | ControlledDenied(denial).
Preconditions: authorization first; target is Paused for vCPU resume.
Postconditions: ResumedToRunnable → Paused→Runnable (L-1), enqueued via S-2
                under C-1 placement, reconsideration via S-3 for the eligible
                pCPU set. ResumedToBlocked → Paused→Blocked (L-1) with the
                retained event record intact; W06 owns any later wake.
                WrongState (including Stopped/Faulted — terminal per parent
                README decision 5) → explicit denial, zero change.
State and ownership change: lifecycle transition; runqueue membership via
                S-2; nothing else. Placement and events were preserved, not
                recomputed.
Concurrency/allocation context: bounded control path; no waiting.
Errors and failure guarantee: denial classes leave zero state; an S-2 enqueue
                failure is an invariant violation surfaced per the P0-W14
                boundary (never silently dropped).
Security/authorization checks: A-1 rights; A-2 handle validity; no bypass.
Logic: authorize → verify Paused → evaluate eligible events and pre-pause
       condition → transition → enqueue → hooks.
Validation: P7-V15 resume rows (eligibility/placement/events preserved);
           blocked-case resume lands in Blocked without a WFI round-trip.
```
