# P3-W06 Concurrency Synchronization — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewed shared-state synchronization semantics — lock
primitives, atomic-ordering policy, lock-order ladder, busy-wait and misuse
rules — required by [P3-W06](../../plans/p3-w06-concurrency-synchronization.md).  
**Owner/change context:** P3-W06 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W06. It establishes the
stage's synchronization baseline: exactly two lock flavors with their
memory-ordering and context contracts, a stage-wide atomic-ordering policy, a
named lock-order ladder covering allocator, registry, statistics, and future
consumers, binding busy-wait constraints, and the misuse checklist that review
and the [P3-W10](../p3-w10-smp-safety-audit/README.md) audit enforce. It
deliberately does **not** define a general barrier library, condition
variables, rwlocks, blocking scheduler locks, or priority inheritance (plan
out of scope), does not restate [P3-W05](../p3-w05-smp-boot-synchronization/README.md)'s
boot protocol (W05 uses only atomics and added no lock), and does not design
any consumer's data structure — W06 owns the *rules*; W07/W08/W09/W11 own
their own shared state under these rules.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, assumed-contract failure boundaries, scope classification |
| [02-architecture-and-state.md](02-architecture-and-state.md) | logical modules, ownership, the ladder model, consumer integration map |
| [03-code-contracts-lock-primitives.md](03-code-contracts-lock-primitives.md) | `SpinLock` / `InterruptSaveSpinLock` type and guard contracts |
| [04-code-contracts-atomic-and-ordering-policy.md](04-code-contracts-atomic-and-ordering-policy.md) | atomic-ordering policy, lock-order ladder, busy-wait and misuse rules |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W06 plan → this
design → Coding Guidelines. Binding constraints:

- The ADR P3 roadmap requires "基础 spinlock、irq-save lock、atomic helpers；
  明确锁顺序" (basic spinlock, irq-save lock, atomic helpers; explicit lock
  ordering); ADR-048 makes lock contention observable; the ADR invariant that
  every cross-CPU operation must define its synchronization semantics makes
  the atomic policy normative, not advisory.
- The task book requires synchronization semantics "sufficient for shared
  allocator/registry/state access and interrupt-sensitive contexts, including
  lock-order and atomic-ordering rules" — the ladder and the atomic policy in
  [04](04-code-contracts-atomic-and-ordering-policy.md) are those rules.
- The plan's out-of-scope list (blocking scheduler locks, condition
  variables, complete rwlock families, priority inheritance, realtime
  locking, detailed algorithms or Rust APIs) bounds the primitive set: the
  "detailed algorithms" reservation means this design fixes *semantics,
  ordering rules, and interfaces*, while the concrete CAS loop instruction
  selection and type-level Rust encoding are implementation-record material
  under the approved build design.
- The plan's consumer list (W07–W10, W12–W15, P4) receives documented
  semantics, "not unreviewed primitive internals" — therefore everything a
  consumer may rely on is stated as a rule or a contract here, and nothing
  else is guaranteed.
- [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md),
  [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md),
  [P3-W04](../p3-w04-per-cpu-runtime/README.md), and
  [P3-W05](../p3-w05-smp-boot-synchronization/README.md) each state "no
  locks; if one appears to be needed, that is a design error to raise (W06
  owns lock semantics)". This design honors that: it adds no lock to their
  delivered surfaces and defines the ladder so their atomic protocols remain
  the authority for their own state.

Classification:

- **Required** for W06 closure: the two lock flavors and guards
  ([03](03-code-contracts-lock-primitives.md)), the atomic-ordering policy,
  the lock-order ladder, the busy-wait constraints, the misuse checklist,
  the consumer integration map, host-side correctness and contention
  evidence within declared limits, and P3-V06 acceptance evidence.
- **Reserved** with recorded triggers: a try-lock-with-timeout variant
  (trigger: a consumer design that needs bounded acquisition rather than
  bounded busy-wait); runtime held-lock tracking for deadlock detection
  (trigger: a consumer design needing runtime checks — until then ladder
  conformance is by review plus the host-side harness); WFE-based lock
  acquisition (trigger: [P3-W07](../p3-w07-cross-cpu-notification/README.md)'s
  event primitive, which owns WFE/SEV semantics); read-copy structures
  (RCU-like) and rwlock (trigger: an approved consumer design with a
  read-dominated shared structure); a priority-inheritance or real-time
  lock (P7/P17 territory per ADR-016/ADR-017).
- **Out of Scope:** blocking scheduler locks and condition variables (P7);
  full rwlock families; any consumer's shared data structure or protocol
  (W07/W08/W09/W11 own those); the boot rendezvous protocol (W05);
  lifecycle transitions (W03); per-CPU area internals (W04); the P2
  allocator's *internal* locking (P2-W04/P2-W05 own it; the ladder consumes
  and audits its boundary); lock-contention telemetry content (W11 owns
  the catalog); guest-visible synchronization (P8+).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Normal mutual exclusion | [lock contracts](03-code-contracts-lock-primitives.md) §2, §4 | P3-V06 (W06-DV01, DV02) |
| IRQ-sensitive critical sections | [irq-save contract](03-code-contracts-lock-primitives.md) §3, [context rules](04-code-contracts-atomic-and-ordering-policy.md) §2 | P3-V06 (W06-DV02, DV03) |
| Atomic state transfer with acquire/release expectations | [atomic policy](04-code-contracts-atomic-and-ordering-policy.md) §3 | P3-V06 (W06-DV04) |
| Busy-wait constraints; non-sleeping contexts | [busy-wait rules](04-code-contracts-atomic-and-ordering-policy.md) §5 | P3-V06 (W06-DV05) |
| Misuse constraints | [misuse checklist](04-code-contracts-atomic-and-ordering-policy.md) §6 | P3-V06 (W06-DV06) |
| Baseline lock ordering (allocator, registry, statistics, future consumers) | [ladder](04-code-contracts-atomic-and-ordering-policy.md) §4; [integration map](02-architecture-and-state.md) §6 | P3-V06 (W06-DV05) |
| Rules integrated with notification, TLB transport, exception, and shared-infrastructure consumers | [integration map](02-architecture-and-state.md) §6 | W06 closure review (W06-DV07) |
| Correctness and contention evidence within declared limits | [workflow](05-implementation-workflow.md) steps 5–6; matrix in [validation](06-validation-and-handoff.md) | P3-V06 (W06-DV01–DV03); repeated SMP contention runs are [P3-W12](../p3-w12-smp-stress-failure-tests/README.md)/[P3-W13](../p3-w13-qemu-smp-regression/README.md) |
| Lock-order and atomic-ordering handoff rules recorded | [handoff](06-validation-and-handoff.md) §3 | W06 closure review (W06-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`):
the repository is a P0 documentation scaffold — no Cargo workspace, no Rust
sources, no synchronization code of any kind. P0–P2 are planning sets (only
P0-W01/P0-W02 have implementation and verification records); their unsafe,
diagnostic, panic, and allocator governance are assumed contracts with the
failure boundaries in [01-scope-and-foundations.md](01-scope-and-foundations.md)
§1.2. Sibling P3 designs are being prepared in parallel on this branch; W06
consumes W04/W05 as upstream contracts and serves W07–W15 and P4 downstream,
referenced by path and P3-Wxx ID without assuming design content beyond the
plans and published sibling designs.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Shared-state access follows declared rules | No rules or primitives exist; W02–W05 each used ad-hoc single-variable acquire/release protocols | The two lock flavors plus the atomic-ordering policy as the stage's only sanctioned synchronization vocabulary | "Reviewed access follows declared rules" requires the rules to exist before the consumers do | W06 (this design) | W06-DV01/DV04 reviews and tests |
| Interrupt-sensitive contexts are covered | No interrupt-sensitive code exists at P3 (P1 baseline posture; [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md) owns exceptional-path integration) | The irq-save flavor and the context rules shipped *now*, so no consumer invents them when interrupts arrive | Retrofitting IRQ rules after consumers exist is exactly the unreviewed divergence P3-V06 forbids | W06 (flavor is stage-local freedom); P1 posture | W06-DV02/DV03 |
| No corruption or unexplained deadlock under contention | Nothing runs concurrently yet | A testable primitive set (host-side per the P0-W08 baseline) with correctness, try-lock, and concurrent-hammer evidence within declared limits | Contention evidence must exist at the primitive level before consumers stress on top | W06; harness per P0-W08 | W06-DV01–DV03 |
| Ladder covers allocator, registry, statistics, future consumers | The named subsystems are plans, not code | The rank table with per-class rules and the leaf rules for allocator and diagnostics internals | A ladder written after consumers fix their orderings is documentation, not authority | W06 (ladder); consumer designs cite it | W06-DV05 review |
| Atomic-ordering expectations are explicit | W02–W05 already fixed acquire/release and one fence | The policy that codifies (and does not contradict) the delivered boot/bring-up protocols | The policy must be consistent with the existing established contracts or label the conflict | W06 policy; W02–W05 protocols | W06-DV04 consistency review |
| Misuse is preventable, not just curable | Nothing to misuse yet | The misuse checklist as a named review artifact consumed by W10 | P3-V06's "reviewed" wording makes the checklist the enforcement surface | W06 checklist; W10 audit | W06-DV06 |

No ledger row requires this design to select crate names, concrete Rust type
encodings, or runtime policy owned elsewhere; no new decision blocker is
outstanding here.

## Resolved design decisions and their authority

1. **Exactly two lock flavors, both data-carrying.** `SpinLock<T>` for
   thread-context mutual exclusion and `InterruptSaveSpinLock<T>` (save/mask
   and restore of the interrupt mask around the critical section) for
   interrupt-sensitive contexts. Rationale: the ADR P3 roadmap names exactly
   these two; the plan's out-of-scope list removes everything richer; two
   flavors keep the ladder auditable. Both expose a guard that yields `&mut
   T`, so the protected data needs no interior atomics while locked.
2. **Lock ordering contract: Acquire on lock, Release on unlock.** The
   guard's `&mut T` access plus the release store at unlock gives every
   critical section a clean happens-before edge to the next acquirer; no
   SeqCst and no fences inside the primitives. Rationale: minimal sufficient
   discipline, consistent with every delivered P3 protocol (W02 mailbox,
   W03 transitions, W05 phases); reviewers can hold the line that SeqCst
   appears nowhere at P3 without a recorded design decision.
3. **The lock-order ladder is a named rank table, not a runtime structure.**
   Five named classes (Lifecycle, Allocator, Infrastructure, Statistics,
   Diagnostics) with strictly increasing acquisition along any hold-chain,
   plus leaf rules for the allocator and diagnostics internals. Rationale:
   the guardrail forbids global manager objects — a runtime lock registry
   would be one; a documented table is reviewable by W10 and citable by
   every consumer design.
4. **Allocation while holding a ladder lock is prohibited at rank ≥ 2.**
   The allocator behaves as a leaf: no P3 code may allocate while holding
   any lock except (future) Diagnostics-class locks, and allocator-internal
   locks never call out. Rationale: the P2 allocator's concurrency contract
   is an assumed P2 deliverable that [P3-W10](../p3-w10-smp-safety-audit/README.md)
   audits; W06 must not build rules that *require* allocating under a held
   lock before that audit exists. A consumer that believes it needs
   allocation inside a critical section raises a design change, not a local
   exception.
5. **IRQ-context rules are contracts now, even though P3 does not enable
   new interrupt sources.** The interrupt posture is the P1 baseline's;
   [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md) owns the
   exceptional-path integration. W06 fixes the rules so they are already
   binding: any lock reachable from a context that can run with interrupts
   enabled or from an exception handler must be the irq-save flavor; data
   reachable from handlers is either lock-free (per the atomic policy) or
   protected by an irq-save lock the handler also takes. Rationale: the
   task book requires interrupt-sensitive coverage in P3; deferring the
   flavor would force consumers to rework at the moment interrupts arrive.
6. **Busy-wait is bounded by construction.** Every spin/poll loop states a
   bound constant with recorded rationale (the W02 `POLL_BOUND` pattern);
   waits are forbidden while holding any lock; WFE/SEV-based waiting is
   Reserved to [P3-W07](../p3-w07-cross-cpu-notification/README.md), which
   owns its semantics. Rationale: the plan names busy-wait constraints
   explicitly; an unbounded spin without a recorded design decision is a
   review failure.
7. **The atomic policy codifies delivered protocols instead of overriding
   them.** Patterns are named (single-variable publication, state
   transitions, independent counters, multi-variable publication) with
   fixed orderings; SeqCst requires a recorded design decision (none exists
   at P3); an explicit fence requires named justification (the only one is
   W05's SMP-ready publish fence). Rationale: W02–W05 already fixed these
   protocols; the policy's authority is consistency — a conflict with a
   delivered protocol is raised per the skill's conflict rules, never
   silently absorbed.
8. **Evidence is host-side primitive evidence plus declared limits.**
   Correctness, try-lock, ordering, and concurrent-hammer tests run on the
   host per the P0-W08 baseline; repeated SMP contention runs on QEMU are
   W12/W13's matrix, executed against the limits W06 declares. Rationale:
   W06's plan asks for "correctness and contention evidence within declared
   test limits" at the primitive level; claiming SMP-scale proof here would
   duplicate and preempt W12/W13.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, the assumed-contract failure boundaries, and the scope split.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module split, ownership, the ladder model, and the consumer integration
   map.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   the lock primitives with
   [03](03-code-contracts-lock-primitives.md) (steps 1–3), then the policy
   artifacts and evidence with
   [04](04-code-contracts-atomic-and-ordering-policy.md) (steps 4–6).
4. Record implementation decisions in
   `../p3-w06-concurrency-synchronization-record.md` and evidence in
   `../../verification/p3-w06-concurrency-synchronization-verification.md`
   only when the work is performed. Validation conditions and the handoff
   checklist are in [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No barrier, condition variable, rwlock, mutex-with-blocking, sleeper, or
priority-inheritance type is designed or authorized by W06; no lifecycle
transition (W03), no rendezvous primitive (W05), no notification send or
receive semantics (W07), no TLB transport request format (W08), no
exception-state layout (P1/[P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md)),
and no telemetry event or counter catalog (W11) is designed here. W06 adds
no lock to any W02–W05 delivered surface and defines no consumer data
structure. A `lazy_static`-style global lock registry, a lock-manager object,
or a `lock<T>()` free function that hides which ladder class is involved is
a scope violation to stop at review. Any P4 consumer synchronization is
designed by P4 against this policy through
[P3-W14](../p3-w14-p4-smp-handoff/README.md), not by extending W06 ad hoc.

## Downstream handoff

- **W07** receives the atomic-ordering policy (its mailbox is
  single-variable publication — Release/Acquire), the busy-wait rules (its
  WFE/SEV waiting owns the Reserved trigger, must state its bounds, and
  must forbid waits under locks), and the misuse checklist.
- **W08** receives `SpinLock` for its single-flight initiation lock, the
  ladder (its lock is class `Infrastructure`), the bounded-wait rules for
  completion polling, and the reactive-wait requirement recorded in
  [04 §5](04-code-contracts-atomic-and-ordering-policy.md) (a CPU waiting
  to initiate must remain able to service its own reception slot).
- **W09** receives the Diagnostics-class rules for console serialization on
  exceptional paths and the fatal-path no-lock rule (Diagnostics only,
  bounded try-lock, best-effort fallback).
- **W10** receives the misuse checklist and the ladder as audit criteria:
  every lock-protected P0–P2/P3 item must cite its ladder class and its
  context rules; every atomic item must cite its policy pattern.
- **W11** receives the Statistics class and the Relaxed-counter rule for
  independent counters, plus the contention-observation hooks (wait/hold
  counting is W11's catalog content, not W06's).
- **W12/W13** receive the declared contention test limits and the
  primitive-level evidence they build stress rows on.
- **W14/P4** receive the policy, ladder, and flavors as the P4-facing
  synchronization contract; P4 designs its own state against them and must
  not import W02–W05 protocol internals as if they were general rules.
