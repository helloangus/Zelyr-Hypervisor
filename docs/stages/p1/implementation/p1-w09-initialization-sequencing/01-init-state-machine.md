# P1-W09 Init State Machine

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W09 detailed design](README.md).

## 1. Phase model

The lifecycle is a total order over eight phases plus one terminal state. A
phase is the interval in which exactly one supplying package's mechanism
establishes its planned contract. The phase is the unit of prerequisites,
markers, failure routing, and review — not a container for the mechanism's
internal state, which stays owned by its package.

| # | Phase identifier | Rust variant | Supplying contract | Phase purpose (exit condition owner) |
|---|---|---|---|---|
| 1 | `entry` | `InitPhase::Entry` | W01 | Reference boot entry accepted: environment validated or rejected per the W01 boundary |
| 2 | `runtime` | `InitPhase::Runtime` | W02 | Minimal Rust `no_std` EL2 runtime established: stack, static data, boot context, panic route, build identity |
| 3 | `capabilities` | `InitPhase::Capabilities` | W03 | Capability inventory produced; required facts present or continuation blocked |
| 4 | `el2-baseline` | `InitPhase::El2Baseline` | W04 | Required EL2 architectural state explicitly owned and consistent |
| 5 | `exceptions` | `InitPhase::Exceptions` | W05 | All four vector categories valid; recoverable/fatal boundary defined |
| 6 | `console` | `InitPhase::Console` | W06 | Early channel established; markers from entry onward materialized |
| 7 | `fatal-path` | `InitPhase::FatalPath` | W07 | Bounded non-recursive fatal reporting ready across pre/post-MMU |
| 8 | `stage1` | `InitPhase::Stage1` | W08 | Host Stage-1 transition complete; execution, console, vectors, fatal path live on mapped memory |
| — | `stable` | `InitPhase::Stable` | W09 outcome | Terminal state: controlled stable idle reached; consumers (W10–W12, P2) may attach |

Each phase has exactly three observable events: `enter`, `complete`, and the
failure route (§4). `enter` asserts the phase's prerequisite set; `complete`
asserts the supplying contract's exit condition is met. Events are recorded in
the tracker ([02-interface-contracts.md](02-interface-contracts.md) §3) even
when they cannot yet be printed (§3).

## 2. Legal transitions

```text
 (firmware handoff, W01 validates)
   Entry.enter -> Entry.complete
     -> Runtime.enter -> Runtime.complete        [W02 then calls run_init_sequence()]
       -> Capabilities.enter -> Capabilities.complete
         -> El2Baseline.enter -> El2Baseline.complete
           -> Exceptions.enter -> Exceptions.complete
             -> Console.enter -> Console.complete
                  [channel available: deferred markers materialize here]
               -> FatalPath.enter -> FatalPath.complete
                 -> Stage1.enter -> Stage1.complete
                   -> Stable (terminal)
```

Rules:

- **T1 Total order.** `enter(P[n])` is legal only if `complete(P[n-1])` is the
  latest recorded event. Phases cannot be skipped, repeated, or reordered at
  runtime; the order is compiled into the sequencer.
- **T2 Monotonicity.** No transition decreases the lifecycle position. The
  state machine has no backward edges and no partial-rollback edges.
- **T3 Terminal failure.** Any phase failure routes to the diagnostic route of
  §4 and terminates normal boot. Failure is modeled as leaving the lifecycle at
  the failing phase with the route's bounded terminal behavior, never as a
  transition to another phase.
- **T4 Single traverser.** The boot CPU traverses the lifecycle exactly once
  per boot. There is no secondary-CPU participation, no re-entry, and no
  hot-path concurrency (P1 has no SMP and enables no interrupt delivery).
- **T5 Stable is terminal.** `stable` has no `complete` event and no outgoing
  normal transition. Faults after `stable` are exception-path events (§4),
  not lifecycle transitions.

Per-step failure and rollback posture (plan work seq 2 demands the statement
per step): every step's failure posture is "terminal via §4 route; retain the
failed phase identifier; no rollback". The retained state at failure is exactly
what the route needs: the phase identifier and the supplying contract's own
failure context. Nothing is un-wound because nothing is re-claimed.

## 3. Marker availability model

Markers are (phase, event) records rendered through the W06 channel in the
format that W06's design fixes; W09 owns only the label tokens of §1 and the
event kinds. Because phases 1–5 complete before the channel is available
(decision 1 of the parent README), the model has two regimes:

- **Deferred regime (phases 1–5).** Events are recorded in the tracker only.
  No output is attempted; early failure handling uses the routes of §4, whose
  early forms are the W01 rejection diagnostic and the W02/P0 panic route.
- **Materialized regime (from `console`).** When W06's mechanism reports the
  channel available, the tracker emits a replay line listing all recorded
  events in order (identifying every stage from entry onward, as P1-V10
  requires), then live markers follow for each subsequent event, including
  `console.complete` itself.

Requirements:

- M1 Every phase transition must be recorded, in both regimes; no phase may
  condition its marker on output availability.
- M2 The replay is emitted exactly once per boot, at the materialization point;
  the materialization point is the W06 availability signal, not a timer or a
  fixed phase number.
- M3 The `stable` state emits the stable-state marker with content owned by
  [P1-W10](../p1-w10-qemu-boot-regression/README.md) (coordination requirement:
  W10 selects the token; W09 guarantees the emission point at `stable` entry).
  Until W10's design fixes the token, W09 requires only that the emission
  happen and be phase-attributed.
- M4 Marker text carries no policy and no capability interpretation; it is
  phase identification only. Capability *content* is W03 output rendered by
  W06.

## 4. Failure-routing matrix

A failure route must be **established by an earlier completed phase** — this is
the lifecycle expression of "no hidden dependency": the route a phase needs may
not be a capability of the phase that is failing. The route column names the
owning contract; the mechanism's internal behavior belongs to that contract.

| Failing phase | Route established by | Expected observable | Terminal behavior |
|---|---|---|---|
| `entry` | W01 rejection boundary | Rejection reason, no runtime markers, no normal continuation | Bounded stop before transfer; QEMU-visible halt/exit per W01 contract |
| `runtime` | W02 bounded failure via P0 panic route | Panic-route report with build identity; phase attributed `runtime` | Terminal; no retry |
| `capabilities` | W03 fail-fast via panic route (fatal path not yet built) | Explicit required-capability absence reason; not an unrelated panic | Terminal |
| `el2-baseline` | W04 failure boundary via panic route | Unsupported-element reason, phase attributed | Terminal |
| `exceptions` | W05 fatal boundary via panic route | Syndrome and location context as available | Terminal |
| `console` | Panic route (best effort; output may be impossible) | At minimum a bounded stop; no silent continuation | Terminal |
| `fatal-path` | W07's own non-recursive readiness boundary | Minimal bounded terminal behavior per W07 contract | Terminal; must not recurse |
| `stage1` | W07 fatal path (ready since phase 7), pre- or post-MMU-on per W08's transition contract | Full fatal report including transition state | Terminal |
| post-`stable` fault | W05 vectors + W07 fatal path | Syndrome/classified unexpected-event report with phase `stable` | Terminal; no lifecycle transition (T5) |

Supplementary routing facts:

- **Unowned window.** Between boot entry and `exceptions.complete` there is no
  hypervisor-owned vector table except the early forms W05 itself installs. A
  spurious exception in phases 1–5 is outside P1's owned failure surface; this
  is a recorded lifecycle limitation handed to
  [P1-W12](../p1-w12-p1-documentation-handoff/README.md), not something the
  sequencer may paper over. If W05's design enables earlier vector
  installation, the window narrows but the matrix routes do not change.
- **Misuse routing.** Sequencer contract violations (wrong phase order, double
  `complete`, `stable` misuse) are invariant violations: they route through the
  bounded fatal path if ready, otherwise through the panic route, and never
  panic from inside a panic/fatal path.
- **Failure evidence.** Every routed failure must leave the phase identifier
  recoverable from the report (W07 phase field or the tracker's position), so
  W10/W11 evidence can attribute failures to phases.

## 5. Partial-init, hidden-dependency, and reserved-boundary rules

These rules are the reviewable core of P1-V15.

- **H1 Declared prerequisites only.** `enter(P[n])` may rely only on the exit
  conditions of phases 1..n-1 as stated in §1 and on the supplying contracts.
  If an adapter needs anything else, the missing prerequisite is recorded as a
  design change to this file — never satisfied by reaching into another
  package's internal state.
- **H2 No partial-init continuation.** No phase observes another phase's
  partially-established state; the only inter-phase observable is a completed
  exit condition. A mechanism that cannot complete exits via §4; it never
  "completes enough".
- **H3 Reserved boundaries.** No phase may reserve, initialize, or probe
  mechanisms of later stages (GIC, DTB discovery, allocators, SMP, Guest). The
  capability inventory reports facts (W03) but configures none of them.
- **H4 Failure-route precedence.** The §4 matrix is checked against the phase
  order: any route required by phase n must be established by a phase < n. A
  route that would need the failing phase itself is a design error caught by
  W09-DV04.
- **H5 Repeat-boot consistency.** The lifecycle must reach `stable` with the
  same phase sequence and the same marker token set on every clean boot; any
  run-to-run difference in the sequence is a failure regardless of final state.
  Evidence expectations are defined in
  [03-implementation-and-review.md](03-implementation-and-review.md) and
  executed by W10/W11.
- **H6 Phase attribution.** Any post-`stable` diagnostic must be attributable
  to `stable` (or its exception context), never to a stale boot-phase value;
  the tracker's read semantics in
  [02-interface-contracts.md](02-interface-contracts.md) §3 guarantee a
  monotone, never-impossible snapshot.
