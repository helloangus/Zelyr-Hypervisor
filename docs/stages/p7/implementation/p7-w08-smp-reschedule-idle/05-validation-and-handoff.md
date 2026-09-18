# P7-W08 Validation Matrix, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W08 detailed design](README.md).  
All rows are planned evidence with objective conditions; none claims that a
test ran. Results go only to
`../../verification/p7-w08-smp-reschedule-idle-verification.md`.

## 1. Validation matrix

| ID | Maps to | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W08-DV01 | P7-V17 | Concurrent distinct-vCPU execution | Run N vCPUs on N pCPUs (QEMU SMP, P3 baseline required) | Each pCPU runs a distinct vCPU simultaneously; no vCPU appears twice (L-2); telemetry attributes each to its pCPU | Concurrent scheduling in the tested environment; not real-hardware SMP behavior |
| W08-DV02 | P7-V17 | pCPU state isolation | Inspect shared-state write sets; run isolation stress with concurrent decisions and idle | Scheduler state mutations are owner-pCPU-only except declared shared structures; no cross-pCPU corruption | Structural isolation; not freedom from all future shared-state bugs |
| W08-DV03 | P7-V18 | Remote work reconsideration | Wake/enqueue a vCPU whose eligible pCPU is remote and possibly idle | The target re-runs its decision in bounded time; work is dispatched; no lost request across tested interleavings | Protocol correctness in tested environments; not a formal proof |
| W08-DV04 | P7-V18 | Remote pause reconsideration | Pause/stop a vCPU running on a remote pCPU | Target ceases Guest execution within the W07 bounded bound; marker precedes signal ordering holds | Transport effect; not the pause semantics themselves (W07 owns those) |
| W08-DV05 | P7-V18 | Placement compliance of remote requests | Drive requests across the placement matrix (P7-W03 cases) under SMP | Every target is eligible (C-1); ineligible/offline targets are refused and diagnosed; no silent fallback | Placement-verified targeting; not placement policy itself |
| W08-DV06 | P7-V18 | Idle non-busy evidence | All-quiet system; sample idle pCPU behavior | Zero scheduling-decision iterations per idle period; single wait per period; no polling loop exists in review | Designed idle in the tested environment; QEMU WFE behavior does not prove hardware power/latency characteristics |
| W08-DV07 | P7-V18 | Idle wake-source coverage | Fire each of DeadlineFired, Reconsideration, CrossCpuNotification at an idle pCPU; plus a spurious wake | Each listed source exits idle exactly once and dispatches or re-decides correctly; spurious wakes are counted and harmless | Closed wake-source list; not notification semantics beyond the scheduler mapping |
| W08-DV08 | P7-V18 | Idle-to-work preservation | Wake idle pCPUs with work whose placement constrains targets | Post-idle dispatch obeys the same L-1/L-3/C-1 checks; no alternate selection path exists (review) | Uniform decision entry; not load balancing (Out of Scope) |
| W08-DV09 | P7-V25 preview | Request-storm and idle-race stress preview | Storm R-1 from all pCPUs at one target; race requests against idle entry | Bounded handler invocations; no lost or duplicated requests under tested density | Preview of W11 stress conditions; full race-density evidence is P7-W11's |
| W08-DV10 | P7-V19/V20 | Accounting coherence at hooks | Count sent/received/refused/iterations/idle transitions against R-4/I-5 hooks | One-to-one coverage; counters never regress or double-count | Hook placement; full W09 semantics owned by P7-W09 |
| W08-DV11 | W08 closure | Scope-exclusion review | Inspect the delta for notification/timer/hotplug/balancing implementations | None present; all transport/timer/wait access behind declared seams | Boundary discipline; not downstream correctness |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. W08-DV01–DV03 and
DV06–DV08 require the P3 SMP and P1/P4 QEMU baselines; without them they are
recorded not run with the owning stage named. QEMU evidence never proves
real-hardware interrupt-latency, WFE, or power behavior; RK3566 evidence is a
later-stage responsibility. No row proves P7-V24+ stress properties, and none
may be reported as doing so.

## 2. Error, security, and observability model

- **Errors.** Internal failure classes: refused targets (ineligible/offline —
  invariant violations surfaced per the P0-W14 boundary, never retried
  silently); transport send failure (degrades to bounded-latency discovery at
  the target's next natural decision point, per
  [01](01-smp-scheduler-architecture.md) §7); deadline-fold failure (idle
  refused, degraded but safe); spurious wakes (counted, correctness-neutral).
  A permanently stalled online target with pending intent is a diagnosable
  invariant violation, not a condition to poll away.
- **Security.** The reconsideration mechanism is hypervisor-internal; Guests
  and external inputs influence it only through already-authorized producers
  (P7-W06 sources, P7-W07 control, P7-W04 deadlines). No Guest-influenced
  length, index, or pointer crosses the protocol; targeting is verified
  against placement and online sets before any signal, so a misbehaving
  producer cannot direct signals at arbitrary pCPUs.
- **Observability.** Every request, delivery, refusal, handler iteration
  count, idle transition, and wake source fires exactly one W09 hook with a
  monotonic timestamp basis; storm behavior is visible through the iteration
  counter; idle behavior is visible through entry/exit and spurious-wake
  counters. Trace fields register under the P0-W13 scheduler namespace via
  [P7-W09](../p7-w09-accounting-diagnostics/README.md); release trimming
  follows P0-W12 (high-frequency reconsideration events trim first; refusals
  and spurious wakes stay in diagnostic snapshots).

## 3. Handoff checklist

Before handing W08 to a reviewer and to consumers:

- Changed-file list respecting the module boundary of
  [01](01-smp-scheduler-architecture.md) §1 (no P3 notification, P6 timer, or
  arch wait-seam implementation files modified).
- Assumed-contract status table from workflow step 1, including deltas and
  blocked prerequisites (especially PC-3/PC-4/PC-5 surfaces).
- W08-DV01–DV11 status with evidence paths and explicit not-run entries
  (naming which rows await the P3/P1/P4 baselines).
- Confirmation: no new `unsafe`, no ABI/public API, no new dependency, no
  transport/timer/hotplug/balancing implementation, no board conditionals.
- Seam inventory handed on: W11 (protocol race classes, isolation rules,
  placement-verified targeting, wake-source closure list for stress);
  W09 (R-4/I-5 hooks); W12 (SMP/idle behaviors for its QEMU matrix, via W11).
- Open items for consumers (not resolved here): W11 owns race-density and
  stress evidence (P7-V24–V27); W09 owns record/trace semantics; hardware
  WFE/interrupt-latency evidence remains later-stage work.
- Record locations: implementation notes to
  `../p7-w08-smp-reschedule-idle-record.md`; evidence to
  `../../verification/p7-w08-smp-reschedule-idle-verification.md` — both
  created only when work or evidence exists.
