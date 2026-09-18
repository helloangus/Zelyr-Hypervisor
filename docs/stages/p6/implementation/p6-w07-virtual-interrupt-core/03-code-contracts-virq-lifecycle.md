# P6-W07 Code Contracts — vIRQ Lifecycle

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W07 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (decisions D1–D8),
[02](02-architecture-and-state.md) (ownership, state model, concurrency).
Authorization-specific contracts are in
[04](04-code-contracts-authorization-and-errors.md).

## 1. Types

### 1.1 Identities

```text
Name and stability:
  VcpuId, VirqId, VirqPriority — validated newtypes (P0-W15 conventions).
  VirqClaim — opaque token issued by claim(). Internal to W07 and its
  named consumers (W08/W09); stable within P6.
Purpose and caller: make target and interrupt identities unforgeable at
  the type level; VirqClaim prevents stale/foreign returns.
Inputs / outputs: constructors are total for validated ranges only:
  VirqId::new(raw, window: &VirqWindow) -> Result<VirqId, InvalidRange>;
  VirqPriority::new(raw, width_bits: u8) -> Result<VirqPriority, InvalidRange>.
Preconditions / postconditions: a VirqId is within the declared SGI/PPI
  bank or SPI window; a VirqPriority fits the declared priority width.
Concurrency/allocation context: value semantics.
Errors: InvalidRange carries the value and the window for diagnostics.
Security/authorization checks: constructors are the validation point for
  Guest-supplied integers ([04] §2).
Validation: host-side unit tests (W07-DV01).
```

### 1.2 `VirqWindow` (intake declaration)

```text
Name and stability: VirqWindow { sgi_ppi_bank: 0..=31 (fixed by the
  architecture), spi_first: VirqId, spi_last: VirqId,
  priority_width_bits: u8 } — built once at init from the W01-reconciled
  capability declaration. Immutable afterwards. Internal to W07; stable
  within P6.
Purpose and caller: the only platform-dependent shape of the bank;
  consumed by VirqId/VirqPriority constructors and bank sizing.
Preconditions: spi window non-empty is not required (an empty declared
  SPI window is legal for a minimal platform); priority width > 0.
Errors: DeclarationConflict on overlapping/inverted ranges.
Security/authorization checks: platform capability data is untrusted
  until the declaration validates.
Validation: W07-DV01 intake exercise.
```

### 1.3 `VirqBank` (per vCPU)

```text
Name and stability: VirqBank — one per vCPU, held by the vCPU object.
  Internal to W07; stable within P6.
Fields:
  pending:      bitset over the window
  active:       bitset over the window
  presented:    bitset over the window (set/cleared only via claim/return)
  occurrences:  saturating counters per bit
  priority:     priority slots per bit (written once per pending episode)
  counters:     telemetry per [02] §7
  lock:         one irq-save spinlock per bank (P3 discipline)
Purpose and caller: sole authority for the vCPU's virtual-interrupt
  software truth (D1); operated by virq_* functions below.
Preconditions: created with the vCPU; sized from VirqWindow; zeroed at
  creation (no inherited state — a fresh vCPU starts with no pending
  work even if the pCPU/GIC has residue, which is W02/W03's cleanup).
Postconditions: drained (empty) before vCPU destruction completes.
Concurrency: all operations take the single bank lock; see [02] §5.
Errors: invariant violations counted and recovered ([02] §4 rule 5).
Security/authorization checks: at the function boundary, not the struct.
Validation: W07-DV01–DV06.
```

## 2. Injection

### 2.1 `virq_inject()`

```text
Name and stability: fn virq_inject(request: VirqRequest) ->
  Result<InjectOutcome, VirqDenial> where VirqRequest { target:
  ValidVcpuRef, class: ProducerClass (HostMechanism{source} |
  GuestRequest{authorized context}), vintid: VirqId,
  priority: VirqPriority }. Internal to W07; producers are W06, the HVC
  path ([04] §3), and — only through future design amendments — later
  Host sources. Stable within P6.
Purpose and caller: the single entry point that makes a virtual
  interrupt pending for a specified vCPU.
Inputs / outputs: validated request -> Accepted{state} | Coalesced |
  QueuedWhilePresented | QueuedWhileActive, or a typed denial.
Preconditions: authorization already performed ([04] §2 — this function
  re-checks the cheap invariants: range by type, target liveness);
  postconditions: on success, `pending[vintid] == true`; occurrence
  count reflects dedupe class; no other datum in any bank changed.
Concurrency: IRQ/VM-exit/mainline safe; O(1); no allocation; single
  bank lock.
Errors and failure guarantee (denials leave all state unchanged):
  InvalidTarget (dead/uninitialized vCPU), InvalidRange (by type),
  InvalidPriority (by type), ClassNotAllowed ([04] §2 rule),
  BankDraining (destroy race — counted, benign).
Security/authorization checks: see [04] §2; no VM-ID/role shortcut.
Logic (pseudocode):
    bank = target.bank(); lock(bank)
    if bank.draining { unlock; return BankDraining }
    if bank.active[vintid] { pending[vintid] = true; occurrences += 1;
        outcome = QueuedWhileActive }            // D6: presented later
    else if bank.presented[vintid] { pending[vintid] = true;
        occurrences += 1; outcome = QueuedWhilePresented }
    else if bank.pending[vintid] { occurrences += 1 (saturating);
        outcome = Coalesced }
    else { pending[vintid] = true; occurrences = 1; priority[vintid] =
        request.priority; outcome = Accepted }
    update counters; unlock(bank)
Validation: W07-DV02 (single), DV04 (repeated), DV05 (denials).
```

## 3. Deferred and unavailable-vCPU semantics

```text
Contract (behavioral, implements D3): virq_inject never observes or
  requires the target's run state. A vCPU that exists but is not running
  simply accumulates pending state (P6-V12 "survive deferred
  presentation"). Only liveness matters:
  - target exists, alive        -> normal path (deferred if absent);
  - target exists, draining     -> BankDraining (counted, no state);
  - target destroyed/unknown    -> InvalidTarget.
  There is deliberately no "kick", "signal", or "wakeup" output: wakeup
  is P7 policy. Consumers that need prompt presentation observe pending
  state at the vCPU's next entry (W08 loads LRs then).
Validation: W07-DV03 (absent-target deferral; dead-target rejection).
```

## 4. Repeated arrival (P6-V14 basis)

```text
Contract (behavioral, implements D5): every arrival increments the
  saturating occurrence counter for the vINTID exactly once:
  - while Pending     -> Coalesced (no second queue entry exists);
  - while Presented   -> QueuedWhilePresented (pending set again; after
                          completion, re-presentation happens naturally);
  - while Active      -> QueuedWhileActive (presented after completion
                          clears active, per D6).
  Occurrence counters are saturating (a storm cannot overflow or grow
  memory); per-episode coalescing counts are exposed to telemetry and
  later to W10's semantics. No loss is possible in P6's model because
  pending is a bit, not a queue: at least one delivery is always owed
  while the bit is set, and the count records suppressed duplicates.
Validation: W07-DV04 (repeat-in-each-state cases).
```

## 5. Presentation protocol (consumed by W08; maintained by W09)

### 5.1 `virq_select_next_pending()`

```text
Name and stability: fn virq_select_next_pending(bank) ->
  Option<VirqSelection> where VirqSelection { vintid: VirqId,
  priority: VirqPriority, claim: VirqClaim }. Internal to W07; sole
  caller is W08 at entry-load and refill ([../p6-w08-gic-virtualization-
  interface/README.md]). Stable within P6.
Purpose and caller: offer the next presentable event, respecting D6.
Inputs / outputs: bank -> highest-urgency pending event not excluded by
  D6, with its claim token.
Preconditions: caller is the presenting context for this vCPU (W08
  entry/refill); postconditions: on Some, `pending == false`,
  `presented == true` for the selection; the bit is claim-locked against
  double presentation.
Concurrency: bank lock; O(window width); no allocation. Callable in
  maintenance context (bounded width, [02] §5).
Errors: none; None simply means nothing presentable.
Security/authorization checks: Host-internal protocol; no guest input.
Logic (pseudocode):
    lock(bank)
    best = None
    for vintid in window where pending[vintid] && !active[vintid] {
        if best.is_none() || priority[vintid] < best.priority
           || (priority == best.priority && vintid < best.vintid) {
            best = vintid }                       // D7 tie-break
    }
    if let Some(v) = best { pending[v] = false; presented[v] = true;
        claim = VirqClaim::issue(bank, v); presented_count += 1 }
    unlock(bank)
Validation: W07-DV02 (selection order), DV04 (exclusion of active).
```

### 5.2 `virq_presentation_returned()`

```text
Name and stability: fn virq_presentation_returned(claim: VirqClaim,
  outcome: PresentationOutcome) -> Result<(), ProtocolViolation> where
  PresentationOutcome is StillPending | BecameActive | Completed.
  Internal to W07; sole callers are W08 (exit purge, eviction) and the
  W09-completion path via W08. Stable within P6.
Purpose and caller: close a presentation: hand the LR's fate back into
  W07 truth without loss or duplicate completion (P6-V16/V17 support).
Preconditions: claim is live and belongs to this bank/vintid; post-
  conditions per [02] §4 rule 3; presented cleared; counters updated.
Concurrency: bank lock; O(1).
Errors: ProtocolViolation on stale/foreign/duplicate claims — counted,
  state forced consistent (presented cleared, pending preserved), never
  a panic; the violation is a consumer-contract bug (Host-attributed).
Security/authorization checks: tokens are unforgeable typed values not
  exposed to Guests.
Logic: validate claim generation/identity; apply outcome per [02] §4;
  clear presented; unlock.
Validation: W07-DV02/DV04; protocol misuse cases exercised in DV07.
```

### 5.3 `virq_report_guest_completion()`

```text
Name and stability: fn virq_report_guest_completion(bank, vintid) -> ()
  — called by W09's processing (through W08's LR-state extraction) when
  the Guest visibly completes the event. Stable within P6.
Purpose and caller: release active state so pending work progresses
  (P6-V17 basis); completion of a not-active vINTID is an anomaly to
  count, not to apply.
Preconditions: bounded O(1); maintenance or exit context.
Postconditions: active cleared; pending untouched (re-presentation is
  selection's job later).
Concurrency: bank lock; O(1).
Security/authorization checks: none (Host-internal correlation; the
  Guest influences this only through architectural EOI behavior, which
  W08/W09 observe, not forge).
Logic: lock; if active { active = false; completed += 1 } else {
  anomaly_count += 1 }; unlock.
Validation: W07-DV04 (completion then re-presentation), DV07.
```

### 5.4 `virq_query_summary()`

```text
Name and stability: fn virq_query_summary(bank) -> VirqSummary {
  any_pending: bool, pending_priority_ceiling: Option<VirqPriority>,
  presented_count, active_count } — read-only snapshot for W08 entry
  planning and diagnostics. Stable within P6.
Concurrency: bank lock; O(window width); no mutation.
Validation: reviewed in W07-DV02/DV06.
```

## 6. Destruction drain

```text
Name and stability: fn virq_drain(bank) -> DrainReport { dropped_pending:
  u32, cleared_presented: u32, cleared_active: u32 } — called from the
  vCPU/VM destruction path (P4 lifecycle hook, assumed contract [01] §5)
  after W08 has purged LRs at the final unload. Implements D8.
Preconditions: no further entry can occur (caller guarantees); any live
  claims were returned by W08's final purge (ordering is: purge, then
  drain; a claim surviving into drain is a ProtocolViolation counted and
  forced-consistent).
Postconditions: bank empty; later injects get BankDraining then, after
  destruction, InvalidTarget.
Concurrency: exclusive destroy context; takes the bank lock.
Errors: none returned; anomalies counted in DrainReport and telemetry.
Security/authorization checks: none (Host lifecycle path).
Validation: W07-DV03 (drain determinism).
```
