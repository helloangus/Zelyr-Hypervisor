# P8-W06 PSCI Virtualization — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The standard, Guest-visible PSCI lifecycle boundary (version and
feature query, CPU_ON/CPU_OFF, affinity status, SYSTEM_OFF/SYSTEM_RESET) that
Linux requires for secondary-vCPU startup and shutdown, per
[P8-W06](../../plans/p8-w06-psci-virtualization.md).  
**Owner/change context:** P8-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W06. It converts the bounded
work-package plan into reviewable contracts for the firmware-interface layer a
Linux Guest uses instead of a private HVC dependency: trap dispatch and
function decoding, request validation, the vCPU and VM lifecycle bridges, and
the malformed-parameter behavior required by the stage validation matrix. It
deliberately does **not** change the P5 hypercall ABI, the P7 scheduler policy,
the P4 Guest entry mechanism, the W04 DTB contract, or any Linux-side artifact;
it only binds them at explicitly stated seams.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

| Assigned work | Load |
|---|---|
| Understand modules, state ownership, lifecycle, concurrency | [01 Architecture and state](01-architecture-and-state.md) |
| Dispatch, version/features, error mapping contracts | [02 PSCI dispatch contracts](02-code-contracts-psci-dispatch.md) |
| CPU_ON / CPU_OFF / AFFINITY_INFO / SYSTEM_OFF / SYSTEM_RESET contracts | [03 CPU and system lifecycle contracts](03-code-contracts-cpu-lifecycle.md) |
| Implement in dependency order | [04 Implementation workflow](04-implementation-workflow.md) |
| Validate and hand off | [05 Validation and handoff](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P8
task book, P8-W06 plan, and the prerequisite documents named below. This
document proposes design only; it is not a completion record and contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → the P8-W02 machine-contract
gate → P8-W06 plan → this design → Coding Guidelines. In particular:

- ADR-008 places the Hypervisor in the PSCI path without taking over the
  Secure World: EL2 handles or proxies the PSCI/SMC calls a Guest must make;
  EL2 never becomes EL3 firmware. Every PSCI function this design exposes is
  answered by EL2 from virtual-machine state.
- ADR-007 and the baseline invariants (§19) make the Guest untrusted: every
  function ID, target MPIDR, entry address, and reserved-bit field arriving
  through the PSCI conduit is validated before use, and a Guest-caused failure
  is contained to the calling VM.
- The task book states that no plan-level document selects the PSCI subset,
  and routes "PSCI mandatory subset" through Specification Investigation in
  detailed design ([task book §8](../../task-book-v0.1.md)). This design
  therefore performs that investigation from the Arm PSCI specification and
  Linux boot requirements, and routes every Guest-visible frozen value (subset
  membership, conduit, reported version) through the
  [P8-W02](../../plans/p8-w02-machine-contract-governance.md) machine-contract
  freeze gate before any code treats it as ABI.
- The plan excludes management policy and Linux driver modifications. The P5
  capability model is untouched: a Guest does not need a capability to call
  its own VM's PSCI surface (it is the standard firmware interface of its
  virtual machine), but PSCI can never reach another VM, Host state, or
  management operations.

Classification:

- **Required:** HVC-conduit dispatch classification; PSCI_VERSION and
  PSCI_FEATURES semantics; CPU_ON, CPU_OFF, AFFINITY_INFO, SYSTEM_OFF,
  SYSTEM_RESET semantics; parameter validation and spec error mapping;
  VM-scoped powerdown containment; diagnostics/telemetry events; the DTB
  `arm,psci` facts consumed (not redefined) from W04; the secondary entry-state
  handoff consumed (not redefined) from W03.
- **Reserved:** CPU_SUSPEND (implemented only if an approved v1 DTB declares
  Guest idle states; the v1 proposal declares none); SYSTEM_RESET2 and other
  post-v1.1 functions; an SMC-conduit variant (trigger: the W02 gate selects
  SMC over HVC); forwarding of Guest PSCI to EL3 firmware (trigger: a future
  approved firmware-proxy requirement).
- **Out of Scope:** the P5 hypercall ABI and capability policy; scheduler
  placement or admission policy decisions (P7); Stage-2 address-space
  mechanics (P4); the DTB node contents themselves (W04); Linux configuration
  and fixture selection (W15/W03); CPU topology values (W02 gate); second-VM
  management, migration, and Secure World behavior (later stages).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| PSCI version/features semantics without private HVC dependency | [02 PSCI dispatch contracts](02-code-contracts-psci-dispatch.md) §2–§3, decisions D1/D2 | P8-V09 scenario S1 |
| CPU_ON and secondary-vCPU startup | [03 CPU and system lifecycle contracts](03-code-contracts-cpu-lifecycle.md) §2, [01 Architecture and state](01-architecture-and-state.md) §4 | P8-V09 S2; consumed by P8-V14 |
| CPU_OFF and affinity/status needs | [03 CPU and system lifecycle contracts](03-code-contracts-cpu-lifecycle.md) §3–§4 | P8-V09 S3 |
| System-off route | [03 CPU and system lifecycle contracts](03-code-contracts-cpu-lifecycle.md) §5, decision D3 | P8-V09 S4 |
| Normal and malformed-parameter acceptance scenarios | [02 PSCI dispatch contracts](02-code-contracts-psci-dispatch.md) §5; [05 Validation and handoff](05-validation-and-handoff.md) matrix | P8-V09 S5; feeds P8-V24 |
| Authority and Host-fact non-leakage review | [01 Architecture and state](01-architecture-and-state.md) §5, decision D6 | P8-V09 + host-independence review |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p8-implementation-designs`): the repository is a P0 documentation
scaffold. `git ls-files` shows no Cargo workspace, no Rust sources, no
implementation records beyond P0-W01, and `docs/stages/p8/implementation/`
contains only its README. Every P0–P7 prerequisite below is therefore a
**planned contract**, consumed as an assumption with an explicit failure
boundary, never as evidenced behavior.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Linux uses the standard PSCI path for secondary/off/system-off behavior (P8-V09) | No PSCI code, register table, or contract exists anywhere in the tree | The dispatch, validation, and lifecycle-bridge contracts in [02](02-code-contracts-psci-dispatch.md) and [03](03-code-contracts-cpu-lifecycle.md) | Without an owned PSCI boundary Linux would need a private HVC dependency, which the task book forbids | P8-W06 (this design) | P8-V09 scenarios S1–S5 (future `../../verification/p8-w06-psci-virtualization-verification.md`) |
| Guest-visible PSCI facts are frozen, not invented | Machine ABI unfrozen; ADR §18 keeps machine values open; W02 plans the governance only | Subset, conduit, and reported-version proposals (decisions D1–D3) routed through the W02 freeze gate with recorded rationale | The task book forbids freezing Guest ABI values outside the reviewed route | P8-W02 gate owns the freeze; this design owns the proposal | P8-V02/V03 review rows covering PSCI |
| CPU_ON starts a secondary vCPU that the scheduler admits | P7 scheduler is planned (`docs/stages/p7/plans/p7-w02-scheduler-admission-lifecycle.md`), not implemented | CPU_ON is specified against the P7 admission/stop seams as assumed contracts with a failure boundary ([03](03-code-contracts-cpu-lifecycle.md) §2) | A CPU_ON that returns SUCCESS while no vCPU runs would break P8-V14 | P7-W14 handoff names the seams; P8-W06 consumes them | P8-V14 secondary-start rows |
| Secondary start uses the boot state Linux expects | The Guest boot state is W03's contract, planned only | Entry-state consumption is delegated to W03; W06 supplies target validation and lifecycle transition only | Duplicate entry-state definitions would drift from the boot contract | [P8-W03](../../plans/p8-w03-linux-boot-contract.md) owns boot state | P8-V04 review; P8-V09 S2 |
| Linux discovers PSCI through the standard DTB node | DTB builder and node contract are W04 scope, planned only | W06 consumes the `arm,psci` node facts (compatible, method, function IDs) from W04; this design defines no DTB bytes | A divergent DTB fact would break "no private-HVC dependency" at boot | [P8-W04](../../plans/p8-w04-guest-dtb-contract.md) | P8-V05 consistency review |
| Malformed PSCI input is contained (feeds P8-V24) | No validation code exists; W05 owns the behavior classification categories | Per-call validation contracts and contained-failure outcomes in [02](02-code-contracts-psci-dispatch.md) §5 | Unvalidated MPIDR/entry-address handling would violate ADR §19 | W05 classification categories; this design applies them | P8-V24 illegal-PSCI scenario (W18) |

No row above requires selecting a final machine value inside this design
alone: subset, conduit, and version are proposals gated by W02, so the only
outstanding blocker is the W02 gate itself, which the task book places before
implementation.

## Resolved design decisions and their authority

1. **Compliance target:** PSCI v1.1 semantics per Arm DEN 0022 (Specification
   Investigation route of task book §8), with the mandatory subset
   {PSCI_VERSION, PSCI_FEATURES, CPU_ON, CPU_OFF, AFFINITY_INFO, SYSTEM_OFF,
   SYSTEM_RESET}; every other function ID answers `NOT_SUPPORTED` and reports
   as unimplemented through PSCI_FEATURES. Rationale: Linux requires a
   standard v0.2+ interface and probes v1.x features; the subset covers
   secondary start, hotplug offlining plus its affinity-status wait, and the
   shutdown/reboot route. The concrete reported version value and subset
   membership are frozen through the W02 gate (decision D2).
2. **Conduit:** HVC as the Guest PSCI conduit, selected by the DTB
   `method = "hvc"` fact owned by W04 and frozen by W02. Rationale: the Guest
   is NS EL1 and ADR-008 keeps EL3 out of the path; the HVC exception route is
   already a required EL2 boundary; routing Guest SMC separately is a policy
   alternative reserved until W02 selects it. All function-ID values
   themselves are fixed by DEN 0022, not chosen here.
3. **VM-scoped system powerdown:** SYSTEM_OFF and SYSTEM_RESET act on the
   calling VM only, never on the Host or other VMs. SYSTEM_OFF moves the VM
   toward its Stopped lifecycle state; SYSTEM_RESET terminates Guest execution
   on all vCPUs and releases the VM toward a re-loadable state from which the
   Host may re-establish the W03 boot inputs. This is stage-local design
   freedom owned by this design, justified by ADR §19 (a Guest-caused fault
   must not affect the Host) and ADR §4.1 VM lifecycle; the Guest-visible
   outcome (execution ceases) is invariant regardless of the Host's restart
   mechanism.
4. **Lifecycle delegation:** CPU_ON performs target validation and a
   lifecycle transition only; the secondary entry register state, artifact
   lifetime, and boot vCPU facts are consumed from W03, and Runnable
   admission, placement, and stop semantics are consumed from P7
   (`p7-w02-scheduler-admission-lifecycle.md`,
   `p7-w03-placement-configuration.md`, `p7-w07-pause-stop-fault.md`). W06
   re-designs neither.
5. **Dispatch classification:** PSCI calls are recognized inside the unified
   HVC exception path by their SMC function-ID namespace; all other HVC
   traffic follows the P5 hypercall route. Because P5's hypercall numbering is
   itself a planned, unimplemented contract, a numbering collision is a named
   prerequisite failure boundary (see
   [02](02-code-contracts-psci-dispatch.md) §6), resolved at integration, not
   by re-numbering either ABI here.
6. **Host-fact non-leakage:** virtual MPIDR values, the reported version, and
   AFFINITY_INFO answers derive exclusively from the VM's virtual topology and
   state. No pCPU identity, Host PSCI result, or physical platform fact
   crosses the boundary; target resolution is scoped to the calling VM, so no
   capability check can be replaced by an ID check (ADR-051/§19).
7. **Internal-failure posture:** Guest-triggerable malformed input produces
   spec error codes; Hypervisor-internal failures never masquerade as PSCI
   status. They escalate through the VM/vCPU fault classification owned by
   P8-W13 and P7-W07; in the P8 model resource exhaustion cannot arise from
   CPU_ON because every topology vCPU is pre-provisioned at VM creation.
8. **Naming:** all contract names in the supporting files are logical names
   owned by this design for review purposes. Physical crate/module placement
   follows the approved workspace decision (ADR-046; P0 build-target work);
   this design fixes no file tree.

## Work breakdown and loading order

1. Read this README, then [01 Architecture and state](01-architecture-and-state.md)
   for the module map, ownership, and concurrency model.
2. Implement in the order given by
   [04 Implementation workflow](04-implementation-workflow.md), loading
   [02](02-code-contracts-psci-dispatch.md) for dispatch/version work and
   [03](03-code-contracts-cpu-lifecycle.md) for lifecycle work when the
   workflow reaches those steps.
3. Record implementation decisions and deviations in
   `../p8-w06-psci-virtualization-record.md` when implementation begins, and
   evidence in `../../verification/p8-w06-psci-virtualization-verification.md`
   when scenarios run. Neither this design nor a record may claim W06
   complete; P8-V09 evidence is the only accepted proof surface.

## Explicitly excluded interfaces

No management-ABI surface, capability right, P5 hypercall number, scheduler
policy hook signature, DTB byte, Linux-side change, EL3 firmware behavior, or
second-VM mechanism is designed or authorized here. The only Guest-visible
surfaces are the PSCI conduit response register state and the
machine-contract facts this design consumes; adding anything beyond them is a
scope conflict to stop at review (at minimum W02 for machine values, W04 for
DTB, W10 for SMP consumption).

## Downstream handoff

- **W10 (Linux SMP bring-up)** receives the CPU_ON contract and its
  ALREADY_ON/INVALID error semantics as the sole secondary-start path
  ([03](03-code-contracts-cpu-lifecycle.md) §2).
- **W14 (machine ABI compatibility)** receives the frozen-after-W02 PSCI
  facts (reported version, subset, conduit) as mandatory compatibility
  dimensions; drift detection compares against them.
- **W16 (automated regression)** receives the P8-V09 scenario list and its
  expected markers from [05 Validation and handoff](05-validation-and-handoff.md).
- **W18 (security isolation regression)** receives the malformed-PSCI
  containment scenarios (unknown function ID, bad target MPIDR, out-of-range
  entry IPA, reserved-bit violations, CPU_ON to a running target) as declared
  contained outcomes.
- **W04** receives the consumed `arm,psci` fact list (compatible string,
  method, function IDs) as a consistency input; it owns the node.
- **W13** receives the VM-scoped internal-failure escalation path (decision
  D7) as a fault-classification consumer.
