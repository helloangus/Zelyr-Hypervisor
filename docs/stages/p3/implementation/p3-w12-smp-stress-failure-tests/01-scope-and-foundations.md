# P3-W12 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W12 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Repeatable evidence for concurrency, failure, and exceptional paths"
requires five concrete artifacts:

1. **Scenario definitions** (S1–S7) with stimulus, observable, pass
   condition, repetition, and proof boundary —
   [02-stress-scenario-matrix.md](02-stress-scenario-matrix.md).
2. **Injection register** distinguishing exercisable from non-inducible
   failure stimuli — [03-failure-injection-and-accounting.md](03-failure-injection-and-accounting.md)
   §2.
3. **Accounting rules** — per-scenario identities over W11 counters,
   allocator metadata, rendezvous results, and corruption sentinels, with
   quiescence obligations — same file §3.
4. **Declared limits** — iteration/repetition bounds that scope the
   P3-V12 claim — same file §4.
5. **Evidence convention** — run/fail/blocked/not-run records with
   destinations — same file §6.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Host-side testing baseline | [P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md) | Host-side test execution with fakes is possible for logic separated from hardware | If no host-side path exists for a scenario W12 classified host-side, that scenario's host rows are reclassified QEMU-only and the change is recorded |
| QEMU automation entry | [P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md) | A declared way to boot the hypervisor image under QEMU and capture diagnostics | Without it, QEMU-executed scenarios are blocked; record blocked, do not improvise a runner (CI/runner work is out of scope) |
| QEMU/CI contracts retained | [P0-W20](../../../p0/plans/p0-w20-ci-baseline.md) | CI consumes declared scenarios through the automation entry; no W12-authored CI files | A CI gap is a P0-W20 issue, recorded; W12 adds no workflow files |
| Observability read surfaces | [P3-W11](../p3-w11-smp-observability/README.md) | Snapshot/aggregate/dump per its contracts; event stream; seam register | If W11 lands with named gaps, the affected scenario's accounting degrades to the available surface and the gap is cited in evidence; W12 must not add parallel counters |
| Mechanisms under test | [P3-W06](../p3-w06-concurrency-synchronization/README.md), [P3-W07](../p3-w07-cross-cpu-notification/README.md), [P3-W08](../p3-w08-tlb-shootdown-transport/README.md), [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md), [P3-W05](../p3-w05-smp-boot-synchronization/README.md), [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) (parallel designs, unread) | Published surfaces each scenario names; refusal/terminal semantics for invalid-target and failure scenarios | A missing or differently shaped surface blocks its scenario; record blocked with the blocking contract; scenarios never reach into internals |
| Allocator and its debug metadata | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md), [P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md) | SMP-safe (per W06/audit) allocation with ownership accounting | If the allocator's accounting is not queryable, S1 falls back to W11/W04 sentinels plus boot-deallocation checks, and the reduced guarantee is recorded |
| Secondary-start failure input | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) | Induced absent-CPU CPU_ON input exercisable via the QEMU entry path; recorded limitation that on-target timeout is not inducible at P3 | If W02's induced input does not exist, S5's QEMU row is blocked; host-side watcher tests remain the S5 evidence |
| Counter-block capacity and sentinels | [P3-W04](../p3-w04-per-cpu-runtime/README.md) | Reserved-region fill pattern; counters zeroed at allocation | If sentinels change, the corruption checks that cite them change with the W04 record — cited, not redefined, here |
| SMP-safety classification of shared state | [P3-W10](../p3-w10-smp-safety-audit/README.md) (parallel, unread) | The audit's classification of P0–P2 mutable infrastructure (e.g. allocator, registries) that scenarios S1/S3 stress, plus its unresolved items | An unresolved audit item touching a stressed mechanism scopes the affected scenario's claim; the scenario records the open item and does not claim coverage the audit withholds |

### 1.3 Why no hidden essential deliverable remains

- Plan step 2 ("define bounded test categories, fault stimuli, success
  conditions, accounting, and known test limits") is fully realized by
  §1.1 items 1–4; nothing in the plan's scope list lacks a scenario.
- Plan step 3 ("integrate test evidence requirements with QEMU matrix and
  stage-closure consumers") is realized by the scenario-to-matrix
  mapping obligation (handoff section of the entry README and W12-DV05)
  and the evidence destinations.
- Plan step 5 ("run or collect … when implementation exists") is honestly
  unsatisfiable today and is carried as the deferred execution phase with
  its own steps and status rules — the contract, not the results, is the
  deliverable of the design phase.

## 2. Scope classification

### 2.1 Required

- S1–S7 scenario definitions with all matrix fields.
- The injection register and its exercisable/not-exercisable split.
- Accounting identities, quiescence obligations, corruption sentinels.
- Declared iteration/repetition limits (recorded constants with
  rationale).
- Determinism and seed rules.
- Evidence classification and destinations; per-scenario evidence
  templates.
- Host-side test code for scenarios expressible with fakes, within the
  P0-W08 baseline, and declared stress entry points for QEMU-executed
  scenarios (test-support only).
- Reviews: scenario-owner surface check, Host-SMP-only boundary check,
  mapping-to-W13 check.

### 2.2 Reserved (must not block a future design; not implemented now)

- Property-based/randomized stimulus generation; trigger: a
  property-testing baseline decision with the P0-W08 owner.
- Soak/long-duration runs beyond declared limits; trigger: stage-closure
  request with cost rationale.
- On-target failure injection (real PSCI failure, device-level timeout);
  trigger: P15 hardware validation scope.
- Fault injection into additional subsystems (allocation-failure fault
  points beyond declared seams); trigger: an approved fault-injection
  design.

### 2.3 Out of Scope

- Performance certification, latency numbers, throughput claims.
- Formal verification/model checking (ADR-049 names it as optional
  further work, not P3 evidence).
- Real-hardware execution and proof (P15).
- CI workflow files, runner implementation, QEMU invocation scripts
  (P0-W20/P0-W09; matrix contract is W13's).
- Guest/vCPU/Stage-2/GIC stress workloads (P4+/P6+).
- Modifying tested mechanisms to make scenarios pass; adding mechanisms
  for testability.
