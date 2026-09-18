# P3-W06 Code Contracts — Lock Primitives

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W06 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths, the exact flag encoding, and the CAS
instruction sequence are fixed in the implementation record under the
approved build design. Every instance carries its ladder-class tag at
construction; the tag is documentation-grade metadata (a constructor
argument), not a runtime registry entry.

## 1. `LadderClass`

```text
Name and stability: LadderClass — enum { Lifecycle, Allocator,
    Infrastructure, Statistics, Diagnostics }; internal; rank order equals
    declaration order (Lifecycle lowest, Diagnostics highest).
Purpose and caller: names the lock-order ladder class of a lock instance;
    every lock constructor requires one; reviewers and the W10 audit read
    it from the construction site.
Preconditions / postconditions: value fixed for the life of the lock.
Concurrency/allocation context: plain value; no atomics.
Errors and failure guarantee: n/a.
Security/authorization checks: n/a.
Logic: plain enum; ranks are defined in
    [04 §4](04-code-contracts-atomic-and-ordering-policy.md).
Validation: W06-DV05 (ladder conformance review reads construction sites).
```

## 2. `SpinLock<T>`

```text
Name and stability: SpinLock<T> — internal; one instance per protected
    shared object; lifetime equals the data's lifetime; no Drop semantics
    beyond the payload's.
Purpose and caller: thread-context mutual exclusion for shared state that
    is never reachable from an interrupt-enabled or exception-handler
    context; constructed and used by the consumer design that owns the
    data (W08's initiation lock is the first P3 consumer).
Inputs / outputs: new(data: T, class: LadderClass) -> Self;
    lock(&self) -> SpinLockGuard<'_, T>;
    try_lock(&self) -> Option<SpinLockGuard<'_, T>>;
    (read-only introspection, e.g. is_locked(), is validation-only and
    diagnostic-grade, never a synchronization input).
Preconditions / postconditions: lock() — on return the caller holds the
    lock and may access T exclusively; unlock happens when the guard drops.
    try_lock() — Some(guard) or None; never blocks; no mutation on None.
State and ownership change: Unlocked -> Locked for the duration of the
    guard; payload ownership (exclusive access) transfers to the guard.
Concurrency/allocation context: no allocation inside lock/try_lock/unlock;
    the acquire loop uses Acquire semantics on success; unlock is a
    Release store. Callers must satisfy the busy-wait and misuse rules
    ([04 §5–§6](04-code-contracts-atomic-and-ordering-policy.md)).
Errors and failure guarantee: no error paths; a corrupted lock word is a
    fatal diagnostic (P0-W14 class), never a returned error.
Security/authorization checks: n/a (not guest-reachable).
Logic (pseudocode):

    lock(self):
        loop:
            if self.flag.compare_exchange(UNLOCKED, LOCKED, Acquire,
                                          Relaxed).is_ok(): break
            spin_hint()                    # subject to busy-wait rules
        return Guard { lock: self }        # &mut T reachable via guard

    unlock(guard):
        guard.lock.flag.store(UNLOCKED, Release)
        forget(guard)                      # drop glue skips the flag

Validation: W06-DV01 (unit: mutual exclusion under concurrent hammering,
    try-lock contention, unlock-reacquire ordering), W06-DV04 (ordering
    consistency with the atomic policy).
```

## 3. `InterruptSaveSpinLock<T>`

```text
Name and stability: InterruptSaveSpinLock<T> — internal; the irq-save
    flavor required by the ADR P3 roadmap; constructed like SpinLock.
Purpose and caller: mutual exclusion in interrupt-sensitive contexts —
    any lock reachable from a context that can run with interrupts enabled
    or from an exception handler; the handler takes the same flavor, whose
    masking makes same-CPU re-entry impossible.
Inputs / outputs: same surface as SpinLock<T> (new/lock/try_lock with
    InterruptSaveGuard).
Preconditions / postconditions: lock() — the interrupt mask (per the P1
    baseline DAIF discipline) is saved and masked before acquisition and
    restored after release; the guard carries the saved value.
State and ownership change: as SpinLock, plus saved-mask bookkeeping in
    the guard.
Concurrency/allocation context: as SpinLock; restore happens after the
    Release store of the unlock (architecture §3) so an interrupt arriving
    during unlock cannot re-enter on the same CPU.
Errors and failure guarantee: as SpinLock; a nested acquisition of the
    same instance on one CPU deadlocks by construction — the misuse rules
    make same-CPU recursion a review failure, and the fatal diagnostic
    path never takes non-Diagnostics locks ([04 §6](04-code-contracts-atomic-and-ordering-policy.md)).
Security/authorization checks: n/a.
Logic (pseudocode):

    lock(self):
        saved = read_and_mask_interrupts()   # unsafe: arch boundary
                                             # (P1 DAIF discipline)
        acquire flag (Acquire)               # as §2
        return InterruptSaveGuard { lock, saved }

    unlock(guard):
        guard.lock.flag.store(UNLOCKED, Release)
        restore_interrupts(guard.saved)      # unsafe: arch boundary

Validation: W06-DV02 (unit: mask-save/restore pairing, same-CPU re-entry
    prevented by masking, acquire/release ordering), W06-DV03 (context
    rule review over construction sites).
```

## 4. Guards

```text
Name and stability: SpinLockGuard<'a, T> / InterruptSaveGuard<'a, T> —
    internal; created only by their lock's lock()/try_lock(); dropped only
    by scope exit (unlock in Drop).
Purpose and caller: the sole accessor for the payload; callers
    dereference via Deref/DerefMut.
Preconditions / postconditions: while live, the holder has exclusive
    access to T; on drop, unlock runs exactly once (plain flavor) or
    unlock + interrupt restore (irq-save flavor).
State and ownership change: see the flavors.
Concurrency/allocation context: guards must not be stored in structures
    that outlive the critical section, passed across a wait, or leaked
    (mem::forget on a guard is a correctness bug class named in the misuse
    checklist).
Errors and failure guarantee: Drop cannot fail.
Security/authorization checks: n/a.
Logic: newtype over &'a Lock<T> plus (irq-save only) the saved mask.
Validation: covered by the flavor tests (W06-DV01/DV02); guard-escape is
    structurally prevented by lifetimes and reviewed per the checklist.
```

## 5. Construction and instance rules

- A lock is declared in the same design that owns the protected data, with
  its `LadderClass` stated and justified in that design; W06 defines no
  instances itself except the reference consumers named in the integration
  map.
- The protected data and the lock are one unit (no separate flag spinning
  on a remote object); no `SpinLock<()>`-style "lock somewhere else"
  arrangements unless the owning design records why the payload cannot
  live with the flag.
- No runtime creation/destruction of locks in P3; new instances appear
  only through consumer designs.
- Validation: W06-DV05 reviews every construction site for class,
  context, and checklist conformance; the census is a W10 audit input.
