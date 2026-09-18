# P6-W04 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W04 detailed design](README.md).

## 1. Scope of validation

W04 validation covers typed send behavior, target attribution and
accounting convergence, and determinate SPI re-routing in the declared
QEMU multi-pCPU environment, plus host-side unit and sequence tests.
QEMU evidence establishes the mechanism in the reference environment; it
does not prove general RPC adequacy, scheduler fitness, real-hardware
correctness, or production-scale routing behavior. Per the task book,
multi-vCPU rows require the evidenced upstream multi-vCPU contract or are
recorded as a stage block.

## 2. Validation matrix

Each validation is recorded as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason, in
`../../verification/p6-w04-smp-interrupt-routing-sgi-verification.md`.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W04-DV01 → P6-V04 | Entry basis | prerequisite review ([workflow](05-implementation-workflow.md) step 1) | inspect W02/W03 evidence + P3 contracts present; partition resolution recorded | all consumed contracts evidenced; partition resolved by citation | entry integrity; not runtime behavior |
| W04-DV02 → P6-V04/V05 | Target forms and decomposition | unit + sequence tests | each form's validation, decomposition coverage, determinism, unassigned-row rejection | forms behave per contract; encodings reproducible | mechanism vocabulary; not delivery on all topologies |
| W04-DV03 → P6-V04 | Eligibility and failure handling | unit tests + QEMU ineligible-target attempt | send toward an offline/failed pCPU in a declared scenario | whole-send rejection with named error, zero writes, accounted | eligibility safety; not P3 lifecycle itself |
| W04-DV04 → P6-V04 | CPU0→CPU1 Host SGI | QEMU multi-pCPU run | send the validation SGI from the boot pCPU to one secondary; observe receipt | receipt attributable to the intended online pCPU; combined-EOI completion via W03 loop; counters match | target-attributed SGI on QEMU; not real-hardware affinity behavior |
| W04-DV05 → P6-V05 | Reverse and multi-target; accounting | QEMU runs | CPU1→CPU0; explicit multi-target set; declared IRM form; accounting view sampled | determinate target accounting; convergence (or labeled failed-target drift); no unexplained loss | accounting determinacy; not general messaging fitness |
| W04-DV06 → P6-V06 | SPI re-route reaches replacement | QEMU run | disable → change to a second pCPU → enable → assert source → observe new-target receipt; route back | recorded target reached before and after change; no delivery to the old target in the accepted path; events emitted | determinate routing change on QEMU; not load-balancing adequacy |
| W04-DV07 → P6-V06 | Rejection paths | sequence tests + QEMU where constructible | enabled-SPI change, busy-SPI change, ineligible target | named rejections; state unchanged; retry-by-consumer documented | protocol determinacy; not consumer robustness (their designs') |
| W04-DV08 → P6-V04–V06 | Non-policy review | static + behavioral review ([01 §5](01-scope-and-foundations.md)) | search for scheduler/reschedule/wakeup/payload/Guest symbols; inspect the generic-event consumer boundary | zero policy-coupled paths; SGI remains a Host mechanism | non-policy commitment; not P7's future design |
| W04-DV09 → W04 closure | Handoff consumability | consumer review | read the boundary as W11 (scenario inputs present?), W13 (accounting/events sufficient?), P7 (mechanism usable without policy?) | each consumer can act without extending W04 | handoff readiness; not downstream completion |

Explicitly **not run** in W04: Guest SGI scenarios (W07/W08/P8 scope),
scheduler integration (P7), real-hardware SMP (P15), production routing
policy (out of scope).

## 3. Error, security, and observability model

- *Error model:* named send and route rejections with unchanged state;
  accounting divergence is reported data, never auto-corrected; no W04
  path panics.
- *Security model:* Host-internal mechanism with no Guest reachability;
  target namespace is pCPU-only; partition rows prevent accidental use of
  unassigned IDs; no VM-identity coupling exists.
- *Observability model:* sampled send events, route-change events, drift
  report, W03 receipt counters; W13 correlates into P6-V24–V26 evidence.

## 4. Handoff checklist

Before handing W04 to a reviewer, provide:

- the exact changed-file list and any `unsafe` delta (expected: none —
  sysreg/MMIO access via the W02 surface);
- the partition resolution record (which SGI rows are P3 transport, P6
  event, W11 validation, unassigned) with the P3-contract citation;
- W04-DV01…DV09 evidence paths and run status, including explicit
  not-run entries (Guest SGIs, scheduler integration, real hardware,
  IRM-heavy topologies);
- the mechanism contract handed downstream: typed targets, validation-
  before-emission, accounting view semantics, route-change protocol and
  its enable/disable interplay with W03;
- confirmation that no scheduler policy, TLB semantics, message layer,
  load balancing, Guest SGI path, or platform-name branch was implemented;
- open items for W11 (scenario SGI and multi-vCPU prerequisites), W13
  (correlation and latency sampling), P7 (its protocol over the generic
  event SGI is its own design) — without resolving their contracts here.

## 5. Future record and verification locations

Implementation facts: `../p6-w04-smp-interrupt-routing-sgi-record.md`
(created when work starts; not created by this design). Verification
evidence: `../../verification/p6-w04-smp-interrupt-routing-sgi-verification.md`
(created only by real verification work). Neither file exists yet; neither
may claim completion.
