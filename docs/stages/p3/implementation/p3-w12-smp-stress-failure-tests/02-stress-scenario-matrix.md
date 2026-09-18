# P3-W12 Stress and Failure Scenario Matrix

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W12 detailed design](README.md).

## 1. How to read a scenario row

Every scenario S1–S7 is specified with fixed fields:

- **Mechanism owners** — the P3/P2 designs whose published surfaces are
  exercised (never modified).
- **Stimulus** — what the harness does, as a fixed schedule; parameters
  (CPU count N, per-CPU iterations K, rounds R) are declared constants
  recorded in the implementation record with rationale
  ([03](03-failure-injection-and-accounting.md) §4).
- **Preconditions** — boot state required (SMP-ready per
  [P3-W05](../p3-w05-smp-boot-synchronization/README.md), online set from
  [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)).
- **Expected observable** — counters/events/dumps/state the run must
  produce ([P3-W11](../p3-w11-smp-observability/README.md) surfaces).
- **Pass condition** — the accounting identities and invariants that
  hold, within declared bounds; never a timing value.
- **Proves / does not prove** — the honest claim boundary, inherited by
  every [P3-W13](../p3-w13-qemu-smp-regression/README.md) matrix row that
  reuses the scenario.
- **Repetition** — what "repeat" means for the scenario (in-boot rounds,
  cold boots, or host-test re-runs).
- **Environment** — host-side (fakes) or QEMU (real cross-CPU progress),
  per the P0-W08/P0-W09 assumed contracts.

Scenario execution order in a QEMU session: S4 (rendezvous is the boot
itself) precedes everything; S1/S2/S3/S7 then run in a fixed order; S5/S6
run last because they perturb or consume the failed/invalid-target state.

## 2. Scenario definitions

### S1 — concurrent allocate/free (allocator stress)

- **Owners:** [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md),
  [P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md) surfaces;
  protection semantics per
  [P3-W06](../p3-w06-concurrency-synchronization/README.md).
- **Stimulus:** all N online CPUs concurrently allocate, touch (write a
  per-CPU pattern), and free page and small-object allocations in a fixed
  K-iteration loop; a bounded subset of allocations is intentionally
  retained and freed at the end (leak-detection variant).
- **Preconditions:** SMP-ready; online set = N; allocator ownership
  metadata queryable.
- **Expected observable:** allocator accounting before/after; W11
  contention counters/events; final SMP-ready dump.
- **Pass condition:** accounting balances (allocated == freed, including
  the retained-set release); no ownership-metadata violation; W04
  reserved-region fill patterns intact (no stray write); no hang within
  the declared iteration bound; all CPUs still online per the registry.
- **Proves:** the allocator's shared-state protection holds under declared
  concurrent pressure on the reference platform. **Does not prove:** all
  interleavings (bounded pattern), performance, hardware behavior, or
  correctness beyond the exercised allocation sizes/orders.
- **Repetition:** K iterations in-boot; R in-boot rounds; repeated across
  cold boots only as part of W13's matrix rows.
- **Environment:** QEMU (real concurrency required).

### S2 — notification storm

- **Owners:** [P3-W07](../p3-w07-cross-cpu-notification/README.md) surface;
  reception slots per
  [P3-W04](../p3-w04-per-cpu-runtime/README.md).
- **Stimulus:** every online CPU sends K notifications in a fixed
  round-robin target schedule covering all online CPUs including
  self-targets; senders quiesce before accounting (per-W11 obligation).
- **Preconditions:** SMP-ready; notification primitive initialized.
- **Expected observable:** W11 `notifications_sent`/`notifications_received`
  per CPU; notification events (when enabled); reception accounting per
  W07's semantics.
- **Pass condition:** per-target sent == received across the online set
  (quiesced); no duplicated or lost arrival per W07's stated exactly/at-
  least-once semantics; no deadlock or lost wakeup within the bound; all
  CPUs online after the storm.
- **Proves:** the notification primitive sustains declared storm rates
  with intact accounting. **Does not prove:** RPC-style delivery
  guarantees beyond W07's contract, real IPI latency, hardware interrupt
  behavior.
- **Repetition:** K sends × R rounds in-boot; matrix re-use per W13.
- **Environment:** QEMU.

### S3 — atomic and shared-counter stress

- **Owners:** [P3-W06](../p3-w06-concurrency-synchronization/README.md)
  primitives; W11 contention seams.
- **Stimulus:** all online CPUs perform K fetch-add operations on shared
  atomic counters and K lock-protected read-modify-write operations on a
  declared shared structure, in fixed schedules with deliberate
  lock-order-safe overlapping.
- **Preconditions:** SMP-ready; W06 lock-order rules published.
- **Expected observable:** final counter values; contention counters;
  lock-order audit check result.
- **Pass condition:** every shared counter equals its expected total
  exactly; no lock-order violation flagged by W06's checks; no livelock
  within the bound (progress counter advances).
- **Proves:** the synchronization primitives preserve updates under
  declared concurrency. **Does not prove:** all memory-ordering properties
  beyond the exercised patterns, fairness, or performance.
- **Repetition:** K × R in-boot; host-side variants with fakes re-run per
  host-test invocation.
- **Environment:** host-side (with fakes where logic separates) and QEMU.

### S4 — repeated rendezvous (boot-order stress)

- **Owners:** [P3-W05](../p3-w05-smp-boot-synchronization/README.md);
  registry per [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md).
- **Stimulus:** repeated cold boots of the same declared configuration;
  every boot's phase sequence, once-only invariants, ready counting, and
  `SmpReadyState` captured.
- **Preconditions:** none beyond the declared boot configuration.
- **Expected observable:** per boot — `boot.phase` sequence, rendezvous
  result, `boot.smp_ready` state, SMP-ready counter dump; across boots —
  identical phase ordering and consistent online-set accounting.
- **Pass condition:** every boot reaches `Ready` or a `Degraded` state
  with complete failed-set accounting; once-only transitions never fire
  twice; per-CPU ready signals appear exactly once per CPU per boot; no
  boot hangs within the rendezvous terminal-condition bound (W05's
  structural guarantee, verified not assumed).
- **Proves:** boot ordering and readiness accounting are repeatable, not
  a one-boot accident. **Does not prove:** timing stability (values are
  informative only), behavior under failing firmware, hardware boots.
- **Repetition:** R cold boots per configuration (R fixed with W13's
  repetition policy).
- **Environment:** QEMU.

### S5 — secondary timeout and failed start

- **Owners:** [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) outcome
  model; registry transitions
  ([P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)); degraded
  accounting ([P3-W05](../p3-w05-smp-boot-synchronization/README.md)).
- **Stimulus:** two parts. (a) Host-side: watcher tests drive the bounded
  poll with a fake mailbox that never arrives, arrives late, and arrives
  with a bad result. (b) QEMU: the W02 induced failure input — CPU_ON
  targeting an identity outside the inventory.
- **Preconditions:** (a) none; (b) declared boot configuration plus the
  induced input.
- **Expected observable:** (a) watcher terminal outcomes per fake case;
  (b) per-CPU failure outcome with phase attribution, `Failed` registry
  state, `Degraded` SMP-ready state with the failed set, quarantine of
  the failed CPU's provisional stack per W02.
- **Pass condition:** every induced failure yields a terminal,
  phase-attributed outcome; failed CPUs stay outside the online set;
  boot continues (reference default) with complete diagnostics; no hang,
  no crash.
- **Proves:** the failure path terminates, attributes, and degrades
  accountably. **Does not prove:** true device-level timeout behavior
  (not inducible at P3 — recorded limitation), recovery/retry (reserved
  out of scope), hardware failures.
- **Repetition:** host-side watcher cases re-run per invocation; QEMU
  induced-failure row per cold boot in the matrix.
- **Environment:** host-side (a) and QEMU (b).

### S6 — invalid and offline targets

- **Owners:** W07/W08 targeting surfaces and their refusal semantics;
  registry `OnlineSet` and `Eligibility`
  ([P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)); W02 start
  targeting.
- **Stimulus:** from an online CPU, issue declared invalid-target calls —
  notification and TLB-transport requests targeting an unknown identity,
  a `Failed` CPU, an excluded-class CPU, and (as the offline stand-in —
  runtime offline is Reserved in W03) a never-started excluded CPU; plus
  a CPU_ON retry of a `Failed` CPU (must be refused per W02's no-restart
  rule).
- **Preconditions:** SMP-ready; at least one failed or excluded CPU
  exists (produced by S5's induced failure or an excluded-class entry in
  the declared topology).
- **Expected observable:** refusal outcomes per W07/W08/W02 semantics;
  refusal events; unchanged registry and online set; unchanged target
  CPU state.
- **Pass condition:** every invalid/offline/failed target is refused
  structurally (untargetable) or by a defined refusal with a diagnostic;
  no crash, no state mutation of the target, no delivery; counters count
  the attempts per the W11 counting rules.
- **Proves:** the targeting-universe boundary holds under abuse.
  **Does not prove:** hotplug/offline semantics (Reserved), guest-initiated
  targeting (P4+), hardware fault behavior.
- **Repetition:** K rounds in-boot; matrix row per cold boot.
- **Environment:** QEMU (host-side for the pure-targeting logic where
  fakes suffice).

### S7 — concurrent logging

- **Owners:** the P0-W12 diagnostic channel contracts
  ([P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)) as
  implemented; W11 events/dumps as content.
- **Stimulus:** all online CPUs emit a fixed per-CPU log/trace pattern at
  mixed declared levels concurrently for K rounds; one run additionally
  exercises the fatal-diagnostic path from a non-boot CPU via the
  declared W09/P1-W07 seam.
- **Preconditions:** SMP-ready; channel initialized per P0-W12.
- **Expected observable:** channel output carrying per-CPU attribution
  for every line; the fatal-path run renders complete crash-context
  content per the P0-W12 minimum.
- **Pass condition:** every emitted line is attributable to its emitting
  CPU (no attribution loss under concurrency); no channel deadlock or
  hang within the bound; the fatal-path run produces the full minimum
  crash content; output volume matches the declared pattern counts
  (subject to the channel's declared filtering/dropping semantics, which
  the evidence must cite — a channel that drops under load does so by its
  own declared rule, and the scenario then asserts the rule, not
  losslessness).
- **Proves:** diagnostics remain usable and attributable under concurrent
  load. **Does not prove:** log-ordering total order across CPUs, output
  completeness beyond the channel's declared semantics, crash-path
  correctness beyond the exercised seam.
- **Repetition:** K rounds in-boot; fatal-path run once per session;
  matrix rows per cold boot.
- **Environment:** QEMU (channel behavior under real concurrency).

## 3. Traceability

| Scenario | Plan scope item | Task-book validation |
|---|---|---|
| S1 | Concurrent allocate/free | P3-V12 (also P3-V13 allocator row) |
| S2 | Notification storms | P3-V12 (with P3-V07 context) |
| S3 | Atomic/shared-counter stress | P3-V12 (with P3-V06 context) |
| S4 | Repeated rendezvous | P3-V12 (with P3-V05) |
| S5 | Secondary timeout | P3-V12 (with P3-V02 failure context) |
| S6 | Invalid/offline targets | P3-V12 (with P3-V07/V08 context) |
| S7 | Concurrent logging | P3-V12 (with P3-V09/V11 context) |

Every plan scope item maps to exactly one scenario; every scenario maps
to at least one task-book validation ID. No scenario asserts guest,
vCPU, Stage-2, or GIC behavior — the review obligation is W12-DV04.
