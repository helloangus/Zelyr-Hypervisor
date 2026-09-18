# P3-W06 Code Contracts — Atomic Ordering Policy, Ladder, and Rules

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W06 detailed design](README.md).

This file is normative policy. Sections §3–§6 are citable by id
(AP-<n>, LOL-<n>, BW-<n>, MIS-<n>); consumer designs and the
[P3-W10](../p3-w10-smp-safety-audit/README.md) audit reference these ids.
The policy codifies the protocols already established by
[P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)–
[P3-W05](../p3-w05-smp-boot-synchronization/README.md); a conflict with a
delivered protocol is a cross-design conflict to resolve with its owner,
never a silent rewording here.

## 1. Scope of the policy

The policy governs every piece of shared mutable state at P3 that is *not*
protected by a W06 lock flavor: the atomic word per record/state/slot that
W02–W05 deliver, the slot words W07/W08 will add, counters, and phase
words. Its unit of applicability is the *variable*, and every variable's
owning design must be able to name the pattern (§3) it uses.

## 2. Context rules (IRQ/exception sensitivity)

- CR-1: A context is *interrupt-sensitive* if it can run with interrupts
  enabled or if it is an exception handler. At P3 the interrupt posture is
  the P1 baseline's (no new interrupt enablement);
  [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md) owns
  exceptional-path integration. The rules below are binding now so they
  hold unchanged when sources are enabled.
- CR-2: Any lock instance reachable from an interrupt-sensitive context
  must be `InterruptSaveSpinLock<T>`.
- CR-3: Data reachable from an exception handler must be either lock-free
  (per §3) or protected by an irq-save lock that the handler also takes.
  Plain `SpinLock` data must never be touched from a handler.
- CR-4: Code running in an exception handler must not allocate, must not
  wait (no busy-wait loops beyond bounded hardware access), and must not
  acquire any non-Diagnostics lock. Long/heavy work is deferred out of the
  handler per the Coding Guidelines.
- CR-5: The P1-W07 fatal diagnostic path may acquire only the Diagnostics
  lock, only with bounded try-lock, and must fall back to best-effort
  lock-free emission (per
  [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md)'s contract)
  rather than wait on a possibly-never-released lock.

## 3. Atomic-ordering pattern table (AP)

| Id | Pattern | Ordering | Notes and existing users |
|---|---|---|---|
| AP-1 | Single-variable publication (one writer publishes one value; readers consume it) | Release store / Acquire load | W02 arrival mailbox; W05 ready flags; W07 slot word (consumer) |
| AP-2 | State-machine transitions on one word | compare_exchange AcqRel (success) / Acquire (failure load) | W03 lifecycle words; W04 install_state; W08 slot control (consumer) |
| AP-3 | Independent counters (no ordering-dependent decision reads them mid-flight) | Relaxed fetch_add/store | W11 counter block; send/arrival counters where independent |
| AP-4 | Multi-variable coherent publication | Release store *plus* one explicit full barrier before it, with recorded justification | W05 `declare_smp_ready` (the only P3 instance); justification: registry + flags + outcomes coherently published |
| AP-5 | Ownership transfer of a payload via flag exclusivity (lock guard) | lock Acquire / unlock Release | the two lock flavors (§ of [03](03-code-contracts-lock-primitives.md)) |

Rules:

- AP-R1: SeqCst is prohibited unless a recorded design decision names the
  variables and the consistency requirement that demands it. No P3
  variable uses SeqCst.
- AP-R2: An explicit fence may appear only under AP-4 with the
  justification recorded in the owning design. Any other fence is a review
  failure.
- AP-R3: A variable's pattern is fixed for its life; switching patterns is
  a design change of the owning package.
- AP-R4: Relaxed (AP-3) is permitted only where no other variable's
  visibility depends on the counter's ordering and no decision consumes
  two counters as a coherent pair; a pair read as one fact is AP-1/AP-4
  territory, not two Relaxed loads.

## 4. Lock-order ladder (LOL)

Ranks ascend; a hold-chain (acquire while holding) must ascend strictly:

| Rank | Class | What it protects | Leaf rule |
|---|---|---|---|
| 1 | `Lifecycle` | Reserved for any future lock-protected lifecycle surface (W03 needs none — its state words are AP-2 atomics) | — |
| 2 | `Allocator` | P2 allocator entry points if a P3-side wrapper lock ever exists; allocator internals are P2-owned | leaf: never call out while held; allocation while holding any rank ≥ 2 lock is prohibited (README decision 4) |
| 3 | `Infrastructure` | cross-CPU coordination tables (W08's single-flight initiation lock is the first instance); future consumer registries | may acquire Diagnostics while held; nothing above |
| 4 | `Statistics` | counter aggregation, telemetry-side structures (W11) | may acquire Diagnostics while held; must not sample lower classes while held |
| 5 | `Diagnostics` | console/log serialization (normal diagnostics; exceptional paths per CR-5) | leaf: emit only; never acquire any other lock while held |

Rules:

- LOL-R1: Acquisitions along any hold-chain strictly ascend. Acquiring a
  lock of equal or lower rank while holding one is a ladder violation —
  fatal-class per P0-W14 if detected at runtime by corruption, and a
  review failure when detected by inspection.
- LOL-R2: Acquiring two locks of the same class in one hold-chain is
  prohibited (no class re-entry).
- LOL-R3: A new class or a rank change is a W06 design change; a consumer
  design needing one raises it rather than inventing a local ordering.
- LOL-R4: Every lock construction site states its class and the rule that
  keeps its critical section bounded; the census feeds W10.
- LOL-R5: The ladder constrains lock acquisition only; atomic (lock-free)
  protocols are not ladder participants, but code must not hold a lock
  while performing a wait (see BW-R2).

## 5. Busy-wait and waiting rules (BW)

- BW-1: Every busy-wait/poll loop states a bound constant with recorded
  rationale and its failure meaning (the W02 `POLL_BOUND` pattern). The
  bound proves "no progress within the bound", not a wall-clock interval
  (no timer exists before the P6 baseline).
- BW-2: No lock may be held while waiting — for a lock, a notification, a
  completion, or any other CPU's progress. Critical sections are bounded
  by construction.
- BW-3: WFE/SEV-based waiting is owned by
  [P3-W07](../p3-w07-cross-cpu-notification/README.md); other packages
  consume its `wait` surface or keep plain bounded polls. WFE/SEV inside
  the lock flavors is prohibited (Reserved; trigger recorded in the entry
  README).
- BW-4 (reactive-wait rule): A CPU that owes reception duties (it is a
  valid target of cross-CPU requests) must remain able to service them
  while it waits for anything. A wait loop that would make the CPU
  unresponsive as a target is prohibited; the owning design interleaves
  its reception consumption into the wait (the pattern W08's initiation
  wait uses). This rule is what makes W08's single-flight wait deadlock-free.
- BW-5: Unbounded spinning requires a recorded design decision naming why
  no bound is knowable; "it should finish quickly" is not a rationale.
  (W07's idle-context `wait` is the sanctioned P3 instance — it is an
  idle terminal state, not a wait for progress, and W06 records it here as
  the cross-referenced exception.)

## 6. Misuse checklist (MIS)

Review-checkable; every item is a review failure or a W10 audit finding:

- MIS-1: Shared mutable state has no declared pattern/class (orphan state).
- MIS-2: A private lock, second flag word, or "uncontended fast path"
  bypassing the flavors.
- MIS-3: Ladder violation (LOL-R1/R2) at a construction or call site.
- MIS-4: Allocation while holding any rank ≥ 2 lock.
- MIS-5: A wait inside a critical section (BW-2), or a non-reactive wait
  on a CPU with reception duties (BW-4).
- MIS-6: Plain-flavor lock reachable from an interrupt-sensitive context
  (CR-2/CR-3); handler code allocating or taking non-Diagnostics locks
  (CR-4).
- MIS-7: SeqCst or an unjustified fence (AP-R1/R2); a pattern switch
  without a design change (AP-R3); incoherent counter-pair reads (AP-R4).
- MIS-8: A guard stored in a long-lived structure, leaked, or a payload
  reference escaping the guard.
- MIS-9: Unbounded spin without a recorded decision (BW-5); a poll loop
  without a bound constant (BW-1).
- MIS-10: Lock recursion (same instance or same class re-entered on one
  CPU).
- MIS-11: Fatal-path code acquiring any non-Diagnostics lock or waiting
  unboundedly (CR-5).
- MIS-12: `unsafe` in a primitive outside the audited flag/payload boundary
  or without a SAFETY comment per the P0-W10 governance.

## 7. Contention evidence limits (declared)

W06 declares the limits within which its primitive-level evidence is
gathered and which W12/W13 build on:

- L-1: Host-side concurrent hammer: N "CPUs" (host threads) per lock,
  N ∈ {2, 4, 8, 32}; each performs K acquire/mutate/release rounds
  (K = 10_000) against a shared counter; passing condition: exact final
  counter, no lost update, bounded test wall-time (recorded), no deadlock
  detected by the harness's global timeout.
- L-2: try_lock contention: concurrent hammer with try_lock; passing
  condition: successes + failures accounted exactly, no corruption.
- L-3: Mixed-flavor run: irq-save flavor exercised with simulated
  mask/restore hooks (host) proving save/restore pairing under contention.
- L-4: Ladder model check: a host-side harness models the rank table and
  asserts the delivered construction-site census is acyclic and
  rule-conformant (no runtime tracking — Reserved).
- Limits statement: these prove the primitives under the modeled memory
  model of the host; they do not prove AArch64 SMP behavior on QEMU or
  hardware — that is W12/W13's declared territory, using these limits as
  the base layer.
