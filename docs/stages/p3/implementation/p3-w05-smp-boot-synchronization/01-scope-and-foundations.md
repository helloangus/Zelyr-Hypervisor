# P3-W05 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W05 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Define and verify boot-time ordering from one-time global initialization
through secondary readiness to SMP-ready" requires four concrete
artifacts:

1. The **phase authority**: a `BootPhase` word with designated actors,
   once-only CAS transitions, and a gate that dispatch and secondary code
   must observe —
   [03-code-contracts-boot-rendezvous.md](03-code-contracts-boot-rendezvous.md)
   §2.
2. The **per-CPU ready gate**: release-signaled per CPU after local
   initialization, acquire-counted by the coordinator — same file §3.
3. The **coordinator and declaration**: a bounded-by-terminals wait, the
   once-only `declare_smp_ready`, and the queryable
   `SmpReadyState` including the degraded record — same file §4–§5.
4. **Evidence**: host-side protocol tests (ordering, once-only, degraded
   accounting) and QEMU rendezvous captures per declared count, per
   [05-validation-and-handoff.md](05-validation-and-handoff.md).

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Bring-up flow with a gate assertion point and terminal outcomes | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) | The requester asserts `GlobalInitPublished` before the first CPU_ON; every attempted CPU reaches a terminal outcome; the entry tail signals W05 after W04 install | If W02's flow lacks the assertion point or terminals, resolve as a cross-design conflict; W05 must not add its own start mechanism |
| Lifecycle states and admission operation | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) | Registry states for terminal accounting; `admit_online` called by this design's coordinator per ready CPU | If admission is refused for a ready CPU, that is an invariant violation (fatal) — the two designs' conditions must agree |
| Local-install completion point | [P3-W04](../p3-w04-per-cpu-runtime/README.md) | Install ends by signaling this design's ready gate | A consumer that signals readiness without completing install violates P3-V03's "usable only after local init" — refuse at review |
| Boot-integration owner for continuation policy | [P1-W09](../../../p1/plans/p1-w09-initialization-sequencing.md) style boot sequencing; P3 integration | Consumes `SmpReadyState` (including `Degraded`) and decides halt-vs-continue | If no owner exists when needed, the recorded default (continue-with-diagnostics on the reference boot) stands, with the open question carried to closure |
| Diagnostics/trace governance | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) | Event namespace for phase transitions and the SMP-ready result | Catalog registration coordinates with [P3-W11](../p3-w11-smp-observability/README.md); no private scheme |

### 1.3 Why no hidden essential deliverable remains

- "Integrate the readiness contract with shared synchronization and test
  consumers" (plan step 3) is realized by the query/event surfaces handed
  to W12/W13 and the phase gate shared with W02 — designed surfaces, not
  implied ones.
- "Collect repeated rendezvous evidence including timing/failure
  observations" (plan step 5) is bounded: W05 defines what a rendezvous
  capture must show and provides an optional counter-based latency
  observation; the repeated executions are W13's matrix, and timing
  semantics remain informative until P6/W11 own them. This is stated in
  the matrix rather than left implicit.

## 2. Scope classification

### 2.1 Required

- `BootPhase` word with `Bootstrap`, `GlobalInitPublished`, `SmpReady`;
  designated-actor transitions; `require_published` gate.
- Per-CPU ready gate (signal + count) and the coordinator wait bounded by
  the attempted-set terminal condition.
- `declare_smp_ready` (once-only, fenced) and `smp_ready_state` query.
- `SmpReadyState::{Pending, Ready, Degraded}` with the failed set
  recorded; phase-transition and result diagnostics.
- Host-side protocol tests (once-only, ordering, degraded accounting,
  refusal paths); QEMU rendezvous captures per declared count.

### 2.2 Reserved (must not block a future design; not implemented now)

- Strict-fail boot profile (halt on any secondary failure); trigger: a
  boot-policy decision by the boot-integration owner.
- Rendezvous-latency telemetry with owned event ids; trigger:
  [P3-W11](../p3-w11-smp-observability/README.md) catalog plus the P6
  timer baseline (the counter read used meanwhile is an informative
  diagnostic, not a timing authority).
- WFE/SEV parking in coordinator/secondary waits; trigger: P3-W07's event
  primitive.
- Post-boot reuse of rendezvous machinery (stop-the-world, quiescence);
  trigger: a later-stage design (P16 owns stop-the-world).

### 2.3 Out of Scope

- Lock primitives and general shared-state access rules (W06); W05 uses
  only atomics and one explicit publish fence.
- Lifecycle transitions and the registry (W03; W05 calls `admit_online`).
- Per-CPU runtime contents and install mechanics (W04; W05 consumes its
  completion signal).
- Start mechanics (W02); notification/IPI (W07); TLB transport (W08).
- Runtime hotplug coordination and scheduler blocking (stage/P7).
- The halt-vs-continue policy decision itself (surfaced, not chosen).
- Guest-visible synchronization (P8+).
