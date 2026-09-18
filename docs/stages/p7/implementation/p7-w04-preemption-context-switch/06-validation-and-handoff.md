# P7-W04 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W04 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none is claimed to have run. Requirement ids
map to the P7 task book §6.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W04-DV01 → P7-V08/V09 | prerequisite/seam review | inspect W01 register rows P7-IN-05/07/08 and the Step-1 record | every assumed seam named with plan path and failure boundary; none blocked for W04 | assumed-contract basis explicit; not that P4/P6 are implemented |
| W04-DV02 → P7-V08 | slice/deadline unit tests | host: `TimeSlice` boundaries, policy-source validation, arm/cancel state machine, spurious-expiry rule, arm-failure asymmetry | all boundaries enforced without clamping; at most one armed deadline per pCPU; asymmetry behaves as specified | deadline bookkeeping correct as specified; not Guest-visible preemption yet |
| W04-DV03 → P7-V08 | preemption-on-target test | QEMU: CPU-bound (non-exiting, non-blocking) Validation Guest on a shared pCPU; observe deadline exits and reconsideration | a CPU-bound Guest returns to EL2 after slice expiry each time; it cannot monopolize the shared pCPU; other eligible vCPUs progress; switch trace carries `SliceExpired` | timer preemption works end-to-end on target; not fairness proportions or performance (W05/W13) |
| W04-DV03b → P7-V08 | reconsideration-trigger coverage | QEMU: repeated exits of each exit class (HVC, WFI-block, IRQ) plus injected reschedule requests | every trigger leads to a reconsideration decision; no dropped intent (loop reconsiders after each take) | trigger set closed and effective; not wakeup-race correctness (W06/W08 evidence) |
| W04-DV04 → P7-V09 | switch-sequence host proof | host fakes: ordering tests (quiesce→gate→activate→arm→enter), all-or-nothing activation, failure containment per stage, isolation trace marks | sequence never violates an ownership rule; activation failure leaves candidate requeued and pCPU idle-capable; repeated-failure escalation fires | sequencing and containment correct as specified; not hardware behavior |
| W04-DV05 → P7-V09 | switch-isolation test (target) | QEMU: A→B→C→A rotation across two VMs with W10's counter/canary/timer/IRQ workloads; W02 checker sampled during rotation | registers/PC/PSTATE continuity per Guest; no cross-VM address-space observation; timer and vIRQ/event state correct for each vCPU after arbitrary interleaving; scheduler invariants hold at samples | declared per-vCPU state is isolated across repeated switches on target; not stress-envelope correctness (W11) or full scheduler policy (W05) |
| W04-DV06 → P7-V08/V09 | failure-boundary and observability review | review the failure model ([architecture and state](02-architecture-and-state.md) §6) against P4/P5 taxonomy; verify `DescheduleReason`/switch trace rendering seam with W09 | no failure path marked `Faulted` for non-Guest causes; every switch carries a reason; diagnostics name stage and owner | failure and trace semantics coherent; rendering quality is W09 evidence |
| W04-DV07 → W04 closure | consumer consumability review | read the design as W05 (loop/trigger/slice seams), W07 (remote pause realization), W09 (reasons/trace), W10 (isolation classes) | each consumer finds its seam named and its obligations listed | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command/input, reviewer, environment, timestamp, and reason.
Host passes do not substitute for QEMU rows; nothing here proves P7-V02–V07
or P7-V10–V30.

## 2. Error, security, and observability model

**Errors.** Recoverable: gate rejections (requeue/idle), activation failures
per stage (candidate containment), deadline arm failures (shared abort /
pinned degrade), spurious expiry (benign). Guarantee: every failure names
its stage and owner, leaves `from`-handling intact, and never marks a
non-Guest failure as `Faulted`. Fatal: invariant-grade mismatches (slot
disagreement, entry failure after activation) per the P0 panic policy.

**Security.** No Guest-influenced input participates in any W04 contract;
the sequence consumes predecessor mechanisms only and touches no registers,
timer registers, GIC/LR state, or VMID internals. The monopolization
guarantee is a security-relevant property: a shared-pCPU dispatch without
an armed deadline aborts by design, so a scheduler or timer bug fails
closed (no Guest runs unpreempted on a shared pCPU) rather than open. The
pinned degrade path is safe exactly because pinned operation is equivalent
to the static baseline that never had a scheduler deadline.

**Observability.** Trace points: deadline armed/expired/cancelled,
trigger observed, switch completed/failed (with stage and reason),
deschedule reason. Together these let diagnostics answer: who was
preempted, by what trigger, after how many switches, and which activation
stage failed (P7-V20/V21 inputs via W09 rendering). The switch rate and
reason mix are W13's baseline material — W04 provides the points, never
performance conclusions.

## 3. Handoff checklist

Before handing W04 to a reviewer, provide:

- the exact changed-file list (expected: preemption/switch modules, fakes,
  tests per the approved layout; no crate/ABI additions);
- W04-DV01…DV07 evidence paths and run status, including explicit not-run
  entries (expected not-run at design acceptance: target rows awaiting
  integration and W10 workloads);
- the recorded `DEFAULT_SLICE` value, its selection rationale per the
  decision-3 rule, and its date;
- confirmation: no new `unsafe`, no timer-register or register access in
  scheduler modules, no queue-internals access, no public API, no trace
  encoding;
- confirmation: the ownership table
  ([architecture and state](02-architecture-and-state.md) §3) is satisfied
  step-by-step in code review; and
- open items: W05 (loop/trigger/slice seams), W07 (remote pause as
  reschedule + control event), W08 (`RescheduleRequest` surfacing), W09
  (reason/trace rendering), W10 (isolation workload coverage), W11
  (rotation stress), W14 (semantics into the P8 handoff) — without
  resolving their contracts.
