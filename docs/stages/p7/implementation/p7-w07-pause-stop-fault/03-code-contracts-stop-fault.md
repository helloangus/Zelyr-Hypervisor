# P7-W07 Code Contracts — Stop and Fault Containment

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W07 detailed design](README.md).  
Scope: authorized stop, automatic guest-fault reaction, and structural
exclusion. Naming authority as stated in
[02-code-contracts-pause-resume.md](02-code-contracts-pause-resume.md).

---

## F-1. `request_vcpu_stop`

```text
Name and stability: request_vcpu_stop(target: VcpuHandleRef, caller:
                    AuthorityRef, cause: StopCause) -> StopOutcome; internal;
                    stable within P7.
Purpose and caller: authorized permanent exclusion of a vCPU from scheduling.
                    Callers: P5-dispatched control path. StopCause ∈
                    { ControlRequest, GuestTerminated } (a classified guest
                    stop/exit result per the P4-W04 stop path may arrive
                    through the exit path directly; see F-2).
Inputs / outputs: authority and handle semantics as P-1 of
                  [02-code-contracts-pause-resume.md](02-code-contracts-pause-resume.md).
                  StopOutcome = StopAccepted | StoppedAlready |
                  ControlledDenied(denial).
Preconditions: authorization (A-1) and handle validity (A-2) before any
               effect; control context.
Postconditions: acceptance on Running targets → stop-pending marker (release)
                then S-3 reconsideration; the exit path commits
                Running→Stopped via F-2's disposition machinery. Acceptance on
                Runnable → dequeue + Running-less transition to Stopped under
                L-1. Acceptance on Blocked → direct transition under L-1.
                Stopped/Faulted → StoppedAlready (idempotent, explicit).
State and ownership change: lifecycle transition to the terminal Stopped
                state via L-1; structural exclusion via L-4 thereafter;
                pending events retained for diagnostics only (parent README
                decision 5).
Concurrency/allocation context: bounded control path; no waiting on remote
                pCPUs; marker-then-reconsideration ordering as P-1.
Errors and failure guarantee: denials leave zero state; a terminal state can
                never be overwritten (L-1 rejects illegal transitions and the
                exit-path disposition gives stop/fault/pause a fixed priority
                — terminal > pause > continue).
Security/authorization checks: A-1 first; no Guest can stop another VM's
                vCPU without authorized capability (ADR-007/013).
Logic: authorize → dispatch per state → for Running: marker + S-3 → hooks
       (W09) with cause.
Validation: P7-V16 stopped-exclusion; P7-V15 remote-running rows.
```

## F-2. `commit_guest_fault`

```text
Name and stability: commit_guest_fault(entity: VcpuRef, fault:
                    GuestFaultReport) -> FaultCommit; internal; stable within
                    P7.
Purpose and caller: the automatic, unauthorized scheduling reaction to a
                    P4-W06-classified GuestFault. Called from the exit path
                    when the classification (F-0 boundary) identifies a
                    Guest-caused, VM-facing fault.
Inputs / outputs: VcpuRef of the faulting vCPU (the current vCPU of this
                  pCPU, per L-2); GuestFaultReport — the classified context
                  produced by the P4-W06 boundary (VM/vCPU identity, fault
                  class, address information it already validated).
                  FaultCommit = FaultedCommitted | DeferredToInvariantPath.
Preconditions: called only from the exit path with a valid F-0
               classification; Hypervisor-invariant failures MUST NOT enter
               this contract (they follow the P0-W14 panic path instead).
Postconditions: FaultedCommitted → Running→Faulted via L-1, structural
                exclusion via L-4, write set limited to this vCPU's scheduling
                state, its accounting record, and its diagnostic snapshot
                (containment, §7 of the architecture file); the pCPU proceeds
                to the scheduler decision for other work. Guest execution of
                this vCPU cannot resume in P7 (terminal).
State and ownership change: as above; no other VM/vCPU state is read or
                modified; no runqueue or placement mutation.
Concurrency/allocation context: pCPU exit context: bounded, allocation-free;
                one L-1 transition; diagnostic snapshot update is a fixed-size
                CPU-local write (P7-W09 surface).
Errors and failure guarantee: an L-1 rejection here means the vCPU was not
                Running (already terminal or paused) — the commit defers to
                the winning path's outcome rather than forcing Faulted; a
                classification that turns out to be a Hypervisor invariant
                failure is DeferredToInvariantPath → the P0-W14 fatal path,
                not a VM-facing event. The hypervisor never panics on the
                Guest-fault branch (P-0).
Security/authorization checks: deliberately NO capability check (parent
                README decision 6); conversely this path grants nothing — it
                only removes the faulting vCPU. The fault report originates
                from the trusted P4 classification boundary and is treated as
                hypervisor-internal; Guest-influenced values inside it were
                validated there.
Logic: verify classification class → L-1 transition → exclusion → W09 hooks
       → return disposition; never touches other entities.
Validation: P7-V16 (faulted vCPU excluded; other VM scheduling unaffected);
            P4-W06 negative-isolation reuse; P7-W11 stress.
```

## F-3. `enforce_terminal_exclusion`

```text
Name and stability: enforce_terminal_exclusion(entity: VcpuRef) -> bool;
                    internal; stable within P7; consumed at two structural
                    points: the admission path (with L-3) and the enqueue seam
                    call sites (with S-2 cooperation).
Purpose and caller: make Stopped/Faulted exclusion structural rather than
                    conventional — the last line of defense behind L-4.
Inputs / outputs: VcpuRef; bool admit.
Preconditions: called before Guest entry and before any enqueue commits.
Postconditions: Stopped/Faulted (and per L-4 any non-runnable state) → no
                entry, no enqueue; the scheduler decision moves on.
State and ownership change: none.
Concurrency/allocation context: hot path: bounded, allocation-free, acquire-
               loads only.
Errors and failure guarantee: none.
Security/authorization checks: none (mechanism check).
Logic: state check under admission serialization; refuse non-runnable.
Validation: P7-V16; property test — a terminal vCPU is never admitted or
            enqueued under any interleaving.
```

## F-4. Exit-path disposition priority

```text
Name and stability: resolve_exit_disposition(entity: VcpuRef, exit:
                    ExitClassification) -> ExitDisposition; internal; stable
                    within P7. ExitDisposition = ContinueScheduling |
                    BecamePaused | BecameStopped | BecameFaulted (extensible
                    only through this design's authority).
Purpose and caller: fix ONE evaluation order at the exit path so concurrent
                    pause/stop/fault/terminal requests cannot overwrite each
                    other: terminal states win, then authorized pause, then
                    continuation (with blocking handled by the P7-W06 block
                    path). Called by the exit path before any re-admission
                    decision.
Inputs / outputs: the vCPU that just stopped executing; the exit
                  classification (P-1 boundary of P7-W06's architecture file).
Preconditions: called exactly once per exit, on the owning pCPU.
Postconditions: exactly one disposition; at most one lifecycle transition per
                exit; accounting/trace hooks (W09) fire once with the final
                disposition and its cause.
State and ownership change: delegates to complete_pause_on_exit (P-2),
                F-1/F-2 commit paths, or the P7-W06 block path.
Concurrency/allocation context: exit context; bounded, allocation-free.
Errors and failure guarantee: unknown classification → treated as
                UnclassifiedExit per P7-W06's failure rules; never silently
                re-admitted.
Security/authorization checks: pause/stop dispositions consume already-
               authorized requests; fault disposition is automatic by design.
Logic: if F-0 fault class → F-2; else if stop marker → F-1 commit path; else
       if pause marker → P-2; else → blocking check (P7-W06 B-1) → continue.
Validation: P7-V15/V16; interleaving tests where pause, stop, fault, and
            block requests all target one vCPU within one exit.
```

## F-5. Accounting and diagnostics hooks (consumed by P7-W09)

```text
Name and stability: on_pause_committed(entity, cause), on_resume_committed(
                    entity, target_state), on_stop_committed(entity, cause),
                    on_fault_committed(entity, fault_class); internal hook
                    points reserved for [P7-W09](../p7-w09-accounting-diagnostics/README.md).
Purpose and caller: single authoritative observation points for control and
                    fault events; called exactly once per committed outcome by
                    P-2, P-6, F-1, F-2.
Preconditions / postconditions: bounded, allocation-free, must not alter the
               owning outcome; failures are accounting bugs surfaced per the
               P0-W14 boundary, never scheduling failures.
Security/authorization checks: none; the hooks record authority outcomes
               (granted/denied) — denials are recorded where the P5 dispatch
               path invokes its own telemetry, not here.
Logic: call-through.
Validation: P7-V19/V20 — one control event, one record; stop-running reason
            distinguishable from fault.
```
