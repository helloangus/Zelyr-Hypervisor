# P7-W09 Code Contracts — Accounting Records and Update Points

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W09 detailed design](README.md).  
Scope: record types and update contracts. Trace events and diagnostics are in
[03-code-contracts-trace-diagnostics.md](03-code-contracts-trace-diagnostics.md).
Names are stage-local design freedom owned by this design (rationale: parent
README decisions 1–4); reconcile with sibling module contracts at
implementation time without changing their semantics.

---

## A-1. `VcpuAccounting`, `PcpuAccounting` record types

```text
Name and stability: struct VcpuAccounting, struct PcpuAccounting per the §3
                    tables of [01-accounting-model.md](01-accounting-model.md);
                    internal to the scheduler observability module; stable
                    within P7. Field order is not an ABI; no raw serialization.
Purpose and caller: the sole sinks for scheduler counter updates; instantiated
                    by the vCPU/per-pCPU object owners and mutated only through
                    A-2..A-5.
Inputs / outputs: typed newtype IDs (VmId, VcpuId, PcpuId), monotonic time
                  readings (T-3 type), enumerations (SwitchReason,
                  WakeEventSource, outcome classes from the producer designs).
Preconditions: a record exists for every live vCPU and online pCPU before any
               hook can fire on it; records are initialized to zero at object
               creation (lifecycle owned by the object's package).
Postconditions: fields follow the coherence rules
                ([01-accounting-model.md](01-accounting-model.md) §4): single
                writer class, monotonic non-decreasing accumulators, durations
                from same-critical-section reading pairs.
State and ownership change: embedded records only; no allocation after
               creation (fixed-size by construction).
Concurrency/allocation context: writer-pCPU-only mutation; aligned atomic
               stores or writer-local plain writes per P3-W06 rules; readers
               accept declared staleness; no cross-CPU RMW.
Errors and failure guarantee: none at rest; an observed invariant breach
               (decreasing accumulator, negative duration) surfaces per the
               P0-W14 boundary.
Security/authorization checks: none — records contain no Guest-controlled
               content (identifiers and enumerations only).
Logic: data + the update contracts below.
Validation: P7-V19 coherence; property test — for random hook sequences,
            every accumulator is consistent with the hook history.
```

## A-2. `record_switch`

```text
Name and stability: record_switch(pcpu, outgoing: Option<(VcpuRef, SwitchReason)>,
                    incoming: Option<VcpuRef>, readings: TimePair) -> void;
                    internal; stable within P7.
Purpose and caller: the single update point for every switch on a pCPU;
                    called by the exit-path disposition (W07 F-4) and the
                    dispatch path exactly once per entity change (including
                    entity→idle and idle→entity).
Inputs / outputs: see signature; TimePair is the monotonic reading pair taken
                  in the same critical section as the disposition.
Preconditions: called from the disposition/dispatch paths only; outgoing and
               incoming are mutually exclusive with idle transitions per the
               decision entry's semantics.
Postconditions: switches_out[reason] and switches_in advance; guest_runtime,
               ready_wait, busy_time, idle_time, context_switches update per
               §3/§4; the trace emission (C-1) fires for this switch.
State and ownership change: records only.
Concurrency/allocation context: writer = this pCPU; bounded, allocation-free,
               IRQ-safe.
Errors and failure guarantee: none; invariant breaches surface per P0-W14.
Security/authorization checks: none.
Logic: compute durations from TimePair; saturate at type width only as a
       diagnosed anomaly (never silent); update; emit trace.
Validation: P7-V19/V20; a switch is visible with its reason and entity
            identities.
```

## A-3. `record_block_wake`

```text
Name and stability: record_block(pcpu, entity, hint, reading) -> void;
                    record_wake(entity, source, outcome, requester, reading)
                    -> void; internal; stable within P7; the bodies behind
                    P7-W06's B-5 hooks.
Purpose and caller: consume the W06 block/wake hook points exactly once per
                    committed outcome.
Inputs / outputs: per B-5 of
                  [P7-W06](../p7-w06-block-wakeup/02-code-contracts-block-path.md);
                  requester is the pCPU that ran the wake (may differ from
                  the vCPU's home pCPU — both recorded for P7-V25 analysis).
Preconditions: producer guarantees one call per committed outcome.
Postconditions: blocks / wakes-by-source-and-outcome advance; blocked_time
               accumulates from block commit to the next wake or terminal
               transition; trace events fire (C-2, C-3).
State and ownership change: records only.
Concurrency/allocation context: IRQ-safe, allocation-free, writer-class per
               §3.
Errors and failure guarantee: none.
Security/authorization checks: none.
Logic: accumulate + emit.
Validation: P7-V19/V20; P7-V14/V25 consumers read coherent per-event counts.
```

## A-4. `record_control_fault`

```text
Name and stability: record_pause(...), record_resume(...), record_stop(...),
                    record_fault(...); internal; stable within P7; bodies
                    behind P7-W07's F-5 hooks.
Purpose and caller: consume the control/fault hook points exactly once per
                    committed outcome; record the authority outcome class for
                    pause/resume/stop (granted/denied) where the producer
                    passes it.
Inputs / outputs: per F-5 of
                  [P7-W07](../p7-w07-pause-stop-fault/03-code-contracts-stop-fault.md);
                  fault carries the P4-W06 fault-class enumeration only.
Preconditions: producer guarantees one call per committed outcome; denials
               are recorded by the P5 dispatch path's own telemetry (T-1),
               not by these hooks.
Postconditions: paused_time, switches_out reasons (PausedControl,
               StoppedControl, FaultedGuest), and VM generation counters
               advance; trace events fire (C-4).
State and ownership change: records only.
Concurrency/allocation context: as A-3.
Errors and failure guarantee: none.
Security/authorization checks: fault class and cause are enumerations; no
               Guest text or raw addresses are stored here (addresses, if any,
               live in the P4-W06 diagnostic boundary and appear only in its
               events).
Logic: accumulate + emit.
Validation: P7-V19/V20 — stop-running reason distinguishable from fault;
            P7-V15/V16 consumers.
```

## A-5. `record_pcpu_event` and VM aggregation

```text
Name and stability: record_pcpu_event(pcpu, kind: PcpuEventKind, details)
                    -> void; aggregate_vm(vm) -> VmSummary; internal; stable
                    within P7; bodies behind P7-W08's R-4/I-5 hooks.
Purpose and caller: consume reconsideration/idle/deadline hooks; provide the
                    read-time VM aggregation of
                    [01-accounting-model.md](01-accounting-model.md) §3/§5.
Inputs / outputs: PcpuEventKind ∈ { Reconsider(Request|Handled|Refused),
                    Idle(Enter|Exit|Spurious|Refused), DeadlineFired,
                    HandlerIterations }; VmSummary per §3 computed fields.
Preconditions: hooks fire once per producer outcome; aggregation is called
               only from diagnostic/observation contexts (never the hot path).
Postconditions: pCPU counters advance; VmSummary reflects a consistent
               per-member snapshot under declared staleness; no aggregate is
               ever stored.
State and ownership change: pCPU records only.
Concurrency/allocation context: record path bounded/IRQ-safe; aggregation is
               a bounded member iteration in observation context.
Errors and failure guarantee: none.
Security/authorization checks: none.
Logic: accumulate; aggregate = sum over members + member-state census.
Validation: P7-V19; W11/W13 consumers (P7-V24–V27, P7-V29) read coherent
            values.
```
