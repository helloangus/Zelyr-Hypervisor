# P7-W08 Code Contracts — Designed Idle

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W08 detailed design](README.md).  
Scope: idle entry, wake sources, wait, and exit. Naming authority as stated in
[02-code-contracts-reschedule.md](02-code-contracts-reschedule.md).

---

## I-1. `enter_idle`

```text
Name and stability: enter_idle(pcpu: PcpuRef) -> IdleWaitOutcome; internal;
                    stable within P7; called by the scheduling loop
                    ([01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md) §3)
                    when selection finds no eligible work.
Purpose and caller: put the pCPU into the designed idle state without
                    busy-looping and without losing a concurrent request.
Inputs / outputs: PcpuRef (self). IdleWaitOutcome = Woke(IdleWakeSource) |
                  WorkArrivedDuringEntry.
Preconditions: called from the decision entry with no eligible local work;
               current-vCPU registration empty (L-3); IRQs of this pCPU may
               wake it.
Postconditions: on return the pCPU is out of the wait and the caller re-runs
                the decision entry. Either a wake source fired (recorded) or
                the two-phase entry observed late work (WorkArrived...). The
                pCPU never exits idle by timeout polling; a host deadline that
                folds a scheduler tick (W04) is a legitimate wake source, not
                a poll.
State and ownership change: per-pCPU idle state and deadline programming via
                PC-5; nothing shared is mutated.
Concurrency/allocation context: two-phase protocol mirroring P7-W06's block
                pairing: (1) set idle-intent (release); (2) re-check intent
                flag and enqueue-visible work (acquire); (3) if anything
                arrived, clear intent and return WorkArrived...; else wait via
                I-3. Allocation-free; no locks held across the wait.
Errors and failure guarantee: deadline-fold failure (P7-W06 B-4 degraded
                rule) → refuse idle: return immediately so the decision loop's
                bounded path handles it; never wait with unobservable
                deadlines.
Security/authorization checks: none (internal state).
Logic: mark → fold deadlines (PC-5, earliest of scheduler tick and folded
       vCPU deadlines) → re-check → wait → record wake source → return.
Validation: P7-V18 idle rows; P7-V25 request-vs-idle-entry race via P7-W11.
```

## I-2. Idle wake sources (closed list)

```text
Name and stability: enum IdleWakeSource { DeadlineFired, Reconsideration,
                    CrossCpuNotification }; internal; stable within P7.
Purpose and caller: the complete set of events that may terminate I-1's wait;
                    enforced by construction at I-3 and recorded at I-1's
                    return.
Members:
  DeadlineFired          — the folded host deadline (PC-5) expired: a
                           scheduler tick (W04) or a folded blocked-vCPU
                           deadline (P7-W06 B-4 deadline home).
  Reconsideration        — the R-2 handler ran for this pCPU: a remote wake,
                           remote pause/stop, or other reconsideration request
                           reached this pCPU.
  CrossCpuNotification   — the P3-W07 reception path delivered a non-
                           scheduler-class notification event that this
                           design's transport registration maps to the
                           scheduler (e.g. TLB-transport completion requiring
                           local action; the mapping is declared, not guessed).
Guarantee: any other exit cause is a spurious wake — counted, diagnosed, and
           still followed by a decision re-run (correctness never depends on
           expectation). Missing a listed source is a lost-wakeup defect.
State and ownership change: none (vocabulary).
Security/authorization checks: none.
Validation: P7-V18 — each source exercised individually; spurious-wake
            counting via P7-W09 hooks.
```

## I-3. `wait_for_event`

```text
Name and stability: wait_for_event(pcpu: PcpuRef) -> void; internal seam;
                    stable within P7; the only waiting primitive idle uses.
Purpose and caller: block the pCPU until an interrupt or event arrives,
                    implemented in the architecture layer (WFE-based) behind
                    the P1/P3 boundaries. Called only by I-1 after its two-
                    phase checks pass.
Inputs / outputs: self pCPU; no return value (the wake source is observed by
                  the interrupt handlers that run before I-1 resumes).
Preconditions: idle intent set; deadlines folded (PC-5); wake-source
               registrations in place (reconsideration handler, notification
               reception); caller has no lock held.
Postconditions: returns when some interrupt/event arrived on this pCPU; no
                spin, no timeout loop.
State and ownership change: none in Core; arch-layer register effects are
                that layer's audited boundary (volatile/ordered per the
                Coding Guidelines MMIO rules).
Concurrency/allocation context: interrupts enabled; strictly no polling
                loop around this seam in Core.
Errors and failure guarantee: an immediate-return implementation (e.g. a
                platform where WFE returns spuriously) is correctness-safe:
                I-1 records a spurious wake and re-decides. QEMU's WFE
                behavior differences never justify a Core-side poll.
Security/authorization checks: none.
Logic: single arch wait call.
Validation: P7-V18 non-busy check (idle pCPU consumes no scheduling
            iterations); P7-V17 isolation (idle pCPU does not disturb
            running pCPUs); real-hardware caveats recorded as not-proven by
            QEMU evidence.
```

## I-4. `exit_idle`

```text
Name and stability: exit_idle(pcpu: PcpuRef, source: IdleWakeSource) -> void;
                    internal; stable within P7.
Purpose and caller: clear the idle state and return control to the decision
                    entry; called by I-1's epilogue after the wait.
Inputs / outputs: self pCPU; the observed wake source (Spurious is a valid
                  diagnostic value).
Preconditions: called with idle intent still set (the wait ended).
Postconditions: idle intent cleared (release) before the decision entry runs;
                wake-source accounting recorded (W09 hook); the pCPU is
                scheduler-visible again as "not idle" before any enqueue
                target choice could consult idle state.
State and ownership change: per-pCPU idle state only.
Concurrency/allocation context: bounded, allocation-free; the ordering (clear
               intent before decision) prevents a requester from treating a
               just-woken pCPU as still idle.
Errors and failure guarantee: none.
Security/authorization checks: none.
Logic: record source (hook) → clear intent → return to caller (decision).
Validation: P7-V18 idle-to-work; ordering review that intent clearing
            precedes dispatch.
```

## I-5. Diagnostics hooks (consumed by P7-W09)

```text
Name and stability: on_idle_entered(pcpu, now), on_idle_exited(pcpu, source,
                    duration_basis, now), on_spurious_wake(pcpu, now),
                    on_idle_refused(pcpu, reason, now); internal hook points
                    reserved for [P7-W09](../p7-w09-accounting-diagnostics/README.md).
Purpose and caller: make idle behavior observable without performance claims;
                    called exactly once per transition by I-1/I-3/I-4.
Preconditions / postconditions: bounded, allocation-free, IRQ-safe; must not
               alter idle outcomes. duration_basis is the monotonic reading
               pair, not a formatted duration (encoding is W09/P0-W13's).
Security/authorization checks: none.
Logic: call-through.
Validation: P7-V18 (idle entered/exited with reasons), P7-V19/V20
            (coherence; spurious wakes counted).
```
