# P7-W12 Regression Matrix and Runner Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W12 detailed design](README.md).

## 1. Composition inputs

The regression composes three declared sources; it invents none of them:

- **P7-W10 workload regressions** — the single-vCPU stability case (P7-V22)
  and the declared scheduler workloads (P7-V23), consumed through the
  [P7-W10 design](../p7-w10-validation-guest-suite/README.md). Workload
  programs and their success markers are W10 property; W12 references them by
  ID and parameters.
- **P7-W11 required scenarios** — the handoff of
  [P7-W11](../p7-w11-stress-invariants/README.md): S1a, S1b rows, S2a–S2e,
  S3a/S3b, S4a–S4c with fixed parameters, pass conditions, and F-signals.
  Short-form derivatives keep the pass condition and markers and may reduce
  repetition counts within the case schema.
- **P7-W02–W08 behavior checks** — lifecycle, placement, preemption, pause,
  wakeup, and SMP semantics asserted through their observable outcomes, as
  consolidated by the P7-W01 input boundary.

Every case runs through the single P0-W09 runner entry with the environment
declared per run. If the runner entry, its capture mechanics, or the W09
telemetry contract is absent or diverges from the assumed contract, W12 is
**blocked** as a whole and records the divergence; it does not build a
substitute runner.

## 2. Case schema

Each case is declared with exactly these fields (the schema is stage-local
design freedom owned by this design):

| Field | Meaning |
|---|---|
| `case-id` | R-01…R-14, stable across evidence and W14 indexing |
| `topology` | pCPU count; VM count; vCPU count and per-VM distribution |
| `placement` | pinned / affinity-mask / dedicated / shared configuration per W03 semantics |
| `payloads` | per-vCPU W10 workload IDs and parameters |
| `controls` | pause / wakeup / affinity / pinning control script executed during the case, if any |
| `success-markers` | declared workload/scheduler observables that must appear (from W10 markers and W09 trace/accounting outcomes) |
| `failure-signals` | applicable F-signal subset from [the W11 taxonomy](../p7-w11-stress-invariants/02-invariant-checks-and-failure-signals.md) §3 |
| `timeout-class` | T1 (short) or T2 (loaded); floor values in §5 |
| `repetitions` | declared count ≥ 1; short-form reductions of W11 rows are declared here |
| `artifacts` | serial capture, diagnostic report, parameter record (always all three on FAIL) |

A case with an empty `success-markers` field is invalid; every case must be
able to pass observably.

## 3. Regression case matrix (P7-V28)

| ID | Topology (pCPU/vCPU) | Placement and payloads | Controls | Success markers (summary) | Failure signals | Timeout class |
|---|---|---|---|---|---|---|
| R-01 | 1/1, 1 VM | pinned-equivalent static baseline (W03) | none | W10 single-vCPU workload completes; accounting coherent | F1–F5 | T1 |
| R-02 | 1/2, 1 VM | shared | none | both vCPUs progress; no duplicate running; completion markers | F1–F5 | T1 |
| R-03 | 2/4, 1 VM | shared | none | four vCPUs progress on two pCPUs; per-pCPU state isolated | F1–F5 | T2 |
| R-04 | 4/4, 1 VM | shared, 1:1 | none | four vCPUs progress concurrently on four pCPUs | F1–F5 | T2 |
| R-05 | 4/8, 1 VM | shared, M:N | none | eight vCPUs progress; no starvation within case window | F1–F5 | T2 |
| R-06 | 4/8, 2 VMs | shared, mixed payloads | none | both VMs progress (short-form S1a) | F1–F5 | T2 |
| R-07 | 2/2, 1 VM | shared | pause/resume of a Running vCPU | pause completes with no Guest execution; resume restores progress | F1–F5 | T1 |
| R-08 | 2/2, 1 VM | shared | pause/resume of a Blocked vCPU | resume preserves eligibility and pending events (W06/W07) | F1–F5 | T1 |
| R-09 | 4/4, 1 VM | shared | VM-level pause/resume | no Guest execution during pause; all vCPUs resume | F1–F5 | T2 |
| R-10 | 1/2, 1 VM | shared | timer wakeup from WFI | blocked vCPU wakes on deadline and progresses | F1–F5 | T1 |
| R-11 | 2/4, 1 VM | pinned pairs | SGI/vIRQ cross-vCPU wakeup | target vCPU wakes exactly once per event | F1–F5 | T2 |
| R-12 | 2/2, 1 VM | shared | Notification/internal-event wakeup | wakeup without lost or duplicate events | F1–F5 | T1 |
| R-13 | 4/8, 1 VM | overlapping affinity masks (S4a configuration) | none | execution only within declared masks | F1–F5 | T2 |
| R-14 | 4/8, 2 VMs | dedicated + shared mix (S4c configuration) | none | dedicated exclusivity holds; both VMs progress | F1–F5 | T2 |

Cases R-06, R-13, and R-14 are short-form derivatives of W11 S1a, S4a, and
S4c respectively; their deeper evidence lives in the W11 verification record.

Coverage note: P7-V28 names the five topology points and the four control
families. R-01…R-06 cover the topologies; R-07–R-09 cover pause/resume;
R-10–R-12 cover wakeup; R-13–R-14 cover affinity and pinning (including
dedicated/shared). The matrix is the complete P7-V28 surface; adding cases
follows the design-change path of the parent README.

## 4. Diagnostics integration

On any failure signal, the case's artifact set must contain, together:

1. the serial capture to the point of failure (runner-owned capture, P0-W09);
2. the scheduler diagnostic content required by P7-V21 (current pCPU, VM/vCPU,
   state, affinity, stop reason, recent and pending events) as emitted through
   the W09/P1 boundary;
3. the case declaration actually used (parameters, repetitions, environment
   declaration).

These are the same artifact classes P7-W11 requires, so a regression failure
can be escalated into stress-level investigation without re-instrumenting.
W12 never filters, truncates, or reformats diagnostics to make a result
classifiable, and never converts a missing-diagnostic failure into a pass.

## 5. Determinate outcomes, timeouts, and repetitions

Result taxonomy:

| Result | Definition |
|---|---|
| PASS | All success markers appeared within the timeout; no failure signal fired; repetitions reproduced the outcome |
| FAIL | A declared failure signal fired, or success markers did not appear by timeout, or repetitions did not reproduce a prior outcome |
| INFRA-BLOCKED | The case could not execute or be judged for reasons outside scheduler behavior: runner fault, missing fixture/workload, host resource exhaustion, incomplete environment declaration |
| NOT-RUN | The case was not attempted; reason recorded |

Rules:

- **Timeout floors (owned by this design):** T1 ≥ 120 s, T2 ≥ 300 s wall-clock
  under the declared environment. Implementation fixes concrete values at or
  above the floors per environment and records them. A timeout is FAIL (with
  artifacts), never a silent skip — an unclassifiable timeout with no
  diagnosable cause is recorded FAIL with the diagnosis gap named, and if
  non-scheduler causes cannot be excluded, INFRA-BLOCKED with the same
  evidence.
- **Flakiness rule:** each case runs its declared repetition count (minimum 1
  for T1, minimum 2 for T2 and for every short-form W11 derivative). Any
  non-reproducing outcome is FAIL per the parent README decision 4.
- **INFRA-BLOCKED is never counted as a pass or a scheduler failure**; it
  names the infrastructure cause and must be resolved or explicitly accepted
  as a limitation in the verification record before the matrix can be
  reported complete for P7-V28.
- **Environment declaration is mandatory per run:** QEMU version and machine
  model, virtualization mode (TCG/KVM) and CPU model, host platform state,
  hypervisor binary identity (commit/profile per the P0 build governance),
  and the pinned toolchain identity. A result without a complete environment
  declaration is invalid evidence.

## 6. The QEMU-versus-architecture limit

Every evidence record carries this statement in substance: results hold for
the declared QEMU `virt` reference environment (ADR-003) and prove the
scheduler contracts as exercised there. They do not define AArch64 semantics,
do not prove Orange Pi 3B (RK3566) hardware behavior, do not substitute for
the MMIO/TLB/cache/barrier hardware rules of the Coding Guidelines, and are
never promoted into a Core or machine-ABI contract. Hardware validation
remains later-stage work and is not claimed by P7.
