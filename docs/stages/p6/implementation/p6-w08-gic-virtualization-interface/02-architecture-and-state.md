# P6-W08 Architecture, Ownership, and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W08 design entry](README.md).

## 1. Position in the P6 event chain and layer placement

W08 is the hardware boundary between W07's software vIRQ truth and the
CPU's virtual interrupt interface:

```text
W07 pending state --select/claim--> W08 LR load (entry/refill)
      ^                                   |
      | return (purge/eviction)           v
      +------------------------- List Registers (hardware)
                                          |
                              Guest runs; ICV interface presents,
                              Guest acknowledges/completes
                                          |
                       maintenance conditions (EOI/underflow/...)
                                          v
                            W09 processing (policy) --uses--> W08
                            primitives (status read, clear, refill)
```

W08 is Arch-domain code: every register it touches is a system-register
access behind the audited `unsafe` boundary. It contains no W07 state
structure, holds no pending queue of its own, and never decides which
event is more important beyond applying W07's selection order (D6).
Core-layer code must not contain any W08 register sequence.

## 2. Logical modules

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `vgic-readiness` | Per-pCPU capability verification, discovery (capacity, widths), safe enable/disable | readiness record per pCPU (discovered values, enabled flag) | W01/W02 facts; pCPU discovery reads | readiness result; recorded discovery values | platform capability *decisions* (W01); physical bring-up (W02) |
| `vgic-lr-table` | The per-pCPU LR table: load (from W07 claims), read/purge, clear; slot lifecycle | LR slot descriptors (which claim occupies which slot; EOI-tracking flags) | W07 selections; W09 refill/clear calls; entry/exit sequences | loaded LRs; purged outcomes returned to W07 | which event is selected (W07); what maintenance means (W09) |
| `vgic-context` | vCPU-context preservation: VMCR image, active-priority image, entry/exit sequences with barrier placement | none (operates on the vCPU record's extension area) | vCPU record; P4 transition points | restored/quiet interface per transition | transition scheduling (P4); run policy (P7) |
| `vgic-maintenance-boundary` | Enable maintenance sources per the W09 contract; expose status reads; provide clear/refill primitives | none (delegates state to hardware + W07) | W09 calls; entry sequence enables | status snapshots; refill results | maintenance policy and correlation (W09) |

One instance of each per online pCPU (readiness, LR table) except
`vgic-context`, which is per-vCPU state operated on the running pCPU. All
hardware access is confined to these modules.

## 3. Core objects and ownership (one owner per datum)

| Datum | Sole owner | Written by | Read by |
|---|---|---|---|
| LR registers (hardware) | W08's `vgic-lr-table` — only on the owning pCPU | load/clear/refill operations | purge/inspection at exit |
| LR slot descriptors (claim ↔ slot mapping, EOI flag) | `vgic-lr-table` | load/return operations | refill, diagnostics |
| VMCR image, active-priority image (per vCPU) | the vCPU record's extension area (P4 contract); contents defined by W08 | entry restore / exit save | diagnostics |
| Virtual-interface enable + maintenance enables | `vgic-readiness` (enable), `vgic-context` (per-transition on/off) | sequences | maintenance handler presence check |
| W07 pending/active/presented state | W07 exclusively | W07 via the protocol | W08 via selection/return only |
| Physical interrupt state | W02/W03 domain | W02/W03 | never W08 (hw=0 rule, D3) |
| Maintenance processing decisions | W09 exclusively | W09 | W08 primitives serve it |

No datum exists in two owners: the LR slot descriptor and the W07 claim
are linked by the claim token, but only W07's protocol functions mutate
lifecycle bits, and only W08 mutates hardware and slot descriptors.

## 4. LR slot lifecycle (per slot, per pCPU)

```text
        load(vINTID, priority, claim)
  Empty ─────────────────────────────> Loaded(pending)
    ^                                    |  Guest acknowledges (ICV IAR)
    |                                    v
    |  purge at exit / W09 clear   Loaded(active, eoi-flag per config)
    |                                    |
    |  return(StillPending)              | Guest completes -> maintenance
    |  return(BecameActive)              v
    +<------ W07 return <-------- completion observed by W09, which
              (Completed)         reports completion to W07 and calls
                                  clear -> Empty (refill may then load)
```

Rules with no exceptions:

1. A slot holds at most one live claim; a claim lives in at most one slot.
2. `load` is only called from an empty slot; `clear` only from a slot
   whose claim has been completed/processed; `purge` (exit) drains every
   non-empty slot via W07 returns.
3. Refill (from W09's no-pending/EOI processing or entry load) never
   overwrites a non-empty slot; it fills only empty slots from fresh W07
   selections (D6: no overwrite).
4. A slot descriptor that does not match the hardware LR (e.g. an invalid
   LR where a live claim was recorded) is an invariant violation: counted,
   the slot forced empty (hardware cleared), the claim returned as
   StillPending (preserving the event), and surfaced as a Host-attributed
   diagnostic.
5. At exit, no slot may remain non-empty after the purge step — the exit
   sequence asserts this, and a violation is recovered by the same
   force-empty path before the vCPU is unloaded.

## 5. Concurrency model

- **Running-pCPU-only (D2):** all hardware and table access occurs in the
  P4 transition context (entry/exit), the maintenance context (W09-driven
  refill on the same pCPU), or per-pCPU bring-up. The P4 transition
  contract guarantees entry/exit exclusivity for the vCPU; the maintenance
  interrupt handler runs on the same pCPU while the vCPU is loaded — the
  only concurrent pair is (transition exit) vs. (maintenance handler), and
  both are serialized by the W03 IRQ-context rules plus the short
  per-pCPU sequence locks each operation already respects (P3
  synchronization contract, `../../../p3/plans/p3-w06-concurrency-synchronization.md`).
  Critical sections are O(LR count), bounded by discovery (D4).
- **IRQ-context boundedness:** maintenance-context work is bounded by the
  discovered LR count plus one W07 selection per freed slot (D6/D8). No
  allocation occurs in any W08 path (all tables sized at readiness/vCPU
  creation).
- **No cross-pCPU operations exist.** If a future stage migrates vCPUs
  across pCPUs, the migration design inherits this rule: state moves only
  through exit-save then entry-restore, never by remote access (Reserved
  note for P7).

## 6. Security model

- Guests have no path to W08: no Guest-visible register, no Guest MMIO
  model, no Guest-chosen LR parameters. All LR fields derive from W07
  claims (validated at injection) and W08 configuration.
- Discovery inputs (identification registers) are hardware facts but are
  still validated against the W01-declared envelope and sanity bounds
  before use (untrusted-until-checked platform data, same posture as
  W05/W06/W07 intakes).
- The `unsafe` surface is the register accessors of the four logical
  modules; each accessor carries a `SAFETY` justification and is
  inventoried; the sequence tables in
  [03](03-code-contracts-interface-context.md) §5 and
  [04](04-code-contracts-lr-presentation.md) §6 are part of the audited
  boundary (Coding Guidelines: QEMU success is not evidence the hardware
  rules can be omitted).

## 7. Telemetry surface (names fixed by this design)

Per-pCPU (and per-transition where noted) saturating counters and trace
events; P6-W13 consumes:

| Counter | Meaning |
|---|---|
| `lr_presented_count` / `lr_purged_pending_count` / `lr_purged_active_count` / `lr_completed_count` | slot lifecycle outcomes (P6-V16/V17 correlation) |
| `lr_pressure_depth_peak` | maximum pending events left un-presented at a load (P6-V16 evidence) |
| `refill_load_count` | maintenance-driven refills |
| `eoi_error_count` | EOI-error maintenance conditions observed |
| `discovery_lr_capacity`, `discovery_priority_bits`, `discovery_preempt_bits` | recorded discovery values (written once at readiness) |
| `invariant_violation_count{kind}` | forced-consistency recoveries (Host-attributed) |
| trace: `lr_loaded`, `lr_purged`, `lr_refilled` | per the P0-W13 trace namespace; W13 owns collection |
