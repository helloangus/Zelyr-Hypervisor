# P7-W01 Input Boundary Register Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W01 detailed design](README.md).

This file defines the artifact W01 produces: its schema, its input inventory,
its evidence-status model, its blocked-prerequisite procedure, and the
normal-entry boundary statement. The produced artifact is the "Input boundary
register" section of `../p7-w01-entry-contract-reconciliation-record.md`
(created when W01 work starts). That section is the sole authoritative home of
P7 input-contract statements; every other P7 document cites it by `P7-IN-xx`
id.

## 1. Logical artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Input boundary register | "Input boundary register" section of the W01 implementation record | task book §2 predecessor table, upstream handoff plans, this contract | per-input P7 consumption statements, evidence status, failure boundaries; it does not restate upstream contracts beyond locating and summarizing them for P7 use |
| Reconciliation review evidence | W01 verification record (created when the review runs) | actual register content, reviewer findings | run/not-run review results per the validation matrix; not part of this design |
| Entry-boundary statement | register §E (inside the register section) | task book §2 Required bullet, P4 handoff facts | the authorized bypass classes and the rule that instances are enumerable; it does not design the admission gate (W02) |
| Implementation record narrative | rest of the W01 implementation record | actual decisions and deviations | changed-artifact list, deviations; no command logs (those live in verification) |

The artifact named in the second column is the sole authoritative home for the
statement in its row. No P7 document may duplicate or contradict it.

## 2. Register schema

Each register row has exactly these seven columns:

| Column | Content rule |
|---|---|
| `ID` | stable id `P7-IN-nn`; never renumbered or reused |
| `Required P7 input` | the task book §2 wording for that predecessor row, quoted or tightly paraphrased |
| `Source authority` | the upstream plan (and, when evidence exists, the handoff/verification record) by repository path |
| `Evidence status` | exactly one of the §4 states, with a linked record when not the default |
| `P7 consumption` | the semantic contract P7 relies on, one to three sentences, mechanism-level only |
| `Failure boundary` | what the consuming P7 packages must do if the input is absent, contradictory, or delivered differently |
| `Primary P7 consumers` | P7-Wxx ids from the plan index consumer map |

Rules: the register is a table, not prose; a row must be reviewable in one
pass; a row must not promise upstream behavior beyond the cited source; a row
must not be marked `available` without a linked, dated record.

## 3. Input inventory (P7-IN-01 … P7-IN-09)

The rows below define the required content of each register row. The
implementer transfers them into the register, verifies each cited path
resolves, and sets the evidence status per §4. Wording here is design
guidance for the register row; the register row is authoritative once
written.

### P7-IN-01 — P0 engineering baseline

- **Source authority:** [P0 task book](../../../../stages/p0/task-book-v0.1.md)
  and the P0 plan index; the P0-W01 repository-baseline record is the only
  implementation-stage evidence tracked today.
- **P7 consumption:** repeatable build/test/QEMU entry points, CI gates,
  unsafe and dependency governance, and the diagnostics/trace namespace. P7
  scheduler code is `no_std`, records any new `unsafe` per governance, and
  emits events only inside the P0 trace namespace.
- **Failure boundary:** if build/test entry or governance differs from the P0
  contracts, all P7 code-bearing packages are blocked; record per §5.
- **Primary P7 consumers:** all W01–W14.

### P7-IN-02 — P1 EL2 runtime and exception boundary

- **Source authority:** [P1 task book](../../../../stages/p1/task-book-v0.1.md);
  P1-W04/W05/W07 plans (EL2 architectural state, exception entry, fatal
  crash diagnostics).
- **P7 consumption:** a stable Non-secure EL2 runtime in which the scheduler
  control loop runs; exception entry that regains EL2 control on Guest exit
  and timer IRQ; a fatal-diagnostic boundary P7 never redefines.
- **Failure boundary:** if EL2 exception entry or fatal diagnostics cannot
  regain EL2 control as assumed, P7-W02 (admission), P7-W04 (preemption), and
  every runtime package are blocked.
- **Primary P7 consumers:** W02, W04, W06–W09.

### P7-IN-03 — P2 platform facts, memory, allocation

- **Source authority:** [P2-W10 P3/P4 handoff contract](../../../../stages/p2/plans/p2-w10-p3-p4-handoff-contract.md):
  CPU inventory, platform facts, allocatable-memory and capability inputs.
- **P7 consumption:** normalized platform CPU topology (used for the
  placement capacity bound and pCPU identity space) and allocator services
  for scheduler objects; P7 never branches on platform names (ADR-043/052).
- **Failure boundary:** if topology facts or allocation services differ,
  P7-W03 (placement capacity/identity) and object creation in W02/W05 are
  blocked.
- **Primary P7 consumers:** W02, W03, W05.

### P7-IN-04 — P3 SMP, pCPU registry, synchronization, notification

- **Source authority:** [P3-W14 P4 SMP handoff](../../../../stages/p3/plans/p3-w14-p4-smp-handoff.md)
  and P3-W03/W04/W06/W07 plans: current logical pCPU identity, per-CPU state
  (with capacity reserved for scheduler/current-vCPU needs), shared-state
  protection and lock-order rules, cross-CPU notification/completion,
  CPU-attributed faults, telemetry rules.
- **P7 consumption:** the online-pCPU registry and lifecycle states that
  placement eligibility is checked against; per-CPU local state slots the
  per-pCPU scheduler attaches to; the synchronization baseline the scheduler
  lock discipline extends; the cross-CPU notification transport remote
  reschedule requests ride on.
- **Failure boundary:** if pCPU identity, registry states, lock-order rules,
  or notification transport differ from the cited contracts, P7-W03
  (eligibility), P7-W05 (per-pCPU run-queue ownership), and P7-W08 (remote
  reschedule, idle) are blocked.
- **Primary P7 consumers:** W02, W03, W05, W08.

### P7-IN-05 — P4 VM/vCPU objects, entry/exit, Stage-2 identity, fault boundary

- **Source authority:** [P4-W09 closeout/P5 handoff](../../../../stages/p4/plans/p4-w09-closeout-p5-handoff.md)
  covering the evidenced Stage-2/Guest-entry/fault facts, the Validation
  Guest asset, fault diagnostics, and QEMU regression; P4-W04 plan for the
  single-vCPU entry/return authority and stop result.
- **P7 consumption:** independent VM and vCPU objects with arch context
  save/restore; a Guest entry/return mechanism P7 can invoke per dispatch;
  Stage-2/VMID address-space identity P7 activates per switch; the
  exit/fault boundary that classifies what returns to EL2. P7's admission
  gate supersedes P4's direct entry for normal operation; P4's direct path
  remains only inside the §6 bypass classes.
- **Failure boundary:** if vCPU context save/restore, entry/return, or
  Stage-2 identity semantics differ, P7-W02 (admission), P7-W04 (switch
  isolation), and P7-W05 (M:N rotation) are blocked.
- **Primary P7 consumers:** W02, W04, W05, W07.

### P7-IN-06 — P5 handles, capability rights, guest-failure classification

- **Source authority:** [P5-W10 closeout/P6 handoff](../../../../stages/p5/plans/p5-w10-closeout-p6-handoff.md):
  evidenced HVC ABI, object-handle/right, controlled Guest-data, structured
  telemetry and regression facts.
- **P7 consumption:** the capability/handle + rights + generation model that
  authorizes lifecycle control actions (pause/resume/stop, configuration
  entry points); the Guest-failure classification that decides whether an
  event faults a VM (`Faulted`) or is a hypervisor invariant violation.
- **Failure boundary:** if the rights model has no right suitable for
  scheduling-policy control, or the failure taxonomy lacks a VM-facing fault
  class, the authorization hooks of P7-W03 and the containment of P7-W07 are
  blocked pending an explicit upstream extension (recorded per §5, not
  invented in P7).
- **Primary P7 consumers:** W02, W03, W07.

### P7-IN-07 — P6 per-pCPU EL2 deadline timer and vCPU timer

- **Source authority:** [P6-W13 telemetry/regression/handoff](../../../../stages/p6/plans/p6-w13-telemetry-regression-handoff.md)
  and the P6-W05 plan: per-pCPU monotonic time and deadline-event mechanism
  with arm/cancel/rearm behavior; vCPU virtual timer save/restore.
- **P7 consumption:** the per-pCPU deadline event that realizes preemption
  deadlines (P7-W04), and per-vCPU virtual-timer state that must be preserved
  across switches (P7-W04 isolation classes). P7 owns no timer programming.
- **Failure boundary:** if no armable per-pCPU deadline mechanism or no
  per-vCPU timer save/restore is delivered, P7-W04 is blocked (timer
  preemption and switch isolation cannot be built on other mechanisms
  without a P6 scope change).
- **Primary P7 consumers:** W04, W06 (wakeup deadlines), W09 (latency
  telemetry).

### P7-IN-08 — P6 vIRQ/event delivery and maintenance behavior

- **Source authority:** [P6-W13](../../../../stages/p6/plans/p6-w13-telemetry-regression-handoff.md)
  and P6-W07/W08/W10 plans: vIRQ lifecycle (pending through completion),
  unavailable-vCPU outcomes, GIC virtualization interface and maintenance
  behavior.
- **P7 consumption:** pending vIRQ/event state preserved per vCPU across
  scheduling, and the event signals that may wake a blocked vCPU. P7 does
  not redesign injection, LR, or maintenance mechanics.
- **Failure boundary:** if pending state is not preserved per vCPU across
  arbitrary descheduling, P7-W04 isolation and P7-W06 wakeup contracts are
  blocked.
- **Primary P7 consumers:** W04, W06, W07.

### P7-IN-09 — P6 timer/IRQ telemetry and latency method

- **Source authority:** [P6-W13](../../../../stages/p6/plans/p6-w13-telemetry-regression-handoff.md):
  structured Host IRQ/timer/vIRQ/maintenance telemetry and the latency
  measurement method.
- **P7 consumption:** the telemetry style and correlation fields that
  scheduler events (dispatch, deschedule, switch reason) must blend with;
  the QEMU integration/regression pattern W12 follows.
- **Failure boundary:** telemetry mismatch degrades W09 observability but
  does not block mechanism work; a mismatch that prevents correlating
  scheduler events at all is recorded per §5 and blocks W09/W12/W13.
- **Primary P7 consumers:** W09, W12, W13.

## 4. Evidence-status model and maintenance rules

Each row's `Evidence status` is one of:

- `contract-mapped (evidence pending)` — the default today: the cited plan
  defines the contract; no implementation/verification record yet evidences
  it. Runtime packages must treat the row as an assumed contract.
- `available (evidence linked)` — a dated implementation/handoff or
  verification record exists at a linked path and matches the row's
  consumption statement.
- `blocked (issue linked)` — the input is absent, contradictory, or
  unevidenced where the task book requires evidence; an issue record is
  linked and the affected packages are stopped.

Transitions: the first state moves to `available` only by linking evidence
reviewed against the row; it moves to `blocked` only per §5. A row never
moves backward from `available` without a new §5 issue. The register records
the date and reviewer of every status change.

Maintenance: adding a row is a reviewed change to the W01 record; removing,
renumbering, or re-scoping a row requires a task-book change (the rows mirror
the task book §2 table one-to-one). W14 consumes the final column states for
the P8 handoff.

## 5. Blocked-prerequisite handling procedure

When reconciliation, implementation, or validation discovers an input that is
absent, contradictory, or unevidenced contrary to its row:

1. **Stop** the affected P7 packages; do not proceed on the assumed contract
   and do not repair the predecessor in P7 (task book §2 rule).
2. **Record** in the W01 record: the row id, the observed mismatch, the
   citing paths, and the affected P7-Wxx packages; set the row's status to
   `blocked (issue linked)`.
3. **Label** the issue `Architecture Change Request` when the fix changes a
   stage boundary or an established module contract, or `ADR Required` when
   it would alter an accepted ADR decision; otherwise record it as a blocked
   prerequisite awaiting upstream evidence.
4. **Notify** consumers: the blocked row's `Primary P7 consumers` are named in
   the issue; their designs and records must reference the issue, not the
   stale assumption.
5. **Resume** only when the upstream contract or evidence lands and the row
   returns to a reviewed state; the resumption decision and date are
   recorded.

The procedure never edits an upstream document from P7 and never silently
narrowed a row's consumption statement to make work proceed.

## 6. Normal scheduler-controlled Guest-entry boundary

The register's final section records the boundary statement W02 must enforce:

> All normal Guest entry and return-after-exit is scheduler-controlled: a
> vCPU enters Guest EL1 only through the P7 admission gate after scheduler
> dispatch, and every Guest exit returns through the scheduler exit boundary
> before any re-entry. Guest entry outside this boundary is an invariant
> violation unless the path belongs to one of the registered bypass classes
> below.

Bounded bypass classes (from the task book Required bullet). Each class is a
name and a scope rule; W02's design must enumerate the concrete instances it
relies on against these classes, and an instance outside these classes is a
review failure:

- **E1 early-boot:** execution on the bootstrap and secondary pCPUs before
  the P7 scheduler is activated (P1/P3 bring-up; no Guest entry occurs in
  this class).
- **E2 bring-up:** bounded non-scheduler Guest-entry paths inherited from P4
  stage sequencing, usable only before scheduler activation and only for the
  P4 first-guest bring-up asset; after scheduler activation this class is
  empty.
- **E3 emergency-debug:** fatal-diagnostic and emergency paths that regain or
  inspect EL2 state when the scheduler itself cannot run (P1 crash
  boundary); they must not resume normal Guest execution.

W01 owns this class list and its review; W02 owns the gate, the activation
transition that ends E1/E2 availability, and the enumeration of instances.
The register also records the compatibility finding for P7-IN-05: P4's
direct-entry path is compatible because it is confined to E2 by P7's
boundary.
