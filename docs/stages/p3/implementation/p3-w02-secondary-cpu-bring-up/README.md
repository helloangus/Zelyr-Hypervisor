# P3-W02 Secondary CPU Bring-Up — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The bounded mechanism by which each expected secondary physical
CPU reaches a defined, diagnosable EL2 bring-up result, required by
[P3-W02](../../plans/p3-w02-secondary-cpu-bring-up.md).  
**Owner/change context:** P3-W02 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W02. It defines both sides of
secondary bring-up: the boot-CPU requester that issues CPU-start calls using
P2-provided platform input, and the secondary-side entry path that reaches
EL2, validates the entry state, confirms its own identity, establishes a
minimal local environment, and reports a per-CPU, per-phase result. It
deliberately does **not** define the lifecycle registry and its transition
machine ([P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) — W02 is a
transition *caller*), per-CPU runtime state
([P3-W04](../p3-w04-per-cpu-runtime/README.md)), the boot rendezvous it
integrates with ([P3-W05](../p3-w05-smp-boot-synchronization/README.md)), or
the general concurrency primitives (P3-W06).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, scope classification, resolved decisions |
| [02-architecture-and-state.md](02-architecture-and-state.md) | the two-sided bring-up architecture, phase model, ownership, concurrency model |
| [03-code-contracts-secondary-entry.md](03-code-contracts-secondary-entry.md) | secondary entry stub, identity confirmation, arrival reporting contracts |
| [04-code-contracts-start-requester.md](04-code-contracts-start-requester.md) | PSCI start call, outcome model, sequencer, timeout watcher contracts |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W02 plan → this
design → Coding Guidelines. Binding constraints:

- The ADR P3 roadmap names PSCI CPU_ON as the secondary-start mechanism
  (ADR-015 context); ADR-008 requires the hypervisor to use existing
  firmware (TF-A/U-Boot/QEMU firmware interfaces) rather than implement EL3.
  The PSCI **conduit and function identifiers are assumed contract from P2's
  PlatformInfo** ([P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md),
  [P2-W02](../../../p2/plans/p2-w02-platform-discovery-normalization.md));
  W02 must not hardcode QEMU's identifiers or pick a conduit by platform
  name (ADR-044, ADR-052).
- ADR-003 makes QEMU `virt` the reference environment; observed QEMU PSCI
  behavior is reference evidence, not an architecture contract.
- ADR-006 constrains the assembly and `unsafe` boundary: the PSCI call is a
  small, audited architecture boundary.
- The task book requires success, failure, and timeout reporting "with CPU
  and phase attribution", and P3-V02 requires that a failure never makes
  SMP state opaque.
- The plan's out-of-scope list removes runtime hotplug/restart, guest PSCI,
  guest SMP, a *detailed* PSCI/GIC mechanism beyond this bounded bring-up
  use, and the policy for continuing after a degraded platform result.
  Continuation policy is therefore surfaced as a decision input for the
  boot-integration owner, not silently fixed here (decision 7).

Classification:

- **Required** for W02 closure: the PSCI CPU_ON start path (conduit-driven),
  the secondary entry path with entry-state revalidation and identity
  confirmation, the provisional per-secondary execution environment, the
  per-CPU outcome model with phase attribution, the arrival mailbox, and
  P3-V02 acceptance evidence.
- **Reserved** with recorded triggers: spin-table startup (trigger: an
  approved platform without PSCI CPU_ON; would need its own release-memory
  and cache-maintenance design); timer-based timeout measurement (trigger:
  the P6 timer baseline; P3 uses a bounded poll); WFE/SEV-idle optimization
  of the poll loop (trigger: integration with P3-W07's event primitive);
  retry of a failed secondary (trigger: an approved lifecycle design that
  owns re-admission).
- **Out of Scope:** runtime CPU hotplug or restart; guest PSCI and guest
  SMP (P8); GIC initialization or interrupt delivery to secondaries (P6 /
  P3-W09); the lifecycle state machine itself (P3-W03); per-CPU runtime
  areas and real per-CPU stacks (P3-W04); rendezvous/barrier semantics
  (P3-W05); general lock primitives (P3-W06); the halt-vs-continue boot
  policy choice beyond making degraded results observable.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Boot CPU requests startup using P2-provided platform input | [requester contract](04-code-contracts-start-requester.md) §2–§3 | P3-V02 (W02-DV01, DV02) |
| Secondary reaches EL2 with a validated entry state | [entry contract](03-code-contracts-secondary-entry.md) §2 | P3-V02 (W02-DV03) |
| Secondary confirms identity | [entry contract](03-code-contracts-secondary-entry.md) §3 | P3-V02 (W02-DV03) |
| Secondary establishes its initial local environment | [architecture](02-architecture-and-state.md) §5, [entry contract](03-code-contracts-secondary-entry.md) §4 | P3-V02 (W02-DV03) |
| Success / timeout / failure reported with CPU and phase attribution | [outcome model](04-code-contracts-start-requester.md) §4–§6, [arrival protocol](03-code-contracts-secondary-entry.md) §5 | P3-V02 (W02-DV04, DV05) |
| Repeatable bring-up evidence for expected CPU counts and an induced failure path | [workflow](05-implementation-workflow.md) steps 5–6; matrix in [validation](06-validation-and-handoff.md) | P3-V02 (W02-DV06, DV07); matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |
| Startup contract recorded for W03–W05, W09, W13 | [handoff](06-validation-and-handoff.md) §3 | W02 closure review (W02-DV08) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): the repository is a P0 documentation scaffold — no Cargo
workspace, no Rust sources, code directories contain only `.gitkeep`. P1 and
P2 are planning sets with no implementation or verification evidence; the
task book §2 explicitly keeps their inputs as *conditions for
implementation*, not claims. Sibling P3 designs (W03–W15) are being prepared
in parallel; this design consumes their published plan goals and references
them by path and P3-Wxx ID without assuming design content.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Startup uses P2-provided platform input | No P3 code exists; P2 plans define the semantic facts (CPU inventory, boot-CPU relation, PSCI capability facts) but deliver no types | A typed consumption boundary: W02 reads start capability (conduit class, CPU_ON function id) from `TopologyInputs` (W01) and fails closed if unusable | Guessing conduit or function ids would couple Core to QEMU and violate ADR-044 | W02 boundary; P2-W10 facts; W01 aggregate | W02-DV01 review that no platform constant exists; DV02 request-path tests |
| Each expected secondary reaches a defined EL2 result | Nothing starts any CPU | The secondary entry path: a positioned entry symbol, entry-state revalidation under the P1 contract, identity confirmation, minimal local environment | "Reaches EL2" is only diagnosable if the entry path revalidates the same contract P1 validates for the boot CPU | W02 entry design; P1-W01/P1-W04 entry and state contracts | W02-DV03 |
| Result reported with CPU and phase attribution | No outcome model exists | `SecondaryStartOutcome` plus `StartPhase` enumeration and the single-writer arrival mailbox | P3-V02's passing condition is CPU/phase-specific reporting; without phases, a failure is opaque | W02 | W02-DV04/DV05 |
| Timeout is detected | No timer exists in P1–P3 scope (P6 owns timers) | A bounded-poll timeout constant with an explicit recorded limitation | Bring-up must terminate; an unbounded wait would hang boot with no diagnostic | W02 (stage-local freedom, recorded); revisit at P6 | W02-DV05 (host-side watcher tests) |
| Provisional execution environment for a starting secondary | P3-W04 (real per-CPU stacks/areas) is a downstream consumer, not a prerequisite | One provisional stack per attempted secondary, allocated from the P2 page allocator before release, with explicit ownership transfer/quarantine rules | A secondary cannot execute C/Rust prologue without a stack, and W02 cannot depend on W04 (plan index ordering) | W02 (stage-local scaffolding, recorded) | W02-DV03; ownership rules reviewed in DV01 |
| Repeatable evidence for expected CPU counts and an induced failure path | No QEMU harness exists (P0-W09 and P3-W13 are plans) | Bring-up diagnostics emitted per CPU; an induced-failure input (CPU_ON targeting an identity outside the inventory) exercisable via the P0 QEMU entry path | P3-V02 requires an induced failure path; QEMU exposes absent-CPU rejection deterministically, true device-level timeout it does not | W02 for diagnostics and the failure input; W13 for the matrix | W02-DV06/DV07; the timeout-on-target limit is stated in the matrix |

No ledger row requires this design to fix a crate name, target, or policy
choice owned elsewhere, so no new decision blocker is outstanding here.

## Resolved design decisions and their authority

1. **Start mechanism: PSCI CPU_ON only.** Rationale: the ADR P3 roadmap and
   task book name PSCI CPU_ON; P2 delivers the conduit and function ids as
   assumed contract; a spin-table path would introduce release-address
   memory and cache-maintenance obligations P3 does not otherwise need.
   Spin-table is Reserved with the trigger recorded in the scope
   classification. Selecting PSCI here is stage-local design freedom within
   the plan's declared start-mechanism boundary, not a new architecture.
2. **Single audited `unsafe` boundary for the PSCI call.** One function
   executes the HVC or SMC instruction selected by the conduit recorded in
   `StartCapabilityFacts` (W01) with a `SAFETY` comment covering register
   clobbers and the exception-level contract inherited from P1. No other
   code may issue firmware calls.
3. **Timeout by bounded poll, not by timer.** The requester polls the
   arrival mailbox for a fixed per-CPU iteration bound. Rationale: no timer
   exists in the P1–P3 baseline (P6 owns timers); the bound is a
   stage-local constant recorded with its limitation — it proves *no
   arrival within the bound*, not a wall-clock timeout. Timer-based
   measurement is Reserved for P6 integration.
4. **Provisional stacks from the P2 page allocator, boot-CPU-allocated.**
   One stack per attempted secondary, allocated and populated before any
   CPU_ON is issued, so all boot-phase allocation stays single-threaded.
   Ownership: on success the stack's ownership passes to the P3-W04
   per-CPU runtime design (which may keep or replace it); on failure the
   stack is quarantined (never freed, never reused within P3) because
   failed-CPU reuse is a lifecycle decision P3-W03 does not make. Size is
   a stage-local constant bounded to entry validation and local
   initialization, not to runtime service.
5. **Arrival protocol: single-writer mailbox per CPU, no locks.** Each
   attempted secondary has one mailbox word plus one result record. The
   secondary writes the result record then release-stores the mailbox
   sentinel; the requester acquire-polls the mailbox. Rationale: the
   protocol needs no lock, keeps W02 clear of P3-W06's lock semantics, and
   its ordering requirements are fully statable with acquire/release.
6. **Identity confirmation is MPIDR-based, cross-checked against the
   start request.** The secondary reads its own MPIDR (P1 baseline
   mechanism), derives the expected identity, and cross-checks the logical
   id passed in the PSCI context id. A mismatch is a start failure with
   phase attribution, never a "best effort" continuation. Rationale:
   ADR-015 demands per-CPU identity certainty before any per-CPU state is
   touched; the guardrail requires MPIDR-typed identity.
7. **Degraded results are observable; continuation policy is surfaced, not
   chosen.** W02 produces per-CPU terminal outcomes; whether boot
   continues with a degraded online set or halts is recorded as an input to
   the boot-integration owner (P3-W05 state model plus the P1-W09-style
   boot sequencing consumer). Rationale: the plan explicitly removes that
   policy from W02's scope; the reference-boot default (continue with
   diagnostics so failures stay repeatable and diagnosable) is recorded as
   the consumers' working assumption, revisitable without redesigning W02.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, scope split, and decisions; confirm the W01 and P2/W1
   prerequisites named there are available as reviewed deliverables.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   requester/secondary split, the phase model, the mailbox protocol, and
   the concurrency rules.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   requester side with
   [04](04-code-contracts-start-requester.md) (steps 1–3), secondary side
   with [03](03-code-contracts-secondary-entry.md) (steps 4–5).
4. Record implementation decisions in
   `../p3-w02-secondary-cpu-bring-up-record.md` and evidence in
   `../../verification/p3-w02-secondary-cpu-bring-up-verification.md` only
   when the work is actually performed. Validation conditions and the
   handoff checklist are in
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No lifecycle transition API (W03 owns the semantics; W02 only calls the
designated transition operations it is named as owner of), no per-CPU data
area or CPU-local accessor (W04), no rendezvous/barrier primitive (W05), no
lock type (W06), no IPI/SGI send path (W07), no TLB transport (W08), and no
guest-visible PSCI surface (P8) is designed or authorized by W02. The
PSCI call is authorized solely for CPU_ON in the boot-CPU bring-up context;
a `SYSTEM_OFF`, `CPU_OFF`, or any other function reaching W02 code is a
scope violation. Any "restart a failed CPU" path is out of scope and must
be stopped at review.

## Downstream handoff

- **W03** receives the bring-up result stream: each attempted CPU's terminal
  outcome and phase-on-failure, plus the named transition operations W02
  requests (request-start, entered-initializing, report-failure). W03 owns
  the state machine those requests drive.
- **W04** receives the provisional-stack ownership transfer on success and
  the quarantine rule on failure; W04's design decides what replaces the
  provisional environment.
- **W05** receives the dispatch-point contract: CPU_ON issuance happens
  only after global-initialization publication, and the requester's
  outcome map is the expected-attempted-set input to the rendezvous.
- **W09** receives the invariant that a secondary entering the fatal
  diagnostic path carries correct CPU attribution from the first
  instruction of the entry stub.
- **W13** receives the repeatable bring-up scenario definition (expected
  counts plus the induced absent-CPU failure input) for the regression
  matrix, and the recorded limitation that on-target timeout induction is
  not available at P3.
- **P4** consumes bring-up only through
  [P3-W14](../p3-w14-p4-smp-handoff/README.md); W02 defines no vCPU or
  guest-start concept.
