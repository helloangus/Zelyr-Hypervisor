# P3-W08 Timeout ownership remediation

Chinese readers can use the [Chinese edition](07-timeout-ownership-remediation.zh-CN.md).

**Status:** Option A direction selected by owner on 2026-10-02; coordinated design correction recorded. Final implementation admission and execution evidence remain pending.
**Scope:** AUD-002 slot reuse, receiver lifetime, timeout and downstream acceptance.
**Version:** v0.1
**Owner/change context:** Owner-directed documentation remediation, 2026-10-02.
**Supersedes:** Pending supersession and unconditional timeout recovery in this package.
**Parent:** [W08 design](README.md); [plan](../../plans/p3-w08-tlb-shootdown-transport.md).

## 1. Baseline and required foundations

Current production source has no P3 transport. P3-V08 requires diagnosable
targeting, completion, concurrency and timeout, not cancellation or guaranteed
recovery of a stalled CPU. This amendment stays within that delegated transport
scope; TransportNoop remains the only P3 bound operation.

| Outcome | Observed design gap | Required foundation / owner | Acceptance |
|---|---|---|---|
| Correct request consumption | Target can load old Pending, then new descriptor after timeout reuse | W08 retains Pending descriptor until target completion | DV06 pause at control read, descriptor read and operation completion |
| Side-effect-free refusal | Publication loop has fallible lookup/notify after earlier writes | W08 preflights all slots before any descriptor write; impossible commit faults are terminal | DV04/DV06 mixed ready/busy targets preserve all publication state |
| Safe reuse | Sequence equality is treated as cancellation authority | W08 permits only Empty/Completed reuse under serialized initiation | DV01/DV06 delayed target and sequence-wrap cases |
| Legal synchronization | W08 holds SpinLock during collection; W06 BW-2 forbids waiting under any lock | Owner-selected A: W06 BW-6 admission without held data-lock guard | DV05 specifies contention/refusal, release and terminal retention; execution pending |

Required: no Pending overwrite, all-target preflight, sampled timeout accounting,
single non-reentrant receiver per CPU, and explicit completion/resource lifetime.
Reserved: cancellation, forced reset, hotplug and multi-request pipelining.
Out of scope: TLBI selection, VMID/IPA semantics and production P3 code.

## 2. Corrected ownership and timeout contract

A serialized initiator may write a descriptor only after an acquire observation
of Empty or Completed for that slot. It then release-publishes Pending with the
new sequence. The target's acquire load of Pending observes the descriptor.
Only the owning CPU consumes it; consumption is non-reentrant (including the
bound operation), and P6 interrupt consumption requires a new reviewed protocol.

Pending is immutable to initiators, including after collection times out.
The receiver may already be between its control and descriptor loads, or may
have executed part of its bound operation. Timeout is neither cancellation,
quiescence, CPU-offline authority nor permission to release referenced resources.
The target finishes all descriptor/resource accesses and its operation before
release-publishing Completed. It never accesses the old descriptor/resources
after that publication; subsequent target-private diagnostics use local values.

Before publishing any target, initiation inspects every effective target. If
any is Pending, return `TargetsBusy { busy, excluded }` without changing any
descriptor, control word or request sequence. The busy mask is an observation
during preflight, not an atomic snapshot; a concurrent completion may make a
retry succeed. Do not wait indefinitely for busy targets or silently remove
them from the requested set. Independent targets remain usable. Refusal changes
no new-request publication state; independent old reception and refusal counters
may still progress, so this is not a frozen global-state snapshot.

An Empty/Completed preflight observation remains stable until commit under
serialized initiation: targets write only Pending→Completed. Save each exact
control value and CAS that value to Pending; arbitrary Empty wildcards are
invalid. Resolve area references, encoding, targeting and notification
prerequisites before any write. A publication CAS failure or impossible W07
failure after publication is terminal and retains slots/resources, never an
ordinary side-effect-free Err or a partial rollback.

Collection latches same-sequence Completed observations into acked; unacked is
effective minus acked when the bound expires. Late completions are valid and
may occur before the caller receives TimedOut. Results describe observations,
not cancellation or a simultaneous snapshot. A later request may reuse a slot
only after acquire-observing Completed. Initiators never clear a Pending slot.

The existing u16 sequence remains an internal equality tag, not a globally
unique operation ID or resource lease. Wrapping is safe only because no slot
is reused while an old receiver can access its descriptor, and serialized
collection prevents another initiator overwriting a current completion.
Different slots can retain older Pending sequences across wrap; Pending always
causes TargetsBusy even when its tag equals the new tag. External tokens or
asynchronous completion waiters require a new identity/lifetime design.

## 3. Static interleaving review and acceptance

Original counterexample: target reads Pending(A); A times out; initiator writes
descriptor B; target reads B under sequence A, then completes the wrong operation
or fails its completion CAS. An atomic descriptor alone does not prevent this.

Corrected cases (design review only, not executed tests):

- Target pauses before descriptor read: preflight observes Pending and refuses
  B without writes; target resumes with A's descriptor.
- Target pauses during operation: the same refusal holds; timeout cannot reclaim
  resources. Completion is published only after the final access.
- Target completes during preflight: the initiator either refuses conservatively,
  or observes Completed and may reuse. Both paths preserve pairing.
- Mixed mask contains ready and Pending targets: inspect all first; no ready
  target is published before the refusal.
- A target never resumes: its slot remains occupied; unrelated target sets work,
  and requests including it receive busy diagnostics. Recovery is not promised.
- Duplicate wake: serialized non-reentrant target consumption sees Completed or
  a new Pending; it cannot consume the old request twice.

DV06 must execute these schedules against the actual future implementation,
including timeout followed by late completion, sequence wrap and a target
stalled for more than one sequence cycle. DV03 must verify publication and
completion visibility. DV05 must cover serialization and receiver reentrancy.
No Host/AArch64 execution is claimed by this static review.

## 4. W08-SYNC: separate cross-package design conflict

Classification: Architecture Change Request at the established module-contract
boundary; no change to an accepted ADR is proposed or selected.

[W06 BW-2](../p3-w06-concurrency-synchronization/04-code-contracts-atomic-and-ordering-policy.md#5-busy-wait-and-waiting-rules-bw)
forbids holding a lock while waiting for completion. The superseded W08 proposal required
holding its initiation SpinLock across collection. BW-4 reactive consumption
does not grant an exception to BW-2. It also contradicts W08's claim that the
bound operation always runs without a held lock. A finite collection count
does not prove bounded lock acquisition, scheduling or wall-clock progress.

The owner selected A on 2026-10-02 ("就按A来"). W06 BW-6 and W08 now
specify single-attempt admission without a held data-lock guard. The original
SpinLock realization above is superseded; BW-2 is unchanged. This resolves the
design-policy choice, not final implementation admission or executed DV05 proof.

## 5. Consumer and completion boundary

W11 distinguishes busy refusals, timeout observations and eventual target
completions; aggregates reconcile at declared quiescence, not at timeout.
W12/DV06 must not reset Pending to clean up a failed test. W13 carries the
same scenario requirements and blocked/not-run status. W14's R7 must convey
that timeout is not completion or reclamation authority. Any future P4 binding
must retain descriptor-referenced objects and translation resources until its
own completion/quiescence contract permits release; P3's opaque u64 does not
supply that lease. P4's current-pCPU-only baseline is unchanged.

AUD-002's dangerous proposed Pending supersession is removed. Overall finding
closure still requires approved coherent W06/W08 contracts, implemented tests
and owning-stage evidence. No production code, unsafe, dependency or public
ABI is added by this amendment.

## 6. Decision record: W08-SYNC (A selected)

Owner-selected option A replaces the initiation SpinLock with a fail-fast protocol
admission state. This is an explicit coordinated change to the W06/W08 boundary,
not renaming a lock to bypass BW-2. It grants no protected-data borrow and no
waiter spins for it; receivers never acquire it. W06 BW-6 explicitly records its
protocol role and constraints, while preserving the ban on waiting with a
SpinLock or other data-lock guard held.

Selected state: Idle → Active(initiator CPU) → Idle. One atomic compare-exchange
attempt admits a call; contention returns TransportBusy without publishing or
advancing the sequence. A private non-copyable call token identifies the sole
initiator; it is not an external completion token. Only its owner may release
admission. Acquire on successful admission and release on return order initiator
sequence/state access; receiver slot publication/completion retain their own
release/acquire ordering. No new request can observe or alter intermediate
initiator state.

With admission held, preflight every target before publication. TargetsBusy
releases admission without new-request writes. Success or TimedOut first copies
the observed result into caller-owned data, then releases admission; subsequent
calls cannot change that returned snapshot. TimedOut leaves Pending slots and
referenced resources intact. Impossible post-publication faults remain terminal,
retaining slots/resources and admission; do not run a cleanup path that exposes
a partially published call as reusable. Ordinary return uses a checked release,
not cancellation or forced reset.

Collection is bounded and holds no SpinLock/data-lock guard. The caller must
enter with no held locks; collection still services its own reception duties.
A TransportBusy caller gets no fairness or automatic retry guarantee. Any retry
loop belongs to its owner, is bounded, and services receiver duties per BW-4;
no hidden spin-until-admitted wrapper is permitted. The P3 bound operation stays
non-reentrant TransportNoop. Later real operations need their own reviewed
bounded context. A halted initiator may keep admission unavailable; this is
diagnosed, not recovered by forcibly stealing its token.

Consequences and acceptance for option A:

- Concurrent calls change from waiting/serialization to one admitted call and
  explicit busy refusals. Published calls still serialize; slot safety is intact.
- W08 errors, pseudocode, workflow and DV05; W06 BW rules; W11 counters;
  W12/W13 contention tests; W14 consumer retry semantics must change together.
- DV05 must pause an admitted initiator at preflight/publication/collection,
  prove competitors promptly refuse without writes, verify reception remains
  possible, and exercise release after success, refusal and timeout.
- Review terminal partial-publication behavior, no stealing/reset, and no lock
  guards throughout collection. This packet is not implementation evidence.

Alternative B retains the existing SpinLock and defines a narrow W06 BW-2
exception specifically for W08. It additionally needs a reviewed bounded
acquisition mechanism (the current generic lock supplies none), reception and
non-reentrancy rules under a held guard, lock-order analysis and a revised bound-
operation call context. It preserves waiting-style callers but expands the
generic synchronization policy and adds more proof obligations; not recommended.

Owner decision on 2026-10-02: **A**, authorized by "就按A来". B is rejected.
This authorizes the coordinated detailed contract and validation-plan revision.
Production coding still requires the resulting design's approval and actual
P3 prerequisites. This does not approve
W04-MAP, W11, S2-MULTIVM-SCOPE or other outstanding remediation decisions.
