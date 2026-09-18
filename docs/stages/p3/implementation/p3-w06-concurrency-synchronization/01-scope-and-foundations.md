# P3-W06 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W06 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Reviewed synchronization semantics sufficient for shared Host SMP state and
interrupt-sensitive execution" requires four concrete artifacts:

1. The **primitive set**: two lock flavors with type and guard contracts
   whose orderings and context rules are stated —
   [03-code-contracts-lock-primitives.md](03-code-contracts-lock-primitives.md).
2. The **policy artifacts**: the atomic-ordering pattern table and the
   lock-order ladder as citable normative rules —
   [04-code-contracts-atomic-and-ordering-policy.md](04-code-contracts-atomic-and-ordering-policy.md)
   §3–§4.
3. The **enforcement surfaces**: the busy-wait rules and the misuse
   checklist, phrased as review-checkable statements a consumer design or
   the [P3-W10](../p3-w10-smp-safety-audit/README.md) audit can cite — same
   file §5–§6.
4. **Evidence**: host-side correctness, try-lock, ordering, and
   concurrent-hammer results within declared limits, plus the declared
   limits themselves —
   [05-implementation-workflow.md](05-implementation-workflow.md) and the
   matrix in [06-validation-and-handoff.md](06-validation-and-handoff.md).

Without (1)–(3) there is nothing for consumers to follow or for P3-V06's
"reviewed access follows declared rules" to check; without (4) the rules are
untested prose, which the plan's acceptance wording excludes.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| No-lock atomic protocols for boot/bring-up state | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md), [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md), [P3-W05](../p3-w05-smp-boot-synchronization/README.md) | Single-variable acquire/release protocols; W05's one justified full-barrier publication; W03 CAS transitions | If a delivered protocol needs an ordering the policy table cannot name, that is a cross-design conflict to resolve with the owner; W06 does not restate or rewrite their protocols |
| Per-CPU area privacy boundary | [P3-W04](../p3-w04-per-cpu-runtime/README.md) | Areas are CPU-private after install; cross-CPU access only through documented shared surfaces | If W04 areas are accessed cross-CPU outside documented surfaces, that is a W10 audit finding (misuse), not a W06 rule change |
| Unsafe governance (justification, inventory, review) | [P0-W10](../../../p0/plans/p0-w10-unsafe-rust-governance.md) | Every non-trivial `unsafe` block carries SAFETY premises and inventory entries | If the P0 governance is absent when W06 code lands, the lock primitives' pointer/atomic `unsafe` cannot be reviewed — record a blocker; do not invent a private scheme |
| Diagnostics/trace governance | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) | Log levels, trace namespaces, contention-event content coordinate with [P3-W11](../p3-w11-smp-observability/README.md) | A missing governance surface blocks only the *telemetry* step, not the semantics; record it and proceed with semantics |
| Panic/failure classification | [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md) | Invariant violations (ladder breach, lock corruption) are fatal-class, distinct from recoverable errors | If the classification differs, the misuse checklist's escalation wording changes by a recorded decision, not silently |
| Host-side test baseline | [P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md) | A host test entry point exists for primitive tests | Without it, W06 evidence is blocked at the evidence step; semantics and contracts remain valid design deliverables |
| Allocator concurrency boundary | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md), [P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md) | The allocator's own locking is P2-owned and internally ordered; P3 consumes entry points | If P2 delivers an allocator that requires P3-side allocation under arbitrary locks, the no-allocation-under-lock rule (README decision 4) becomes a cross-stage conflict — raise it; do not weaken the rule locally |

### 1.3 Why no hidden essential deliverable remains

- "Integrate the rules with notification, TLB transport, exception, and
  shared-infrastructure consumers" (plan step 3) is realized by the consumer
  integration map ([02 §6](02-architecture-and-state.md)) plus the rule
  citations each consumer design is required to carry — designed surfaces,
  not implied ones. The consumer designs are written against these rules;
  W06's integration obligation is that the rules exist, are citable, and
  cover every context those consumers named in their plans.
- "Collect correctness and contention evidence within declared test limits"
  (plan step 5) is bounded two-sided: W06 declares the limits and delivers
  primitive-level evidence; the repeated SMP execution of consumer-level
  stress belongs to W12/W13, stated in the matrix rather than left implicit.
- "Record lock-order and atomic-ordering handoff rules" (plan step 6) is
  the handoff section of [06-validation-and-handoff.md](06-validation-and-handoff.md)
  — a deliverable, not an afterthought.

## 2. Scope classification

### 2.1 Required

- `SpinLock<T>` with `lock`/`try_lock` and its guard; `InterruptSaveSpinLock<T>`
  with interrupt-mask save/restore semantics.
- Atomic-ordering pattern table (publication, transition, counter,
  multi-variable publication) with the SeqCst and fence restrictions.
- Lock-order ladder: five named classes, rank ordering, leaf rules.
- Busy-wait constraints (bounds, no-wait-under-lock, WFE/SEV reserved) and
  the reactive-wait rule for CPUs with reception duties.
- Misuse checklist (review-checkable, W10-consumable).
- Consumer integration map and handoff rules.
- Host-side primitive evidence within declared limits.

### 2.2 Reserved (must not block a future design; not implemented now)

- Try-lock-with-timeout; trigger: a consumer design needing bounded
  acquisition rather than bounded spinning.
- Runtime held-lock tracking / deadlock detection; trigger: a consumer
  design needing runtime checks (W11 may record hold/wait statistics — that
  is telemetry, not detection).
- WFE-based lock acquisition; trigger: [P3-W07](../p3-w07-cross-cpu-notification/README.md)
  owns WFE/SEV semantics.
- RCU-like read-copy structures and rwlock; trigger: an approved
  read-dominated consumer design.
- Real-time or priority-inheriting locks; trigger: P7 scheduler design /
  ADR-017 territory.

### 2.3 Out of Scope

- Blocking scheduler locks, condition variables, full rwlock families,
  priority inheritance, realtime locking (plan out of scope; P7 policy).
- Consumer shared-state designs (W07/W08/W09/W11 own their state under
  these rules).
- Boot rendezvous and phase protocol (W05); lifecycle transitions (W03);
  per-CPU area internals (W04).
- P2 allocator internal lock design (P2's contract; audited by W10).
- Telemetry catalog, contention-event ids, rates (W11).
- Guest-visible synchronization and any guest-facing primitive (P8+).
- Concrete instruction-sequence selection for the CAS loops and the exact
  Rust type encoding (implementation-record material under the approved
  build design; the semantics here are the contract).

## 3. Resolved decisions — authority notes

The entry README carries the numbered decisions; the authority basis, in
governing order:

- Decisions 1, 6: ADR P3 roadmap (spinlock, irq-save lock, atomic helpers,
  explicit lock ordering) plus the plan's scope sentences — stage-local
  freedom is only in *how* the named requirements are bounded, not whether.
- Decision 2: consistency with the established W02–W05 module contracts
  (frozen-contract tier); nothing new is invented.
- Decision 3: ADR guardrails (no God objects) and the plan's requirement of
  a documented baseline ordering.
- Decision 4: prudence under the assumed P2 allocator contract; recorded as
  a W06-owned rule that a consumer may challenge only through a design
  change.
- Decision 5: task book's interrupt-sensitive requirement; the P1 interrupt
  posture is the upstream condition (assumed contract, §1.2).
- Decision 7: frozen-contract consistency duty (skill conflict rules).
- Decision 8: plan's "within declared test limits" and the P0-W08 baseline.
