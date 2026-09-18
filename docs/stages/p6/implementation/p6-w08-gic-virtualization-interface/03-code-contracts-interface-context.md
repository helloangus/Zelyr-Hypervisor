# P6-W08 Code Contracts — Readiness, Context Preservation, Entry/Exit

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W08 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (decisions D1–D8),
[02](02-architecture-and-state.md) (ownership, lifecycle). LR presentation
contracts are in [04](04-code-contracts-lr-presentation.md).

Register fields are named at field level (vINTID, state, priority, group,
hardware-mapped, EOI-tracking); exact encodings and instruction spellings
are fixed against the specification revision locked at implementation
(task book §8) and recorded in the implementation record. Every accessor
is Arch-domain `unsafe` with a `SAFETY` justification.

## 1. Discovery and readiness

### 1.1 `VirtInterfaceCaps` and `vgic_readiness()`

```text
Name and stability: VirtInterfaceCaps { lr_count: u8 (discovered,
  sanity-bounded 4..=16), priority_bits: u8, preempt_bits: u8,
  sysreg_interface_available: bool, virtual_interface_available: bool }
  plus fn vgic_readiness(pcpu) -> Result<VirtInterfaceCaps, ReadinessError>.
  Internal to W08; stable within P6.
Purpose and caller: one-time per-pCPU discovery and gating (D1, D4);
  called at per-pCPU bring-up after W02 local readiness.
Inputs / outputs: pCPU identity -> discovered envelope or precise failure.
Preconditions: W01 conclusions declare virtualization support for the
  platform; W02 local GIC readiness acknowledged for this pCPU (both
  assumed contracts, [01] §5).
Postconditions: on success, the discovery values are recorded (telemetry)
  and the interface remains DISABLED until a vCPU entry enables it; on
  failure, presentation is unavailable on this pCPU and the reason is
  identifiable.
Concurrency: bring-up context; exclusive.
Errors and failure guarantee (named cases):
  CapabilityMismatch — pCPU discovery contradicts the W01-declared
                       envelope (or sanity bounds) -> disabled, diagnosis.
  SysregInterfaceUnavailable — system-register interface not usable at
                       EL2 -> disabled (memory-mapped interface fallback
                       is explicitly NOT attempted; D1).
  PhysicalNotReady    — W02 local readiness not acknowledged -> blocked
                       pending W02, not a local workaround.
Security/authorization checks: discovery reads are hardware facts
  validated against the declared envelope before any use.
Logic (pseudocode):
    require w01_declares_virtualization()?; require w02_local_ready(pcpu)?
    verify system-register interface usable at EL2?
    caps.lr_count      = discover(); require 4 <= lr_count <= 16
    caps.priority_bits = discover(); caps.preempt_bits = discover()
    require caps within w01_declared_envelope()?
    leave virtual interface disabled; record discovery telemetry
Validation: W08-DV01 (including deliberately mismatched fixtures where
  the platform harness permits).
```

## 2. Interface context (per vCPU)

### 2.1 `VcpuVirtInterfaceContext`

```text
Name and stability: VcpuVirtInterfaceContext — W08's extension-area
  record held in the vCPU's arch-extension area (assumed P4 contract,
  [01] §5). Internal to W08; stable within P6.
Fields:
  vmcr_image:        validated image of the virtual interface control
                     state (Guest run controls as of last exit)
  active_priority_image: image of the active-priority registers as of
                     last exit
  lr_occupancy_at_exit: summary (count by outcome; no claim data —
                     claims returned to W07 at purge)
Purpose and caller: the preserved virtual-interface state that makes
  execution resumable and cross-vCPU contamination impossible (D5).
Preconditions: zeroed/initialized at vCPU creation (disabled, no active
  state); present whenever the vCPU exists.
Postconditions: after exit, the images match the hardware state as
  purged; after entry, hardware matches the images.
Concurrency: written only in the exclusive transition context; read by
  diagnostics. (The images are transition-local; the [02] §5 rules cover
  the maintenance-vs-exit overlap.)
Errors: none stored; inconsistencies are sequence violations caught at
  the next transition and recovered per [02] §4 rule 4/5.
Security/authorization checks: contents are hypervisor-generated; no
  Guest memory involved.
Validation: W08-DV03.
```

## 3. Entry sequence

### 3.1 `vgic_on_entry()`

```text
Name and stability: fn vgic_on_entry(vcpu, context: &VcpuVirtInterfaceContext)
  -> Result<(), EntryError> — called by the P4 entry path (after W06's
  vtimer_on_entry, before Guest execution resumes), on the hosting pCPU.
  Arch-domain. Stable within P6.
Purpose and caller: make the vCPU's virtual interface live: restore
  controls and active state, present pending work, enable the interface.
Inputs / outputs: context -> hardware programmed; pending work loaded
  from W07 selections.
Preconditions: readiness passed on this pCPU; transition exclusivity
  held (P4); the interface was disabled at last exit (invariant checked,
  recovered if not).
Postconditions: VMCR and active-priority state match the context; up to
  lr_count LRs loaded in W07 selection order (D6); virtual interface
  enabled; maintenance sources per the W09 contract enabled; all prior
  programming globally visible before Guest entry (§5 barrier table).
Concurrency: transition context; O(lr_count).
Errors: none returned; invariant anomalies recovered internally and
  counted ([02] §4).
Security/authorization checks: none beyond protocol (Host path).
Logic (pseudocode):
    if interface_enabled { force-disable; count invariant }   // §4 rule 5
    restore vmcr_image; restore active_priority_image
    for _ in 0..lr_count {
        match virq_select_next_pending(w07 bank) {   // [04] §2
          Some(sel) => load_lr(slot, sel, hw=0, group per W02 policy)
          None      => break
        }
    }
    pressure_depth = pending_remaining_summary; count peak
    enable maintenance sources per w09_contract()
    enable virtual interface
    dsb_sy()                          // §5 table: before ERET
Validation: W08-DV02 (presentation occurs), DV03 (context restored),
  DV04 (pressure path), DV05 (priority carriage).
```

## 4. Exit sequence

### 4.1 `vgic_on_exit()`

```text
Name and stability: fn vgic_on_exit(vcpu, context: &mut
  VcpuVirtInterfaceContext) -> ExitReport — called by the P4 exit path
  (after W06's vtimer_on_exit hooks are consistent with the P4
  sequencing; ordering fixed at integration), on the hosting pCPU.
  Arch-domain. Stable within P6.
Purpose and caller: return the interface to a clean, vCPU-independent
  state and hand every observable outcome back into W07 truth (D5) —
  the step that makes P6-V16 ("no loss or overwrite") and the P6-V18
  isolation share structural.
Inputs / outputs: context -> ExitReport { completed: list length,
  still_pending, became_active, eoi_errors, unexpected_conditions }
  for diagnostics and W09 correlation.
Preconditions: transition exclusivity held; W09 maintenance processing
  for this pCPU is quiesced or serialized per [02] §5.
Postconditions: every non-empty LR purged and its outcome returned to
  W07 exactly once; VMCR/active-priority images saved; interface and
  maintenance enables off; hardware leaves no state that a different
  vCPU could inherit.
Concurrency: transition context; O(lr_count).
Errors: none returned; invariant anomalies recovered (force-clear,
  claims preserved as StillPending) and counted.
Security/authorization checks: none beyond protocol (Host path).
Logic (pseudocode):
    drain critical maintenance state per w09_boundary()   // before purge
    for slot in 0..lr_count where occupied(slot) {
        state = read_lr(slot)
        outcome = match state {
          pending-held          => StillPending,
          active-held           => BecameActive,
          completed-by-guest    => Completed,     // W07 completion inside
          invalid/contradiction => { count invariant; StillPending }
        }
        virq_presentation_returned(claim(slot), outcome)  // [04] §3
        clear_lr(slot); update slot descriptor
    }
    save vmcr_image; save active_priority_image
    disable maintenance enables; disable virtual interface
    dsb_sy()                          // §5 table: before "unloaded"
    assert no slot occupied           // recovered if violated
Validation: W08-DV03 (no loss across exits), DV04 (excess preserved),
  DV06 (isolation).
```

## 5. Register sequence and barrier contract

| Sequence point | Required ordering | Rationale |
|---|---|---|
| Enable system-register interface (readiness) before any ICH access | program order + ISB after enable | the virtual-interface registers are only architecturally defined once the interface is enabled; the ISB makes the enable effective before first use |
| VMCR / active-priority restore before LR loads at entry | program order (system registers) | LRs must be loaded against the restored control state, not the previous vCPU's |
| LR writes before virtual-interface enable at entry | program order | enabling exposes LRs; never expose a half-written table |
| All entry programming before ERET into the Guest | DSB SY | the Guest must observe the complete presentation state; ERET alone does not order prior system-register writes for all observers |
| Maintenance-state drain before LR purge at exit | program order | completion evidence must be captured before the LRs that produced it are cleared |
| Purge/return before interface disable at exit | program order | returns reflect observed state; disabling first would make outcomes unobservable |
| Disable before the vCPU is regarded as unloaded | DSB SY | no other vCPU may start on this pCPU until the prior vCPU's interface state is globally quiesced (P6-V18 isolation share) |
| ISB after any enable/disable toggle before dependent decisions | ISB | status reads and enable bits must not be speculatively ordered across the toggle |

Reserved bits: every control register is written as read-modify-write with
reserved bits preserved as observed; no field is written by bit-or into an
unaware image. Exact mnemonics and any additional architecture-mandated
ordering are locked against the specification revision at implementation
and recorded; this table is the reviewable contract the implementation is
checked against (Coding Guidelines hardware-rule emphasis).

## 6. Explicitly excluded here

No Guest access path, no hardware-mapped LR programming, no maintenance
policy, no vCPU migration support, no serialization format for the context
record — each is Reserved or Out of Scope per
[01](01-scope-and-foundations.md) §6 and named so a coding agent cannot
mistake silence for permission.
