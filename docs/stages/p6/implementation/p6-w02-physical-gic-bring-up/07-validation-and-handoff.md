# P6-W02 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W02 detailed design](README.md).

## 1. Scope of validation

W02 validation covers BSP global bring-up (P6-V02) and AP-local bring-up
(P6-V03) in the declared QEMU reference environment, plus host-side
sequence tests on the abstracted register surface. QEMU evidence
establishes the documented usable state in the reference environment only;
it does not prove real-hardware correctness, and it does not prove routing,
Guest delivery, or a real-hardware support tier (plan closure wording).
Hardware-tier validation is out of P6 (P15 owns Orange Pi 3B).

## 2. Validation matrix

Each validation is recorded as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason, in
`../../verification/p6-w02-physical-gic-bring-up-verification.md`.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W02-DV01 → P6-V02 | Entry basis | prerequisite review ([workflow](06-implementation-workflow.md) step 1) | inspect W01 record + P3/P2/P0 evidence actually present | all assumed contracts located and evidenced; gaps recorded as blocks | entry integrity; not runtime behavior |
| W02-DV02 → P6-V02 | BSP Host-GIC initialization | QEMU single-CPU boot | boot to `DistributorReady` + boot-CPU `LocalReady`; read back CTLR/IROUTR/priority/enable state | declared usable state reached; read-back matches the configuration record; residuals handled diagnostically | global safe state on QEMU; not real-hardware, routing, or Guest delivery |
| W02-DV03 → P6-V02 | Confirmation-before-configuration | sequence review + QEMU probe logging | verify the probe phase precedes any state-changing write (backend log in host tests; probe telemetry in QEMU) | zero writes before probe agreement in every path | safety ordering; not hardware independence from firmware state |
| W02-DV04 → P6-V02 | Local readiness state machine and failure paths | host-side sequence tests (fake backend) + injected failures | exercise mismatch, wake timeout, SRE failure, double invocation | each failure publishes the named `LocalFailed` reason once; no partial posture; WAKER never left asleep | state-machine correctness; not QEMU/hardware failure behavior |
| W02-DV05 → P6-V03 | AP-local independence | QEMU multi-CPU boot (per the declared environment's CPU count) | each online pCPU publishes its own `LocalReady`; compare per-CPU records | every evidenced online pCPU independently initialized; no shared BSP-local state (record diff check); boot-CPU record produced by the same Phase-B path | per-pCPU independence on QEMU; not hardware AP behavior at scale |
| W02-DV06 → P6-V02/V03 | Readiness ledger | unit tests + QEMU consumer read | write-once enforcement; acquire/release visibility; queries consistent with published states | no double publication; consumers never observe intermediate states | ledger correctness; not downstream consumer logic |
| W02-DV07 → P6-V02 | Failure-to-P3 mapping | QEMU failure injection (where the environment permits, e.g. malformed expected identity via a test construct) | a forced local failure yields the P3 failed-CPU outcome and boot continues or stops per the P3 contract | local failure never global-panics; global failure takes the initialization-error path | containment; not production fault handling (W12 owns robustness evidence) |
| W02-DV08 → P6-V02 | Residual-state scenario | host sequence tests + QEMU observation | residual survey-and-clear report produced; enabled state verified quiet (no interrupt before consumer registration) | residuals logged and cleared; INV-1 read-back clean | initial-state cleanliness; not firmware-residual behavior on all hardware |
| W02-DV09 → P6-V02 | Unsafe and layering review | static review | `unsafe` inventory delta limited to `gic-regaccess` with SAFETY comments; no board/SoC/QEMU constants; no platform-name branch | zero unreviewed `unsafe`; layering rules hold | controlled boundary; not long-term audit completeness (P0 governance owns the inventory) |
| W02-DV10 → P6-V02 | Telemetry review | event inspection in QEMU run | the five event kinds emitted with expected payloads; residuals aggregated | events present, bounded, name-free of platform identity | observability; not W13 correlation |
| W02-DV11 → W02 closure | Handoff consumability | consumer review | read the readiness/initial-state record as W03 (can I dispatch safely?), W04 (can I change routes?), W05 (is my PPI surface defined?) | each consumer can act without re-deriving bring-up | handoff readiness; not downstream completion |

Explicitly **not run** in W02: real-hardware bring-up (P15), Guest delivery
(W11 scenarios), storm behavior (W12/W13), dispatch correctness (W03).

## 3. Error, security, and observability model

- *Error model:* named failure reasons at both scopes; local failures are
  pCPU-attributed and contained (P3 consequences); global failures stop
  initialization through the established fatal path (P1 diagnostics own
  presentation). No W02 path panics on a local failure; no path continues
  after a global failure.
- *Security model:* Secure configuration untouched; probe-before-write;
  residual firmware state treated as untrusted and quiesced; frame bounds
  enforced at the access surface; no Guest input exists in W02.
- *Observability model:* five trace-event kinds (phases, probe results,
  residuals, failures) with bounded payloads; the readiness ledger is the
  queryable state; W13 later correlates the events into the P6 telemetry
  record (P6-V24).

## 4. Handoff checklist

Before handing W02 to a reviewer, provide:

- the exact changed-file list and the `unsafe` inventory delta;
- the pinned GIC specification revisions (from the W01 record) and the
  register-class table location;
- W02-DV01…DV11 evidence paths and run status, including explicit not-run
  entries (real hardware, Guest delivery, storm);
- the initial-state facts handed to consumers: every SGI/PPI/SPI disabled,
  lowest default priority, SPIs determinately routed to the boot pCPU
  (IRM=0), combined-EOI posture (EOImode=0), no preemption grouping;
- confirmation that no dispatch, classification, SGI-send, route-change,
  timer-PPI, maintenance, virtualization-configuration, or Guest-visible
  interface was implemented;
- open items: W03 (registration surface must be built on the ledger and
  the disabled baseline), W04 (routing change over the initial state),
  W08 (virtualization probe evidence location), W13 (telemetry
  correlation) — without resolving their contracts here.

## 5. Future record and verification locations

Implementation facts: `../p6-w02-physical-gic-bring-up-record.md` (created
when work starts; not created by this design). Verification evidence:
`../../verification/p6-w02-physical-gic-bring-up-verification.md` (created
only by real verification work). Neither file exists yet; neither may claim
completion.
