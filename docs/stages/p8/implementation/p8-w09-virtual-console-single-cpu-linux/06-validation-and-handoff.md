# P8-W09 Validation and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W09 detailed design](README.md).

## 1. Validation matrix

All rows are planned evidence; none asserts that a test has run. Evidence
destination:
`../../verification/p8-w09-virtual-console-single-cpu-linux-verification.md`
(do not create before evidence exists). Stage/Milestone IDs are from
[04](04-single-vcpu-boot-path.md).

| ID | Requirement → test/review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W09-DV01 → gate | machine-fact single-source review | inspect console facts registration | base/INTID/ID constants from approved machine facts; consistent with W04 DTB node (DV01 also covers capacity constants documented) | freeze respected; value correctness is W02/P8-V02/V03 |
| W09-DV02 → P8-V12 (early) | Stage A register behavior | host-side tests + minimal boot M1–M2 | readback/gating/RAZ-WI exact per [02 §2](02-code-contracts-console-frontend.md); earlycon output present | frontend subset correctness; not driver-grade completeness (unimplemented regs are RAZ/WI by design) |
| W09-DV03 → P8-V12/V13 (Stage B) | GIC + RX IRQ round trip | scripted input at kernel stage; M3/M5 markers | pl011 probe/bind marker present; injected kernel-stage input visible | SPI path + RX path; not userspace readiness |
| W09-DV04 → P8-V13 (Stage C) | timer-dependent boot | M4 marker + a timed-sleep marker in boot | clock/clockevent registered; sleep completes ([W08](../p8-w08-linux-timer-integration/README.md) contracts exercised) | timer integration at boot level; not the full W08 matrix |
| W09-DV05 → P8-V13 (Stage D) | interactive initramfs shell | input script: login/prompt → injected command → response | M7 round trip in the retained log: injected command line **and** Guest response present; M0–M7 all reached | the P8-V13 completion bar; not SMP (W10), not distribution content (W15) |
| W09-DV06 → P8-V13 | retained-log fidelity | compare log snapshot against known console traffic; check drop counters | log is a faithful record; drops (if any) counted; marker table matches ([03 §5–§6](03-code-contracts-console-backend-and-input.md)) | evidence integrity; not Host-side export policy |
| W09-DV07 → P8-V12 | containment scenarios | malformed access set ([02 §5](02-code-contracts-console-frontend.md)): out-of-window, bad widths, TX flood, CR toggling, read-only writes | declared contained outcomes only; Host serial/Host state untouched; no other-VM effect (per-VM state); W13 diagnostic context populated | guest-untrusted containment of the console; not whole-VM isolation (W18 owns the aggregate) |
| W09-DV08 → P8-V03 | ownership/isolation review | audit that no Guest path reaches Host serial or cross-VM console state | none exists; backend boundary holds (seam is the only Host touchpoint) | console containment; not general security regression (W18) |
| W09-DV09 → W16/W19 | consumer readiness review | marker table + seam review against W16's automation and W19's reuse needs | marker table sufficient for automated verdicts; seam reusable without Guest-ABI change | handoff readiness; not W16/W19 execution |

Record each validation as passed/failed/blocked/not run with command, input,
environment, timestamp, reason. Reaching `start_kernel` satisfies at most
DV02/DV03 partially — P8-V13 requires DV05 (Stage D interactive round trip)
and DV06. A green Stage D is the precondition [W10](../p8-w10-linux-smp-bringup/README.md)
builds on.

## 2. Error, security, and observability model

- **Error model:** Guest-visible device errors do not exist for the P8
  subset (no framing/overrun generation); contained classes are malformed
  access (W05-classified, VM-scoped diagnostic), backend failure (silent
  device + internal escalation, never a Guest hang — TX is store-and-forward),
  and overflow (counted drops). No console path can panic the Host for
  Guest-reachable input (ADR §19).
- **Security model:** the Guest is untrusted on every access; bytes are
  payload, never control; input is Host-controlled and bounded; log content
  is untrusted data parsed with exact-pattern matching, and the M7 round
  trip plus Hypervisor-side telemetry are the anti-forgery controls
  ([04 §4](04-single-vcpu-boot-path.md)). Host serial ownership is
  structural: the Guest has no path to it (decision D6).
- **Observability:** per-access telemetry classes, TX byte/drop counters,
  RX inject/drop counters, IRQ assertion events, milestone report with
  offsets — all bounded, prunable, runtime-filterable (ADR-048); consumed by
  W13 (diagnostics), W16 (verdicts), W17 (baselines). No unbounded buffers.

## 3. Handoff checklist

Before handing W09 to a reviewer, provide:

- the exact changed-file list and M1–M6 → actual-unit mapping as implemented;
- the approved W02 gate values used (base, INTID, ID constants) and their
  authority location; the documented RX-queue/log capacities;
- DV01–DV09 evidence paths with run status, including explicit not-run
  entries (e.g. DV05 blocked if W03/W15 fixtures are not ready — recorded as
  blocked, not skipped);
- the retained boot log path (evidence artifact) for the highest stage
  reached, with its marker report;
- confirmation that no virtio surface, custom register, Host serial driver,
  or SMP path was introduced; inventory of new `unsafe` with SAFETY
  justification (expected only at arch MMIO-access boundaries owned by the
  arch layer);
- confirmation of seam agreement with W06/W07/W08 designs (Step 5/6 seams);
- open items: W02 gate status; W15 fixture readiness; W19 reuse decision;
  W16 automation consumption — without resolving their contracts here.
