# P7-W09 Accounting, Trace, and Diagnostics — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** vCPU/pCPU/VM scheduler accounting, structured trace events, switch
reasons, and failure diagnostics required by
[P7-W09](../../plans/p7-w09-accounting-diagnostics.md).  
**Owner/change context:** P7-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W09. It converts the bounded
work-package plan into concrete mechanics: the accounting records and
single-writer update rules that keep counters coherent across scheduler
events; the switch-reason vocabulary that makes every stop-running event
distinguishable; the scheduler trace-event set with its fields, registered
under the P0-W13 namespace governance; and the bounded, deterministic failure
diagnostic for scheduler faults — explicitly without a crash-dump claim. It
consumes the P0-W12 diagnostic-semantics and P0-W13 trace-namespace
contracts and does not restate or redesign them; it deliberately does **not**
design a metrics backend, buffer encoding, a crash-dump framework, a
management protocol, or any performance conclusion.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| Accounting model, coherence rules, ownership, aggregation | [01-accounting-model.md](01-accounting-model.md) |
| Implement accounting records and update points | [02-code-contracts-accounting.md](02-code-contracts-accounting.md) |
| Implement trace events, switch reasons, and failure diagnostics | [03-code-contracts-trace-diagnostics.md](03-code-contracts-trace-diagnostics.md) |
| Execute the ordered workflow | [04-implementation-workflow.md](04-implementation-workflow.md) |
| Plan or review validation and closure | [05-validation-and-handoff.md](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P7 task book](../../task-book-v0.1.md), the
[P7-W09 plan](../../plans/p7-w09-accounting-diagnostics.md), and
[P7-W01](../p7-w01-entry-contract-reconciliation/README.md)'s recorded input
boundary. This document is a proposed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W09 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-048 (telemetry is first-class, structured, compilable-trimmable,
  runtime-filterable) and ADR-049 (layered validation) shape everything here;
  observability is a designed interface, not accumulated prints (Plan Agent
  guardrail).
- The P0-W12 diagnostic contract
  ([plan](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)) owns log
  levels, the human-log/structured-trace/metrics separation, release
  trimming, minimum panic information, and version identity. W09 classifies
  its outputs into those channels; it does not redefine them.
- The P0-W13 trace-namespace contract
  ([plan](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md)) owns
  the event-domain classification (including the scheduler domain), naming,
  versioning, compatibility, and new-event review rules. W09 registers events
  under those rules; it does not invent a parallel naming scheme.
- The P3-W11 observability contract
  ([plan](../../../p3/plans/p3-w11-smp-observability.md)) owns CPU-attribution
  conventions; P6-W13
  ([plan](../../../p6/plans/p6-w13-telemetry-regression-handoff.md)) hands P7
  its telemetry/latency-correlation facts and limits.
- Task book §2: accounting must not regress or double-count (P7-V04); P7-V21
  demands failure diagnostics **without** a crash-dump claim.

Classification:

- **Required:** vCPU runtime/switch/preemption/block/wakeup/CPU-change
  counters; pCPU busy/idle/reconsideration counters; VM-level aggregation;
  the switch-reason vocabulary; scheduler trace events with required fields
  (VM/vCPU/pCPU/reason/time, wake source); the bounded recent-transition
  diagnostic snapshot; the scheduler failure report content; P0-W12/P0-W13
  compatibility; coherence guarantees; the P7-V19–V21 evidence plan.
- **Reserved:** a metrics aggregation backend, ring-buffer transport,
  per-guest steal-time ABI, lock-contention heat maps, live counters over any
  management interface, and runtime filter UIs — the record *shapes* (fields,
  semantics) are designed here; their transport/bulk plumbing is not.
- **Out of Scope:** metric backend, buffer encoding, crash-dump framework,
  management protocol, performance conclusions or KPIs (P7-W13 owns the
  baseline method), new diagnostic channels, and all P8+ mechanisms.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect lifecycle, switch, wakeup, pause observability inputs | [assumed contracts](01-accounting-model.md) §2 | [workflow](04-implementation-workflow.md) step 1 review |
| Required accounting and aggregation observations | [accounting model](01-accounting-model.md) §3–§5, [contracts](02-code-contracts-accounting.md) | P7-V19 |
| Trace/reason and scheduler-failure diagnostic content | [trace/diagnostic contracts](03-code-contracts-trace-diagnostics.md) | P7-V20, P7-V21 |
| Compatibility with P0 telemetry governance | [governance mapping](03-code-contracts-trace-diagnostics.md) §4 | [workflow](04-implementation-workflow.md) step 4 review |
| Evidence and handoff to W11, W13, W14 | [validation and handoff](05-validation-and-handoff.md) | P7-V19–V21 evidence locations; handoff checklist |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p7-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no implemented crates, no
telemetry implementation; `docs/stages/p7/implementation/` holds only the
stage README. P0–P6 are planned, not implemented; every input W09 consumes is
an **assumed contract** cited by plan path, each with a failure boundary in
[01-accounting-model.md](01-accounting-model.md) §2.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Counters observable and coherent across scheduler events (P7-V19) | Absent; no code exists | Accounting records with single-writer update rules at the producer hook points (W02/W04/W06/W07/W08 seams) | Coherence requires exactly one authority per increment; scattered updates double-count | Producer packages (hooks); P7-W09 (records/rules) | P7-V19; P7-V04 accounting clause |
| Events associate VM/vCPU/pCPU/reason/time (P7-V20) | Absent | Trace-event set with declared fields registered under P0-W13 governance | An event without declared identity/context fields cannot be correlated | P0-W13 (governance); P7-W09 (event set) | P7-V20 |
| Stop-running reason distinguishable (P7-V20) | Absent | Switch-reason vocabulary covering W04/W06/W07 dispositions | Preemption, block, pause, stop, and fault must be separately countable | P7-W09 (vocabulary); producers (occurrences) | P7-V20 |
| Failure reports current pCPU, VM/vCPU/state/affinity/reason/recent and pending events; no crash-dump claim (P7-V21) | Absent | Bounded recent-transition snapshot + failure-report content, emitted via P0-W12 channels | Diagnosability must not depend on a crash-dump framework that does not and may not exist here | P7-W09; P0-W12 (channels) | P7-V21 |
| pCPU and VM aggregation | Absent | pCPU-owned counters; VM aggregation computed on read | Aggregation must never duplicate state that can diverge | P7-W09 | P7-V19 |
| P0 telemetry governance compatibility | P0-W12/W13 plans exist as contracts; implementations do not | Classification of every W09 output into P0-W12 channels and P0-W13 naming | Governance is only real if every event registers under it | P0-W12/W13; P7-W09 | Review step 4 |
| Evidence handoff to W11, W13, W14 | Verification directory empty (`.gitkeep` only) | Validation matrix and record paths defined here | Consumers rely only on declared, locatable evidence | P7-W09 | `../../verification/p7-w09-accounting-diagnostics-verification.md` |

No row invents an upstream mechanism. If P7-W01's reconciliation (P7-V01)
marks any assumed input missing or contradictory, the affected step is blocked
and recorded; it is not repaired in W09.

## Resolved design decisions and their authority

1. **Producers own occurrences; W09 owns records.** The lifecycle, switch,
   block/wake, control, and SMP paths (W02/W04/W06/W07/W08) call the hook
   points this design defines; only those hooks mutate accounting state. No
   counter is incremented anywhere else. Rationale: this is the only
   structure that makes "no regress, no double-count" (P7-V04/V19) checkable.
   Authority: plan scope; producer contracts
   ([P7-W06](../p7-w06-block-wakeup/README.md) B-5,
   [P7-W07](../p7-w07-pause-stop-fault/README.md) F-5,
   [P7-W08](../p7-w08-smp-reschedule-idle/README.md) R-4/I-5).
2. **One writer class per record.** vCPU accumulators are written only by the
   pCPU executing that vCPU (plus its control transitions); pCPU counters only
   by the owning pCPU; VM aggregation is computed on read, never stored.
   Cross-CPU reads may observe bounded staleness, declared per record.
   Rationale: avoids cross-CPU read-modify-write entirely on the hot path.
   Authority: stage-local design freedom owned here within P3-W06's
   synchronization rules.
3. **Mechanism-neutral switch reasons.** The `SwitchReason` vocabulary
   describes why a vCPU stopped running and what the pCPU did next; it encodes
   no policy judgment (no priority classes, no fairness claims). Authority:
   plan scope; ADR-057/ADR-017 leave policy undecided — the vocabulary must
   not prejudge it.
4. **Ready-wait accounting is the steal-time basis, not steal time.** Each
   vCPU accumulates runnable-but-not-running time; any Guest-visible steal
   ABI is P8+/machine-ABI territory and explicitly excluded. Rationale: the
   ADR P7 stage list names steal time; the measurable, policy-free basis is
   ready-wait. Authority: task book §1 ("runtime accounting、steal time");
   stage-local boundary decision owned here.
5. **Trace events are declared, then registered.** Every event's name, domain
   placement, fields, and version behavior follow P0-W13's rules; this design
   fixes the event set and fields, and implementation registers them through
   the governance mechanism — never as ad-hoc strings. Rationale: P0-W13's
   purpose is to prevent temporary strings from becoming interfaces.
   Authority: P0-W13 contract.
6. **Diagnostics are bounded, deterministic, and crash-dump-free.** A
   fixed-capacity per-vCPU recent-transition record (CPU-local writes, no
   allocation) plus a fixed-order failure report emitted through the P0-W12
   channels. No crash-dump framework is used or implied. Rationale: P7-V21
   explicitly disclaims crash dumping; bounded snapshots keep failure paths
   allocation-free per the Coding Guidelines. Authority: plan scope;
   [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md).
7. **Diagnostics never echo Guest-controlled bytes.** Identifiers, numeric
   states, and enumerated reasons only; any Guest-influenced value appears as
   a validated number (e.g. fault class, IPA as hex number), never as text.
   Rationale: failure diagnostics are Guest-reachable in effect (a Guest can
   cause the fault being diagnosed) and must not become an injection or
   log-flooding channel. Authority: ADR-007; Coding Guidelines untrusted-input
   rules; [P5-W09](../../../p5/plans/p5-w09-telemetry-safe-logging-regression.md)
   safe-logging direction.
8. **Release trimming follows P0-W12, with a fixed priority.** High-frequency
   events (wake, reconsideration, per-switch trace) are trimmable; block/
   decline, control, fault, refusal, and diagnostic events are the minimum
   set retained for failure analysis. Rationale: P0-W12 requires compilable
   trimming and runtime filtering; the priority fixes what a production build
   can still diagnose. Authority: P0-W12 contract; stage-local priority owned
   here.

## Work breakdown and loading order

1. Read [01-accounting-model.md](01-accounting-model.md) for the record
   inventory, coherence rules, aggregation, and assumed contracts with
   failure boundaries.
2. Implement the accounting records and update points per
   [02-code-contracts-accounting.md](02-code-contracts-accounting.md).
3. Implement trace events, the switch-reason vocabulary, and the diagnostic
   snapshot/report per
   [03-code-contracts-trace-diagnostics.md](03-code-contracts-trace-diagnostics.md).
4. Follow [04-implementation-workflow.md](04-implementation-workflow.md).
5. Record planned and actual evidence per
   [05-validation-and-handoff.md](05-validation-and-handoff.md).
   Implementation notes go to `../p7-w09-accounting-diagnostics-record.md`
   (created when work starts); evidence to
   `../../verification/p7-w09-accounting-diagnostics-verification.md`.
   Neither this design nor the record may claim W09 complete.

## Explicitly excluded interfaces

No metrics backend, transport, ring buffer, crash-dump framework, management
protocol, log-format ABI, performance KPI, public API, or wire format is
designed or authorized by W09. Field lists are semantic content; their
encoding belongs to the P0-W12/W13-governed telemetry implementation and,
where an external format ever exists, to a versioned ABI design. Adding any
excluded surface is a scope conflict stopped at review. The hook-point
inventory this design shares with producers is in
[01-accounting-model.md](01-accounting-model.md) §2 and §6.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md), W11, W13, and W14 consume
this package:

- **[P7-W11](../p7-w11-stress-invariants/README.md)** receives the counters,
  switch reasons, and coherence rules as the evidence instruments its stress
  and fairness checks (P7-V24–V27) read; starvation and fairness checks are
  expressed over ready-wait and switch counters, not ad-hoc prints.
- **[P7-W13](../p7-w13-performance-baseline/README.md)** receives the
  accounting records, trace event set, and the declared measurement limits as
  the sole data source for its static-versus-scheduled baseline (P7-V29);
  W13 draws no number this design does not define.
- **[P7-W14](../p7-w14-documentation-p8-handoff/README.md)** receives the
  observability contract, diagnostic boundaries (no crash-dump claim), and
  the P0-governance registration status for the closure documentation and P8
  handoff (P7-V30).
- **[P7-W12](../p7-w12-qemu-regression/README.md)** (indirect, via W11/W13)
  receives determinate counter/trace expectations its QEMU matrix (P7-V28)
  can assert.
