# P7-W11 Stress Scenario Matrices

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W11 detailed design](README.md).

## 1. Scenario construction rules

Every stress scenario is a declared tuple, not free-form testing. A scenario
declaration names, and may only name:

- **Topology:** pCPU count, VM count, vCPU count and per-VM distribution, and
  the placement configuration (pinned / affinity-masked / dedicated / shared)
  expressed with the P7-W03 configuration semantics.
- **Payload:** one or more P7-W10 workload programs per vCPU with their
  declared parameters (iteration counts, event rates, marker outputs). W11
  defines payload parameters only; workload programs and their output markers
  are owned by the [P7-W10 design](../p7-w10-validation-guest-suite/README.md).
- **Event timing profile:** when the scenario injects timer/SGI/vIRQ/Notification/pause
  events, the injection source (which vCPU/pCPU), the offset distribution
  around the target vCPU's block/wakeup/pause transitions, and the recorded
  seed of any pseudo-random offset generator.
- **Check set:** the INV-* catalog entries evaluated during the run, from
  [the checks-and-signals file](02-invariant-checks-and-failure-signals.md).
- **Objective outcome:** the pass condition and the F-signal set that
  classifies failure, per the same file §3.

A scenario runs only when the P7-W01 input boundary confirms the underlying
contracts are evidenced: W02 lifecycle and admission, W03 placement, W04
preemption, W05 M:N/multi-VM, W06 block/wakeup, W07 pause/stop/fault, W08
SMP/idle, W09 observability, W10 workloads, plus the P0 runner entry and P1
fatal-diagnostics boundary. If any contract is absent or contradictory, the
affected scenario is **blocked** and recorded as such; it is never run against
an assumed behavior, and the gap is never repaired inside W11.

The pCPU/vCPU counts in this file are test-topology parameters for the QEMU
reference environment (ADR-003). They are not claims about host or target
hardware capabilities.

## 2. S1 — M:N and overcommit stress (P7-V24)

Goal: sustained multi-VM, overcommitted execution shows no corruption,
starvation, or livelock. The task-book gate is the 4pCPU/8vCPU/two-VM
configuration plus 2× overcommit; 4× is informational.

| ID | Class | Composition and precondition | Expected observable | Pass condition | Failure signal |
|---|---|---|---|---|---|
| S1a | Required gate | 4 pCPU, 2 VMs, 8 vCPUs (4 per VM), all shared, all vCPUs CPU-bound (W10 CPU-bound workload); M:N overcommit 2:1 | All 8 vCPUs accumulate execution; both VMs emit progress markers; run completes and reports accounting | INV-1–INV-8 hold for the full duration; every vCPU shows sustained nonzero execution in every fairness window; completion markers present before the watchdog deadline | F1–F5; absence of completion markers by deadline → F3 |
| S1b | Required | 2× overcommit matrix: (1 pCPU/2 vCPU), (2/4), (4/8), single VM per row, mixed payloads: CPU-bound + periodic-WFI + HVC-heavy vCPUs per W10 suite | Blocked vCPUs release their pCPU and later resume; CPU-bound vCPUs are preempted; mixed progress continues | Same as S1a per row; additionally no vCPU of the row starves across any fairness window | F1–F5 |
| S1c | Informational, reported separately | 4× overcommit: (2 pCPU/8 vCPU) and (4 pCPU/16 vCPU), mixed payloads | Progress and invariant observability at 4× depth | Same objective conditions as S1a; a failure here is a recorded finding and a P7 limitation, not a stage-gate failure | F1–F6, recorded as finding |

Proves: within the declared duration and payload coverage, the implemented
scheduler sustains M:N operation at the gate configuration without any
objective fault signal, and blocked/running/mixed mixes coexist.

Does not prove: absence of races not exercised by the payloads; any fairness
quality beyond "no starvation over the declared windows" (proportional
fairness is out of scope); behavior on real hardware; any performance property
(W13's domain).

## 3. S2 — race stress around block, wakeup, and pause (P7-V25)

Goal: events arriving before, during, and after blocking transitions, and
pause/resume concurrent with event delivery, lose no wakeup/timer/vIRQ and
preserve placement guarantees. Race windows are swept statistically, not
deterministically forced: the injector emits events at offsets drawn from a
declared distribution around the target transition, and the seed of every draw
sequence is recorded. Same-pCPU and cross-CPU injection variants are both run
where the row marks them; the cross-CPU variant exercises the P3 notification
transport through the W08 contract.

| ID | Class | Composition and precondition | Expected observable | Pass condition | Failure signal |
|---|---|---|---|---|---|
| S2a | Required | 1 pCPU, 1 VM, 2 vCPUs; vCPU-B executes periodic WFI; vCPU-A injects timer and SGI events at offsets before B's WFI entry | B either handles the pending event without blocking or wakes immediately; no lost event (event counter vs handled counter reconciles) | INV-7 holds; no duplicate running (INV-1); completion markers reconcile injected vs handled events | F2, F5; counter mismatch → F4 |
| S2b | Required | 2 pCPU, 1 VM, 2 vCPUs pinned one per pCPU; A injects while B is in blocked/WFI state (cross-CPU injection) | B becomes runnable and runs without lost or duplicate wakeup | INV-1, INV-2, INV-7 hold across the sweep | F1–F5 |
| S2c | Required | Same as S2b; injection timed at the block-exit transition (wakeup window) | Wakeup races never produce a stranded blocked vCPU or a double-run | Same as S2b | F1–F5 |
| S2d | Required | 2 pCPU, 1 VM, 2 vCPUs; pause request concurrent with block entry/exit and with event delivery; resume concurrent with pending events | Pause completes with no Guest execution; resume preserves eligibility, placement, and pending-event delivery (W07 semantics) | INV-3, INV-7, INV-8 hold; W07 completion conditions observable via W09 trace | F1–F5 |
| S2e | Required | 4 pCPU, 2 VMs, 8 vCPUs, mixed payloads; all S2a–S2d race profiles superimposed at load | No lost wakeup or corruption under concurrent race pressure | All invariants hold; event reconciliation closes for every vCPU | F1–F5 |

Proves: the wakeup, pause, and event paths survive the swept windows at the
declared offsets and repetition counts without an objective fault signal.

Does not prove: race freedom outside the swept offset distributions; timing
behavior on hardware; that the scheduler is free of latent races the payloads
cannot reach.

## 4. S3 — invariant and fairness stress (P7-V26)

Goal: long-duration mixed runs with the full check set enabled, plus
no-starvation observation. A **fairness window** is a declared observation
interval within the run; the run declares window length and sampling count.
Within every window, every vCPU that was continuously Runnable for the whole
window (equal scheduling class, per W05 semantics) must accumulate nonzero
scheduled execution. A continuously runnable vCPU with zero execution across a
window is a starvation finding.

| ID | Class | Composition and precondition | Expected observable | Pass condition | Failure signal |
|---|---|---|---|---|---|
| S3a | Required | 4 pCPU, 2 VMs, 8 vCPUs, all CPU-bound, shared; full INV-1–INV-8 check set active; declared fairness windows sampled throughout | Coherent accounting at window boundaries and end of run; nonzero per-vCPU execution in every window | INV-1–INV-8 hold; accounting audit (INV-4) closes; no starvation in any window | F2, F3, F4 |
| S3b | Required | Same topology; payloads mixed CPU-bound + WFI + SGI-active + HVC-heavy; windows sampled across block/wakeup churn | Same, under blocking churn | Same as S3a | F1–F5 |

Proves: the P7 invariant set is checkable under load and holds for the
declared durations, and continuously runnable equal-class vCPUs are not
starved over the declared windows.

Does not prove: any fairness ratio or share claim; invariant truth outside
observed states; correctness beyond the check set's observability (a violation
invisible to W09's observable surface is not detected — that limit is part of
the recorded evidence).

## 5. S4 — placement stress (P7-V27)

Goal: valid-but-extreme placement configurations obey configuration under
sustained load. Invalid-configuration rejection is P7-W03 evidence (P7-V07)
and is not re-tested here except where a stress row exercises it incidentally.

| ID | Class | Composition and precondition | Expected observable | Pass condition | Failure signal |
|---|---|---|---|---|---|
| S4a | Required | 4 pCPU, 1 VM, 8 vCPUs with deliberately overlapping affinity masks (all vCPUs eligible on the same pCPU subset); CPU-bound payloads | Execution never occurs on an ineligible pCPU; contention resolves by scheduling, not by violation | INV-5 holds across the run; per-pCPU execution only within declared masks (W09 placement observability) | F2, F5; mask breach → F2 |
| S4b | Required | 2 pCPU, 1 VM, 4 vCPUs all pinned to one pCPU; 4 additional vCPUs pinned to the second; CPU-bound payloads | Pinned groups timeshare within their pCPU; no pinned vCPU executes elsewhere | INV-5 holds; pinned behavior retains the W03 static-baseline contract | F1–F5 |
| S4c | Required | 4 pCPU, 2 VMs; VM-A vCPUs dedicated to pCPUs 0–1, VM-B vCPUs shared on pCPUs 2–3, plus one cross-VM overlap row per the W03 mixed semantics; mixed payloads | Dedicated pCPUs run only their owning vCPUs; shared pCPUs honor affinity; both VMs progress | INV-5 holds; dedicated exclusivity observable; both VMs emit progress markers | F1–F5 |

Proves: placement semantics survive adversarial-but-valid configurations
under load.

Does not prove: load-balancing quality, migration policy behavior (reserved),
or any hardware topology assumption.

## 6. Repetition, seed, and duration policy

Values below are **floors owned by this design** (decision 6 of the entry
README). Implementation may raise them with recorded rationale; lowering any
floor requires a new design decision. Duration is wall-clock time under the
declared environment, recorded per run together with the environment
declaration.

| Family | Minimum repetitions per scenario | Minimum duration per repetition | Seed rule |
|---|---|---|---|
| S1 | 3 | 300 s (S1a, S1b rows); 300 s (S1c informational) | Not seed-dependent except for payload parameters; any seeded parameter is recorded |
| S2 | 5 per scenario, covering the declared offset distribution | 120 s or 1000 injected-event windows, whichever completes later | Required: every run records the PRNG seed and offset sequence identifier of each injector |
| S3 | 2 | 600 s | As S1 |
| S4 | 3 | 300 s | As S1 |

Additional rules:

- A **failed run is re-attempted for classification only** (see
  [checks and signals](02-invariant-checks-and-failure-signals.md) §4); a
  failed classification run that reproduces is recorded as reproduced; one
  that does not is recorded as non-reproduced with full context. Either way
  the original failure stands as a finding.
- Repetitions of a scenario may be interleaved with other scenarios to
  distribute host-environment drift; the schedule is recorded.
- Any run whose environment declaration is incomplete (missing QEMU version,
  host state, binary identity, or seed where required) is invalid evidence and
  is recorded as not-run-for-evidence with the defect named.

Evidence for every scenario goes to
`docs/stages/p7/verification/p7-w11-stress-invariants-verification.md`, with
raw serial captures, trace extracts, and per-run parameter/seed records under
`docs/stages/p7/verification/p7-w11-stress-invariants-artifacts/` (created at
execution).
