# P8-W07 Code Contracts — vGIC MMIO Register Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W07 detailed design](README.md).  
**Modules covered:** M1 (access routing), M2 (Distributor), M3
(Redistributor/interface registers). Delivery/EOI/SGI flow contracts are in
[03](03-code-contracts-interrupt-flow.md).

All names are logical contract names (README decision D7); pseudocode is an
implementation outline, not runnable production code. Register semantics cite
Arm IHI 0069 (GICv3); the implemented subset is the minimum-Linux set of
README decision D2. Every numeric Guest-visible address/INTID value is
machine-gated ([01 §2](01-architecture-and-state.md)).

## 1. Access routing contract

```text
Name and stability: vgic_mmio_access(vcpu, ipa, size, is_write, value) -> AccessOutcome — internal
Purpose and caller: sole MMIO entry from the Stage-2 data-abort path for the
  GIC regions; routes to Distributor or Redistributor handlers
Inputs (all Guest-influenced, untrusted):
  ipa      — faulting Guest IPA (validated as in-region by caller routing)
  size     — access width (1/2/4 bytes for the implemented subset)
  is_write — direction
  value    — register value for writes (untrusted)
Outputs:
  Routed      -> handler ran, value/readback produced
  Unmapped    -> caller applies the W05-classified fault outcome (Reject path)
Preconditions:
  - vcpu is Running in Guest context (VM-exit); VM GIC state exists
  - ipa falls in the machine-gated GICD or GICR region set (region identity
    decided here; bounds enforced here as defense in depth)
Postconditions:
  - a Routed access completes with fixed bounded work and returns a defined
    value (reads) or applied-transition (writes)
  - exactly one telemetry event per access class (read/write/reject)
State and ownership change: delegated to the register handlers below; never
  any object creation/destruction on this path
Concurrency/allocation: takes per-vCPU lock and/or VM GIC lock per §1 lock
  order of [01 §6](01-architecture-and-state.md); no allocation; no polling;
  worst case is one register transition
Errors and failure guarantee: out-of-region/unaligned-where-required/miswidth
  access -> Unmapped or RAZ/WI per IHI 0069 rules; never mutates state
  partially; never faults the Host
Security/authorization checks: region membership bounds-check; width/alignment
  rules; Redistributor access is accepted only for the accessing vCPU's own
  frame except the GICR_TYPER-probe pattern used by Linux enumeration (see §4)
Logic (pseudocode):
  fn vgic_mmio_access(vcpu, ipa, size, is_write, value) -> AccessOutcome:
      if in_region(gicd_region, ipa):
          off = ipa - gicd_region.base
          return vgic_distributor_access(vcpu, off, size, is_write, value)
      if let Some((owner, off)) = redistributor_frame_for(vcpu, ipa):
          return vgic_redistributor_access(owner, off, size, is_write, value)
      return Unmapped            # caller applies W05 classification
Validation: region/width/alignment review; W18 illegal-MMIO scenarios
```

## 2. Distributor register subset (M2)

Implemented and their semantics (IHI 0069 names; access class per GICv3 for
one-secure-state Guests):

| Register | Semantics owned here |
|---|---|
| GICD_CTLR | Enable/disable group 1. Disable must not destroy pending state (latch-and-defer, [01 §4](01-architecture-and-state.md)); re-enable presents deferred pending work through M5 |
| GICD_TYPER | Machine-gated values: ITLinesNumber consistent with the machine SPI count; CPU_Number = topology size; LPIS=0 (no ITS in P8) |
| GICD_IIDR | Machine-gated implementer/variant values (never Host hardware reads) |
| GICD_ISENABLERn / GICD_ICENABLERn | SPI enable set/clear with write-one-to-set/clear semantics; unknown bits read as 0; enabling a latched-pending SPI defers presentation to M5 |
| GICD_ISPENDRn / GICD_ICPENDRn | Software pending set/clear for SPIs. Set → M5 pending request (P6-W07). Clear of a not-presented pending SPI clears the latch; clear of an active interrupt is architecturally a no-op here and is classified per W05 if it arrives in a state the model rejects |
| GICD_ICACTIVERn / GICD_ISACTIVERn | Active-flag mirror maintenance, translated to P6-W07 active/completed transitions |
| GICD_IPRIORITYRn | Per-INTID priority byte; carried into presentation by P6-W08 basic-priority carriage |
| GICD_IGROUPRn / GICD_IGRPMODRn | Group1 presentation for this single-secure-state Guest; stored for readback, no Secure-world behavior |
| GICD_ICFGRn | Edge/level configuration stored and honored by the delivery semantics of the owning device models ([W08]/[W09]) through M5 requests |
| Any other GICD offset (incl. GICD_SGIR) | RAZ/WI. Note: GICv3 SGIs are generated via ICC_SGI1R_EL1; a GICD_SGIR write is a legacy-v2 register and must never inject ([03 §2](03-code-contracts-interrupt-flow.md)) |

```text
Name and stability: vgic_distributor_access(vcpu, off, size, is_write, value) -> AccessOutcome — internal
Purpose and caller: Distributor register decode/emulation; called by §1
Inputs: offset within GICD frame, width, direction, value
Outputs: readback or applied transition; RAZ/WI outcome for unimplemented
Preconditions: offset < GICD frame size (enforced by §1)
Postconditions: SPI-table transitions are exact and atomic per access; one
  telemetry event; readback always reflects committed state
State/ownership change: per-VM Distributor state only ([01 §3.1]); M5
  lifecycle calls for pending/active requests
Concurrency/allocation: VM GIC lock; bounded; no allocation
Errors and failure guarantee: unimplemented -> RAZ/WI (defined, non-failing);
  invalid INTID fields in writes -> bits masked, no table indexing outside
  bounds (checked arithmetic per Coding Guidelines)
Security/authorization checks: INTID range checks; SPI-vs-SGI/PPI separation
  (Distributor registers affect SPIs only; SGI/PPI bits in Distributor
  enable/pending registers are architecturally reserved and masked)
Logic (pseudocode):
  fn vgic_distributor_access(vcpu, off, size, is_write, value):
      lock(vcpu.vm.gic_lock):
          match decode_gicd(off):
              CTLR      => if is_write { vcpu.vm.gicd.ctlr = value & CTLR_MASK;
                                          if !enabled: defer_all_pending() }
                           else readback(vcpu.vm.gicd.ctlr)
              TYPER/IIDR=> readback(machine_facts)          # never Host reads
              ISENABLERn=> for bit in set_bits(value & SPI_MASK(n)):
                               spi_table[INTID(n,bit)].enabled = true
                               if spi_table[...].pending: M5.defer_present(INTID)
              ISPENDRn  => for bit in set_bits(value & SPI_MASK(n)):
                               M5.request_pending(vcpu, INTID(n,bit))   # [03 §4]
              ICPENDRn  => for bit in set_bits(value & SPI_MASK(n)):
                               M5.clear_pending_if_not_presented(...)
              IPRIORITYR=> spi_table[INTID].priority = byte_field(value)
              _         => RAZ_WI()
      telemetry(...)
Validation: register-by-register review against IHI 0069; P8-V10 boot rows;
  W18 illegal-offset/width scenarios
```

## 3. SPI INTID allocation contract (data)

```text
Name and stability: MachineInterruptTable — machine-gated data; internal
Purpose and caller: authoritative SPI allocation for the VM; read by M2
  bounds checks and by device models ([W08] timer PPI, [W09] console SPI)
Contents: SGI 0–15 (per-CPU); PPI 16–31 with arch-timer PPI per [W08]'s
  timer policy; declared SPI set including <CONSOLE-SPI-INTID> ([W09]);
  SPI max count reported in GICD_TYPER
Preconditions: populated only from approved machine facts (W02 gate);
  immutable at runtime
Postconditions: all INTID indexing derives from this table; no handler
  hard-codes an allocation
Errors: VM creation fails if the table is absent or inconsistent with the
  DTB facts reviewed under P8-V05
Validation: consistency review against W04 DTB facts; W14 compat dimensions
```

## 4. Redistributor register subset (M3, GICR frame)

Implemented:

| Register | Semantics owned here |
|---|---|
| GICR_TYPER | Per-vCPU facts: Processor_Number = topology index; last-frame bit set on the highest-index frame (Linux uses it to end enumeration) |
| GICR_WAKER | The Quiescent handshake: write to clear ProcessorSleep → NotQuiescent (RegisterWritePending behavior may be modeled as immediately cleared for P8's synchronous model — recorded as stage-local simplification); set ProcessorSleep → Quiescent. Linux's secondary path polls this; semantics must match IHI 0069 observable behavior |
| GICR_ISENABLER0 / GICD_ICENABLER0-equivalents | SGI (0–15) and PPI (16–31) enables for this vCPU — the frame that Linux programs at secondary init; timer PPI enable is the [W08] consumed fact |
| GICR_ISPENDR0 / GICR_ICPENDR0 | Per-vCPU pending set/clear for SGI/PPI via M5 |
| GICR_IPRIORITYRn / GICR_ICFGR0/1 | Per-vCPU priority/config for 0–31 |
| GICR_IGROUPR0 / IGRPMODR0 | Group1 readback for 0–31 |
| Other GICR offsets | RAZ/WI |

```text
Name and stability: vgic_redistributor_access(vcpu, frame_owner, off, size, is_write, value) -> AccessOutcome — internal
Purpose and caller: per-vCPU GICR emulation; called by §1
Inputs: frame owner (vCPU), offset, width, direction, value
Outputs: readback or applied transition; RAZ/WI for unimplemented
Preconditions: §1 accepted the region; the accessing vCPU's frame is
  identified by address (the machine-gated frame map)
Postconditions: SGI/PPI transitions exact and atomic; WAKER transitions are
  observably consistent with [01 §5] quiesce/resume hooks
State/ownership change: per-vCPU Redistributor state ([01 §3.2]); M5 calls
Concurrency/allocation: vCPU lock (frame owner) or vcpu+VM lock when M5
  resolves cross-vCPU effects; bounded; no allocation
Errors and failure guarantee: unimplemented -> RAZ/WI; bad fields masked;
  no partial transition
Security/authorization checks: a vCPU may program only frames the machine map
  presents to it; Linux's enumeration probe pattern (reading remote TYPER) is
  read-only by construction — no contract grants remote writes
Logic (pseudocode): same shape as §2 handler over the GICR decode table with
  owner = frame_owner
Validation: per-register review; P8-V10 secondary rows; W10 consumption
```

## 5. Quiesce/resume hook contracts

```text
Name and stability: vgic_quiesce(vcpu) / vgic_resume(vcpu) — internal hooks
Purpose and caller: called by CPU_OFF ([W06 §3](../p8-w06-psci-virtualization/03-code-contracts-cpu-lifecycle.md))
  and by the vCPU entry path across P7 transitions
Inputs: vCPU
Outputs: withdrawn presentation (via P6-W08/W09 boundaries) with pending/
  enable state preserved; restored interface presentation at resume
Preconditions: quiesce — vCPU leaving Guest execution; resume — vCPU about
  to (re)enter Guest execution
Postconditions: after quiesce, no capacity of the vCPU's is held outside P6
  bookkeeping; after resume, ICC state is consistent with the carried vCPU
  context; repeated quiesce/resume cycles are state-identical
State/ownership change: presentation state only; no enable/pending mutation
Concurrency/allocation: bounded; may touch VM GIC lock per lock order; no
  allocation
Errors and failure guarantee: withdrawal failure is an internal fault (W13
  path), never silent state leak
Security checks: none beyond P6's target isolation
Logic (pseudocode):
  vgic_quiesce: P6_W08.withdraw_all_presented(vcpu); P6_W09.settle(vcpu)
  vgic_resume:  P6_W08.restore_interface_presentation(vcpu)
Validation: repeated-lifecycle (hotplug cycle S3b of [W06]); W10 per-CPU
  bring-up rows
```

## 6. Malformed-access classification and diagnostics

Applied W05 categories (W05 owns the taxonomy; this table binds it):

| Guest behavior | Class | Observable outcome |
|---|---|---|
| In-region, implemented register, legal width | Direct | Emulated per this file |
| In-region, unimplemented offset | Direct (RAZ/WI) | Read 0 / write ignored + telemetry |
| Out-of-region or width/alignment-violating access | Reject | Stage-2 fault class per W05 with VM-scoped diagnostic (W13 context: faulting IPA, size, vCPU, nearest region) |
| INTID out of range in register fields | Direct (masked) | Bits ignored, telemetry counter |
| Sustained invalid-access storm | Reject + contained | Per W05/W18: VM continues, counters/telemetry show the storm; no Host degradation |

Diagnostic context contract for W13: every Reject outcome records {vCPU id,
virtual faulting IPA, access size/direction, region class} in the VM's
fault-diagnostic slot — bounded, overwritable, never growing.
