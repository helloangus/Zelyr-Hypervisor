# P6-W03 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W03 detailed design](README.md).

## 1. Scope of validation

W03 validation covers classification and outcome correctness (host-side
exhaustive tests), lifecycle behavior with real delivery (QEMU), and
declared storm smoke (bounded synthetic high-rate SGI delivery in QEMU).
QEMU evidence establishes behavior in the reference environment; it does
not prove production DoS resistance, device-framework adequacy, or
real-hardware correctness. Guest-side scenarios are W11; robustness
integration is W12.

## 2. Validation matrix

Each validation is recorded as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason, in
`../../verification/p6-w03-physical-interrupt-lifecycle-verification.md`.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W03-DV01 → P6-V20 | Entry basis | prerequisite review ([workflow](05-implementation-workflow.md) step 1) | inspect W02 evidence + P1/P3 contracts present | all consumed contracts evidenced | entry integrity; not runtime behavior |
| W03-DV02 → P6-V20 | State authority | architecture review | check every state row of [02](02-architecture-and-state.md) §2 against the implementation | each transition has exactly the designed owner; no second EOI/acknowledge site | ownership totality; not consumer correctness |
| W03-DV03 → P6-V20 | Classification totality | exhaustive unit tests | full encoded value space per pinned revision; boundary IDs | one named outcome per value; typed construction infallible in-range | classification; not hardware emit behavior |
| W03-DV04 → P6-V20 | Ordinary dispatch + registration | QEMU with registered test consumer (SGI and PPI classes) | enable, deliver, observe ack→handler→complete, counters | consumer invoked with correct context; exactly one EOI; stats exact | lifecycle on QEMU; not SPI device realism |
| W03-DV05 → P6-V20 | Spurious / unknown / consumerless | QEMU + unit tests | forced no-pending acknowledge; OutOfSupported delivery via test construct; enabled-but-unregistered ID | each takes its named row: no-EOI/complete-as-data/consumerless-complete; diagnosable; no panic, no invalid index | safe outcomes; not malicious-hardware resistance |
| W03-DV06 → P6-V20 | Repeated and simultaneous | QEMU scenarios (per W11 coordination) | same-ID repeat during handling; multiple pending IDs | no loss (each delivery accounted), documented policy holds, no state corruption | documented policy; not full GIC semantics coverage (W10) |
| W03-DV07 → P6-V22 | Bound exit and containment | host synthetic sequences + QEMU declared storm smoke | exceed DISPATCH_BOUND; declared high-rate SGI rate | exit between iterations only; hardware re-delivery completes the remainder; invariants hold; no hang | bounded containment within declared limits; not production DoS resistance |
| W03-DV08 → P6-V20 | Guest isolation review | static + behavioral review | confirm VM-oblivious dispatch; no Guest state access; guest-caused anomalous ID (via W11 later) cannot panic | zero Guest-coupled paths in dispatch | isolation; not W07/W12 virtual-side evidence |
| W03-DV09 → P6-V24 | Telemetry | event inspection in QEMU runs | outcome counters + rate-limited events observed; per-pCPU attribution correct | bounded, attributable, correlatable output | observability; not W13 correlation completeness |
| W03-DV10 → W03 closure | Handoff consumability | consumer review | read the boundary as W04 (SGI consumer surface), W05 (PPI path), W07 (producer boundary), W12 (outcome vocabulary), W13 (counter surface) | each consumer can act without extending W03 | handoff readiness; not downstream completion |

Explicitly **not run** in W03: SPI routing changes (W04), timer semantics
(W05), virtual interrupts (W07–W09), Guest-observable scenarios (W11),
production DoS claims (out of scope by the task book).

## 3. Error, security, and observability model

- *Error model:* five named outcomes; no panic path from interrupt data;
  consumerless/unknown are diagnosed and bounded; a leaked active
  interrupt is structurally unreachable (INV-B) and its would-be symptoms
  (wedge on priority registers) are named in the design for reviewers.
- *Security model:* hardware and Guest-influenced values are data, never
  control flow; registration is a Host-internal surface with owner tags;
  Guest-facing authorization does not exist here (W07 + P5 boundary).
- *Observability model:* per-pCPU/per-ID counters, once-per-crossing
  threshold events, rate-limited anomaly events, bound-exit events; W13
  correlates (P6-V24) and measures latency (P6-V25) on this surface.

## 4. Handoff checklist

Before handing W03 to a reviewer, provide:

- the exact changed-file list and any `unsafe` delta (expected: none —
  all hardware access via the W02 surface);
- the fixed constants (DISPATCH_BOUND, rate-limit window/threshold) with
  rationale and their measured effect note for W13;
- W03-DV01…DV10 evidence paths and run status, including explicit
  not-run entries;
- the lifecycle contract handed downstream: single acknowledge/complete
  path, combined-EOI rule, registration-before-enable, consumer
  obligations, named outcomes, counter surface;
- confirmation that no SGI policy, route change, trigger-type surface,
  scheduler hook, virtual-interrupt path, device framework, or second
  entry path was implemented;
- open items for W04 (SGI consumer registration shape), W05/W08 (first
  consumers of the registration path), W10 (priority semantics over the
  flat posture), W12 (storm integration), W13 (latency hooks) — without
  resolving their contracts here.

## 5. Future record and verification locations

Implementation facts: `../p6-w03-physical-interrupt-lifecycle-record.md`
(created when work starts; not created by this design). Verification
evidence: `../../verification/p6-w03-physical-interrupt-lifecycle-verification.md`
(created only by real verification work). Neither file exists yet; neither
may claim completion.
