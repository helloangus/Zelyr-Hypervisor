# P8-W10 Linux SMP Bring-up — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Linux 2- and 4-vCPU integration path over the PSCI, DTB, vGIC,
timer, and scheduler foundations — virtual topology, secondary-vCPU bring-up
pipeline, per-CPU integration sequence, hotplug/offlining, and the declared
SMP stability scenario set — per
[P8-W10](../../plans/p8-w10-linux-smp-bringup.md).  
**Owner/change context:** P8-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W10. It is an **integration**
design: it introduces no new device, ABI, or lifecycle mechanism. It binds
the sibling contracts — [W04](../p8-w04-guest-dtb-contract/README.md)
topology facts, [W06](../p8-w06-psci-virtualization/README.md) CPU_ON/OFF,
[W07](../p8-w07-linux-vgicv3/README.md) per-CPU GICR/SGI,
[W08](../p8-w08-linux-timer-integration/README.md) per-CPU time and WFI,
[P7](../../../p7/plans/p7-w14-documentation-p8-handoff.md) scheduler seams —
into the ordered path by which 2- and 4-vCPU Linux boots, enumerates, starts
secondaries, and survives the declared stability workloads (P8-V14/V15). The
stability scenarios are specified here as semantic contracts; their
automation is [W16](../../plans/p8-w16-automated-linux-regression.md) and the
fixture is [W15](../../plans/p8-w15-reproducible-linux-fixture.md).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned work | Load |
|---|---|
| Topology facts, bring-up pipeline, lifecycle interactions, concurrency | [01 Architecture and state](01-architecture-and-state.md) |
| Secondary bring-up and per-CPU integration contracts | [02 Secondary bring-up contracts](02-code-contracts-secondary-bringup.md) |
| Stability scenario contracts for P8-V15 | [03 Stability scenario contracts](03-stability-scenarios.md) |
| Implement in dependency order | [04 Implementation workflow](04-implementation-workflow.md) |
| Validate and hand off | [05 Validation and handoff](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P8
task book, P8-W10 plan, and the sibling contracts cited above. This document
proposes design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → the P8-W02
machine-contract gate → P8-W10 plan → this design → Coding Guidelines. In
particular:

- ADR-015 makes SMP an early foundation capability and ADR-016 keeps
  vCPU↔pCPU binding a scheduler policy — W10 consumes placement; it never
  decides it. The [W11](../../plans/p8-w11-scheduler-linux-integration.md)
  scheduler-integration package owns 1:1/M:N cooperation; W10 owns the
  bring-up path and stability evidence that W11 builds on.
- ADR-007/§19 (Guest untrusted, containment) apply to every SMP path: a
  malformed CPU_ON, a crashing secondary, or an interrupt storm is contained
  to the VM ([W06](../p8-w06-psci-virtualization/README.md)/[W13](../../plans/p8-w13-guest-fault-diagnostics.md)
  contracts).
- Task book §8 routes CPU-topology values through the
  [W02](../../plans/p8-w02-machine-contract-governance.md) gate; W10 fixes
  the semantics of topology consumption, not the values.
- The plan excludes scheduler redesign, topology values, Host SMP bring-up
  (P3's), real-board scale claims, and final stress implementation (W16
  automates the declared scenarios of [03](03-stability-scenarios.md)).

Classification:

- **Required:** virtual-topology consumption (DTB ↔ virtual MPIDR ↔ vCPU
  identity coherence); the secondary bring-up pipeline (PSCI CPU_ON →
  lifecycle admission → W03 secondary state → per-CPU GICR/timer/SGI/idle
  init); hotplug/offline path (CPU_OFF + AFFINITY_INFO cycles); the six
  declared stability scenarios with their observables; race/lost-event
  diagnostic integration (P3-W08 TLB transport, P7-W09 accounting); 2- and
  4-vCPU matrices.
- **Reserved:** topology values beyond 4 vCPUs; M:N/shared-CPU scheduling
  scenarios (W11 owns; W10's scenarios run under the current scheduler);
  CPU-hotplug stress beyond the declared cycle scenarios; power-management/
  cpuidle states beyond WFI (none in the v1 machine).
- **Out of Scope:** Host pCPU bring-up (P3); scheduler algorithm/policy
  changes (P7/W11); ITS/MSI (excluded stage-wide); real-hardware SMP
  behavior (P15); stress automation implementation (W16).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| CPU enumeration | [01 Architecture and state](01-architecture-and-state.md) §2, [02 Secondary bring-up contracts](02-code-contracts-secondary-bringup.md) §2 | P8-V14 enumeration rows |
| PSCI secondary start | [02 Secondary bring-up contracts](02-code-contracts-secondary-bringup.md) §3 (via [W06 §2 of 03](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md)) | P8-V14 secondary-start rows |
| Per-CPU timer/IRQ | [02 Secondary bring-up contracts](02-code-contracts-secondary-bringup.md) §4 | P8-V14 per-CPU rows |
| SGI | [02 Secondary bring-up contracts](02-code-contracts-secondary-bringup.md) §4 (via [W07 §2 of 03](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md)) | P8-V14 SGI rows |
| Scheduler, idle/WFI | [02 Secondary bring-up contracts](02-code-contracts-secondary-bringup.md) §4 (via [W08 §4 of 03](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md), P7 seams) | P8-V14 idle rows |
| Busy/thread/sleep/affinity/interrupt/scheduler scenarios | [03 Stability scenario contracts](03-stability-scenarios.md) | P8-V15 |
| Race/lost-event/TLB diagnostics dependencies | [01 Architecture and state](01-architecture-and-state.md) §6, [03 §3](03-stability-scenarios.md) | P8-V15 observability rows |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p8-implementation-designs`): documentation scaffold only — no Cargo
workspace, no Rust sources, no implementation records for P1–P8. Every
prerequisite below is a planned contract consumed as an assumption with a
failure boundary; the parallel-written sibling designs (W04–W09) are
referenced by slug and consumed per their plans.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| 2/4-vCPU Linux enumerates expected CPUs (P8-V14) | No SMP code or topology contract exists | Topology coherence contract of [02 §2](02-code-contracts-secondary-bringup.md) (DTB nodes ↔ virtual MPIDRs ↔ vCPU count) | Linux enumerates from DTB; a mismatch between DTB, MPIDR, and vCPU objects breaks enumeration at the first secondary | W04 owns DTB facts; W02 gates values; W10 owns coherence | P8-V14 enumeration rows (future `../../verification/p8-w10-linux-smp-bringup-verification.md`) |
| Secondaries start via standard PSCI (P8-V09/V14) | W06 CPU_ON contract planned; P7 admission planned | The bring-up pipeline of [02 §3](02-code-contracts-secondary-bringup.md) binding CPU_ON → admission → W03 entry state | A secondary that enters with wrong state or never gets scheduled fails Linux's spin-table-free bring-up immediately | W06/W03/P7-W02 seams | P8-V14 secondary rows |
| Per-CPU GIC/timer/SGI work on every vCPU | W07 per-vCPU model and W08 per-vCPU timer planned | The per-CPU init sequence contract of [02 §4](02-code-contracts-secondary-bringup.md) | Linux secondaries run GICR init, timer registration, and idle loops before they are useful | W07/W08 own the mechanisms; W10 owns the sequence | P8-V14 per-CPU rows |
| Declared stability workloads expose no unresolved race/lost event (P8-V15) | No scenario definitions exist | The six scenario contracts of [03](03-stability-scenarios.md) with observables and failure bounds | "Stable" is untestable without declared scenarios, observables, and containment behavior | W10 owns scenario semantics; W16 automates; W15 supplies workload content | P8-V15 rows |
| Crashing/hotplugged secondaries contained | No lifecycle integration exists | Hotplug/offline and fault-path contracts of [02 §5–§6](02-code-contracts-secondary-bringup.md) | CPU_OFF/AFFINITY_INFO cycles and a faulted secondary must leave consistent state | W06 errors; P7-W07 fault states; W13 diagnostics | P8-V15/V24-relevant rows |
| Topology values frozen | Machine ABI unfrozen (ADR §18) | Placeholder tokens (`<VCPU-COUNT>`, `<VCPU-MPIDR(n)>` rule) owned by the W02 gate | Values frozen outside the gate violate task-book routing | W02 gate owns values | P8-V02/V03 review rows |
| Host SMP substrate exists | P3 is planned only (`docs/stages/p3/plans/p3-w14-p4-smp-handoff.md`) | Consumed as assumed contract: pCPU identity, IPI/notification, TLB transport | Multi-vCPU execution on multiple pCPUs stands on the Host SMP substrate | P3-W14 handoff names it | P8-V15 race rows (with P7-W09/P3-W08 telemetry) |

No row above selects a final machine value inside this design alone; the
outstanding decision is the W02 gate (vCPU count, MPIDR mapping rule),
placed by the task book before implementation.

## Resolved design decisions and their authority

1. **Topology model: flat virtual sockets.** The v1 machine presents N
   vCPUs (`<VCPU-COUNT>` ∈ {1,2,4} for P8 fixtures) as one package with N
   cores, virtual MPIDRs assigned by the rule `<VCPU-MPIDR(n)>` (affinity
   level 0 = n, higher levels zero) — proposed here as stage-local design
   freedom and frozen by the W02 gate with
   [W04](../p8-w04-guest-dtb-contract/README.md) (DTB representation).
   Rationale: flat topology keeps Linux's scheduler configuration simple,
   avoids implying cache/cluster facts the P8 machine does not model, and
   matches the plan's exclusion of topology values beyond identity.
2. **One bring-up path only.** Secondary start is exclusively
   [W06 CPU_ON](../p8-w06-psci-virtualization/README.md) (no spin tables, no
   bootreg placeholders — the DTB declares `enable-method = "psci"` per
   W04). Rationale: the task book's "no private HVC dependency" and one
   reviewed path is testable; spin tables would need Guest RAM protocol
   state that the machine contract does not declare.
3. **Synchronous bring-up, asynchronous scheduling.** CPU_ON returns
   SUCCESS once the transition is committed ([W06](../p8-w06-psci-virtualization/README.md)
   contract); Linux's secondary then runs to its idle loop on its own
   schedule. W10 adds no waiting, polling, or barrier of its own —
   synchronization inside the Guest is Linux's; synchronization inside the
   Hypervisor is P3/P7's. This keeps the pipeline free of new cross-vCPU
   protocol.
4. **Pre-provisioned vCPUs; CPU_ON never creates.** All topology vCPUs
   exist at VM creation ([W03](../p8-w03-linux-boot-contract/README.md)
   boot-contract consequence); CPU_ON/CPU_OFF only transition lifecycle.
   Rationale: removes resource-exhaustion from the SMP path and matches
   W06's contract.
5. **Stability scenarios are semantic contracts, not implementations.**
   [03](03-stability-scenarios.md) defines workload classes, injected
   conditions, observables, and failure bounds; W15/W16 turn them into
   fixture content and automation. Rationale: the plan excludes "final
   stress implementation" from W10 while requiring "declared SMP stability
   workloads" — the contract/automation split is the stage's established
   pattern (as with the marker model of
   [W09](../p8-w09-virtual-console-single-cpu-linux/README.md)).
6. **Diagnostics over prints.** Lost-event/race detection relies on
   Hypervisor-side telemetry (P7-W09 accounting, W07/W08 interrupt/timer
   events, P3-W08 TLB transport counters) correlated with the Guest console
   log ([W09](../p8-w09-virtual-console-single-cpu-linux/README.md)
   faithful-record rule) — never on Guest-printed "all good" claims alone
   (same anti-forgery reasoning as W09's M7).
7. **Naming and placement:** logical names only; placement follows the
   approved workspace decision; no board/SoC/QEMU constants (ADR-043).

## Work breakdown and loading order

1. Read this README, then [01 Architecture and state](01-architecture-and-state.md)
   for the topology model, pipeline, and concurrency rules.
2. Implement in the order given by
   [04 Implementation workflow](04-implementation-workflow.md), loading
   [02](02-code-contracts-secondary-bringup.md) for bring-up/hotplug work
   and [03](03-stability-scenarios.md) for scenario definitions.
3. Record implementation decisions in
   `../p8-w10-linux-smp-bringup-record.md` when implementation begins and
   evidence in
   `../../verification/p8-w10-linux-smp-bringup-verification.md` when
   scenarios run. Neither file may claim W10 complete; P8-V14/V15 are the
   proof surfaces.

## Explicitly excluded interfaces

No scheduler policy, topology value, Host SMP mechanism, spin-table
protocol, new device, PSCI extension, or stress-automation implementation is
designed or authorized here. W10's additions are the integration contracts
and scenario semantics this design declares; anything beyond them is a scope
conflict to stop at review (at minimum W02 for topology values, P7/W11 for
scheduling, W16 for automation).

## Downstream handoff

- **W11 (scheduler Linux integration)** receives the bring-up path, the
  per-CPU integration sequence, and the continuity obligations
  ([02 §4, §6](02-code-contracts-secondary-bringup.md)) that 1:1 and M:N
  scheduling must preserve.
- **W12 (memory model)** receives the multi-vCPU Stage-2 concurrency
  surface ([01 §5](01-architecture-and-state.md)) as the isolation context
  for its RAM-size matrix.
- **W13 (fault diagnostics)** receives the secondary-fault diagnostic
  context requirements ([02 §6](02-code-contracts-secondary-bringup.md)).
- **W14 (ABI compatibility)** receives the topology/MPIDR/PSCI-start facts
  as compatibility dimensions.
- **W15/W16** receive the scenario contracts of
  [03](03-stability-scenarios.md) as fixture/automation inputs and the
  2/4-vCPU matrices of [05 §1](05-validation-and-handoff.md).
- **W17–W20** receive the stability evidence boundaries and the not-run
  discipline of [05 §3](05-validation-and-handoff.md) as the factual basis
  for closeout (W20).
