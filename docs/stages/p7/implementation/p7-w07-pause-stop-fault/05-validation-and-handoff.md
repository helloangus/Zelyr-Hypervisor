# P7-W07 Validation Matrix, Error Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W07 detailed design](README.md).  
All rows are planned evidence with objective conditions; none claims that a
test ran. Results go only to
`../../verification/p7-w07-pause-stop-fault-verification.md`.

## 1. Validation matrix

| ID | Maps to | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W07-DV01 | P7-V15 | Pause state matrix | Drive P-1 against Running (local), Running (remote), Runnable, Blocked, Paused, Stopped, Faulted, Offline | Every row's outcome matches the §3 table; denials leave zero state | Per-state pause safety; not snapshot/migration pause variants (Reserved) |
| W07-DV02 | P7-V15 | Remote pause completion | Pause a vCPU running on another pCPU; observe via telemetry/exit hooks | The target ceases Guest execution in bounded time (bounded by P7-W04 preemption) without unbounded spinning anywhere | Remote completion semantics; not the transport internals (W08) |
| W07-DV03 | P7-V15 | VM pause leaves no Guest executing | Multi-vCPU VM; request P-3; sample admission and running state | After acceptance, no member is admitted (P-5) and all running members reach Paused; completion predicate (P-4) flips exactly then | The §5 completion model; not VM-wide fault policy |
| W07-DV04 | P7-V15 | Resume preservation matrix | Pause each state; resume; compare eligibility, placement, pending events | Runnable/Running pre-pause → Runnable with identical placement; Blocked pre-pause without events → Blocked; with events → Runnable; events preserved exactly | Preservation semantics; not placement policy re-derivation (C-1 owns it) |
| W07-DV05 | P7-V15 | Pause/resume race tests | Interleave pause, wake (P7-W06), dispatch, resume | No lost pause, no lost event, no duplicate run (L-2), exactly one disposition per exit (F-4) | Race safety within tested interleavings; W11 amplifies density |
| W07-DV06 | P7-V16 | Terminal exclusion | Stop and fault vCPUs; attempt entry, enqueue, resume | Terminal vCPUs are never admitted, enqueued, or resumed; attempts produce explicit denials/refusals | Structural exclusion (F-3); not restart semantics (Out of Scope) |
| W07-DV07 | P7-V16 | Guest-fault containment | Fault vCPU A of VM-1 while VM-2's vCPUs run; compare VM-2 scheduling state and counters | VM-2 scheduling state and accounting unchanged; A excluded; no global panic (P-0) | Containment of the tested fault classes; not all future Guest error policy (P4-W06 boundary) |
| W07-DV08 | P7-V16 | Authorization ordering | Attempt pause/resume/stop with no-right, wrong-type, stale, revoked authority (P5 classes) | Denial precedes any state effect; no role/VM-ID bypass; outcomes are the P5 controlled denial classes | Capability-gated lifecycle control; not P5's own semantics (A-1) |
| W07-DV09 | P7-V19/V20 | Accounting coherence at hooks | Count pause/resume/stop/fault events against F-5 hooks | One committed event, one record; stop-running reason distinguishable from fault | Hook placement; full W09 semantics owned by P7-W09 |
| W07-DV10 | W07 closure | Scope-exclusion review | Inspect the delta for management API, fault policy, restart, snapshot logic | None present | Boundary discipline; not downstream correctness |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. QEMU-based rows
require the P1/P4 baselines; without them they are recorded not run with the
owning stage named. No row proves P7-V17+ or any W06-external wakeup property,
and none may be reported as doing so.

## 2. Error, security, and observability model

- **Errors.** Control errors are the P5 controlled denial classes plus the
  lifecycle WrongState class — all explicit, none silent, none partially
  applied. The fault path distinguishes GuestFault (VM-facing, contained,
  terminal for the vCPU in P7) from Hypervisor invariant failure (P0-W14
  fatal path) exactly at the F-0 classification boundary; W07 never reclassifies.
  Completion states (PausePending, Complete) are normal observable states,
  not errors.
- **Security.** Every lifecycle control action is capability-authorized
  (A-1) against a P5-validated handle (A-2) before any effect; denials leak
  only their class, not object state. The guest-fault reaction is
  intentionally unauthorized but grants nothing — it only removes the
  faulting vCPU (ADR-019 containment). Guests cannot pause, stop, or resume
  another VM's vCPUs without authorized capability; control paths allocate
  nothing from Guest-influenced sizes and bound all iteration.
- **Observability.** Every committed control/fault event fires exactly one
  F-5 hook with cause, disposition, and authority outcome (granted/denied);
  pause-pending/completion states are queryable for diagnostics; failure
  diagnostics (a stuck Pending completion, an unexpected denial storm) are
  reported through the P7-W09 diagnostic snapshot with the VM/vCPU/state/
  generation context. Trace fields register under the P0-W13 scheduler
  namespace via P7-W09; release trimming follows P0-W12.

## 3. Handoff checklist

Before handing W07 to a reviewer and to consumers:

- Changed-file list respecting the module boundary of
  [01](01-pause-stop-fault-architecture.md) §1 (no capability store, P4
  classification, or runqueue files modified).
- Assumed-contract status table from workflow step 1, including deltas and
  blocked prerequisites (especially any L-1 edge gap).
- W07-DV01–DV10 status with evidence paths and explicit not-run entries.
- Confirmation: no new `unsafe`, no ABI/public API, no new dependency, no
  management surface, no VM fault policy.
- Seam inventory handed on: W08 (remote postcondition, completion-detector
  duty on exit paths, idle honoring control reconsideration), W09 (F-5 hooks,
  SwitchReason entries), W10 (pause/stop/fault scenario contract, guest-
  observable fault consequence), W11 (race classes, containment invariants).
- Open items for consumers (not resolved here): W08 owns transport and idle
  ordering; W09 owns record/trace semantics; W11 owns race-density evidence.
- Record locations: implementation notes to
  `../p7-w07-pause-stop-fault-record.md`; evidence to
  `../../verification/p7-w07-pause-stop-fault-verification.md` — both created
  only when work or evidence exists.
