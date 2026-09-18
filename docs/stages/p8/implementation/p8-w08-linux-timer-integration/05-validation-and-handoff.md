# P8-W08 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W08 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none asserts that a test has run. Evidence
destination:
`../../verification/p8-w08-linux-timer-integration-verification.md` (do not
create before evidence exists). Scenario IDs T1–T4 and the T5a–T5f set of
[02 §4](02-code-contracts-timer-regs.md) are referenced from the README
mapping table and the workflow.

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W08-DV01 → gate | machine-fact single-source review | inspect `VmTimeFacts` population | frequency/offset/class from approved machine facts only (Step 1); consistent with W04 DTB facts | freeze respected in code; value correctness is W02/P8-V02/V03 |
| W08-DV02 → P8-V11 T1 | counter/time and monotonicity | Linux boot; read clocksource across boot phases, WFI sleeps, and (with W10) on all vCPUs | time advances monotonically; all vCPUs coherent (I1/I2); frequency uniform | the time-source contract; not Host timing accuracy or real-hardware behavior |
| W08-DV03 → P8-V11 T2 | timer IRQ delivery | Linux clockevent under load: periodic and one-shot modes | interrupts arrive per programmed deadlines; EOI/re-assert follows level semantics | delivery path (§2); not interrupt throughput |
| W08-DV04 → P8-V11 T3 | sleep/timeout | fixture: timed sleeps across WFI (idle) and busy (non-idle) paths | timeouts fire at programmed deadlines on the programming vCPU (I3/I4) | deferred-delivery and wakeup rules; not scheduling fairness |
| W08-DV05 → P8-V11 T4a | preemption continuity | forced preemption of a vCPU with a near deadline ([W11](../../plans/p8-w11-scheduler-linux-integration.md) M:N scenario) | deadline delivered exactly once after resumption; no time jump (I5) | continuity contract; not scheduler policy |
| W08-DV06 → P8-V11 T4b | high-frequency reprogramming | fixture-bounded reprogram storm (T5a/T5b) | bounded host work (I6); Guest time stays monotonic; Host responsive | robustness; not performance KPIs (W17 owns baselines) |
| W08-DV07 → P8-V11 T4c | multi-vCPU delivery | with [W10](../p8-w10-linux-smp-bringup/README.md): per-CPU clocks/ticks active on 2/4 vCPUs | each vCPU receives only its own timer events (I3); tick correctness on all CPUs | target-vCPU semantics; not SMP scale beyond the declared fixtures |
| W08-DV08 → P8-V24 | malformed programming containment | T5a–T5f from W18's illegal-timer set | declared contained outcomes; no Host fault; stopped-vCPU deadline handled (T5f) | guest-untrusted containment; not full fault taxonomy (W13) |
| W08-DV09 → P8-V03 | host-independence review | audit frequency/offset/counter path for Host-derived exposure | none beyond the offset relation; all values machine-gated | path-level host independence; not whole-machine review (W02) |
| W08-DV10 → W14 | compatibility-dimension review | compare timer facts against W14's drift checklist | counter class, frequency, timer class, PPI INTID listed as dimensions | compat readiness; not drift-test execution (W16) |

Record each validation as passed/failed/blocked/not run with command, input,
environment, timestamp, reason. Clocksource registration or `start_kernel`
alone satisfies only part of DV02 — P8-V11 requires the delivery, sleep,
preemption, and WFI rows. Timing rows use functional assertions (deadlines
met, monotonicity, coherence) with generous environment-dependent tolerances
declared in the fixture (W15/W16); none of this matrix is a real-time or
performance claim.

## 2. Error, security, and observability model

- **Error model:** Guest-visible errors do not exist for well-formed timer
  programming (it always "works"); contained classes are malformed register
  access (W05-classified), stale expiry (benign, counted), and internal
  faults (W13 escalation). The deliberate fail-toward-delivery bias
  ([03 §1](03-code-contracts-expiry-wakeup.md)) means internal uncertainty
  produces spurious delivery — Guest-visible and benign — never silence.
- **Security model:** the Guest is untrusted on every register value;
  arithmetic is checked; deadline data never selects hypervisor behavior;
  the physical counter/timer are Hidden-classified; wakeup sources are
  hypervisor-defined, so a Guest cannot pin or spam another vCPU through WFI
  mechanics. Containment for storm scenarios is bounded work per trap (I6).
- **Observability:** program/expire/deferred/wakeup/noop events, bounded
  per-vCPU counters, P6-W13-method latency observations feeding W17; all
  prunable and runtime-filterable (ADR-048); no unbounded buffers.

## 3. Handoff checklist

Before handing W08 to a reviewer, provide:

- the exact changed-file list and M1–M6 → actual-unit mapping as implemented;
- the approved W02 gate values used (frequency, offset policy, timer class)
  and their authority location;
- DV01–DV10 evidence paths with run status, including explicit not-run
  entries (e.g. multi-vCPU rows await W10; preemption rows await W11's M:N
  scenarios);
- confirmation that no P6 timer state was re-owned, no scheduler policy
  changed, no DTB bytes written, and no Host-hardware-derived value reached
  the Guest; inventory of new `unsafe` with SAFETY justification (expected
  only at arch register-access boundaries owned by the arch layer);
- confirmation of Step 6 seam agreement with W07/W09/W10/W11 designs;
- open items: W02 gate status; WFE/event-stream classification follow-up
  (W05); W14/W16/W18 consumption readiness — without resolving their
  contracts here.
