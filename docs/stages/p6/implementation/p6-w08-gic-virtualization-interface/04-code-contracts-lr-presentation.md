# P6-W08 Code Contracts — LR Presentation, Pressure, Maintenance Boundary

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W08 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (D3–D8),
[02](02-architecture-and-state.md) (slot lifecycle),
[03](03-code-contracts-interface-context.md) (sequences, context).

## 1. Slot table

### 1.1 `LrTable` (per pCPU)

```text
Name and stability: LrTable { slots: [LrSlot; lr_count] } where
  LrSlot { state: Empty | Occupied, claim: Option<VirqClaim>,
           eoi_tracked: bool } — sized from discovery (D4). Internal to
  W08; stable within P6.
Purpose and caller: the software mirror of the hardware LRs, linking
  slots to W07 claims so purge/refill outcomes return to the right
  event; operated by the load/clear/refill functions below and the
  entry/exit sequences ([03] §3–§4).
Preconditions: created at readiness; all slots Empty; never accessed
  off the owning pCPU (D2).
Postconditions: descriptor state always mirrors the hardware LR;
  mismatches are invariant violations handled per [02] §4 rule 4.
Concurrency: running-pCPU-only access; serialization per [02] §5.
Errors: invariant paths are internal, counted, force-consistent.
Security/authorization checks: none (Host-internal; claims are
  unforgeable typed tokens, W07 lifecycle contracts
  (`../p6-w07-virtual-interrupt-core/03-code-contracts-virq-lifecycle.md` §1.1).
Validation: W08-DV02–DV04.
```

## 2. Load (presentation)

### 2.1 `load_from_selection()`

```text
Name and stability: fn load_from_selection(table, sel: VirqSelection)
  -> Result<(), LoadError> — internal to W08; called by the entry
  sequence and by refill ([4]). Stable within P6.
Purpose and caller: program one software LR from a W07 claim (D3).
Inputs / outputs: selection (vINTID, priority, claim) -> success or
  NoFreeSlot.
Preconditions: a slot is Empty; virtual interface may be enabled or
  being brought up (entry path orders loads before enable, [03] §5).
Postconditions: one slot Occupied with the claim recorded; the LR
  carries: vINTID = sel.vintid, hw = 0, group per the W02 physical
  policy (Group 1, Non-secure — assumed contract [01] §5),
  priority = carriage per [3], state = pending; the descriptor mirrors
  it.
Concurrency: transition/refill context on the owning pCPU; O(1).
Errors: NoFreeSlot (caller stops filling — D6 policy, not an error
  condition for the caller); invariant paths internal.
Security/authorization checks: fields derive from the validated claim
  and configuration only.
Logic (pseudocode):
    slot = first_empty(table)?                      // else NoFreeSlot
    field = build_lr(vintid = sel.vintid, hw = 0, group = GROUP_POLICY,
                     priority = carriage(sel.priority), state = pending)
    write_lr(slot.index, field)
    record descriptor { Occupied, claim: sel.claim, eoi_tracked: per
                        maintenance config }
    lr_presented_count += 1
Validation: W08-DV02 (single presentation), DV05 (priority carriage).
```

## 3. Priority carriage (D7)

```text
Contract (behavioral): carriage(virq_priority) -> lr_priority_field:
  the W07-validated priority value is mapped into the LR priority field
  at the discovered priority_bits width — a pure, total re-encoding
  (shift/mask with the declared widths), preserving order (a
  numerically-lower W07 priority remains higher-urgency in the LR).
  The VMCR image loaded at entry carries the Guest's run-control view
  with P6 defaults (declared constants of this design: pass-through of
  the W02 physical policy's baseline run controls, with no Guest-tunable
  fields in P6). No preemption, no Guest-chosen priorities, no priority
  translation beyond width mapping — semantics are W10's.
  Rationale for owning the encoding here: it is the only place the
  discovered width and the W07 value meet; owning it prevents ad-hoc
  shifts at call sites (Coding Guidelines: typed conversions, no naked
  arithmetic).
Validation: W08-DV05 (order-preserving carriage at width boundaries).
```

## 4. Pressure policy (D6)

### 4.1 `fill_to_capacity()` and the no-loss rule

```text
Name and stability: fn fill_to_capacity(table) -> PressureReport {
  loaded: u8, left_pending: u32 } — internal to W08; called at entry
  load and by refill. Stable within P6.
Purpose and caller: implement the bounded pressure policy: load W07
  selections until no slot or no selection remains; the rest stays
  pending in W07 untouched.
Preconditions: running pCPU; W07 selection available; postconditions:
  no slot overwritten; left_pending recorded for telemetry
  (lr_pressure_depth_peak).
Concurrency: O(lr_count); no allocation.
Errors: none (shortfall is the normal pressure outcome).
Security/authorization checks: none beyond protocol.
Logic: loop { sel = virq_select_next_pending()?; if None or
  load_from_selection == NoFreeSlot { break } ; loaded += 1 }.
Validation: W08-DV04 (over-capacity case: more pending than slots →
  no loss, no overwrite, excess pending).
```

The no-loss argument, for review: pending events not presented remain
bits in W07's bank (single owner, D1 of W07); the exit purge returns
every loaded claim (StillPending/BecameActive/Completed), so nothing
loaded is lost either; pressure only defers presentation. This is the
structural claim P6-V16 tests.

## 5. Maintenance boundary (D8)

### 5.1 `maintenance_status()` / `clear_completed_slot()` / `refill()`

```text
Name and stability:
  fn maintenance_status() -> MaintenanceSnapshot  — atomic-enough read
    of the maintenance status and EOI-error state (reads are of
    hardware state; the caller, W09, owns interpretation).
  fn clear_completed_slot(table, slot) -> CompletionOutcome — validates
    the slot's descriptor against the hardware state, returns the
    outcome (Completed/Contradiction), clears the slot, and (for
    Completed) routes the guest-visible completion into W07 via the
    W09-driven path (W07 lifecycle contracts
    `../p6-w07-virtual-interrupt-core/03-code-contracts-virq-lifecycle.md` §5.3).
  fn refill(table) -> PressureReport — after W09 has processed
    completions, fill freed slots via [4.1].
  Internal to W08; sole caller is W09 (assumed contract, [01] §5).
  Stable within P6.
Purpose and caller: give W09 the completed-presentation/reusable-
  capacity boundary without letting policy into W08.
Preconditions: running pCPU; W09 processes in maintenance context with
  bounded work (its design's obligation); postconditions: cleared slots
  are Empty; completions reported exactly once (descriptor cleared with
  the hardware state — a repeated report is W09's duplicate-protection
  concern, backed by [02] §4 rule 4 detection).
Concurrency: maintenance context; O(freed slots); no allocation.
Errors: Contradiction outcomes are counted and force-consistent.
Security/authorization checks: none beyond protocol (Guest influences
  outcomes only through architectural EOI behavior, which the hardware
  reports).
Logic: thin, explicit primitives per above; no policy.
Validation: W08-DV04 (progress under pressure with W09), DV06
  (protocol misuse counted).
```

## 6. Register sequence contract for LR operations

| Sequence point | Required ordering | Rationale |
|---|---|---|
| LR field constructed wholesale, written once | single volatile system-register write | LR fields are interpreted as a unit; partial programming must never be observable |
| Slot descriptor recorded after the hardware write completes | program order | the descriptor must never claim occupancy the hardware does not have (mismatch handling is for faults, not for lazy writes) |
| Hardware LR read before descriptor-based return at purge | read first, then match | the hardware state is the truth at purge time; the descriptor only identifies the claim |
| LR clear (invalid write) before the slot is reused | program order + part of entry DSB coverage | reusing a slot whose LR was not cleared would present a stale event |
| Completion clear before refill load of the same slot | program order | a freed slot must be empty in hardware before the next event loads |

Exact encodings, access mechanics, and any additional ordering mandated by
the locked specification revision are recorded in the implementation
record; this table is the review contract ([03] §5 covers the transition
sequences; this covers slot operations).
