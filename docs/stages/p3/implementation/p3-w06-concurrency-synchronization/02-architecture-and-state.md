# P3-W06 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W06 detailed design](README.md).

## 1. Objects and ownership

| Object | Count | Owner | Mutable parts | Immutability |
|---|---|---|---|---|
| `SpinLock<T>` instances | one per protected shared object, declared by the owning consumer design | the consumer that owns the protected data | the locked flag word | the ladder-class tag is fixed at construction |
| `InterruptSaveSpinLock<T>` instances | one per interrupt-sensitive shared object | the owning consumer design | locked flag + saved-mask bookkeeping | ladder-class tag fixed at construction |
| Atomic-ordering policy | one per stage | W06 (this design) | changes only via design change | normative text, citable by id |
| Lock-order ladder | one per stage | W06 | changes only via design change | normative rank table |
| Misuse checklist | one per stage | W06; enforced by review and [P3-W10](../p3-w10-smp-safety-audit/README.md) | changes only via design change | normative checklist |
| Declared contention test limits | one per stage | W06; executed by [P3-W12](../p3-w12-smp-stress-failure-tests/README.md)/[P3-W13](../p3-w13-qemu-smp-regression/README.md) | recorded constants with rationale | limit values recorded in the implementation record |

W06 introduces no global runtime structure. The ladder and policy are
documented rules, not objects; there is deliberately no lock registry, no
lock-manager, and no runtime that "knows" all locks (guardrail: no God
object).

## 2. Logical modules

| Logical module | Responsibility | Inputs | Outputs | Owned state | Non-responsibility |
|---|---|---|---|---|---|
| A. Lock primitives | the two flavors, guards, try-lock, ordering and context contracts | ladder class tag at construction | mutual exclusion with stated happens-before edges | the lock instances themselves (owned with the data) | what the data means; consumer protocols |
| B. Atomic-ordering policy | the pattern table, SeqCst/fence restrictions | delivered W02–W05 protocols (consistency check) | citable ordering rules per pattern | none (normative text) | rewriting delivered protocols |
| C. Ladder and misuse rules | rank table, leaf rules, misuse checklist | consumer inventory (from plans/designs) | review criteria; W10 audit criteria | none (normative text) | runtime detection (Reserved) |
| D. Evidence harness | host-side tests for A within declared limits | primitive implementations | W06-DV01–DV03 evidence | none (test-side) | QEMU SMP stress (W12/W13) |

Dependency direction: D tests A; B and C are consumed by every P3 package
that declares shared state; A's instances are declared by consumer designs
and constructed by their owning modules.

## 3. The concurrency model of the primitives themselves

- **Locked flag:** one atomic word per lock. `lock()` is a
  compare-exchange-based acquire loop or equivalent (`Acquire` on success);
  `unlock()` is a `Release` store. `try_lock()` is a single
  compare-exchange (`Acquire` on success, no wait).
- **Data access:** the guard dereferences `T` through the locked flag's
  mutual exclusion; the `Acquire`/`Release` pair plus the flag's exclusivity
  makes `&mut T` sound without further fencing. No `unsafe` beyond the small
  audited boundary that derives `&mut T` from the shared payload under the
  flag's exclusion — SAFETY-commented per the P0-W10 governance.
- **Interrupt-save flavor:** on `lock()`, save and mask the interrupt mask
  (P1-baseline DAIF discipline), then acquire; on `unlock()`, release, then
  restore. Restoring after release (not before) prevents an interrupt
  handler from re-entering the lock on the same CPU mid-unlock. Ordering
  semantics are otherwise identical to the plain flavor.
- **No blocking:** neither flavor sleeps, yields, or waits on another CPU's
  progress beyond the acquire spin; the acquire spin itself is subject to
  the busy-wait rules ([04 §5](04-code-contracts-atomic-and-ordering-policy.md))
  — at P3 the acquire loop is unbounded by *construction of the holder*
  (critical sections are bounded by rule), and the holder-side bounds are
  the misuse rules' subject.
- **Destruction:** locks live as long as their data; no lock is created or
  destroyed at runtime in P3 (no hotplug, no dynamic subsystems). A lock
  whose data would outlive it is a design error to raise.

## 4. State machine (per lock instance)

```text
Unlocked --lock()/try_lock() success--> Locked { guard live }
Locked   --guard drop (unlock)------> Unlocked
Locked   --try_lock()---------------> fail (returns None; no mutation)
```

There are no other states. A `try_lock` failure does not mutate. Double
unlock is impossible by ownership (the guard is the only unlock path); a
guard escaping its scope is prevented by the borrow checker and forbidden
from being stored by the misuse rules. If an implementation ever observes a
lock word inconsistent with these states, that is memory corruption — fatal
diagnostic path per the P0-W14 classification, not a recoverable error.

## 5. Boundaries and prohibited shortcuts

- No consumer may implement a private lock, a second flag word, or a
  "usually uncontended so let's skip it" path: any mutual exclusion on
  shared state goes through these flavors, citably tagged with its ladder
  class.
- No consumer may widen a critical section to include a wait (WFE, poll of
  another CPU, notification `wait`) — the reactive-wait rule in
  [04 §5](04-code-contracts-atomic-and-ordering-policy.md) names the one
  structured exception pattern (reception-duty interleaving in bounded
  initiation waits, designed by W08).
- No consumer may alias the payload outside the guard (raw-pointer
  shenanigans to "borrow early") — that is `unsafe` outside the audited
  boundary and fails the P0-W10 review.
- The ladder is not advisory: a consumer design that needs a new ladder
  class or a different rank proposes a W06 design change; a silent new
  class in consumer code is a review failure and a W10 finding.

## 6. Consumer integration map

| Consumer | Rule surface consumed | Contract point |
|---|---|---|
| W07 (notification) | atomic policy (single-variable publication for the slot word), busy-wait rules (WFE/SEV ownership, bounds, no wait under lock), misuse checklist | [04](04-code-contracts-atomic-and-ordering-policy.md) §3, §5, §6 |
| W08 (TLB transport) | `SpinLock` (single-flight initiation), ladder class `Infrastructure`, reactive-wait rule, bounded completion poll | [03](03-code-contracts-lock-primitives.md) §2; [04](04-code-contracts-atomic-and-ordering-policy.md) §4–§5 |
| W09 (exception/interrupt) | Diagnostics class rules, fatal-path no-lock rule, irq-save flavor for any interrupt-sensitive future surface | [03](03-code-contracts-lock-primitives.md) §3; [04](04-code-contracts-atomic-and-ordering-policy.md) §4, §6 |
| W10 (audit) | misuse checklist + ladder + atomic policy as classification evidence requirements | [04](04-code-contracts-atomic-and-ordering-policy.md) §3–§6 |
| W11 (observability) | Statistics class, Relaxed counter rule, contention hooks (content only) | [04](04-code-contracts-atomic-and-ordering-policy.md) §3–§4 |
| W12/W13 (stress, regression) | declared contention limits; primitive evidence as the base layer | [05-implementation-workflow.md](05-implementation-workflow.md) step 6; [06-validation-and-handoff.md](06-validation-and-handoff.md) |
| W14/P4 (handoff) | full policy + ladder + flavors as the synchronization contract | [06-validation-and-handoff.md](06-validation-and-handoff.md) §3 |
