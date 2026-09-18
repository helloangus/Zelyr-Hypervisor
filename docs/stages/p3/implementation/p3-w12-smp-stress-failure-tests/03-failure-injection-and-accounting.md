# P3-W12 Failure Injection, Accounting, Limits, and Evidence

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W12 detailed design](README.md).

## 1. Principles

- **Inject through seams, not surgery.** A fault stimulus is a declared
  seam the owning design exposes (a fake transport, a bad target
  identity, an absent-CPU start input) — never a patch to the
  mechanism's internals.
- **Declare what cannot be injected.** A stimulus that is impossible at
  P3 is listed as not-exercisable with its downstream owner; it is never
  simulated silently and never claimed.
- **Accounting beats intuition.** Pass conditions are identities over
  declared observables; "it looked fine" is not a result.

## 2. Failure-injection register

| Injection | Method and owner | Exercisable at P3 | Limits / what it cannot induce |
|---|---|---|---|
| Never-arriving start result | Host-side fake arrival mailbox driving W02's bounded-poll watcher (S5a) | Yes, host-side | Only the watcher logic; not firmware/PSCI behavior |
| Late/garbled start result | Same fake mailbox, late and bad-result variants (S5a) | Yes, host-side | Same boundary |
| Absent-CPU start | W02's induced failure input: CPU_ON to an identity outside the inventory, via the QEMU entry path (S5b) | Yes, QEMU (deterministic rejection per W02's matrix note) | Not a true device-level timeout; a refusing firmware is a different failure than a hanging one |
| Failed-CPU re-targeting | CPU_ON/notification/TLB requests targeting a `Failed` or excluded-class CPU (S6) | Yes | Requires a failed/excluded CPU to exist (from S5b or declared topology) |
| Unknown identity targeting | Requests with an identity absent from the inventory (S6) | Yes | Pure software stimuli |
| Transport fault (lost/duplicated notification or TLB request) | Fake transport seam in host-side variants where W07/W08 logic separates from delivery (S2/S6 host variants) | Yes for logic, host-side only | The real delivery path under QEMU cannot be made to lose requests at P3; storm accounting is the real-path evidence |
| Real PSCI/firmware failure | None declared | **No** — not exercisable at P3 | Recorded gap; owner: P15 hardware validation |
| Allocator OOM under concurrency | Declared allocation-failure seam if the P2 allocator designs expose one; otherwise retained-set exhaustion at a safe bound (S1 variant) | Conditional — check at implementation time | If neither seam exists, the OOM-under-stress row is recorded not-exercisable with owner P2-W04/W05 |
| Timer-based timeout | None | **No** — no timer in P1–P3 (P6 owns timers) | W02's bounded poll is the only P3 timeout semantics; S5 tests exactly that, no more |

The register is maintained in the implementation record; a scenario whose
injection turns out unexercisable is recorded **blocked** (not failed, not
passed) with the register row cited.

## 3. Accounting rules

### 3.1 Sources

| Source | Used by | Rule |
|---|---|---|
| W11 counter snapshot/aggregate ([P3-W11](../p3-w11-smp-observability/README.md)) | S1–S7 | Read per the snapshot contract; quiescence obligations below; snapshot values are recorded in evidence verbatim |
| W11 SMP-ready dump | S4, S6, and every session start/end | Captured at boot and after the last scenario of a session |
| Allocator ownership metadata ([P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md)/[P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md)) | S1 | Before/after totals recorded; balance identity below |
| Registry / `OnlineSet` ([P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)) | all | Online set re-checked after every scenario; membership changes must be explainable by S5/S6 injections only |
| W04 reserved-region fill pattern ([P3-W04](../p3-w04-per-cpu-runtime/README.md)) | S1–S3, S7 | Corruption sentinel: pattern intact unless a scenario documents a legal writer (none does at P3) |
| Rendezvous result / phase events ([P3-W05](../p3-w05-smp-boot-synchronization/README.md)) | S4 | Sequence and once-only checks |
| Channel output ([P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md)) | S7 | Attribution and declared-semantics checks |

### 3.2 Identities (pass-condition vocabulary)

- S1: `allocated == freed` per CPU and in aggregate, including the
  retained set's final release; zero ownership-metadata violations.
- S2: `Σ notifications_sent[cpu]` (accepted sends) `== Σ
  notifications_received[cpu]` over the quiesced online set; per-target
  equality where W07's semantics support per-target accounting.
- S3: each shared counter `== K × N` (its expected total) exactly.
- S4: per boot, `ready_count + failed_count == attempted_count`; each
  CPU's ready signal occurs exactly once; phase transitions strictly
  increase.
- S5: every attempted CPU has exactly one terminal outcome; `Failed`
  count equals the failed set in `Degraded`.
- S6: target-state unchanged (registry state, online membership) for
  every refused call; attempts counted.
- S7: emitted pattern count per CPU matches the declared schedule, subject
  to the channel's declared filtering/dropping semantics (cited, not
  assumed lossless).

### 3.3 Quiescence obligations

Snapshots are best-effort-coherent (W11 architecture §5). Scenarios that
need exact identities (S2, S3) must quiesce the relevant producers before
snapshotting — all senders finished their send loops and an inter-CPU
order point (a barrier per W06's primitives or rendezvous-completion
ordering) separates stimulation from accounting. The quiescence step is
part of each scenario's declared schedule; without it, an identity
mismatch is inconclusive and must be recorded as such, never reported as
a failure of the mechanism.

## 4. Declared test limits

Iteration and repetition constants (K, N, R, rounds) are fixed in the
implementation record at design-implementation time with a stated
rationale per constant and a revisit trigger. Constraints the values must
satisfy:

- Every loop in every scenario has a finite bound; no scenario may wait
  indefinitely on any condition (the only unbounded-looking wait, S4's
  rendezvous, is bounded by W05's terminal-condition guarantee, which S4
  verifies rather than assumes).
- Bounds are large enough to exercise the declared interleavings and
  small enough to keep a QEMU session within a practical session length
  (the rationale must state the observed order of magnitude once first
  runs exist).
- The declared limits are the scope of the P3-V12 claim: results within
  the limits are evidence; behavior beyond them is unknown and recorded
  as such.

## 5. Determinism, seeds, and QEMU nondeterminism

- Schedules are deterministic (round-robin, rings, fixed orders). Where a
  scenario offers a seeded shuffle to widen interleaving coverage, the
  seed is a recorded constant in the evidence; the same seed reproduces
  the same schedule.
- QEMU wall-clock timing, host load, and TCG vs KVM execution mode vary
  between runs. Scenarios therefore assert ordering, accounting, and
  invariants — never durations. Two runs with different timing but
  identical accounting both pass; a run whose accounting differs fails
  with the divergence recorded.
- Execution mode (TCG/KVM), QEMU version, host CPU count, and guest CPU
  count per run are recorded in the evidence environment block; runs are
  compared within the same declared environment class, and the declared
  environment is stated per verification record.

## 6. Evidence classification and destinations

Per-scenario, per-run status vocabulary — mutually exclusive:

- **planned** — contract exists, no run attempted.
- **run-passed** — run completed; all identities/invariants/sentinels
  held; within declared limits.
- **run-failed** — run completed or bounded-aborted; a declared pass
  condition broke; evidence includes the divergent accounting and the
  diagnostics.
- **blocked** — no run possible: a prerequisite (mechanism, seam, runner,
  environment) is missing; the blocking contract and register row cited.
- **not-run** — deliberately deferred (e.g. deferred to W13's matrix);
  reason recorded.

Every record entry carries: scenario id, status, date/time, environment
block (host, QEMU version and mode, CPU counts), the exact entry-point
invocation, the captured observables (dumps, counter snapshots, channel
output), and the reviewer-facing divergence analysis for anything not
`run-passed`.

Destinations:

- W12's own evidence:
  `docs/stages/p3/verification/p3-w12-smp-stress-failure-tests-verification.md`
  (created only when evidence exists), with raw captures attached or
  referenced from it under the same directory using the scenario-id
  naming convention fixed in the implementation record.
- Matrix execution across 1/2/4/8 counts:
  [P3-W13](../p3-w13-qemu-smp-regression/README.md) verification record —
  W12 must not duplicate or pre-claim it.
- Implementation decisions (constants, seam acceptances, register
  updates): `../p3-w12-smp-stress-failure-tests-record.md` (created only
  when implementation begins).

No status may appear in any design or record without its evidence entry;
`planned` is the only honest status before implementation exists.
