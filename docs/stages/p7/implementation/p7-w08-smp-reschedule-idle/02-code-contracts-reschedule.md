# P7-W08 Code Contracts — Reconsideration Requests

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W08 detailed design](README.md).  
Scope: the reconsideration request protocol and handler. Idle contracts are
in [03-code-contracts-idle.md](03-code-contracts-idle.md). Names are
stage-local design freedom owned by this design (rationale: parent README
decisions 1–3); reconcile with sibling module contracts at implementation
time without changing their semantics.

---

## R-1. `request_reconsideration`

```text
Name and stability: request_reconsideration(target: PcpuRef) -> RequestResult;
                    internal to the scheduler reconsideration module; stable
                    within P7. This is the concrete implementation of the
                    S-3 seam that P7-W06 (W-1 wake path) and P7-W07 (P-1/F-1
                    remote control) call.
Purpose and caller: cause a pCPU to re-run its scheduling decision promptly.
                    Callers: W06 wakeup enqueue, W07 remote pause/stop,
                    W04 deadline machinery (local only), future internal
                    producers.
Inputs / outputs: PcpuRef — target from the C-1 eligible set.
                  RequestResult = Signaled | SelfTargeted | Refused(target).
Preconditions: caller already made its state effect visible (event record,
               pause marker, enqueue) with release ordering BEFORE calling;
               target is an online pCPU (PC-6) within the caller's eligible
               set (C-1).
Postconditions: Signaled → target.intent was set (release) and exactly one
                transport signal was issued for this flag transition; the
                target re-runs its decision in bounded time. SelfTargeted →
                flag set, no signal; the local loop observes it at its next
                decision point. Refused → flag untouched, diagnostic recorded;
                the caller's own state effect stands (it is never rolled back
                here).
State and ownership change: one word of the target's intent flag; transport
                side effects via the P3-W07 surface only.
Concurrency/allocation context: IRQ-safe, allocation-free, bounded; safe for
                concurrent senders (PC-3 guarantee) and coalesces naturally
                (flag is level, not count).
Errors and failure guarantee: transport send failure on an online target →
                RequestResult still records the attempt; degradation per §7 of
                [01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md)
                (bounded-latency discovery at the target's next natural
                decision point); refused offline/ineligible targets are
                invariant violations surfaced per the P0-W14 boundary.
Security/authorization checks: none — hypervisor-internal mechanism; Guests
                influence it only through already-authorized producers (W06
                sources, W07 control).
Logic: per §4 of the architecture file; not runnable production code.
Validation: P7-V18 remote rows; P7-V25 race stress (request-vs-idle-entry,
            coalescing storms) via P7-W11.
```

## R-2. `handle_reconsideration_irq`

```text
Name and stability: handle_reconsideration_irq(pcpu: PcpuRef) -> void;
                    internal; stable within P7; registered as the reception
                    hook of the P3-W07 notification path for scheduler-class
                    events.
Purpose and caller: the target-side half of the protocol; called by the
                    P3-W07 reception path (and by the P6-W04-evidenced Host
                    SGI mechanism beneath it) in the target pCPU's interrupt
                    context.
Inputs / outputs: the receiving pCPU; no return value.
Preconditions: interrupt context of pcpu; the transport attributed this
               signal to pcpu (PC-4 target attribution).
Postconditions: intent flag cleared then decision re-run inside the bounded
                loop of §4; any request coalesced during the loop triggers one
                more iteration; the handler terminates bounded.
State and ownership change: intent flag; whatever the decision entry itself
                changes (dispatch via L-1/L-3).
Concurrency/allocation context: hard IRQ context: bounded, allocation-free,
                no blocking; the bounded loop's bound is a declared constant
                reviewed against storm behavior.
Errors and failure guarantee: a signal with no pending intent (spurious or
                duplicate delivery) is a counted diagnostic; the handler still
                runs one decision pass (correctness never relies on the signal
                being expected).
Security/authorization checks: signal attribution is the transport's duty
               (PC-3/PC-4); this handler re-checks nothing about the sender.
Logic: clear-flag → decision → re-check → repeat (bounded) per §4.
Validation: P7-V17/V18; P7-V25 storm amplification via P7-W11; spurious-
            signal unit test.
```

## R-3. `PcpuRef` targeting rules

```text
Name and stability: PcpuRef + the targeting predicate; internal; stable
                    within P7.
Purpose and caller: guarantee that reconsideration targets are always drawn
                    from the C-1 eligible set and the PC-6 online set; used by
                    every R-1 caller.
Inputs / outputs: PcpuRef — validated reference to an online pCPU.
Preconditions / postconditions: an R-1 call with a target outside the
               caller's eligible set is unrepresentable where the type system
               allows, and a checked refusal otherwise.
State and ownership change: none.
Concurrency/allocation context: none (pure predicate).
Errors and failure guarantee: refusal with diagnostics; no retry.
Security/authorization checks: placement (C-1) is a scheduling-correctness
               boundary, not an authority boundary; capability checks happened
               in the producers.
Logic: intersection of eligible set (C-1) and online set (PC-6); empty
       result is an invariant violation (C-1 guarantees ≥1 eligible pCPU for
       an enqueued entity).
Validation: P7-V06/V18 placement matrix; unit tests for empty-intersection
            refusal.
```

## R-4. Accounting and trace hooks (consumed by P7-W09)

```text
Name and stability: on_reconsideration_requested(target, result, now),
                    on_reconsideration_handled(pcpu, iterations, now),
                    on_reconsideration_refused(target, reason, now); internal
                    hook points reserved for
                    [P7-W09](../p7-w09-accounting-diagnostics/README.md).
Purpose and caller: single authoritative observation points for remote
                    scheduling activity; called exactly once per outcome by
                    R-1/R-2.
Preconditions / postconditions: bounded, allocation-free, IRQ-safe; must not
               alter protocol outcomes; failures are accounting bugs surfaced
               per the P0-W14 boundary.
Security/authorization checks: none.
Logic: call-through.
Validation: P7-V19/V20 — sent/received/refused counters coherent; iterations
            observable for storm diagnosis.
```
