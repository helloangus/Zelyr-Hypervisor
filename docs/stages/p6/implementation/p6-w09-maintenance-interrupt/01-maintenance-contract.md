# P6-W09 Maintenance Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W09 detailed design](README.md).  
**Audience:** load this file before any code-bearing step of the
[workflow](02-workflow-and-validation.md). Register, MMIO, and barrier work in
this file is governed by the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) (MMIO,
registers, barriers, IRQ-context sections) and the P0 unsafe inventory
([P0-W10](../../../p0/plans/p0-w10-unsafe-rust-governance.md)).

## 1. Logical module and boundary

Maintenance processing is one increment inside the logical Host-GIC
virtualization boundary that the P6-W08 design establishes
(`../p6-w08-gic-virtualization-interface/README.md`). It is not a new module
tree, crate, or file layout; the final Rust module path is bound in the
implementation record once W08's design exists. This design owns the
maintenance increment's behavior and names only.

| Aspect | Statement |
|---|---|
| Responsibility | Recognize maintenance conditions; correlate completed presentation slots to their vIRQs; release slots through W08; advance vIRQ lifecycle through W07; admit bounded follow-on work; diagnose unexpected conditions. |
| Owned mutable state | Per-vCPU maintenance diagnostic counters and the raw status value of the last unexpected condition (see §2). Nothing else. |
| Inputs | The maintenance status register read of the local virtual CPU interface (volatile MMIO/system-register access per the W08 access layer); the W08 in-flight presentation records; the W07 vIRQ lifecycle entry points. |
| Outputs | Slot releases, vIRQ lifecycle completions, refill admissions, telemetry events, diagnostic counters. |
| Non-responsibility | Physical IRQ classification and spurious handling (W03); LR selection policy (W08); pending-queue discipline (W07); masking/priority semantics (W10); Guest-visible vGIC model (P8); fairness, coalescing, and IRQ-pressure guarantees (excluded by plan). |
| Failure boundary | Contained diagnostics for unexpected-but-survivable conditions; fatal-invariant escalation (P0-W14 classification) for orphan completions and state-integrity loss; never a silent repair. |

## 2. State and lifecycle

W09 adds no new authoritative runtime state beyond two per-vCPU diagnostic
accumulations, both preallocated with the vCPU:

```text
maintenance_unexpected_count : unsigned counter   (saturating)
maintenance_last_unexpected  : raw status snapshot (opaque bits + capture order)
```

The authoritative mutable state W09 acts on is owned elsewhere, and W09's
single-writer transition is the only maintenance path into it:

```text
Presentation slot (owner: W08 LR allocation table):
    FREE -> PRESENTED(lr, vIRQ) -> COMPLETED(reported by controller)
         -> RECONCILED (W09 transition) -> FREE

vIRQ lifecycle (owner: W07):
    ... -> PRESENTED -> COMPLETED (via W07 completion contract,
         invoked only by W09's maintenance reconciliation in the
         hardware-presentation path) -> ...
```

Lifecycle rules:

- A slot may be reconciled at most once per presentation epoch; re-reports are
  counted no-ops (invariant I2 below).
- Reconciliation order follows the controller's reported completion set; W09
  does not reorder or prioritize (final ordering policy is excluded by plan).
- On vCPU teardown the counters are dropped with the vCPU (W07 lifecycle
  destruction contract); W09 owns no cross-vCPU or global state.
- No state introduced here survives Host reboot or is serialized anywhere.

## 3. Interface contracts

All names are **internal** to the Host GIC virtualization boundary
(stage-local design freedom owned by the parent design, decision 6 of its
README). They are not public API, not ABI, and not versioned. Signatures are
logical; the final Rust realization must satisfy the contracts below, not the
reverse.

### 3.1 `MaintenanceConditions` (type)

```text
Name and stability: MaintenanceConditions; internal; stable for P6 only.
Purpose and caller: logical bit set naming the recognized maintenance
  condition classes; produced by decode_maintenance_status, consumed by
  maintenance_irq_entry and telemetry.
Inputs / outputs: constructed from one raw status register read; exposes
  membership queries per class and the raw value.
Preconditions / postconditions: none beyond a completed controller read.
State and ownership change: value type; no shared state.
Concurrency/allocation context: stack-local; no allocation.
Errors and failure guarantee: carries an "unrecognized pattern" state instead
  of failing; the caller routes it to report_unexpected_maintenance.
Security/authorization checks: none (Host-internal value; never
  Guest-writable).
Logic: bit-set of { CompletionOrReusableSlot, UnderPending, EoiCountZero,
  NoPending } plus `unrecognized(raw)`; exact controller bit mapping is the
  Specification Investigation item resolved in workflow step 2 and recorded
  in the implementation record — the type must not embed bit spellings until
  that reconciliation lands.
Validation: decode unit tests against the reconciled spec mapping
  (W09-DV04's review basis).
```

### 3.2 `maintenance_irq_entry`

```text
Name and stability: maintenance_irq_entry; internal; stable for P6 only.
Purpose and caller: single entry point for the maintenance PPI; called by
  the W03 physical-IRQ lifecycle when the acknowledged INTID classifies as
  the virtual CPU interface maintenance interrupt (GICv3 PPI, maintenance —
  exact INTID confirmed in workflow step 2 spec reconciliation).
Inputs / outputs: none (acts on local pCPU state); returns nothing; all
  outcomes are state transitions and telemetry.
Preconditions: executing at EL2 in IRQ context on the pCPU whose virtual
  CPU interface raised the interrupt; W08 per-vCPU presentation state for a
  bound vCPU exists or the bound-vCPU-absent rule (below) applies; W03
  dispatcher guarantees single-threaded IRQ delivery per pCPU.
Postconditions: every reported completed slot is either reconciled exactly
  once or, if unrecognizable, left untouched with diagnostics; capacity
  freed in this invocation may have been refilled at most once; the physical
  interrupt is completed through the W03 contract before return; no pending
  work item was lost or duplicated.
State and ownership change: advances W08 slots FREE<-RECONCILED and W07
  vIRQs to COMPLETED; mutates only W09's own counters otherwise.
Concurrency/allocation context: IRQ context — no allocation, no blocking,
no unbounded loops (bound = reported completion count for reconciliation;
  capacity-freed count for refill); locking only via the established W03/W08
  per-pCPU ordering; if the maintenance interrupt fires while no vCPU is
  bound to the pCPU, process recognized controller state safely (count and
  release nothing) and return — a repeated pattern here is diagnostic, not
  fatal.
Errors and failure guarantee: orphan completion (reported slot with no
  in-flight record) does not mutate lifecycle state; it increments the
  counter, snapshots state for the dump, and escalates per §4 E2.
  Unrecognized status does not mutate lifecycle state (§4 E3).
Security/authorization checks: not Guest-reachable; no Guest input is
  interpreted on this path.
Logic (outline, not production code):
  1. raw = read maintenance status (volatile, per W08 access layer;
     required ordering barriers per the reconciled specification)
  2. cond = decode(raw); if unrecognized -> report_unexpected_maintenance(raw);
     complete the physical IRQ via W03; return
  3. for each reported completed slot (bound = controller-reported count):
       if in-flight record exists:
         if already reconciled -> count duplicate (no-op, I2)
         else advance W07 vIRQ to COMPLETED via W07 contract,
              release slot via W08 contract
       else -> orphan: escalate per §4 E2 (stop processing further
              completions this invocation after containment)
  4. freed = number of slots released in step 3
     admit up to `freed` pending vIRQs via the W08 admission contract
     (queue order, no reordering); if pending work remains, emit the
     deferred-remainder event
  5. emit maintenance telemetry event set (§5); complete physical IRQ
     per the W03 sequencing rule (decode-then-EOI ordering fixed in
     workflow step 4 against the reconciled specification)
Validation: W09-DV02 (reusable-slot progression), W09-DV03
  (Guest-completion), W09-DV05 (repeat), W09-DV06 (cross-vCPU),
  W09-DV07 (bounded-work review).
```

### 3.3 `decode_maintenance_status`

```text
Name and stability: decode_maintenance_status; internal.
Purpose and caller: translate one raw status register read into
  MaintenanceConditions; called only by maintenance_irq_entry.
Inputs / outputs: raw register value -> MaintenanceConditions.
Preconditions: the access satisfies the volatile/barrier rules of the W08
  access layer and the reconciled specification's read ordering.
Postconditions: pure function of its input; no side effects.
State and ownership change: none.
Concurrency/allocation context: none; no allocation.
Errors and failure guarantee: never fails; returns the unrecognized state
  for any pattern whose classes cannot be determined.
Security/authorization checks: reserved bits are treated as
  unrecognized-pattern contributors, never as class bits (reserved-bit
  handling per Coding Guidelines).
Logic: mask to defined fields per the step-2 spec reconciliation; map
  fields to logical classes; any set reserved/unknown portion -> include
  raw in the unrecognized state.
Validation: decode unit review (W09-DV04 review basis).
```

### 3.4 `reconcile_completed_slots`

```text
Name and stability: reconcile_completed_slots; internal.
Purpose and caller: perform the completed-slot loop of
  maintenance_irq_entry; kept as a named unit so W12's orphan case and the
  duplicate-completion idempotency rule have one reviewable home.
Inputs / outputs: the reported completion set -> count of slots released.
Preconditions / postconditions: as §3.2 steps 2–3; releases only slots
  whose in-flight records exist and were not yet reconciled.
State and ownership change: W08 slot release + W07 vIRQ completion per
  released slot; W09 duplicate/orphan counters otherwise.
Concurrency/allocation context: IRQ context; loop bound is the reported
  set size; no allocation.
Errors and failure guarantee: orphan -> containment + escalation, no
  lifecycle mutation for the orphaned slot; already-reconciled -> counted
  no-op.
Security/authorization checks: none (Host-internal).
Logic: see §3.2 steps 2–3; the separation exists for testability and for
  the W12 FI-C case reference, not as a public boundary.
Validation: W09-DV02/DV03; W12 FI-C consumes the orphan branch.
```

### 3.5 `refill_freed_capacity`

```text
Name and stability: refill_freed_capacity; internal.
Purpose and caller: admit pending vIRQs into capacity freed in the current
  invocation; called by maintenance_irq_entry after reconciliation.
Inputs / outputs: freed-slot count -> count actually admitted.
Preconditions: reconciliation of this invocation has finished; W08
  admission contract is callable in IRQ context (a W08 design obligation —
  failure boundary noted in the parent README ledger).
Postconditions: admitted <= freed; admitted items are removed from the
  pending set by the W07/W08 contracts, not by W09; any remainder stays
  pending with no loss.
State and ownership change: slot PRESENTED transitions and queue removal
  are performed by the W08/W07 contracts on W09's request.
Concurrency/allocation context: IRQ context; loop bound = freed count;
  no allocation; no policy choice beyond consuming the queue in its own
  order.
Errors and failure guarantee: if the admission contract reports
  inability to admit, stop with the remainder pending (never drop, never
  spin); emit the deferred-remainder event.
Security/authorization checks: admission targets only the vCPU bound to
  this pCPU; cross-vCPU admission is structurally impossible here and is
  additionally covered by W09-DV06.
Logic: while admitted < freed and queue reports a ready item: request
  admission via W08; count.
Validation: W09-DV01 (over-capacity), W09-DV05 (repeat progress),
  W09-DV06 (isolation).
```

### 3.6 `report_unexpected_maintenance`

```text
Name and stability: report_unexpected_maintenance; internal.
Purpose and caller: containment path for unrecognized or contradictory
  maintenance status; called by maintenance_irq_entry.
Inputs / outputs: raw status snapshot -> none.
Preconditions: none.
Postconditions: unexpected counter incremented (saturating); raw snapshot
  recorded; one telemetry event emitted; no lifecycle state mutated by this
  call.
State and ownership change: only W09's own two diagnostic fields.
Concurrency/allocation context: IRQ context; no allocation; snapshot is a
  fixed-size value, not a formatted string.
Errors and failure guarantee: cannot fail; never panics; a persistent
  recurrence is surfaced by the counter for W12/W13 review rather than
  escalated at runtime.
Security/authorization checks: none.
Logic: saturating increment; store snapshot; emit unexpected-condition
  event with raw value and capture order; return.
Validation: W09-DV04 (injected unexpected condition).
```

## 4. Invariants and error model

Invariants (review basis for workflow step 5):

- **I1 — No loss.** A vIRQ that enters the pending set is either eventually
  presented or remains pending with a diagnostic; capacity pressure never
  destroys it (P6-V16).
- **I2 — No duplicate completion.** Each presentation epoch of a slot
  reconciles at most once; duplicate reports are counted no-ops; no vIRQ
  completes twice through this path.
- **I3 — No active-state leak.** A reconciled slot returns to FREE in the
  same invocation; no state retains "active" after reconciliation.
- **I4 — Bounded progress.** Each maintenance invocation performs work
  bounded by its controller read; sustained pending work progresses across
  invocations without starvation guarantees claimed (fairness excluded).
- **I5 — Cross-vCPU containment.** The path touches only the vCPU bound to
  the local pCPU's virtual CPU interface; cross-vCPU mutation is structurally
  excluded and tested (W09-DV06).

Named failure cases:

- **E1 — Duplicate completion report.** Class: benign; counted no-op; no
  state change; telemetry only.
- **E2 — Orphan completion (no in-flight record).** Class: hypervisor
  invariant violation (state-integrity loss); contained for the current
  invocation (no mutation), full per-vCPU presentation snapshot recorded,
  escalated on the W12 impossible-state path; fatal per P0-W14
  classification. Never auto-repaired.
- **E3 — Unrecognized status pattern.** Class: hardware/specification
  diagnostic; contained (§3.6); no state mutation; recurring recurrence is
  an investigation, not a runtime escalation.
- **E4 — Admission contract refusal.** Class: bounded-deferral; remainder
  stays pending (I1); deferred-remainder event emitted.

## 5. Telemetry

Required logical event set (identifiers registered under the P0-W13 namespace
contract; names are not fixed here):

| Logical event | Payload classes | Consumer |
|---|---|---|
| maintenance entry | pCPU, bound vCPU (or none) | W13 P6-V24 coverage |
| condition set | decoded classes, raw value class | W13, W12 review |
| slot reconciled | vIRQ id, slot index class, completion source (Guest EOI vs reusable) | W13 latency correlation |
| refill admitted | count, remaining-pending flag | W13 P6-DOC-05 maintenance frequency |
| unexpected condition | raw snapshot, saturating count | W12 review |
| deferred remainder | count | W13, W10 deferred semantics |

## 6. Security and authorization

No Guest-reachable interface exists on this path. Guest influence is indirect
(Guest EOI behavior on presented interrupts) and is interpreted only through
the architectural controller state after the W08 presentation rules; a Guest
cannot name a slot, a vIRQ, or another vCPU through maintenance processing.
Guest-caused anomalies on this path (for example, spurious EOI patterns) are
counted and contained as GuestFault-class diagnostics per the W12 boundary;
they must never degrade into E2-style invariant failures of Host bookkeeping
— the separation between E1 (benign, Guest-influenceable) and E2 (orphan,
Host-integrity) is the load-bearing distinction and is reviewed in workflow
step 5.
