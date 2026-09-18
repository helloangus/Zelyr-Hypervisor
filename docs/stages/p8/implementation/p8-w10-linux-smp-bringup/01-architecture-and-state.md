# P8-W10 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W10 detailed design](README.md).

## 1. Logical modules

W10 is an integration layer; its modules coordinate existing owners rather
than owning mechanisms.

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| M1 Topology registry (SMPT) | The single coherent map: vCPU index ↔ virtual MPIDR ↔ DTB CPU node ↔ scheduling entity; consistency enforcement at VM creation | The topology map (machine-gated values) | Machine facts (W02 gate), W04 DTB node facts | Lookups for W06 target validation, W07 frame map, W04 consistency review | Does not choose values; does not place vCPUs (P7-W03) |
| M2 Bring-up pipeline (SMPP) | Orchestrate CPU_ON → lifecycle admission → pending W03 entry state → scheduler placement → first entry | None (every step is an owning contract's operation) | CPU_ON calls ([W06](../p8-w06-psci-virtualization/README.md)), P7 admission results | Transitioned, schedulable secondary vCPUs | Does not implement admission, placement, or entry-state construction |
| M3 Per-CPU integration sequencer (SMPI) | Verify/observe that each secondary completes the per-CPU init sequence (GICR, timer, SGI, idle) before being counted ready | Per-vCPU readiness record (bounded, diagnostic) | [W07](../p8-w07-linux-vgicv3/README.md)/[W08](../p8-w08-linux-timer-integration/README.md) telemetry, console markers ([W09](../p8-w09-virtual-console-single-cpu-linux/README.md)) | Readiness state for diagnostics/evidence | Not a gate that blocks Guest execution — the Guest owns its init order; this observes and records |
| M4 Hotplug/offline integrator (SMPH) | Bind CPU_OFF/AFFINITY_INFO cycles to lifecycle + quiesce hooks; verify state-identical re-ON | Per-vCPU cycle counters (diagnostic) | CPU_OFF/AFFINITY_INFO ([W06](../p8-w06-psci-virtualization/README.md)), quiesce hooks ([W07 §5 of 02](../p8-w07-linux-vgicv3/02-code-contracts-vgic-mmio.md), [W08 §5 of 03](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md)) | Stopped/started vCPUs with consistent per-CPU state | Does not implement quiesce logic (owners do) |
| M5 Stability scenario engine (SMPS) | Define, bound, and interpret the declared scenarios of [03](03-stability-scenarios.md); correlate telemetry + console log into scenario verdicts | Scenario run records (bounded) | Telemetry streams, console log, fixture workload progress markers | Scenario verdicts (passed/failed/inconclusive) for W16 | Not the workload implementation (W15/W16); not a KPI collector (W17) |
| M6 Diagnostics integration (SMPD) | Map race/lost-event symptoms to the P3-W08/P7-W09/W07/W08 telemetry and the W13 diagnostic context | One diagnostic slot per vCPU (bounded) | All modules | W13-consumable context | Not a fault taxonomy (W13) |

Layering: no board/SoC/QEMU constants; all topology values are machine-gated
data (README decision D7).

## 2. Topology facts and coherence (decision D1)

```text
Name and stability: VmTopology — internal; machine-gated, immutable after VM creation
Contents:
  vcpu_count:  <VCPU-COUNT>       — {1, 2, 4} for the P8 fixtures
  mpidr(n):    <VCPU-MPIDR(n)>    — affinity0 = n, higher levels 0 (proposed rule;
                                  frozen by the W02 gate with W04's DTB facts)
  per-vCPU: scheduling entity (P7), GICR frame ([W07 §2](../p8-w07-linux-vgicv3/01-architecture-and-state.md)),
            timer state ([W08 §3](../p8-w08-linux-timer-integration/01-architecture-and-state.md))
Creation-time consistency checks (fail VM creation):
  C1  DTB CPU-node count == vcpu_count, and each node's reg (MPIDR) matches mpidr(n)
  C2  W07 GICR frame map covers exactly the topology ([W07 §2](../p8-w07-linux-vgicv3/01-architecture-and-state.md))
  C3  W06's target-validation table derives from this map, not a second list
      ([W06 dispatch §3](../p8-w06-psci-virtualization/02-code-contracts-psci-dispatch.md))
  C4  the boot vCPU (W03 contract) is topology member 0
```

Coherence C1–C4 is the substance of the P8-V14 enumeration rows: Linux
enumerates from DTB (C1), addresses secondaries by MPIDR (C3), and the
Hypervisor presents frames/timers per member (C2). Any divergence is a
machine-contract review failure (P8-V05/P8-V02/V03), fixed in the contract —
never patched in code.

## 3. Core objects and ownership

No new long-lived object class; three per-vCPU diagnostic records owned by
M3/M4/M6:

- `cpu_readiness` (M3): per-vCPU {gicr_done, timer_done, idle_entered} flags
  derived from the owners' telemetry; diagnostic only — the Guest never
  reads it and the Hypervisor never gates on it (decision D3: the Guest owns
  its init order).
- `hotplug_cycle_count` (M4): per-vCPU counter of OFF/ON cycles; bounded,
  wrap-tolerant; input to the repeated-lifecycle scenario S3b.
- `smp_diagnostic` (M6): per-vCPU bounded slot for the W13 context (last
  lifecycle transition, last telemetry anomalies).

The authoritative SMP state remains exactly where the owners put it: vCPU
lifecycle (P7), pending entry state (vCPU object, W06/W03), GICR/interface
state ([W07](../p8-w07-linux-vgicv3/01-architecture-and-state.md) §3.2),
timer state ([W08](../p8-w08-linux-timer-integration/01-architecture-and-state.md)
§2). W10 owns no second copy of any of them.

## 4. Bring-up lifecycle (end-to-end view)

```text
VM creation      topology built and checked (C1–C4); boot vCPU enters (W03)
Linux boot CPU   enumerates N CPUs from DTB; calls PSCI CPU_ON per secondary
  per secondary: W06 target validation (C3) -> P7 admission (Offline->Runnable,
                 pending W03 secondary entry state) -> placement (P7-W03 policy)
                 -> first entry at entry IPA, x0=context_id
  secondary:     Linux arch init -> GICR wake/enable ([W07]) -> per-CPU timer
                 registration ([W08]) -> SGI/IPI availability -> idle loop (WFI,
                 [W08 §4 of 03](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md))
online:          Linux reports the CPU present; M3 readiness flags observed
hotplug:         Linux CPU_OFF -> [W06] stop + quiesce hooks -> AFFINITY_INFO
                 polling sees OFF -> later CPU_ON re-ON; cycle must be
                 state-identical (S3b)
fault:           secondary fault -> P7-W07 vCPU Faulted + W13 context; Linux
                 observes a hung/dead CPU and uses its own hotplug recovery;
                 Hypervisor state stays consistent ([02 §6](02-code-contracts-secondary-bringup.md))
```

## 5. Concurrency model

- **Guest-side paths** (CPU_ON/CPU_OFF handling) inherit the W06 contracts:
  bounded locks, no waiting on other vCPUs, no allocation, VM-exit context.
- **Cross-pCPU execution:** 2/4-vCPU fixtures may place vCPUs on distinct
  pCPUs (P7-W03 placement); every shared-structure access on the SMP paths
  (VM GIC lock, device lock, per-VM tables) uses the owners' declared lock
  orders — [W07 §6](../p8-w07-linux-vgicv3/01-architecture-and-state.md)
  (vCPU before VM GIC), [W09 §5](../p8-w09-virtual-console-single-cpu-linux/01-architecture-and-state.md)
  (device), P3's ordering rules otherwise. W10 introduces no new lock.
- **Stage-2/TLB surface:** concurrent vCPUs touch the shared
  GuestAddressSpace; mapping changes and shootdowns follow the P4 contracts
  and the P3-W08 TLB transport (`docs/stages/p3/plans/p3-w08-tlb-shootdown-transport.md`).
  W10's scenarios exercise this surface; W12 owns the memory-isolation
  matrix. W10 introduces no new TLB semantics.
- **IPI fan-out:** Linux IPIs travel the [W07 SGI](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md)
  §2 path; issuance is non-joining on both the W07 and P7 sides, so a Guest
  cannot make one vCPU's IPI storm block another vCPU's Hypervisor work
  (bounded-work property per access).

## 6. Race/lost-event diagnostic dependencies (plan work sequence 5)

| Symptom class | Detecting evidence | Owner of the evidence |
|---|---|---|
| Lost wakeup / hang | WFI block/telemetry + timer expiry events vs. vCPU progress | [W08](../p8-w08-linux-timer-integration/README.md) telemetry; P7-W06 |
| Lost/duplicated interrupt | W07 injection/EOI outcome events + counters vs. Guest-handled IRQ counts (console log) | [W07](../p8-w07-linux-vgicv3/README.md) telemetry |
| Stale TLB/state after migration | P3-W08 shootdown counters + Stage-2 fault anomalies (W13 context) | P3-W08 transport; P4 facts |
| Scheduler starvation/accounting drift | P7-W09 runqueue/accounting telemetry | P7-W09 |
| Corrupted console evidence | [W09](../p8-w09-virtual-console-single-cpu-linux/README.md) faithful-record rule + drop counters | W09 backend |

M5 correlates these streams per scenario ([03 §3](03-stability-scenarios.md));
a verdict requires the Hypervisor-side streams, not the Guest log alone
(README decision D6).

## 7. Failure boundaries of assumed prerequisite contracts

| Prerequisite | Assumed contract (cite) | Failure boundary if different |
|---|---|---|
| W06 CPU_ON/OFF/AFFINITY_INFO | `../p8-w06-psci-virtualization/README.md` contracts | Any behavioral divergence (errors, state semantics) breaks the pipeline's first step; raise at the W06/W10 seam |
| W03 secondary boot state | `docs/stages/p8/plans/p8-w03-linux-boot-contract.md` | Without a secondary entry state, [02 §3](02-code-contracts-secondary-bringup.md) blocks |
| W04 DTB CPU nodes | `docs/stages/p8/plans/p8-w04-guest-dtb-contract.md` | C1 mismatch fails the consistency review; fix in W04's contract |
| W07 per-vCPU GIC + SGI | `../p8-w07-linux-vgicv3/README.md` | If per-vCPU state or SGI path is missing, [02 §4](02-code-contracts-secondary-bringup.md) rows block |
| W08 per-vCPU timer + WFI | `../p8-w08-linux-timer-integration/README.md` | If continuity (I5) or WFI sources are missing, idle rows block |
| P7 admission/placement/stop/accounting | `docs/stages/p7/plans/p7-w02-scheduler-admission-lifecycle.md`, `p7-w03-placement-configuration.md`, `p7-w07-pause-stop-fault.md`, `p7-w09-accounting-diagnostics.md`, handoff `p7-w14-documentation-p8-handoff.md` | A missing seam blocks the corresponding pipeline stage; `Architecture Change Request` against the seam, no local scheduler patch |
| P3 Host SMP + TLB transport | `docs/stages/p3/plans/p3-w14-p4-smp-handoff.md`, `p3-w08-tlb-shootdown-transport.md` | Without the substrate, multi-pCPU placement scenarios block (single-pCPU time-sliced runs remain possible and are the recorded fallback for bring-up debugging — a validation limitation, not an implementation mode change) |
| W09 console baseline | `../p8-w09-virtual-console-single-cpu-linux/README.md` | Without the green Stage D 1-vCPU baseline, SMP stages are blocked (build-up discipline of [04](04-implementation-workflow.md)) |
| W02 machine gate | `docs/stages/p8/plans/p8-w02-machine-contract-governance.md` | Unapproved `<VCPU-COUNT>`/MPIDR rule ⇒ no implementation may embed them |
