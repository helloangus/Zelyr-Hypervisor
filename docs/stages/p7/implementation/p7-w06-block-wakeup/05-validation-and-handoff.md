# P7-W06 Validation Matrix, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W06 detailed design](README.md).  
All rows below are planned evidence with objective conditions; none claims that
a test ran. Actual results are recorded only in
`../../verification/p7-w06-block-wakeup-verification.md`.

## 1. Validation matrix

| ID | Maps to | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W06-DV01 | P7-V13 | Block release review/test | Run a no-event WFI/WFE workload; observe pCPU dispatches other work or idles within one scheduler decision | No-event block releases pCPU capacity; no busy re-entry of the blocked vCPU | Scheduler-visible blocking; not that all wake sources work |
| W06-DV02 | P7-V13 | Preexisting-event abort | Post an eligible event before the exit reaches B-1; repeat across interleavings | Every trial yields WokeImmediately or an immediate re-dispatch; the vCPU never strands Blocked | Poll correctness under tested interleavings; not proof of zero races at arbitrary concurrency (W11 amplifies) |
| W06-DV03 | P7-V14 | Per-source wake matrix | Drive TimerExpiry, VirtualIrq, Notification (internal producer, W-5), Internal events against a Blocked vCPU | Each source alone produces exactly one Runnable transition and one enqueue; no source lost | Source coverage through the single wake entry; not Notification-object semantics (Reserved) |
| W06-DV04 | P7-V14 | Before/during/after-block race tests | Host-side interleaving harness around B-1/W-1; boundary at intent store, poll, L-1 commit | No lost wakeup, no duplicate running (L-2 holds), every outcome accounted | Race safety within tested interleavings and declared bounds; not a formal proof of the protocol |
| W06-DV05 | P7-V14 | Invalid-state wakeup exclusion | Drive W-1 against Paused/Stopped/Faulted/Running/Runnable states | Exclusion table (W-7) holds in every state; excluded wakes record events without eligibility | Exclusion determinism; not P7-W07 pause policy itself |
| W06-DV06 | P7-V14, P7-V18 | Deadline-home timer wake | Blocked vCPU with guest-timer deadline; home pCPU idles; deadline fires | vCPU wakes exactly once, is re-placed per placement rules; host monotonicity (P6-W05) unaffected | Fold + wake path; not hardware timer behavior — QEMU success does not prove real-hardware timing |
| W06-DV07 | P7-V04/V19 | Accounting coherence at hook points | Count block/immediate-wake/wake events against B-5 hooks | One-to-one hook coverage; counters never regress or double-count | Hook placement; not full W09 semantics (P7-W09 owns those) |
| W06-DV08 | W06 closure | Review of scope exclusions | Inspect the delta for wait-queues, wakeup APIs, P6 mechanics, policy | None present; all cross-package calls go through declared seams | Boundary discipline; not downstream package correctness |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. QEMU-based rows
require the P1/P4 QEMU baselines; without them they are recorded not run with
the owning stage named. No row here proves P7-V15–V12 outside W06's scope, and
none may be reported as doing so.

## 2. Error, security, and observability model

- **Errors.** W06 adds no Guest-facing error surface. Internal failure classes:
  transition rejection (L-1) → `NotBlocked(TransitionRejected)`, state stays
  consistent; enqueue/reconsideration surface failure (S-2/S-3) → invariant
  violation surfaced per the P0-W14 panic/failure-classification boundary
  (never silently absorbed); deadline-fold failure → block proceeds, idle
  refused (degraded, diagnosable). Guest-caused anomalies (spurious WFI loops,
  timer storms) degrade to accounting pressure and bounded IRQ work, never to
  hypervisor panic.
- **Security.** The block path consumes only trusted exit classification (P4
  boundary); the wake path accepts only producer-authorized events (P6-W07
  rights for vIRQ; P5 dispatch for Guest control). No capability, handle, or
  Guest pointer crosses this module. Guest behavior cannot create a lost
  wakeup, duplicate run, or invalid wake without violating an assumed
  upstream invariant — in which case the containment duty is W02/W07's, and
  W06's exclusion table minimizes the blast radius.
- **Observability.** Every block and wake outcome passes exactly one W09 hook
  with the exit hint / source and a monotonic timestamp; the vocabulary
  (`BlockExitHint`, `WakeEventSource`, `WakeOutcome`, `BlockDeclineReason`) is
  the trace field basis registered under the P0-W13 scheduler namespace by
  [P7-W09](../p7-w09-accounting-diagnostics/README.md). Release builds may
  trim high-frequency wake events per P0-W12 rules; block/decline events are
  minimum-payload and always available in diagnostic snapshots.

## 3. Handoff checklist

Before handing W06 to a reviewer and to consumers:

- Changed-file list, with the module boundary of
  [01](01-block-wakeup-architecture.md) §1 respected (no runqueue, timer, or
  vIRQ files touched).
- Assumed-contract status table from workflow step 1, including any deltas and
  blocked prerequisites.
- W06-DV01–DV08 status with evidence paths and explicit not-run entries.
- Confirmation: no new `unsafe`, no ABI/public API, no new dependency, no
  scheduling policy, no P6 mechanics modified.
- Seam inventory handed on: W09 hooks (B-5), W05 enqueue call points, W08
  reconsideration call points and the idle/deadline interlock, W07
  `post_internal_event` usage, W10 behavioral contract for WFI/wake scenarios.
- Open items for consumers (without resolving them here): W08 owns the
  reconsideration transport and idle ordering; W07 owns pause/stop outcomes at
  the L-1 serialization point; W11 owns race-density evidence (P7-V25).
- Record locations: implementation notes to
  `../p7-w06-block-wakeup-record.md`; evidence to
  `../../verification/p7-w06-block-wakeup-verification.md` — both created only
  when work or evidence exists.
