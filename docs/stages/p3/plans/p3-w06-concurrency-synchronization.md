# P3-W06 — Concurrency synchronization

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Establish reviewed synchronization semantics sufficient for shared Host SMP state and interrupt-sensitive execution.

## Scope

Normal mutual exclusion, IRQ-sensitive critical sections, atomic state transfer with acquire/release expectations, busy-wait constraints, non-sleeping contexts, misuse constraints, and baseline lock ordering for allocator, registry, statistics, and future consumers.

## Out of scope

Blocking scheduler locks, condition variables, complete rwlock families, priority inheritance, realtime locking, and detailed algorithms or Rust APIs.

## Work sequence

1. Inspect W04/W05 and P0 unsafe/diagnostic governance.
2. Define required synchronization contexts, ordering rules, and prohibited misuse.
3. Integrate the rules with notification, TLB transport, exception, and shared-infrastructure consumers.
4. Review mutable-state ownership and IRQ constraints against ADR safety and layering requirements.
5. Collect correctness and contention evidence within declared test limits.
6. Record lock-order and atomic-ordering handoff rules.

## Acceptance and closure

P3-V06 passes when reviewed shared-state access follows declared atomic/lock/IRQ rules and contention evidence has no corruption or unexplained deadlock.

## Handoff

W07–W10, W12–W15, and P4 may rely on the documented semantics, not on unreviewed primitive internals.
