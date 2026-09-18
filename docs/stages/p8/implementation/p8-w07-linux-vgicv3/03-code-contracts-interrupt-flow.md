# P8-W07 Code Contracts — Interrupt Flow (SGI, Injection, EOI, Presentation)

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W07 detailed design](README.md).  
**Modules covered:** M4 (SGI generation), M5 (injection/EOI bridge). Register
model contracts are in [02](02-code-contracts-vgic-mmio.md).

All names are logical contract names (README decision D7); pseudocode is an
implementation outline, not runnable production code. The P6 contracts are
consumed by citation (README decision D4):
`docs/stages/p6/plans/p6-w07-virtual-interrupt-core.md` (vIRQ lifecycle),
`.../p6-w08-gic-virtualization-interface.md` (presentation capacity),
`.../p6-w09-maintenance-interrupt.md` (maintenance). If their delivered
shape differs from the assumptions in [01 §7](01-architecture-and-state.md),
this file blocks rather than forks.

## 1. Bridging rule

Every transition in this file is expressed as a call into a P6 lifecycle
operation. W07 never queues, reorders, or coalesces vIRQ state itself; its
only added semantics are the GIC register meanings that produce those calls
(priority carriage parameters, edge/level configuration, enable/mask
gating).

## 2. SGI generation (ICC_SGI1R_EL1 trap)

```text
Name and stability: vgic_sgi_generate(caller: &Vcpu, sgi_reg: u64) -> () — internal
Purpose and caller: the only Guest-to-Guest interrupt path in P8; Linux IPIs
  (reschedule, TLB, function-call) arrive here; consumed by W10's SMP matrix
Inputs: raw ICC_SGI1R_EL1 value (untrusted): TargetList, Affinity levels,
  INTID (SGI 0–15), IRM (all-peers or targeted)
Outputs: none (side effects only — per-target injection requests + telemetry)
Preconditions:
  - caller Running in Guest context; ICC_SGI1R_EL1 trapped per the W05/P6-W08
    system-register classification
  - VM topology exists (machine-gated virtual MPIDRs)
Postconditions:
  - for each validly targeted vCPU of the calling VM: one P6-W07 injection
    request for (target, INTID, priority from caller's interface state)
  - IRM=all handling resolves "all peers" against the virtual topology only
  - invalid bits/target components are dropped for that component with one
    aggregated telemetry event; no partial-state residue
State and ownership change: none of its own — target pending state changes
  only through P6-W07
Concurrency/allocation: holds caller vCPU lock then VM GIC lock (order per
  [01 §6](01-architecture-and-state.md)); per-target request issuance may
  cross pCPUs via the P6/P7 kick seams — issuance only, never joining/waiting;
  no allocation (SGI count ≤ 16, targets ≤ topology size)
Errors and failure guarantee: malformed affinity fields or INTID ≥ 16 ->
  request dropped + telemetry; a target that cannot accept (quiesced vCPU)
  follows P6-W07's unavailable-target policy; the calling vCPU is never
  blocked by a remote target
Security/authorization checks: target resolution strictly within the calling
  VM's topology (same single-point discipline as
  [W06 dispatch §3](../p8-w06-psci-virtualization/02-code-contracts-psci-dispatch.md));
  IRM expansion masked to topology size
Logic (pseudocode):
  fn vgic_sgi_generate(caller, sgi_reg):
      intid = SGI1R.intid(sgi_reg)
      if intid > 15: telemetry(rejected); return
      targets = if IRM(sgi_reg) == ALL_PEERS:
                    caller.vm.topology.peers_of(caller)
                else:
                    resolve_affinity3..0_and_targetlist(sgi_reg, caller.vm.topology)
      n_invalid = 0
      for t in targets:
          if t.is_valid_topology_member():
              P6_W07.request_inject(t, VirqRequest{
                  intid, src: caller.id, kind: SGI,
                  priority: caller.interface.pmr_scope_priority(),
                  config: caller.sgi_ppi_table[intid].config })
          else: n_invalid += 1
      P6_W07.kick_runnable_targets()        # scheduler seam, issuance only
      if n_invalid > 0: telemetry(sgi_partial_drop, n_invalid)
Validation: P8-V10 SGI rows (self-SGI, broadcast, targeted, invalid-target);
  W10 SMP matrix consumes this as the Linux IPI path; W18 SGI-abuse scenarios
```

## 3. Injection entry contract (device-facing)

```text
Name and stability: vgic_request_irq(vm_or_vcpu_target, intid, config, source) -> InjectOutcome — internal
Purpose and caller: the single entry device models use to raise a Guest
  interrupt: [W08] timer PPI expiry, [W09] console SPI; also the physical
  forwarded-SPI path when P6 provides one
Inputs: target (vCPU for SGI/PPI, VM+routed-INTID for SPI), INTID (already
  machine-table-validated by the caller), config (edge/level from the GIC
  register state), source tag for telemetry
Outputs: InjectOutcome {accepted-pending, coalesced (repeated-arrival policy
  of P6-W07), rejected-invalid, deferred-unavailable-target}
Preconditions:
  - INTID is in the MachineInterruptTable ([02 §3](02-code-contracts-vgic-mmio.md))
  - caller is a hypervisor-side device/bridge, not Guest-reachable directly
Postconditions:
  - the vIRQ's Guest-visible pending state becomes observable exactly per the
    GIC semantics: pending latches even if disabled/masked; presentation only
    when {enabled (CTLR/ISENABLER), not masked (PMR), group-enabled
    (IGRPEN1)} and P6-W08 capacity allows
  - for level-configured INTIDs the level source semantics are the device
    model's obligation; this layer tracks the latched view only
State and ownership change: P6-W07 pending state (owner: P6 lifecycle);
  latched-view fields in [01 §3] tables as derived mirrors
Concurrency/allocation: bounded; callable from device-exit contexts; no
  allocation; may issue P7 kick for a blocked target (issuance only)
Errors and failure guarantee: invalid INTID/config -> rejected + telemetry;
  unavailable target -> deferred per P6-W07; never loses an accepted request
  silently (coalescing is P6-W07's declared repeated-arrival semantics, not
  silent drop)
Security/authorization checks: caller identity is hypervisor-internal; INTID
  re-validated against the machine table (defense in depth)
Logic (pseudocode):
  fn vgic_request_irq(target, intid, config, source):
      if !machine_table.contains(intid): telemetry(rejected); return Rejected
      gate = compute_gate(target, intid)     # enabled/masked/group from [01 §3] tables
      outcome = P6_W07.request_inject(target_owner, VirqRequest{intid, config,
                            priority: table_priority(intid), source})
      if gate.allows_presentation():
          P6_W08.try_present(target, intid)   # capacity permitting; else stays pending
      telemetry(outcome, source)
      return outcome
Validation: P8-V10 timer/SPI rows; consumed by [W08]/[W09] designs; stress
  rows of [05 §1](05-validation-and-handoff.md)
```

## 4. EOI and deactivate contract

```text
Name and stability: vgic_eoi(caller: &mut Vcpu, eoir_value: u64) -> () — and —
  vgic_deactivate(caller, dir_value: u64) -> () — internal
Purpose and caller: ICC_EOIR1_EL1 / ICC_DIR_EL1 traps; maps Guest completion
  onto the P6-W07 active→completed transition; Linux issues EOI from its IRQ
  exit path
Inputs: register value (INTID field untrusted)
Outputs: none; observable effect is state transition + possible maintenance
  capacity release (P6-W09)
Preconditions: caller Running; an active view consistent with P6-W07 state
Postconditions:
  - EOImode=0 model (P8 subset): priority-drop and deactivate are combined —
    the INTID moves active→completed exactly once per presented activation
  - a spurious/duplicate EOI (INTID not active) is contained: telemetry
    event, state unchanged (declared benign per IHI 0069 rules)
  - maintenance state (P6-W09) observes the completion so LR capacity can be
    released
State and ownership change: P6-W07 lifecycle state via its completion
  operation; no register-table mutation other than derived mirrors
Concurrency/allocation: vCPU lock; bounded; no allocation
Errors and failure guarantee: invalid INTID masked; unknown-active is benign;
  never blocks, never faults the Host
Security/authorization checks: completion applies only to the calling vCPU's
  own active state — no cross-vCPU completion is expressible
Logic (pseudocode):
  fn vgic_eoi(caller, eoir):
      intid = EOIR.intid(eoir) & INTID_MASK
      if !machine_table.contains(intid): telemetry(rejected); return
      match P6_W07.complete(caller, intid):        # active -> completed
          Ok(())       => { P6_W09.notify_completion(caller, intid); telemetry(eoi) }
          Err(NotActive)=> telemetry(eoi_spurious)     # benign, contained
Validation: P8-V10 pending/active rows (state fidelity through
  set-pending→presented→EOI cycles); stress rows (no lost/duplicate
  completion); W18 storm scenarios
```

## 5. Presentation and pressure integration (consumed)

```text
Name and stability: vgic_presentation_tick(vcpu) -> () — internal
Purpose and caller: called at vCPU entry (after [01 §5] resume) and after any
  M5 transition that could enable presentation; drives the P6-W08 selection
  of pending work into available List-Register capacity
Inputs: vCPU
Outputs: presentation attempts for eligible pending vIRQs in priority order
Postconditions:
  - eligible = enabled ∧ not-masked ∧ group-enabled ∧ target runnable
  - over-capacity pending work remains pending (P6-W07 deferred); no loss,
    no duplicate presentation
  - P6-W09 maintenance events release capacity; the next tick presents
    newly eligible work — bounded progress, not fairness (plan excludes
    optimization)
State and ownership change: P6-W08 presentation state via its operations
Concurrency/allocation: bounded per tick (attempts ≤ capacity + small
  constant); no allocation; runs in vCPU-entry context
Errors and failure guarantee: internal presentation failure -> W13 fault
  path; guest-visible rule is "eventually delivered, never lost, never
  duplicated" (P6-W07/P6-W09 evidence underpins this claim)
Logic (pseudocode):
  fn vgic_presentation_tick(vcpu):
      for intid in vcpu.pending_eligible_by_priority():     # bounded scan
          if P6_W08.has_capacity(vcpu):
              P6_W08.present(vcpu, intid)
          else:
              return     # remaining work stays pending; maintenance will recall
Validation: P8-V10 over-capacity and maintenance-adjacent rows (with the
  P6-W08/W09 evidence boundary); W10 per-CPU interrupt rows
```

## 6. Consumer obligations recorded here

- **[W08](../p8-w08-linux-timer-integration/README.md)** injects its timer
  PPI exclusively through §3 and honors the level/edge config from the GIC
  table.
- **[W09](../p8-w09-virtual-console-single-cpu-linux/README.md)** injects the
  console SPI through §3 and must never assume delivery timing (only
  eventual-delivery semantics).
- **[W10](../p8-w10-linux-smp-bringup/README.md)** treats §2 as the Linux IPI
  path and counts SGI success via telemetry, not by private markers.
- **[W13](../../plans/p8-w13-guest-fault-diagnostics.md)** consumes the
  diagnostic context of [02 §6](02-code-contracts-vgic-mmio.md).
- Any consumer needing a new register behavior extends this design first;
  silent register emulation outside [02 §2/§4] tables is a review failure.
