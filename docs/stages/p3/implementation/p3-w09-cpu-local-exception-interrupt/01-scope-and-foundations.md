# P3-W09 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W09 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Make the P1 exception and interrupt diagnostic foundation correct for
every online physical CPU" requires four concrete artifacts:

1. The **per-CPU exception-local state**: slot contents, entry/exit
   bookkeeping, nesting bound —
   [03-code-contracts-exception-local-state.md](03-code-contracts-exception-local-state.md).
2. The **attribution rule**: three ranks resolving who/where a faulting
   CPU is under any initialization state —
   [04-code-contracts-attribution-and-logging.md](04-code-contracts-attribution-and-logging.md)
   §2.
3. The **shared-resource rules**: fatal-path locking strategy and
   concurrent-logging discipline for the one cross-CPU resource on an
   exceptional path — same file §3–§4.
4. **Evidence**: boot-role and secondary-role exceptional-path
   demonstrations (intentional synchronous faults per the P1-W05
   acceptance pattern), attribution correctness, and
   simultaneous-logging behavior —
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

Without (1) the "non-overlapping" requirement is unenforceable; without
(2) identity is wrong exactly when it matters most; without (3) multi-CPU
faults produce deadlocks or noise; without (4) P3-V09 has nothing to
review.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan/design | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Exception entry that can emit context; bounded non-recursive paths | [P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md), [P1-W07](../../../p1/plans/p1-w07-fatal-crash-diagnostics.md) | Vector coverage, origin classification, syndrome capture, fatal field set, non-recursion | If the delivered P1 contract freezes a *global* save/scratch region or context buffer, that is an Architecture Change Request with the P1 owner — W09 does not fork the entry path |
| `current()` valid from the first post-install instruction; `ExceptionLocalSlot` reserved | [P3-W04](../p3-w04-per-cpu-runtime/README.md) 04 §2, 03 §5.3 | Register-based locality; slot placement/sizing; header fields | If the slot is absent/undersized: cross-design conflict with the W04 owner; no second per-CPU storage |
| Secondary entry attribution from the first instruction | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) 03 §3, README handoff | MPIDR-based identity; pre-console failure still attributed via the outcome map | If W02's stub cannot attribute pre-identity faults, the rank-2 rule degrades to rank 3 for those paths — recorded, not improvised |
| Lifecycle state readable for attribution | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) 04 §2 | `state_of(logical)` lock-free read | Attribution reads are best-effort under fault (the registry itself may be the faulting object — rank rules cover this) |
| Lock/context rules | [P3-W06](../p3-w06-concurrency-synchronization/README.md) | Diagnostics class; CR-2–CR-5; MIS-6/MIS-11 | Conflicts are raised with the W06 owner; W09 does not weaken the fatal-path rule |
| Diagnostics/trace governance | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md), [P3-W11](../p3-w11-smp-observability/README.md) | Levels, namespaces, catalog ownership (W11) | Missing governance blocks only the emission-format step, not the state/attribution design; recorded, not invented |

### 1.3 Why no hidden essential deliverable remains

- "Integrate exceptional-path requirements with audit, telemetry, stress,
  and regression consumers" (plan step 3) is realized by the
  non-overlap invariant (→ W10), the attribution field set and emission
  points (→ W11), the concurrent-fault/logging scenarios (→ W12/W13),
  and the handoff section — designed surfaces, not implied.
- "Review no-cross-CPU-scratch-state and shared-logging constraints"
  (plan step 4) is the DV04 review plus the W10 audit item — named
  checks, not prose.
- "Collect boot/secondary exceptional-path acceptance evidence" (plan
  step 5) is bounded: W09 defines the scenarios and their single-pass
  evidence; the repeated matrix is W13's (stated in the validation file).

## 2. Scope classification

### 2.1 Required

- `ExceptionLocalSlot` contents: nesting depth (bounded), in-exception
  flag, current vector class note, fatal record area (per-CPU),
  attribution snapshot fields.
- Entry/exit bookkeeping contracts (`exception_begin`/`exception_end`
  semantics as design-level steps the P1 entry path integrates).
- Three-rank attribution rule with mandatory attribution metadata.
- Fatal integration: per-CPU record first; console under W06 CR-5
  (bounded try-lock, best-effort fallback, interleave marker).
- Concurrent-logging rules for normal diagnostics (Diagnostics lock).
- Boot-role and secondary-role exceptional-path evidence, single-pass,
  within declared limits.

### 2.2 Reserved (must not block a future design; not implemented now)

- Enabling any host interrupt source; IRQ dispatch design; trigger: an
  approved host-IRQ design (P3 preserves the P1 posture).
- Per-CPU IRQ stack switching; trigger: the same design.
- Timer/fault-grade interrupt handling; trigger: P6.
- Deeper nested-exception capture; trigger: an approved design needing
  it (P3's bound is the P1-W05 non-recursion requirement, made per-CPU).
- Crash storage/transport beyond the P1 console; trigger: P1-W07's
  recorded out-of-scope, later stage.

### 2.3 Out of Scope

- Vector-table mechanics, saved-register layout, origin classification
  (P1-W05 contract; W09 consumes and relocates *state*, not mechanics).
- GIC initialization/dispatch/virtualization; guest interrupts; vIRQ
  (P6/P8).
- Scheduler preemption, deferred work, softirq-like machinery (P7).
- Telemetry catalog, event ids, log formats (W11).
- Board-specific exception handling (ADR-043/ADR-052).
- Modifying W02's entry path, W03's registry semantics, or W04's area
  layout.

## 3. Resolved decisions — authority notes

The entry README carries the numbered decisions; authority basis:

- Decision 1: plan scope ("legal per-CPU exception entry/local context",
  "non-overlapping exception state") + W04's slot reservation; the
  CPU-private shape is the only structural reading.
- Decision 2: P3-V09 wording ("correct CPU identity ... on supported
  boot and secondary CPU exceptional paths"); W02/W04 guarantees are the
  frozen-contract tier; rank boundaries are stage-local freedom owned
  here.
- Decision 3: authority order (P1 contract > this design's convenience)
  and the skill's conflict rules; the plan's out-of-scope line on
  vector/context layout.
- Decision 4: P1-W05's non-recursive requirement and P1-W07's bounded-
  fatal requirement, made per-CPU; bound value is stage-local freedom
  recorded with rationale.
- Decision 5: P1-W05's handoff sentence ("IRQ dispatch and GIC behavior
  remain explicitly unimplemented") + plan out-of-scope; the
  interrupt-sensitive task-book requirement is met by binding rules +
  posture.
- Decision 6: P3-V09's safe-simultaneous-logging wording + W06 CR-5;
  bounded-try-lock-with-fallback is stage-local freedom with recorded
  rationale (an attributed interleave beats a deadlock).
- Decision 7: P3-V11's CPU-attribution requirement; the field set
  aligns with W11's plan scope (hardware identity, logical identity,
  boot/secondary role).
