# P7-W05 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W05 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none is claimed to have run. Requirement
ids map to the P7 task book §6.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W05-DV01 → P7-V10–V12 | prerequisite/seam review | inspect W01 register rows and the Step-1 record; check W02–W04 contracts | every seam named with plan path and failure boundary; scenario feasibility per register status recorded | assumed-contract basis explicit; not that prerequisites are implemented |
| W05-DV02 → P7-V10 | ADR-016/057 conformance review | review modules against [scope and foundations](01-scope-and-foundations.md) §4/§5 | one entity type; VM never schedulable; policy behind the seam; no Reserved features present | conformance of the design; not performance or final-algorithm suitability (ADR-057 open) |
| W05-DV03 → P7-V10 | queue/policy unit + property tests | host: FIFO order, duplicate/capacity/remove, rotation no-starvation, enqueue rule matrix | discipline holds under random interleavings; rotation gives every runnable entity turns; rule matrix exact | queue/policy correct as specified; not multi-pCPU or Guest-visible behavior |
| W05-DV04 → P7-V10 | loop simulation | host: `run_scheduler_loop` against fakes, all branches incl. rejection bounding and idle hook | every iteration terminates in a defined outcome; no unbounded retry; fairness-window simulation satisfied | loop logic correct as specified; not target behavior |
| W05-DV05 → P7-V10 | M:N matrix on target | QEMU: S1 (1:2), S2 (2:4), S3 (4:8) with W10 workloads; invariant sampling during runs | all vCPUs progress with zero duplicate or non-runnable execution events; checker clean throughout | M:N overcommit works on target at declared topologies; not stress-envelope (W11) or fairness proportions |
| W05-DV06 → P7-V11/V12 | multi-VM + fairness on target | QEMU: S4 (two VMs, CPU/HVC/WFI mix) and fairness-window assertions across S1–S4; S5/S6 as they become runnable with W08/W06 land | every equal-class continuously runnable vCPU accumulates positive scheduled time in every window; neither VM loses opportunities while eligible vCPUs exist | basic no-starvation fairness and multi-VM progress; no proportional-fairness claim; race correctness is W06/W08/W11 evidence |
| W05-DV07 → W05 closure | consumer consumability review | read the design as W08 (idle hook, enqueue-else obligation, request point), W10 (scenario workloads), W11 (fairness/assertion hooks), W12 (matrix automation) | each consumer finds its seam and obligations named | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command/input, reviewer, environment, timestamp, and reason.
Host passes do not substitute for the QEMU rows; nothing here proves
P7-V02–V09 or P7-V13–V30.

## 2. Error, security, and observability model

**Errors.** Recoverable: gate rejections (requeue, bounded), activation
failures (W04 containment passthrough), non-eligible enqueue targets
(`NoneEligible` + W08 obligation). Programming-error class: duplicate or
non-runnable enqueue attempts are rejected by the discipline and traced at
audit grade. Invariant-grade: capacity overflow at runtime, discipline
bypass — escalate via the W02 audit/fatal path. No W05 failure path panics
directly; terminal classification follows the P0 policy through the audit
discipline.

**Security.** The loop admits only through the W02 gate with W03
eligibility — queue position never substitutes for authorization or
eligibility ("affinity 生效" is enforced at both enqueue and gate). No
Guest-influenced input reaches the loop; the scheduler cannot be
re-prioritized by a Guest no matter how it shapes its exits, because v0
has no weight/priority input at all (a security property of the minimal
policy). Queue ownership prevents cross-CPU tampering with another pCPU's
ready set.

**Observability.** Trace points: enqueue/requeue (with target and depth),
dispatch, deschedule (W04 reason taxonomy), idle-hook entry/exit,
rejection counters, fairness-window verdicts (instrumented builds). These
give W09 queue-depth and switch-mix material and give W11/W12 objective,
mechanical pass conditions. A stuck or starved vCPU is diagnosable as:
state + placement + queue membership + last dispatch/deschedule marks
(P7-V21 input via W09).

## 3. Handoff checklist

Before handing W05 to a reviewer, provide:

- the exact changed-file list (expected: entity/queue/policy/enqueue/loop
  modules, instrumentation, tests per the approved layout);
- W05-DV01…DV07 evidence paths and run status, including explicit not-run
  and blocked-with-reason entries (expected at design acceptance: S2–S6
  target rows awaiting W06/W08 dependencies);
- the recorded `DEFAULT_SLICE` (with W04), fairness-window `SLACK`
  derivation, and queue-capacity constant with their rationale;
- confirmation: no new `unsafe`, dependencies, public API, or Reserved
  features (weights/priorities/stealing/balancing); one enqueue path; VM
  never schedulable;
- confirmation: the scenario table and fairness condition are consumed
  verbatim by the instrumentation (no per-scenario constant tuning); and
- open items: W08 (idle hook semantics, remote reschedule point,
  wake-on-eligibility obligation), W10 (S1–S6 workload coverage), W11
  (stress + fairness campaigns), W12 (automated matrix), W09 (trace/
  accounting rendering), W14 (semantics into the P8 handoff) — without
  resolving their contracts here.
