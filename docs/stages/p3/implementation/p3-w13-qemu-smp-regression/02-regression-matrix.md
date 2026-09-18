# P3-W13 Regression Matrix

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W13 detailed design](README.md).

## 1. Row model

Every row is: **row id → CPU counts → source scenarios/assertions →
repetition → capture set → pass condition → proves / does not prove**.
Assertions cite the owning designs verbatim; a row fails when any cited
assertion breaks, is unobservable, or the run hits a declared bound.
Pass conditions never consume timing values.

Count axis: exactly `c ∈ {1, 2, 4, 8}` — the declared task-book scope and
W01's inventory bound. `c = 1` is not a degenerate row: it asserts the
single-CPU boot path stays correct under the same contract (no CPU_ON
issued, rendezvous trivially completes, matrix machinery identical).

## 2. R-series rows

### R1 — boot and SMP readiness (per count c)

- **Assertions:** [P3-W05](../p3-w05-smp-boot-synchronization/README.md)
  phase sequence `Bootstrap → GlobalInitPublished → SmpReady` exactly
  once each; `SmpReadyState` is `Ready` (or `Degraded` only when a failure
  row legitimately produced it); rendezvous result accounting
  `ready + failed == attempted`; [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)
  outcomes terminal for every attempted secondary.
- **Capture:** boot log, phase/rendezvous events, SMP-ready result, W03
  registry state dump.
- **Pass condition:** all assertions hold within the rendezvous terminal
  bound; no boot hang; diagnostics complete on any degraded outcome.
- **Proves:** repeatable ordered boot to a declared readiness state at
  count c on the reference platform. **Does not prove:** boot timing
  quality, firmware-failure behavior, hardware bring-up.

### R2 — online-set correctness (per count c)

- **Assertions:** [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)
  online set has exactly `c` members (all declared present CPUs online);
  no duplicate admission diagnostics fired; eligibility gates answered
  `NotEligible` for nothing left usable-but-unaccounted.
- **Capture:** registry state dump; online-set snapshot; admission event
  stream.
- **Pass condition:** online-set equality and admission accounting exact.
- **Proves:** lifecycle authority maintains its invariants across counts.
  **Does not prove:** hotplug/offline semantics (Reserved).

### R3 — per-CPU isolation and identity (per count c)

- **Assertions:** [P3-W04](../p3-w04-per-cpu-runtime/README.md) isolation
  diagnostics: distinct area and stack addresses per online CPU; header
  identity matches the declared topology (W01 mapping); `current()`-based
  attribution correct on every CPU (via
  [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md)'s
  attributable diagnostics where present).
- **Capture:** isolation diagnostic block from each CPU.
- **Pass condition:** all c CPUs render correct, distinct local state.
- **Proves:** per-CPU foundations hold at count c. **Does not prove:**
  stack-overflow safety (guard pages Reserved), exception correctness
  under real interrupt load (P3-W09's scope).

### R4 — observability and attribution (per count c)

- **Assertions:** [P3-W11](../p3-w11-smp-observability/README.md)
  SMP-ready counter dump renders c attributed rows; lifecycle,
  rendezvous, and (where exercised) notification/transport events present
  with attribution; no platform-name content.
- **Capture:** the dump and event stream.
- **Pass condition:** attribution complete; counters plausible against R1
  accounting (e.g. `start_requests_sent == c − 1`).
- **Proves:** observability is live and CPU-attributed at count c.
  **Does not prove:** performance; event-stream completeness for trimmed
  classes.

### R5 — stress deployment (per count c; scenarios per §5)

- **Assertions:** [P3-W12](../p3-w12-smp-stress-failure-tests/README.md)
  S1/S2/S3/S7 pass conditions at the depths declared in §5, at count c.
- **Capture:** per-scenario evidence per W12's template.
- **Pass condition:** every deployed scenario `run-passed` within its
  declared limits.
- **Proves:** the stressed mechanisms hold under declared concurrent
  pressure at count c. **Does not prove:** exhaustion of interleavings,
  performance, soak behavior, hardware.

### R6 — cross-CPU event regression (per count c ≥ 2)

- **Assertions:** S2 notification-storm accounting (sent == received,
  quiesced) plus, where [P3-W08](../p3-w08-tlb-shootdown-transport/README.md)
  delivers its transport in P3 scope, its request/completion accounting
  at count c; self-target and all-target patterns included.
- **Capture:** W11 notification/transport counters and events.
- **Pass condition:** exact accounting at count c; no deadlock within
  bounds.
- **Proves:** the cross-CPU event path sustains declared traffic between
  c CPUs. **Does not prove:** real IPI latency, GIC delivery semantics
  (P6), Stage-2 TLB invalidation semantics (P4+).

### R7 — allocator stress (per count c)

- **Assertions:** S1 accounting identity and sentinels at count c at the
  §5 depth.
- **Capture:** allocator accounting before/after; counter snapshot; W04
  sentinel check.
- **Pass condition:** balance exact; sentinels intact; all CPUs remain
  online post-stress.
- **Proves:** concurrent allocation safety at count c within declared
  limits. **Does not prove:** OOM-under-pressure behavior unless the S1
  OOM variant is exercisable (W12 register), fragmentation behavior,
  hardware.

### R8 — failure path (per count c)

- **Assertions:** S5b absent-CPU start and S6 invalid/offline targeting
  produce refusals/terminal outcomes per the owning designs; online set
  unchanged by refused calls; degraded accounting exact when a failure
  is induced.
- **Capture:** failure diagnostics, registry state, refusal events.
- **Pass condition:** every injected failure terminates with
  phase-attributed outcomes; no crash; no state corruption; boot
  continues (reference default) or degrades with complete accounting.
- **Proves:** failure paths stay diagnosable at every supported count.
  **Does not prove:** true device-level timeout (not inducible — W12
  register), recovery/retry (Reserved), hardware failure behavior.

## 3. Session structure

One campaign session at count c: R1–R4 from the boot itself; then the S
scenarios in W12's fixed order (S1, S2, S3, S7; R6's storm is S2's
deployment); then S5b/S6 (R8) last. A session ends with a final registry/
counter dump. Rows R1–R4 are inseparable from boot; R5–R8 reuse W12
entry points. A row's failure marks the session's later rows blocked-
by-failure unless the divergence analysis shows independence — recorded,
never assumed silently.

## 4. Repetition policy

- **Cold boots:** R13 cold boots per count per campaign (constant fixed
  in the implementation record with rationale and revisit trigger).
  R1–R4 are asserted on every cold boot; the S-scenario depths of R5–R8
  run on a declared subset of boots (at least one full-depth session per
  count per campaign) to keep session length practical while every boot
  still exercises the boot-path rows.
- **What repetition is for:** exposing timing/rendezvous/allocator races
  (plan scope). Each boot is independently asserted; a single divergent
  boot is a `run-failed` for the campaign row — the other boots do not
  dilute it.
- **What repetition is not:** a proof of race absence. The honest claim
  is "no divergence observed within R13 boots in the declared
  environment"; P3-V13's wording ("meets declared criteria over repeated
  cold boots") is satisfied exactly by that statement and never
  strengthened.
- **Same-session repetition:** S-scenario in-boot rounds (R rounds) are
  W12's constants; the matrix does not multiply them.

## 5. Scenario deployment mapping

| W12 scenario | Rows | Counts | Depth policy |
|---|---|---|---|
| S1 allocator stress | R5, R7 | 1, 2, 4, 8 | full depth each count; K/R constants per W12's record |
| S2 notification storm | R5, R6 | 2, 4, 8 (S2 is vacuous at c = 1; R6 does not exist at c = 1) | full depth |
| S3 atomic/counter stress | R5 | 1, 2, 4, 8 | full depth |
| S4 repeated rendezvous | R1 (every cold boot is an S4 iteration) | 1, 2, 4, 8 | R13 boots = S4 repetition; S4 pass conditions checked per boot |
| S5 secondary timeout/failure | R8 (S5a host-side per W12; S5b in-session) | 1, 2, 4, 8 | S5b once per count per campaign |
| S6 invalid/offline targets | R8 | 1, 2, 4, 8 | once per count per campaign (needs a failed/excluded CPU: produced in-session at c where S5b runs, or via declared excluded topology) |
| S7 concurrent logging | R5 | 1, 2, 4, 8 | full depth; fatal-path exercise once per campaign (any count, declared which) |

Mapping rules: a scenario not deployed at a count is recorded `not-run`
for that row/count with the reason (e.g. vacuity at c = 1); the mapping
is reviewed against W12's matrix design (W13-DV05) so the two contracts
cannot drift.
