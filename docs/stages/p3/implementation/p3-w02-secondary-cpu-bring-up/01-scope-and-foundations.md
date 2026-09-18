# P3-W02 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W02 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries the
checklist requires.

### 1.1 What must concretely exist for the plan goal to be true

"Make each expected secondary physical CPU reach a defined and diagnosable
EL2 bring-up result" requires five concrete artifacts:

1. A **request path**: a function that, per candidate CPU, issues exactly
   one CPU-start call using P2-recorded conduit and function identifiers and
   maps the firmware return value to a named outcome —
   [04 §2–§3](04-code-contracts-start-requester.md).
2. An **entry path**: a positioned entry symbol reached by the started CPU,
   which revalidates the P1 entry contract, confirms identity, and
   establishes a minimal local environment —
   [03 §2–§4](03-code-contracts-secondary-entry.md).
3. A **reporting path**: the single-writer mailbox and outcome records that
   make the result CPU-attributed and phase-attributed —
   [03 §5](03-code-contracts-secondary-entry.md),
   [04 §4](04-code-contracts-start-requester.md).
4. A **termination path**: the bounded-poll watcher that converts "no
   arrival" into a terminal outcome instead of a hang —
   [04 §6](04-code-contracts-start-requester.md).
5. **Evidence**: host-side tests for the pure logic (outcome mapping,
   watcher, sequencer) plus QEMU capture per declared count and for the
   induced absent-CPU failure, per
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| CPU topology inputs: Present-class candidates, boot designation, start-capability facts (conduit, CPU_ON id, availability) | [P3-W01](../p3-w01-cpu-topology-inputs/README.md), seeded by [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) | A frozen `TopologyInputs` whose `StartCapabilityFacts` name the conduit class and CPU_ON function id, or record their absence | If capability facts are absent or unusable: fail closed per CPU with `StartUnavailable` before any CPU_ON; do not guess identifiers (ADR-044) |
| Entry state contract: Non-secure EL2 entry with defined register/memory assumptions | [P1-W01](../../../p1/plans/p1-w01-reference-boot-contract.md), [P1-W02](../../../p1/plans/p1-w02-minimal-rust-el2-runtime.md) | The boot CPU entered Non-secure EL2 under documented assumptions; secondaries started by PSCI enter an equivalent state on the reference platform | If a secondary's revalidation fails: report failure at phase `EntryStateValidated` and park the CPU; never continue on assumed register state |
| EL2 architectural baseline reproducible per CPU | [P1-W04](../../../p1/plans/p1-w04-el2-architectural-state-baseline.md) | The baseline register program that produced the boot CPU's EL2 state is a bounded, repeatable sequence a secondary can apply to itself | If the baseline is documented as boot-CPU-only or not repeatable per CPU, that is an Architecture Change Request against the P1 contract — W02 must not invent its own trap policy |
| Page allocator contract for boot-phase allocation | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md) | Aligned page allocation with explicit OOM failure, callable on the boot CPU before secondary release | Allocation failure at this point is a fatal boot-critical failure (P0 panic policy); W02 does not fall back to static image arrays |
| Lifecycle transition operations (request-start, entered-initializing, report-failure) | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) | Transition operations with exactly the owners named in W03's design; W02 calls, never implements | If W03's design assigns a transition owner that conflicts with W02's flow, resolve as a design conflict between the two designs before implementation |
| Boot-phase publication ordering | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) | Global initialization (including W01 intake and all provisional-stack allocation) is complete and published before any CPU_ON is issued | If sequencing cannot be asserted, the requester fails closed with a boot-phase diagnostic; releasing secondaries into a half-initialized world is forbidden |

### 1.3 Why no hidden essential deliverable remains

- The plan's step 1 ("confirm topology, P1 EL2 entry, and P2 CPU-start
  prerequisites") is the precondition table above, checked by the
  workflow §1 precondition review.
- The "detailed start mechanism" the plan reserves to "an approved design
  decision" is exactly decisions 1–6 of the entry README; this design is
  that approval record.
- The induced failure path required by the plan's step 5 exists because
  QEMU's PSCI rejects CPU_ON for an identity that is not an implemented
  CPU; W02 treats that rejection as first-class evidence input
  (W02-DV07). On-target wall-clock timeout induction is genuinely
  unavailable at P3 and is recorded as a stated validation limit, not a
  silent gap.

## 2. Scope classification

### 2.1 Required

- PSCI CPU_ON request path with conduit-driven call, return-code mapping,
  and one-audit-boundary `unsafe`.
- Secondary entry stub: entry-state revalidation, MPIDR identity
  confirmation with context-id cross-check, transition to the
  initializing state, minimal local environment, result reporting.
- Provisional per-secondary stacks with allocation-before-release and
  success-transfer/quarantine-on-failure ownership.
- Arrival mailbox protocol (single-writer, acquire/release) and the
  per-CPU outcome model with phases.
- Bounded-poll timeout watcher.
- Per-CPU bring-up diagnostics under the P0 logging/trace governance.
- Host-side unit tests and QEMU evidence per the validation matrix.

### 2.2 Reserved (must not block a future design; not implemented now)

- Spin-table startup path; trigger: an approved platform lacking PSCI
  CPU_ON (would require release-memory, cache-maintenance, and
  firmware-cooperation design).
- Timer-based timeout measurement; trigger: P6 timer baseline.
- WFE/SEV (event-signal) idle in the poll loop and the parked loops;
  trigger: integration with the P3-W07 notification primitive.
- Retry or re-admission of failed secondaries; trigger: an approved
  lifecycle design that owns recovery (P3 has none by plan).
- CPU_OFF / shutdown / other PSCI functions; trigger: later-stage designs
  that need them.

### 2.3 Out of Scope

- The lifecycle state machine and online admission (P3-W03).
- Real per-CPU stacks, per-CPU areas, CPU-local accessors (P3-W04).
- Rendezvous/barrier semantics and SMP-ready (P3-W05).
- Lock primitives and general atomic policy (P3-W06).
- IPI/SGI notification and TLB transport (P3-W07/P3-W08).
- Per-CPU exception/interrupt enablement beyond the P1 baseline checks
  (P3-W09).
- Guest PSCI, guest SMP, vCPU objects (P8/P4+).
- The halt-vs-continue choice for degraded boots (surfaced to the
  boot-integration owner; decision 7).
- Runtime hotplug/restart (stage and plan out of scope).
