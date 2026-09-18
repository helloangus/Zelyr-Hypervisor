# P8-W10 Code Contracts — Secondary Bring-up and Per-CPU Integration

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W10 detailed design](README.md).  
**Modules covered:** M1 (topology), M2 (pipeline), M3 (per-CPU sequencer),
M4 (hotplug). Scenario contracts are in
[03](03-stability-scenarios.md).

All names are logical contract names (README decision D7); pseudocode is an
implementation outline, not runnable production code. Every step below
delegates to an owning sibling contract; W10's contracts are the orchestration
and the consistency obligations between them.

## 1. Consumed contracts (the pipeline's moving parts)

| Step | Contract consumed | Owner |
|---|---|---|
| CPU_ON semantics, ALREADY_ON/INVALID errors | [W06 CPU lifecycle §2](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md) | W06 |
| Secondary entry register state (PC, x0=context_id, EL1, MMU off) | W03 boot contract (`docs/stages/p8/plans/p8-w03-linux-boot-contract.md`) | W03 |
| Lifecycle admission Offline/Stopped → Runnable | P7-W02 (`docs/stages/p7/plans/p7-w02-scheduler-admission-lifecycle.md`) | P7 |
| Placement of the admitted vCPU | P7-W03 (`docs/stages/p7/plans/p7-w03-placement-configuration.md`) | P7 |
| GICR wake/enable, SGI/PPI frame init | [W07 Redistributor §4](../p8-w07-linux-vgicv3/02-code-contracts-vgic-mmio.md) | W07 |
| SGI (IPI) generation/reception | [W07 SGI §2](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md) | W07 |
| Per-CPU timer registration semantics, continuity | [W08 registers](../p8-w08-linux-timer-integration/02-code-contracts-timer-regs.md), [W08 continuity §5](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md) | W08 |
| Idle/WFI blocking and wakeups | [W08 WFI §4](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md), P7-W06 | W08/P7 |
| CPU_OFF quiesce hooks | [W06 CPU_OFF §3](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md), [W07 hooks §5](../p8-w07-linux-vgicv3/02-code-contracts-vgic-mmio.md), [W08 hooks §5](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md) | W06/W07/W08 |

## 2. Topology registry contract (M1)

```text
Name and stability: topology_build(machine_facts, dtb_facts) -> Result<VmTopology, TopologyError> — internal
Purpose and caller: construct and verify the coherence map C1–C4 of
  [01 §2](01-architecture-and-state.md) at VM creation
Inputs: approved machine facts (<VCPU-COUNT>, MPIDR rule), W04 DTB CPU-node facts
Outputs: the immutable VmTopology, or a creation failure
Preconditions: machine facts approved by the W02 gate; DTB facts produced by
  the W04 contract
Postconditions:
  - C1–C4 all hold or creation fails with a structured error (no partially
    coherent VM ever boots)
  - the map is immutable afterwards; all consumers (W06 validation, W07
    frames, evidence) derive from it
State and ownership change: topology object owned by the VM
Concurrency/allocation: VM-creation context (allocation permitted here, in
  contrast to the VM-exit paths); once built, read-only
Errors and failure guarantee: TopologyError names the failed check (C1–C4);
  no fallback topology is ever substituted
Security/authorization checks: values originate from approved machine facts
  only; no Host CPU identity participates
Logic (pseudocode):
  fn topology_build(mf, dtb) -> Result<VmTopology>:
      check C1: dtb.cpu_nodes.len() == mf.vcpu_count
                and all(n.mpidr == mf.mpidr(n) for n in dtb.cpu_nodes)?
      check C2: gicr_frame_map.covers(mf)?                 # W07 facts
      bind C3: psci_target_table = derive(mf)              # W06 consumes
      check C4: dtb.boot_cpu == mf.mpidr(0)?
      VmTopology { .. }
Validation: DV01 consistency review; P8-V05; W14 topology dimensions
```

## 3. Secondary bring-up pipeline contract (M2)

```text
Name and stability: smp_secondary_bringup(caller, target_mpidr, entry_ipa,
  context_id) -> PsciStatus — internal; the W06 CPU_ON handler's SMP-side
  orchestration body
Purpose and caller: make the standard PSCI secondary start the ONLY bring-up
  path and record its stages; called from [W06 dispatch](../p8-w06-psci-virtualization/02-code-contracts-psci-dispatch.md)
Inputs: as CPU_ON (all untrusted)
Outputs: PSCI status to the caller ([W06 §2](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md)
  owns the response semantics)
Preconditions:
  - topology built (C1–C4); CPU_ON in the frozen subset
  - the target is a pre-provisioned topology member (decision D4 — no
    creation path exists)
Postconditions:
  - SUCCESS: exactly the [W06 §2](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md)
    postconditions (pending W03 entry state, P7 admission committed), plus
    stage telemetry {validated, admitted, placed-later, entered-later}
  - every error path leaves zero residue (W06's guarantee, inherited)
State and ownership change: target vCPU lifecycle + pending entry state
  (owner: vCPU/P7); stage telemetry (M2)
Concurrency/allocation: as [W06 §2](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md)
  (target lifecycle lock, no allocation, no waiting); the later first-entry
  work is not this call's
Errors and failure guarantee: inherited from W06 verbatim; no new error
  classes are introduced at this layer
Security/authorization checks: inherited from W06 (single-point VM-scoped
  target resolution); nothing here may widen the accepted target set
Logic (pseudocode):
  fn smp_secondary_bringup(caller, target_mpidr, entry_ipa, context_id):
      # the body IS W06's cpu_on; this contract pins the SMP obligations:
      t = validate_psci_target_mpidr(caller, target_mpidr)?     # C3-derived
      ... W06 cpu_on body ...                                   # [W06 §2]
      telemetry(stage: committed, vcpu: t.id)
      # M3 observes 'entered' later from entry telemetry; never gates here
Validation: P8-V14 secondary rows; S3b repeated lifecycle; W18 CPU_ON abuse
  (inherited S5c/S5d/S5e)
```

## 4. Per-CPU integration sequence contract (M3)

The sequence below is what a healthy secondary's evidence must show, in
order. It is an observation contract (decision D3): M3 records readiness
from the owners' telemetry; it never blocks or paces the Guest.

| Stage | Guest-observable action | Hypervisor-side evidence (owner) |
|---|---|---|
| P1 | secondary enters at entry IPA, x0=context_id | entry telemetry ([W06]/P4) |
| P2 | GICR wake + SGI/PPI enables for this CPU | [W07](../p8-w07-linux-vgicv3/README.md) access telemetry (WAKER, ISENABLER0 writes) |
| P3 | per-CPU timer registered; first tick programmed | [W08](../p8-w08-linux-timer-integration/README.md) program telemetry |
| P4 | first SGI send/receive involving this CPU | [W07 SGI](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md) telemetry |
| P5 | idle loop entered (WFI) | [W08 WFI](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md) block telemetry |
| P6 | Linux marks the CPU online (console marker, [W09](../p8-w09-virtual-console-single-cpu-linux/README.md)) | marker in faithful log |

```text
Name and stability: smp_record_readiness(vcpu, stage: P1..P6) -> () — internal
Purpose and caller: per-CPU readiness bookkeeping for diagnostics/evidence;
  called from the owners' telemetry hooks
Inputs: vCPU, stage
Outputs: updated readiness record; completion event when P1–P6 all observed
Preconditions: vCPU is a topology member
Postconditions: monotone per boot cycle; reset on CPU_OFF (S3b re-ON starts
  a new observation cycle); never gates execution
State and ownership change: readiness record only
Concurrency/allocation: bounded; telemetry-context safe; no allocation
Errors: none (diagnostic)
Security checks: none (hypervisor-internal)
Logic: set flag; if all six: telemetry(secondary_ready, vcpu)
Validation: P8-V14 per-CPU rows; evidence completeness for W16 verdicts
```

## 5. Hotplug/offline integration contract (M4)

```text
Name and stability: smp_hotplug_cycle_record(vcpu, phase: OffRequested | OffDone |
  OnRequested | OnDone) -> () — internal
Purpose and caller: bind the CPU_OFF/AFFINITY_INFO round trip to observable
  state so the S3b repeated-lifecycle scenario has a contract
Inputs: vCPU, phase (from W06 handler telemetry and entry telemetry)
Outputs: cycle counter; state-consistency assertion points
Preconditions: vCPU is a topology member
Postconditions:
  - OffDone implies the quiesce hooks ([W07 §5](../p8-w07-linux-vgicv3/02-code-contracts-vgic-mmio.md),
    [W08 §5](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md))
    completed — W06's CPU_OFF ordering guarantee, here made observable
  - OnDone after a prior OffDone starts a fresh readiness cycle (M3)
  - AFFINITY_INFO answers during the cycle are consistent with the phases
    ([W06 §4](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md))
State and ownership change: cycle counter only
Concurrency/allocation: bounded; telemetry-context safe
Errors: inconsistency (e.g. OnDone without lifecycle evidence) is an internal
  fault (W13 path) — a consistency violation, not a diagnostic event
Security checks: none beyond W06's
Logic: update phase; check adjacent-phase consistency; count completed cycles
Validation: S3b (OFF→ON→OFF state-identical); W16 hotplug regression rows
```

## 6. Secondary fault path contract (M6 + P7/W13)

```text
Name and stability: smp_secondary_fault(vcpu, context) -> () — internal
Purpose and caller: a secondary that faults (Guest-caused Stage-2 fault,
  unsupported operation, abort class) must degrade the VM, not the Host or
  its sibling vCPUs
Inputs: faulting vCPU, W05/W13-classified context
Outputs: P7-W07 vCPU Faulted transition request; W13 diagnostic context
  (VM/vCPU id, PC/PSTATE-class info, syndrome, exit reason — per W13's
  declared fields); sibling vCPUs unaffected
Preconditions: fault classified by the owning path (W05 categories)
Postconditions:
  - the faulting vCPU stops executing; its per-CPU state is quiesced via the
    same hooks as CPU_OFF
  - Linux's own recovery (hotplug-off of the dead CPU) can proceed against
    the consistent lifecycle state; the Hypervisor makes no attempt to
    resume or repair the Guest
  - repeated faults are contained per-scenario (S6 storm bound)
State and ownership change: vCPU lifecycle (P7-W07); diagnostic slot (M6)
Concurrency/allocation: bounded; VM-exit context
Errors: internal failure during fault handling escalates via W13 (never a
  Host panic for a Guest-caused fault — ADR §19)
Security checks: context derives from the faulting vCPU only
Logic (pseudocode):
  fn smp_secondary_fault(vcpu, ctx):
      smp_diagnostic_record(vcpu, ctx)          # bounded slot
      request_vcpu_fault_stop(vcpu)             # P7-W07 seam (quiesce hooks run)
      telemetry(secondary_fault, vcpu, ctx.class)
Validation: P8-V15 S6 containment; W13/W18 consumption
```

## 7. Consumer obligations recorded here

- **[W11](../../plans/p8-w11-scheduler-linux-integration.md)** must preserve
  the pipeline's invariants (pre-provisioned members, W03 entry state on
  first entry, quiesce-hook ordering) under M:N scheduling; any change is a
  revision of the owning designs first.
- **[W12](../../plans/p8-w12-linux-memory-model.md)** exercises the shared
  Stage-2 surface under concurrent vCPUs ([01 §5](01-architecture-and-state.md)).
- **[W13](../../plans/p8-w13-guest-fault-diagnostics.md)** consumes §6's
  context and M6's slots.
- **[W15](../../plans/p8-w15-reproducible-linux-fixture.md)/[W16](../../plans/p8-w16-automated-linux-regression.md)**
  implement the workload content and automation of
  [03](03-stability-scenarios.md); they must not invent additional
  pass/fail criteria.
- Any consumer needing >4 vCPUs, cluster topology, or cpuidle states extends
  the owning designs (Reserved triggers, README classification).
