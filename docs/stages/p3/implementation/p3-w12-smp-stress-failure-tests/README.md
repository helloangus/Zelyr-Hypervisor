# P3-W12 SMP Stress and Failure Tests — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The repeatable stress/failure evidence contract for Host SMP
concurrency, failure, and exceptional paths required by
[P3-W12](../../plans/p3-w12-smp-stress-failure-tests.md).  
**Owner/change context:** P3-W12 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W12. It defines the bounded
test categories, fault stimuli, success conditions, accounting rules,
repetition and determinism policy, and evidence destinations that make
P3-V12 assessable: concurrent allocate/free, notification storms,
atomic/shared-counter stress, repeated rendezvous, secondary timeout,
invalid/offline targets, and concurrent logging. It deliberately does
**not** run the tests (evidence is collected only when the P3
implementation exists), does not implement CI scripts or the QEMU runner
(the P0-W09 automation entry and P0-W20 CI own those; the matrix execution
that reuses these scenarios at 1/2/4/8 CPU counts is
[P3-W13](../p3-w13-qemu-smp-regression/README.md)), does not certify
performance, does not prove real-hardware behavior, and does not design
any guest or Stage-2 stress workload.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) (the
harness code this package may authorize is host-side and
hypervisor-reuse test code only, within the P0 host-side testing
baseline). It then loads only the linked supporting file needed for its
assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, prerequisite failure boundaries, resolved decisions |
| [02-stress-scenario-matrix.md](02-stress-scenario-matrix.md) | the S1–S7 scenario definitions: stimulus, observable, pass condition, proves/does-not-prove, repetition |
| [03-failure-injection-and-accounting.md](03-failure-injection-and-accounting.md) | injection methods, accounting sources, determinism/seed rules, declared limits, evidence classification |
| [04-workflow-validation-and-handoff.md](04-workflow-validation-and-handoff.md) | ordered workflow, validation matrix, error model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document defines a test contract; it contains no results, and no scenario
has run. Nothing here may be read as asserting that P3 mechanisms work.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W12 plan → this
design → Coding Guidelines. Binding constraints:

- The task book requires repeatable stress/failure evidence with
  explainable accounting and no corruption, permanent hang, or undiagnosed
  failure within documented test limits (P3-V12), and its validation
  matrix allocates allocator, notification, atomic, barrier,
  concurrent-log, and failure-path stress to this package.
- The plan's out-of-scope list removes performance certification,
  exhaustive formal verification, real-hardware proof, CI-script
  implementation, and future guest stress workloads. "Run or collect …
  when implementation exists" (plan step 5) makes the *contract* the
  deliverable and the *results* a later, honestly recorded activity.
- ADR-049's validation strategy (unit + host-side + QEMU integration +
  fuzz/property where applicable) bounds the techniques; ADR-003 makes
  QEMU the reference environment whose results never define hardware
  semantics; the Plan/Coding guides require every validation to state
  what it does not prove.
- Host-side testability follows the P0 host-side testing baseline
  ([P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md),
  assumed contract, boundary in
  [01](01-scope-and-foundations.md) §1.2): scenarios that can be expressed
  against P3 logic with fakes run host-side; scenarios that need real
  cross-CPU progress run under QEMU via the P0-W09 entry.
- Observability inputs come from
  [P3-W11](../p3-w11-smp-observability/README.md) (counters, events,
  dumps); accounting obligations that W12's scenarios place on snapshot
  quiescence are W12's to state ([03](03-failure-injection-and-accounting.md)
  §3), per the W11 read/aggregation split.

Classification:

- **Required** for W12 closure: the S1–S7 scenario matrix with stimuli,
  expected observables, pass conditions, prove/does-not-prove statements,
  and repetition policy; the failure-injection method register; the
  accounting and determinism rules; the declared test limits; the
  evidence classification (planned/run/failed/blocked/not-run) and
  destinations; and the review evidence that the contract itself is
  coherent.
- **Reserved** with recorded triggers: randomized/property-based stimulus
  generation beyond the declared seeded rules (trigger: a property-testing
  baseline decision with the P0-W08 owner); long-duration/soak
  repetitions beyond declared limits (trigger: a stage-closure request
  with recorded cost rationale); on-target secondary-timeout induction
  (trigger: hardware validation scope, P15); fault injection beyond the
  declared software seams (trigger: an approved fault-injection design).
- **Out of Scope:** performance numbers or latency claims; formal
  verification; real-hardware execution (P15); CI workflow files and the
  QEMU runner implementation (P0-W20/P0-W09); guest, vCPU, and Stage-2
  stress (P4+); modifying the mechanisms under test to make a scenario
  pass; any completion or pass claim in this design.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Define bounded test categories, fault stimuli, success conditions | [scenario matrix](02-stress-scenario-matrix.md) §2 (S1–S7) | P3-V12 (W12-DV01) |
| Define accounting and known test limits | [accounting](03-failure-injection-and-accounting.md) §3, §4 | P3-V12 (W12-DV02) |
| Repeatability (repeated rendezvous, repetition policy, determinism) | [matrix](02-stress-scenario-matrix.md) §3 (S4), [determinism](03-failure-injection-and-accounting.md) §5 | P3-V12 (W12-DV02) |
| Failure paths: secondary timeout, invalid/offline targets | [matrix](02-stress-scenario-matrix.md) §2 (S5, S6), [injection register](03-failure-injection-and-accounting.md) §2 | P3-V12 (W12-DV01) |
| Evidence assessable: accounting, hangs, corruption, diagnostics | [accounting](03-failure-injection-and-accounting.md) §3, [evidence](03-failure-injection-and-accounting.md) §6 | P3-V12 (W12-DV03) |
| Integrate with QEMU matrix and stage-closure consumers | [handoff](04-workflow-validation-and-handoff.md) §4 | W12 closure review (W12-DV05) |
| Review: Host SMP only; no guest/Stage-2 assertions | [workflow](04-workflow-validation-and-handoff.md) step 5 | W12 closure review (W12-DV04) |
| Run/collect evidence when implementation exists; honest status | [workflow](04-workflow-validation-and-handoff.md) steps 6–7, [evidence](03-failure-injection-and-accounting.md) §6 | P3-V12 (W12-DV06); execution deferred until P3 implementation exists |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P0 documentation scaffold only — no workspace, no sources,
no QEMU runner, no test harness, and no verification evidence for any P3
package. The mechanisms this plan stresses (allocator, notification,
synchronization, rendezvous, secondary start, logging) are planned
packages (P2-W04/W05, P3-W06/W07/W02/W05 and the P0-W12 channel), not
implementations. Sibling P3 designs W01–W05 are present as proposed
designs; W06–W10 are parallel and referenced by path and P3-Wxx ID without
assuming content. Each ledger row below states the missing foundation the
plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Repeatable evidence beyond a one-time boot exists | No P3 code, no harness, no evidence anywhere | The scenario contract: S1–S7 with stimuli, pass conditions, repetition, limits — designed before any run | "Repeatable" cannot be evaluated post hoc; each scenario must fix what repetition means and what a pass is before evidence exists | W12 (this design) | W12-DV01 contract review |
| Accounting is explainable | No accounting source is defined | Per-scenario accounting identities over W11 counters, allocator metadata, and rendezvous results, with quiescence obligations | P3-V12's "explainable accounting" requires declared identities (e.g. sent == received), not log inspection | W12 (identities); W11 (surfaces); W04's fill pattern as a corruption sentinel | W12-DV02/DV03 |
| Hangs are assessable | No bound exists for any loop | Per-scenario iteration bounds and a bounded-wait rule so "no permanent hang within declared limits" is decidable | An unbounded scenario cannot fail cleanly; the declared limit is what P3-V12 scopes the claim to | W12 (limits, recorded constants) | W12-DV02 |
| Corruption is detectable | No detector exists | Declared corruption sentinels: allocator ownership metadata, W04 reserved-region fill pattern, registry/online-set invariants re-checked post-stress | P3-V12 names corruption; without declared sentinels it is unfalsifiable | W12 (declares which sentinels each scenario checks); owners own the mechanisms | W12-DV03 |
| Failure paths (timeout, invalid/offline targets) are exercisable | No failure stimulus exists; true on-target timeout is not inducible at P3 | The injection register: software seams (host fakes), QEMU-observable stimuli (absent-CPU CPU_ON), invalid-target calls — and recorded non-inducible items | A failure path never exercised is a claim, not evidence; the register separates exercisable from not-exercisable honestly | W12 (register); W02's induced-failure input; W07/W08 refusal paths | W12-DV01 |
| Evidence is assessable and distinguishable | No verification records exist for P3 | Evidence classification (planned/run-passed/run-failed/blocked/not-run) with command, environment, timestamp, and destination per record | The plan requires distinguishing planned, run, failed, and blocked states | W12; destinations per the stage layout | W12-DV06 |
| QEMU matrix integration | No matrix exists ([P3-W13](../p3-w13-qemu-smp-regression/README.md) is a plan) | Scenario-to-matrix mapping rules: which S-scenarios appear at which W13 rows and counts | W13 consumes scenarios; without the mapping, W12 and W13 would duplicate or contradict | W12 (scenario side); W13 (matrix side) | W12-DV05 |

No ledger row requires inventing a runner, a CI system, or guest
workloads; the open prerequisite risks are in [01](01-scope-and-foundations.md)
§1.2.

## Resolved design decisions and their authority

1. **Contract-first, results-later.** This design's deliverable is the
   scenario contract; execution evidence is collected only when the P3
   implementation exists and is recorded solely in the verification
   record. Rationale: the plan says so explicitly ("Run or collect … when
   implementation exists"), and the checklist forbids planned items
   masquerading as evidence.
2. **Seven bounded scenario groups, one per plan-scope area.** S1
   concurrent allocate/free, S2 notification storm, S3 atomic/shared-counter
   stress, S4 repeated rendezvous, S5 secondary timeout, S6 invalid/offline
   targets, S7 concurrent logging. Each names its mechanism owners, and no
   scenario invents a mechanism ([02](02-stress-scenario-matrix.md) §2).
   Rationale: the plan's scope list is exactly these; a one-to-one mapping
   keeps traceability to P3-V12 trivial.
3. **Pass conditions are accounting- and invariant-based, never
   timing-based.** A scenario passes when declared counters balance,
   declared invariants hold, and declared sentinels are intact within the
   declared iteration bounds — never because it finished fast or slow.
   Rationale: the plan excludes performance certification; W11's
   informative-only timing rule must not be undermined by a back door.
4. **Determinism by construction; the only randomness is seeded.**
   Stimulus patterns are fixed schedules (round-robin, neighbor rings,
   boot-repeat); where a scenario offers a seeded shuffle, the seed is
   recorded in the evidence and repetition with the same seed must
   reproduce the schedule (QEMU's own timing nondeterminism is scoped in
   [03](03-failure-injection-and-accounting.md) §5). Rationale:
   P3-V12's "repeatable" must mean more than "ran more than once".
5. **Failure injection is limited to declared software seams and
   QEMU-observable stimuli.** The register
   ([03](03-failure-injection-and-accounting.md) §2) lists each injection
   (host-fake transport fault, invalid/offline target calls, absent-CPU
   CPU_ON), its owner, and what it cannot induce (true device-level PSCI
   timeout at P3 — recorded limitation inherited from W02's matrix note).
   Rationale: honest failure coverage needs a boundary between what was
   injected and what was merely imagined.
6. **Every scenario declares its Host-SMP-only proof boundary.** No S
   scenario asserts guest, vCPU, Stage-2, or GIC behavior; W13's matrix
   inherits this boundary per row. Rationale: the plan's review step 4;
   stage boundaries (task book §2).
7. **Stress harness code is bounded and layered.** Where scenarios need
   in-hypervisor stimulus code (e.g. a stress entry point reachable under
   the P0 QEMU entry path), it is declared per scenario as test-support
   code exercising published P3 surfaces only — it adds no new mechanism,
   lock, or allocation path, and host-side variants prefer fakes. CI
   wiring remains P0-W20's. Rationale: the plan excludes CI-script work
   but the scenarios still need a declared, reviewable way to execute;
   leaving it undeclared would push improvisation into implementation.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, the assumed-prerequisite boundaries (W06/W07/W08 and the P0
   host-test/QEMU baselines are assumed contracts), and the decisions.
2. Read [02-stress-scenario-matrix.md](02-stress-scenario-matrix.md) for
   S1–S7: each scenario's owners, stimulus, expected observable, pass
   condition, prove/does-not-prove, and repetition.
3. Read [03-failure-injection-and-accounting.md](03-failure-injection-and-accounting.md)
   for the injection register, accounting identities and quiescence
   rules, declared limits, determinism/seed policy, and evidence
   classification and destinations.
4. Execute per [04-workflow-validation-and-handoff.md](04-workflow-validation-and-handoff.md):
   contract reviews first (steps 1–5), then scenario execution and
   evidence recording only when the implementation exists (steps 6–7).
5. Record implementation decisions in
   `../p3-w12-smp-stress-failure-tests-record.md` and evidence in
   `../../verification/p3-w12-smp-stress-failure-tests-verification.md`
   only when produced. Until real records exist, P3-V12 is *planned*, and
   this design claims nothing.

## Explicitly excluded interfaces

No hypervisor mechanism is designed here: no lock, notification, TLB
transport, allocator, rendezvous, or lifecycle change; scenarios consume
published P3 surfaces and W11 read surfaces only. No CI workflow, runner
binary, or QEMU invocation script is designed (P0-W09/P0-W20 own the
automation contracts; W13 owns the matrix contract). No guest image,
vCPU workload, or Stage-2 stimulus is designed (P4+). No performance
metric, benchmark, or timing assertion is defined. A scenario that
requires modifying a mechanism's internals to observe or inject is a
design conflict to stop at review — injection uses the declared seams or
is recorded as not exercisable.

## Downstream handoff

- **W13** receives S1–S7 as the matrix's stress and failure content, the
  repetition/determinism rules, and the honest-status classification;
  W13 fixes which scenarios run at which CPU counts and repetition
  depths per row.
- **W15** receives the scenario-to-P3-V12 traceability and the
  evidence-destination conventions for the stage documentation package.
- **W14/P4** receives the declared test limits and the proved/does-not-
  prove boundary as part of the P4 handoff (P4 acceptance review
  consumes it per the plan's Handoff section).
- **P15** inherits the recorded hardware gap: on-target timeout induction
  and real-hardware stress remain P15 responsibilities; nothing in W12's
  QEMU evidence substitutes for them.
