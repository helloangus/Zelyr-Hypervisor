# P8-W10 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W10 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none asserts that a test has run. Evidence
destination:
`../../verification/p8-w10-linux-smp-bringup-verification.md` (do not create
before evidence exists). The 2-vCPU matrix gates the 4-vCPU matrix: a row
below marked "2vCPU gate" must pass before the corresponding 4-vCPU row is
attempted.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W10-DV01 → gate | topology coherence review | creation-time checks C1–C4 with injected inconsistencies | each check fails creation with its named error; coherent configs build ([02 §2](02-code-contracts-secondary-bringup.md)) | coherence enforcement; value correctness is W02/P8-V02/V03 |
| W10-DV02 → P8-V14 (2vCPU gate) | enumeration + secondary start | boot 2-vCPU Linux; observe `smp: Brought up 2 CPUs`-class marker and P1–P2 telemetry | both CPUs enumerated; secondary entered via PSCI CPU_ON path with W03 state; no private path used | the standard SMP bring-up path; not scheduler fairness (W11) |
| W10-DV03 → P8-V14 (2vCPU gate) | per-CPU GIC/timer/SGI | P3–P5 readiness records for both vCPUs; IPI ping test | per-CPU GICR/timer/SGI evidence complete on both; SGI round trips work | per-CPU integration; not stress behavior (S5) |
| W10-DV04 → P8-V14 | idle/WFI on all vCPUs | idle both CPUs; verify WFI block + wake telemetry | no busy-loop; wakeups only from declared sources ([W08 §4 of 03](../p8-w08-linux-timer-integration/03-code-contracts-expiry-wakeup.md)) | idle integration; not power behavior |
| W10-DV05 → P8-V14 | 4-vCPU enumeration + start | repeat DV02/DV03 at 4 vCPU | all four CPUs enumerated and brought up via the same path | scale within the declared machine; not beyond-declared scale (Reserved) |
| W10-DV06 → P8-V15 S1–S2 | busy + oversubscription | run [03](03-stability-scenarios.md) S1, S2 at 2 and 4 vCPU | all observables hold for declared durations; verdicts `passed` | concurrent execution and preemption consistency; not performance |
| W10-DV07 → P8-V15 S3 | sleep/wakeup churn | run S3 | no missed deadline (I4); envelope respected | the lost-wakeup detector; not real-time latency |
| W10-DV08 → P8-V15 S4 | affinity migration | run S4 | no time jumps (I1/I5); TLB/shootdown counters consistent | continuity under placement changes; not balancing policy |
| W10-DV09 → P8-V15 S5 | interrupt-heavy mix | run S5 | W07 fidelity counters reconcile; no lost wakeup; Host responsive | SMP-scale interrupt robustness; not throughput (excluded) |
| W10-DV10 → P8-V15 S6 / P8-V24 | fault/storm containment | run S6 script (panic, SGI storm, loop, malformed bursts) | siblings progress; contained outcomes match owning contracts; W13 context populated | containment at SMP scale; not full security regression (W18 aggregates) |
| W10-DV11 → P8-V22 (input) | repeated boot with secondaries | repeated 2/4-vCPU boots ([W16](../../plans/p8-w16-automated-linux-regression.md) harness) | no stale VMID/TLB/vCPU/IRQ state detectable via the declared telemetry; verdicts reproducible | repeatability; not formal state-space coverage |
| W10-DV12 → W14 | compatibility-dimension review | compare topology/MPIDR/PSCI-start facts against W14's checklist | all listed as dimensions | compat readiness; not drift execution |

Record each validation as passed/failed/blocked/not run with command, input,
environment, timestamp, reason, and the scenario verdict record reference.
Enumeration or secondary-entry alone does not satisfy P8-V14's per-CPU rows,
and no single S-row satisfies P8-V15 — the matrix is cumulative. All timing
envelopes are fixture-declared and generous: passing is a semantics claim,
never a performance claim.

## 2. Error, security, and observability model

- **Error model:** three classes — Guest-lifecycle errors (W06 codes,
  inherited verbatim), Guest faults (P7-W07 Faulted + W13 context, sibling-
  safe), and internal faults (Hypervisor-side, escalate via W13, never
  Guest-visible as success). Scenario failures are evidence, not exceptions:
  they are recorded with severity per [03 §1](03-stability-scenarios.md).
- **Security model:** the Guest is untrusted on every SMP-relevant input
  (CPU_ON targets, GIC/SGI abuse, storms); containment is per-VM with
  sibling-vCPU isolation demonstrated by S6; no scenario can degrade the
  Host (ADR §19); no private escape hatch exists because the bring-up path
  is singular (decision D2).
- **Observability:** the [01 §6](01-architecture-and-state.md) evidence map
  is the design's telemetry contract — every symptom class has a named
  Hypervisor-side stream; verdicts require telemetry + log agreement
  (anti-forgery, decision D6); all counters bounded, events prunable and
  filterable (ADR-048); run records are bounded and reproducible
  (`scenario_correlate` auditability).

## 3. Handoff checklist

Before handing W10 to a reviewer, provide:

- the exact changed-file list and M1–M6 → actual-unit mapping as implemented;
- the approved W02 gate values used (`<VCPU-COUNT>`, MPIDR rule) and their
  authority location;
- DV01–DV12 evidence paths with run status and verdict records, including
  explicit not-run/blocked entries (e.g. 4-vCPU rows blocked pending the
  2-vCPU gate; M:N rows owned by W11);
- the green-baseline evidence for [W09](../p8-w09-virtual-console-single-cpu-linux/README.md)
  Stage D on which SMP stages were built;
- confirmation that no scheduler policy, topology value, spin-table path, or
  Host SMP mechanism was introduced, and that all lifecycle logic remains in
  the owning contracts; inventory of new `unsafe` with SAFETY justification
  (expected only in arch-layer boundaries owned elsewhere);
- confirmation of seam agreement with W06/W07/W08/W09 and the P7 seams
  (workflow Steps 2–5);
- open items: W02 gate status; P3 substrate evidence status; W15 fixture
  content; W16 automation readiness; W11's M:N follow-up — without
  resolving their contracts here.
