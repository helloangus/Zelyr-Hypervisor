# P7-W09 Accounting Model: Records, Coherence, Aggregation

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W09 detailed design](README.md).

## 1. Logical module boundary

W09 contributes one observability logical module — the **scheduler
accounting/diagnostic recorder** — plus per-object record structures embedded
in the objects other designs own. It owns no scheduling decision and no
transition.

| Unit | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| Record definitions & update rules | Define every counter, its width, writer class, and unit | Record types only | Hook calls from producer paths | Updated records | When hooks fire (producers decide) |
| Time-accumulation engine | Turn monotonic reading pairs into accumulated durations | None (writes into records) | Hook calls carrying monotonic stamps | runtime/ready-wait/blocked/idle accumulators | The monotonic clock itself (P6-W05) |
| Trace event set | Declare scheduler events and fields under P0-W13 governance | Event/field declarations | Hook calls | Trace emissions through the P0-W12-governed channel | Encoding, buffering, transport |
| Diagnostic snapshot & report | Maintain bounded recent-transition records; render the failure report | Per-vCPU snapshot records | Hook calls; failure invocation | The P7-V21 diagnostic content | Crash dumps; Guest-controlled text |

## 2. Assumed upstream contracts and failure boundaries

All unevidenced in the current tree. Failure boundary per row: **stop the
affected step, record for P7-W01 reconciliation (P7-V01); Architecture Change
Request on contradiction — never patch locally.**

| ID | Assumed contract | Source (plan path) | Fails if |
|---|---|---|---|
| T-1 | Diagnostic channel semantics: stable log levels, human-log vs structured-trace vs metrics separation, release-trimming rules, version identity association | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md) | W09 outputs have no channel classification to attach to; do not invent channels |
| T-2 | Trace namespace governance: scheduler domain exists, naming/versioning/compat rules, new-event review | [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) | Events cannot be registered without ad-hoc strings — prohibited |
| T-3 | Monotonic per-pCPU time basis with entry/exit monotonicity | [P6-W05](../../../p6/plans/p6-w05-el2-generic-timer.md) | Durations cannot be computed without regression risk; do not substitute another clock |
| T-4 | CPU-attribution conventions for observability (hardware + logical identity) | [P3-W11](../../../p3/plans/p3-w11-smp-observability.md) | pCPU attribution diverges from the established convention |
| T-5 | P4 event categories and VM/vCPU/PC/IPA correlation conventions | [P4-W07](../../../p4/plans/p4-w07-repeatability-telemetry.md) | Scheduler events cannot be correlated with P4 exit/fault events |
| H-1..H-5 | Producer hook points: W02 lifecycle transitions, W04 switch/preemption outcomes, W06 B-5 block/wake hooks, W07 F-5 control/fault hooks, W08 R-4/I-5 SMP hooks | [P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md)..[P7-W08](../p7-w08-smp-reschedule-idle/README.md) designs | A scheduler event occurs with no hook; coverage is incomplete and P7-V19 untestable |
| T-6 | P5-W09 telemetry-safe-logging regression constraints (logging under untrusted influence stays safe) | [P5-W09](../../../p5/plans/p5-w09-telemetry-safe-logging-regression.md) | Diagnostics could become a Guest-driven flooding/injection channel |
| T-7 | P6-W13 telemetry/latency-correlation handoff facts and limits | [P6-W13](../../../p6/plans/p6-w13-telemetry-regression-handoff.md) | IRQ→wake latency correlation breaks with the established P6 basis |

## 3. Record inventory

Field lists are semantic content; representation follows the Coding
Guidelines (typed newtypes for IDs, checked arithmetic, no raw struct
serialization as contract).

**VcpuAccounting** (one per vCPU; embedded in the vCPU's scheduler state):

| Field | Meaning | Writer | Notes |
|---|---|---|---|
| `guest_runtime` | accumulated time executing Guest code | owning pCPU at exit/disposition hooks | monotonic duration pairs (T-3) |
| `ready_wait` | accumulated time Runnable but not Running (steal-time basis, decision 4) | pCPU making the selection decision, at dispatch | measured from enqueue/wake to dispatch |
| `blocked_time`, `paused_time` | accumulated time in the state | transition hooks (block commit, pause commit, resume/wake) | |
| `switches_in`, `switches_out` | dispatch/extraction counts, by `SwitchReason` | exit-path disposition hook (W04/W06/W07 F-4) | one increment per disposition |
| `preemptions` | extractions against a still-runnable vCPU by deadline or reconsideration | disposition hook | subset of `switches_out` |
| `blocks` | committed Blocked transitions | W06 B-1 hook | |
| `wakes` | wake outcomes by source and outcome class | W06 W-1 hook | coalesced events count once at wake |
| `pcpu_changes` | count of home-pCPU changes | placement/enqueue path | distinguishes affinity-permitted moves |
| `last_pcpu` | most recent owning pCPU | dispatch hook | diagnostic, racy-read declared |

**PcpuAccounting** (one per pCPU; embedded in the per-pCPU scheduler context):

| Field | Meaning | Writer |
|---|---|---|
| `busy_time`, `idle_time` | accumulated executing/idle durations | decision-entry and idle hooks |
| `idle_periods`, `spurious_wakes`, `idle_refusals` | idle behavior counts | I-1/I-3/I-4 hooks |
| `context_switches` | entity changes on this pCPU | disposition hook |
| `reconsider_sent`, `reconsider_recv`, `reconsider_refused` | R-1/R-2/R-3 outcomes | R-4 hooks |
| `handler_max_iterations` | high-water mark of R-2's bounded loop | R-2 hook |
| `rq_depth_last`, `rq_depth_max` | eligible-work depth observed at each decision entry | decision-entry hook (event-driven, not periodic) |
| `deadline_events` | scheduler-relevant deadline firings | W06 W-3 / W04 hooks |

**VmAccounting** (computed, never stored): the sum over member vCPUs of
runtime, switches, blocks, wakes, preemptions, plus member-state counts and
the pause-generation state from P7-W07. Computed on read by iterating members
under declared staleness rules; a stored aggregate could diverge and is
prohibited.

## 4. Coherence rules (P7-V19, P7-V04)

1. Exactly one writer class per field (§3 tables); any second writer is a
   review failure.
2. Increments happen only inside the declared hook points; no convenience
   increments at call sites.
3. All accumulators are unsigned, monotonically non-decreasing; 64-bit width;
   an observed decrease is an invariant violation surfaced per the P0-W14
   boundary, not clamped silently.
4. Durations are differences of T-3 monotonic readings taken at hook
   boundaries; the reading pair is taken atomically with the state change the
   hook records (same critical section) so accumulation matches the
   transition, not the logging.
5. Cross-CPU reads are declared eventually-consistent with bounded staleness
   (one transition's worth); no reader performs read-modify-write on another
   pCPU's record.
6. Event counting vs. coalescing is fixed per producer contract (e.g. W06
   W-1 counts coalesced sources once at outcome time), so stress storms
   cannot make counters diverge from observable behavior.

## 5. Aggregation and observation

- VM aggregation (computed on read) serves P7-V19's VM-level rows and W11/W13
  consumers; the iteration bound is the fixed VM member count (no dynamic
  VM/vCPU creation in P7, per the task book Reserved list).
- Counter observation surfaces: diagnostics (human channel, T-1) and the
  structured snapshot emitted on failure; no management protocol is designed
  (Out of Scope) — P7-W14 documents what exists, P7-W13 consumes what is
  recorded.
- Every observation is associated with the build/version identity per T-1
  (P0-W16 alignment is P0's; W09 only carries the association point).

## 6. Hook-point inventory (produced by sibling designs, consumed here)

| Hook | Producer design | Records fed |
|---|---|---|
| lifecycle transition commit | [P7-W02](../p7-w02-scheduler-admission-lifecycle/README.md) (assumed H-1) | state-time accumulators, switches |
| exit disposition (`resolve_exit_disposition`) | [P7-W07](../p7-w07-pause-stop-fault/README.md) F-4 / W04 deadline path | switches by reason, preemptions, guest_runtime |
| block/immediate-wake (`on_vcpu_blocked`, `on_wake_immediate`) | [P7-W06](../p7-w06-block-wakeup/README.md) B-5 | blocks, blocked_time |
| wake outcome (`post_wake_event`) | P7-W06 W-1 | wakes by source/outcome |
| control/fault commits (`on_pause/stop/fault_committed`) | P7-W07 F-5 | paused_time, switches by reason |
| reconsideration (`on_reconsideration_*`) | [P7-W08](../p7-w08-smp-reschedule-idle/README.md) R-4 | reconsider counters |
| idle (`on_idle_entered/exited`, spurious, refused) | P7-W08 I-5 | idle counters |
| deadline firing | P7-W06 W-3 / W04 | deadline_events |

If any producer design renames or relocates its hooks, the rename is
reconciled there and recorded in the W09 implementation record; W09 never
re-implements a producer's transition to "also count it".
