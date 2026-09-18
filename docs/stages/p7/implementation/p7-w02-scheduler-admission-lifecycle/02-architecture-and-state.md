# P7-W02 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W02 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `lifecycle` | legality of every run-state change; the only writer of run state | per-vCPU lifecycle cell (state + lock + authority metadata of the last transition) | `LifecycleEvent` + `TransitionContext` from any caller path | accepted transition (new state) or typed rejection; trace semantics event | runqueue membership, placement, timers, policy |
| `admission` | the single normal-entry gate and exit boundary; scheduler activation; bypass register | per-pCPU `current_vcpu` slot reservation protocol; scheduler-mode flag; static bypass table | gate requests from the control loop (W05), exit outcomes from the P4 mechanism | `EntryPermit` or typed rejection; `PostExitControl` | who is picked next (W05); what the Guest does (P4) |
| `audit` | invariant checking over snapshots | none (pure functions over snapshots) | scheduler snapshots (lifecycle cells + pCPU `current_vcpu` slots + queue membership summaries) | `InvariantReport` with per-invariant verdicts | runtime monitoring policy (W09), stress harness (W11) |

Dependency direction (normative): `audit` → `admission` → `lifecycle`; the
policy layer (W05) → `admission` + `lifecycle`; `lifecycle` depends only on
ids, errors, trace semantics, and P3 synchronization primitives. No reverse
edge exists. Physical CPU and vCPU remain separate objects in different
owners (pCPU registry: P3; vCPU objects: P4; per-pCPU scheduler slot: this
design's admission module attached to the P3 per-CPU area).

## 2. Core objects and ownership

| Object | Owner | Mutable state | Written by | Lifetime |
|---|---|---|---|---|
| vCPU lifecycle cell | the vCPU object (P4); lifecycle module owns its interior | `run_state: VcpuRunState`; `last_transition: TransitionRecord` | `lifecycle` engine only (INV-4) | vCPU lifetime; P7 has no destruction path |
| `current_vcpu` slot | per-pCPU scheduler state (P3-W04 reserved area) | `Option<VcpuRef>` | `admission` gate/exit path only, and only while executing on that pCPU | pCPU online lifetime |
| scheduler-mode flag | hypervisor-wide, written once by `activate_scheduler` | `Inactive → Active` | boot sequence (P1/P3 bring-up integration) | hypervisor lifetime |
| bypass register | static, reviewable table compiled in | none at runtime (read-only) | review process (E1–E3 instances) | static |
| transition trace | emitted events (semantics here, rendering W09) | none (append-only) | `lifecycle` engine | consumer-defined |

Single-writer rule: every mutable fact above has exactly one writing module;
any second writer is a design violation. Cross-CPU mutation of a lifecycle
cell is performed only via the engine's locked entry points (the *request*
may arrive cross-CPU, e.g. a remote pause; the state change is still applied
under the cell's lock — the remote-flow mechanics, including how a
currently-running vCPU is stopped first, are W07/W08 and must route through
these same engine calls).

## 3. vCPU lifecycle state machine

States are exactly the ADR §4.1 set. The machine (edges in
[the transition table](03-code-contracts-lifecycle.md) §3):

```text
                 MakeRunnable            Dispatch                Deschedule
     Offline ----------------> Runnable <------- Running <-------+
        |                          |  ^                          |
        |                          |  |      Wake                (re-queue)
        |                          |  +--------------------------+
        |                          |              ^                |
        |                          |              |                |
        |                     Pause|         Block|                |Block
        |                          v              |                |
        |     Resume           Paused <-----------+----------------+
        |      +--------------------^
        v      v
      Stopped <--- (Stop from Offline/Runnable/Running/Blocked/Paused)
      Faulted <--- (Fault from Running, via P5-classified guest fault)

     Stopped, Faulted: terminal in P7 (no outgoing edges)
```

Reading notes:

- `Deschedule` covers slice expiry, voluntary exit with retained eligibility,
  and any exit where the vCPU remains dispatchable (W04/W05 decide which).
- `Block` is legal only from `Running`: a vCPU blocks by executing (e.g.,
  WFI-class behavior); the event *sourcing* is W06.
- `Pause` from `Running` requires the pCPU owner path (W07) to first bring
  the vCPU out of the Guest; the engine sees the `Pause` event only after
  the Guest is no longer executing — this sequencing constraint is part of
  the edge's contract.
- A vCPU paused while `Blocked` stays `Paused`; whether an event arrives
  during the pause is deferred-event bookkeeping owned by W07, not a new
  run-state (Plan guide prohibition on hidden state machines applies: W07
  must declare its auxiliary state and route changes through declared
  mechanisms).
- `Stop` from `Offline` covers configured-but-never-run teardown; `Stop`
  from `Running` requires the same pCPU-owner sequencing as `Pause`.

Rejected transitions (examples, all typed rejections, never silent no-ops):
`Dispatch` from anything but `Runnable`; `Wake` from `Runnable`/`Paused`
(a `Wake` arriving at a `Paused` vCPU is rejected at the engine and becomes
deferred-event bookkeeping in W07's owned auxiliary state); `Resume` from
anything but `Paused`; `MakeRunnable` from `Running`; any event on
`Stopped`/`Faulted`.

## 4. Admission boundary and scheduler activation

```text
 bring-up (E1)           activation        steady state (bypass set empty)
 pCPUs running P1/P3 ------------------>  control loop on each online pCPU:
 bring-up code, no         (one-time)       pick (W05) -> admit_for_entry
 Guest entry                                -> P4 entry -> Guest
                                              -> exit -> complete_exit
                                              -> reconsider (W04/W05)
 E2 paths: legal only pre-activation; empty after.
 E3 paths: fatal/emergency; never resume normal Guest execution.
```

Contract: after activation, a successful Guest entry implies (a) the gate
returned `EntryPermit`, (b) the vCPU state is `Running`, (c)
`pcpu.current_vcpu == Some(vcpu)`. Any observed Guest entry violating these
is an INV-2/INV-3 failure with an E3-class explanation required in the
diagnostics.

## 5. Concurrency model

- **Locks.** One lifecycle lock per vCPU cell, P3-provided spinlock class
  (W02 does not design the primitive). The lifecycle lock is a leaf: no
  code holds it while acquiring any other lock. The per-pCPU
  `current_vcpu` slot needs no lock (owner-executed, IRQ-masked per P3
  rules during gate/exit critical sections). Queue locks (W05), registry
  locks (P3), and timer locks (P6) are never held while taking a lifecycle
  lock — callers either transition first then enqueue, or pop a candidate,
  release the queue, then gate it.
- **IRQ context.** Timer/event callbacks (P6) may raise engine events.
  Engine IRQ-context entries: table lookup + state store + trace-emission
  flag; no allocation, no queue mutation, no cross-CPU spin. Paths needing
  more (e.g., enqueuing a woken vCPU) defer to a bounded post-IRQ step —
  the exact wakeup protocol is W06 and must respect this bound.
- **Lock hold bounds.** Critical sections are O(table lookup); no loops,
  no tracing I/O beyond the P0 in-kernel trace buffer, no allocation while
  holding the lifecycle lock.
- **Ordering.** State transition precedes queue publication ("a vCPU is
  discoverable on a queue only after it is `Runnable`"); the cross-CPU
  observability window between the two steps is the wakeup-race surface
  whose closed protocol is W06's to own, under the frozen invariant that no
  wakeup is lost (P7-V14).
- **Cross-CPU.** Remote requests (pause/stop/reschedule) never write a
  remote pCPU's `current_vcpu`; they request, and the owning pCPU applies.
  Transport semantics: P3-W07; flow design: W07/W08.

## 6. Security and authorization model

- The engine accepts control events (`Pause`, `Resume`, `Stop`) only with a
  `TransitionContext` carrying the caller role and a capability-receipt
  reference produced upstream by the P5 rights check (hook owned by W07 for
  lifecycle control, W03 for configuration). Missing/invalid context is a
  typed rejection (`ControlNotAuthorized`), recorded; the engine performs
  no capability lookup itself (ADR-013: capability model lives in P5).
- Guest-caused faults reach the engine only as pre-classified `Fault`
  events from the fault path (P4 exit boundary + P5 taxonomy); the engine
  never inspects Guest-provided data. Guest input is therefore structurally
  unable to influence transition legality.
- Scheduling policy must not leak into these generic core APIs: no policy
  parameter (slice, weight, class) appears in any contract in
  [lifecycle contracts](03-code-contracts-lifecycle.md).

## 7. Failure model

| Condition | Classification | Engine behavior |
|---|---|---|
| Invalid transition requested | Guest-adjacent control error (recoverable) | typed rejection `Illegal{from, event}`; state unchanged; rejection trace emitted |
| Control event without authority context | authorization rejection (recoverable) | `ControlNotAuthorized`; state unchanged; recorded |
| Guest-caused fault (P5-classified) | VM-facing | `Fault` accepted from `Running`; vCPU `Faulted`; contained per W07 |
| Hypervisor invariant violation (checker or engine assertion) | fatal (P0 panic policy) | immediate fatal diagnostics (P1 boundary); no state laundering |
| Gate called with ineligible candidate | recoverable | typed rejection; caller (control loop) decides next action (W05/W08) |

The engine never panics on any rejection path; panics are reserved for the
invariant row above.

## 8. Telemetry semantics (rendered by W09)

Emit (semantically, at engine/gate boundaries): transition-accepted
`{vcpu, from, event, to, ctx-role}`; transition-rejected
`{vcpu, from, event, reason}`; gate-result `{pcpu, vcpu, verdict, reason}`;
activation `{}`; bypass-use `{class, path-id}` — E3 use is always
 noteworthy. Field names here are semantic; encoding, filtering, and rates
are W09. Rejected transitions in steady state are diagnostics-grade;
unbounded rejection loops are an integration smell the control loop (W05)
must avoid by construction.
