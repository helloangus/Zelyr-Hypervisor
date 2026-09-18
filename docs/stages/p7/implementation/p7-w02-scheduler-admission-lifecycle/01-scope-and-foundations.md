# P7-W02 Scope, Foundations, and Prerequisite Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W02 detailed design](README.md).

## 1. Goal-to-baseline ledger (detail)

The entry README carries the §1 ledger table. This section records the
per-outcome foundation analysis behind it, per the implementation-design
checklist.

### 1.1 What must concretely exist for the plan goal to be true

The plan goal is "a scheduler-controlled normal execution admission boundary
and an explicit, testable vCPU lifecycle." Making that true requires, at
minimum:

1. A named state type and a total transition table (every `(state, event)`
   pair resolves to a successor or a typed rejection) — without totality,
   "testable" is undefined.
2. Exactly one entry path (`admit_for_entry`) and one exit path
   (`complete_exit`) that all normal Guest execution crosses, plus a
   statically enumerable bypass register — otherwise "scheduler-controlled"
   cannot be reviewed.
3. A scheduler activation point that separates pre-scheduler bring-up
   (bypass classes E1/E2 legal) from steady state (empty bypass set).
4. A mechanical invariant checker for the single-running invariants,
   runnable host-side and usable as a QEMU assertion hook.
5. Trace emission points for every accepted and rejected transition, with
   semantics (not encoding) fixed so W09 can render them.

Items 1–5 are the foundation deliverables this design specifies; each is
missing today because nothing exists in the repository except documents.

### 1.2 Prerequisites supplied by other packages

| Prerequisite | Delivering package / plan path | What W02 assumes | Failure boundary if delivered differently |
|---|---|---|---|
| P0 build/test/governance/trace namespace | P0 plans (P7-IN-01); W01 register | host-side test entry, `unsafe` governance, trace event namespace | W02 blocked for code steps; record per W01 §5 procedure |
| EL2 runtime and exception boundary | P1 plans (P7-IN-02) | EL2 control is regained on Guest exit and timer IRQ before scheduler code runs | W02 gate is untestable on target; host-side contracts still proceed; integration blocked |
| pCPU identity, per-CPU state, sync primitives | P3-W14 + P3-W04/W06 (`../../../../stages/p3/plans/p3-w14-p4-smp-handoff.md`) | stable logical pCPU id, per-CPU slots for `current_vcpu`, leaf-lock-compatible spinlocks | W02 concurrency model is blocked; record ACR against P3 lock-order contract |
| VM/vCPU objects, entry/exit mechanism, fault boundary | P4-W09 + P4-W04 (`../../../../stages/p4/plans/p4-w09-closeout-p5-handoff.md`) | vCPU object with arch context; an entry mechanism the gate can call; exit classifies into `ExitOutcome` | W02 cannot bind gate to mechanism; blocked; do not redefine P4 boundary |
| Capability rights + failure taxonomy | P5-W10 (`../../../../stages/p5/plans/p5-w10-closeout-p6-handoff.md`) | a rights check upstream of control events; Guest-fault vs invariant classification | authorization hooks blocked (W03/W07 affected); classification blocked — record per procedure |
| Timer/IRQ events as wake sources | P6-W13 + P6-W05/W07 (`../../../../stages/p6/plans/p6-w13-telemetry-regression-handoff.md`) | bounded event callbacks that may raise `Wake`/IRQ-context events | only IRQ-context entry points blocked; core lifecycle proceeds |

Each row's mismatch is escalated through the W01 register procedure; W02
never repairs the predecessor.

## 2. Itemized scope classification

**Required (W02 owns and must deliver):**

- R1 `VcpuRunState` enum, exactly the ADR §4.1 states.
- R2 `LifecycleEvent` set: `MakeRunnable, Dispatch, Deschedule, Block, Wake,
  Pause, Resume, Stop, Fault`.
- R3 Total transition legality table with typed rejection.
- R4 Transition engine with leaf-lock discipline and IRQ-context entry
  points.
- R5 Admission gate `admit_for_entry`; exit boundary `complete_exit`.
- R6 Scheduler activation transition and the E1–E3 bypass instance register.
- R7 `TransitionContext` authority metadata (caller role + capability
  receipt reference) and its enforcement for control events.
- R8 Invariants INV-1…INV-5 and the checker.
- R9 Telemetry semantics (accepted/rejected transition, activation, bypass
  use) handed to W09.
- R10 Guest-fault vs invariant-violation classification seam consistent
  with the P5 taxonomy.

**Reserved (must remain possible, not designed now):** vCPU restart from
`Stopped`; dynamic vCPU creation producing `MakeRunnable` from a
not-yet-existing object; more lifecycle states for nested virtualization or
migration (ADR-020/023 long-term reservations). R1–R8 must not preclude
these, but no path, API, or state is added for them.

**Out of Scope:** everything listed in the README's classification plus:
scheduling policy of any kind (W05), placement rules (W03), timer/deadline
mechanics (W04), wakeup event sourcing (W06), pause/stop policy flows and
remote-running handling (W07), idle and remote reschedule (W08), counter
definitions and trace encoding (W09), guest workloads (W10), stress
campaigns (W11), QEMU matrix (W12), performance (W13), documentation/handoff
(W14), and any P8 mechanism.

## 3. Mechanism vs policy split (explicit)

| Concern | Classification | Owner |
|---|---|---|
| State set, transition legality | Mechanism (generic lifecycle authority) | W02 |
| Admission gate existence and semantics | Mechanism | W02 |
| Scheduler activation point | Mechanism | W02 |
| Bypass class instances and their enumeration | Mechanism boundary | W02 enumerates; W01 owns classes |
| Which vCPU runs next, in what order | Policy (P7 stage scope) | W05 |
| Whether `Deschedule` re-queues at head or tail | Policy | W05 |
| Slice lengths and deadline values | Policy | W04 (constants) / W05 (policy source) |
| When a vCPU blocks or wakes | Mechanism edges; event sourcing is policy-adjacent runtime behavior | Edges: W02; sourcing: W06 |
| Pause/stop/fault control flows | Mechanism edges; control policy (who may pause) is authorization | Edges: W02; authorization: P5 model via W07/W03 hooks |
| Scheduling-policy rights naming | Authorization contract | P5 rights model; W03/W07 consume |

This table is normative for reviewers: code placed in the lifecycle/admission
modules that encodes a policy choice from the lower half is a review failure.

## 4. Predecessor mapping without redefinition

### 4.1 P4 vCPU execution facts → P7 run states

| P4-era fact (P4-W04/W09 plans) | P7 state mapping |
|---|---|
| Configured vCPU not yet entered | `Offline` (configuration complete, `MakeRunnable` pending) |
| Guest executing between entry and exit | `Running` |
| Returned to EL2 awaiting re-entry (P4 re-entry loop) | `Runnable` under P7 (the scheduler, not the exit loop, decides re-entry) |
| Defined stop result (P4 stop path) | `Stopped` |
| Guest-caused fatal condition per P5 classification | `Faulted` |
| WFI/WFE-detected wait (P4 diagnostics) | `Blocked` edge — sourcing owned by W06; P4 only observes |

No P4 contract is redefined: P4 keeps its entry/exit/stop mechanisms; P7
routes normal use of them through the gate. The mapping is a compatibility
statement recorded for P7-V02 review, and an input to the W01 register
(P7-IN-05).

### 4.2 P3 pCPU registry → gate inputs

| P3 state (P3-W03 plan) | Gate meaning in W02 |
|---|---|
| `Online` | eligible host for dispatch (subject to placement, W03) |
| `Present`/`Initializing`/`Starting` | not dispatchable; no scheduler loop runs there yet |
| `Failed` | never dispatchable; vCPUs eligible on it are re-placed per W03/W08 rules |

The gate itself does not read the registry: it consumes the W03 eligibility
predicate, which encodes these rules. W02 fixes only that a pCPU without an
activated scheduler loop cannot appear in a successful `admit_for_entry`.

### 4.3 P5 failure taxonomy → `Fault` event sourcing

`Fault` is raised only by the fault-handling path when the P5/Guest-failure
classification decides the condition is VM-facing. Hypervisor invariant
violations never become `Fault`; they are fatal per the P0 panic policy.
This seam is asserted in review (W02-DV02) and exercised by W07/W11.
