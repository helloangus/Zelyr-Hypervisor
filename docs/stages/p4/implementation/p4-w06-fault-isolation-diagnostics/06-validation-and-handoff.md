# P4-W06 Validation, Error/Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W06 detailed design](README.md).

## 1. Validation matrix

Diagnosis correctness is proven host-side; isolation and containment are
proven on target through the
[P4-W05](../p4-w05-validation-guest/README.md) scenarios, recorded by
[P4-W08](../p4-w08-qemu-integration-regression/README.md), and counted by
[P4-W07](../p4-w07-repeatability-telemetry/README.md). This matrix defines
what W06's evidence must show; it is a plan until the verification record
exists.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W06-DV01 | P4-E02 categorization | decode unit tables + review | synthesized ESR sweep incl. unknown/reserved | every W04 class decodes with documented detail; unknowns represented | decode correctness; not on-target routing |
| W06-DV02 | P4-F04 address reconstruction | unit tests + revision review | boundary-width FAR/HPFAR pairs; unavailable cases | composition per pinned revision; `Unavailable` never guessed | reconstruction rule; not hardware fidelity beyond QEMU agreement |
| W06-DV03 | Context completeness (Guest/vCPU, PC, state, IPA, access, mapping, reason) | field review + unit test | assert every `ExitDiagnostic` field populated or explicitly unavailable | no silently empty field on any class row | context contract; not that every future field need is met |
| W06-DV04 | Agreement classification | exhaustive pair tests | status family × query result × access | every pair classified per [03 §3.3](03-code-contracts-exit-classification.md); Mismatch never silently dropped | stale-translation detection logic; not that the property holds on target (DV08) |
| W06-DV05 → P4-V06 | Defined exits distinguishable (translation/permission/WFI/WFE/illegal/unknown) | on-target IS rows via W05/W08 | run scenarios; compare `ExitDiagnostic` per row | each class yields a distinct record shape with `Match` | distinguishability for P4's classes; not a full AArch64 fault taxonomy |
| W06-DV06 → P4-V08 | Permission write/execute distinguishable; immediate effect | on-target IS-04/IS-05 | marked RO store; marked XN execute; protect-then-access ordering per W02 | distinct access types in records; both `Match`; no intervening unmap needed | write/execute enforcement diagnosed; not all permission corner cases |
| W06-DV07 → P4-V07 | Unmapped, RAM-boundary, Hypervisor-owned range blocked; EL2 live after stop | on-target IS-01/IS-02/IS-03 | three probe configurations (O1 channel); post-stop liveness via W07 run record | each faults as `Stage2Translation`, probe membership held, `Match`, EL2 continues | Stage-2 blocks the probed ranges and stays diagnosable; not DMA/IOMMU isolation or every host range |
| W06-DV08 → P4-V07/V08 | No stale-translation reliance | Mismatch monitor across all isolation runs | `mapping_agreement` audited in every run record | zero `Mismatch` verdicts; the injected Mismatch unit case is caught | ledger/hardware agreement for the probed paths; not future-recycling correctness |
| W06-DV09 → P4-V09 | Guest-fault containment; domain separation | containment review + on-target fault runs | review the two capture paths; run IS-06/IS-07; audit that no Guest fault reached the fatal path | every Guest fault stopped VM-facing with retained diagnosis; host path untouched | containment for exercised classes; not an exhaustive fault-injection campaign |
| W06-DV10 | Unclassified fail-closed | unit + on-target VG-011 (if delivered) | `UnknownSync` record retains raw ESR; stop is VM-facing | `Unclassified` never re-enters, never panics, fully retained | fail-closed behavior for unknowns; not unknown-condition completeness |
| W06-DV11 | Expectation/matrix coherence | joint review with W05/W07/W08 | `IS-T1` vs `VG-T<n>` vs run-record consumers, same versions | no drift; refinements acknowledged per W05 §6 | expectation provenance; not scenario correctness itself (W05) |
| W06-DV12 | No-unsafe and layering review | static review | audit W06 surface vs inventory; module layering | zero new `unsafe`; no descriptor bits/sysregs outside `exit-decode`; no board/QEMU names | controlled surface; not functional correctness |

Record each row as **passed / failed / blocked / not run** with command or
review input, environment, date, and reason. On-target rows prove the QEMU
reference environment only, never real-hardware semantics (W01 A7). Rows
DV04–DV09 are blocked until the M2–M6 upstream evidence exists; blocked rows
name the waiting W01 row.

## 2. Error model

- Guest-caused conditions are values (`ExitDiagnostic`, `MatchVerdict`),
  never errors and never host-fatal (W01 A2); `Unclassified` and `Mismatch`
  are data that fail evidence, not the Hypervisor.
- Host-authored contract violations (`ProbeInsideMappedRegion`,
  `ProbeMisaligned`, decode cross-check divergence) are fatal
  episode-setup-class conditions escalating through the P0 failure
  classification (M5) — they indicate a broken build contract, not a Guest
  event.
- `FactsUnavailable` and upstream signature mismatches are blocked
  prerequisites recorded per W01 §4; W06 never substitutes guesses.
- `ReportError::BufferTooSmall` surfaces through the annotated-truncation
  fallback; a report or dump is never silently absent.

## 3. Security model

- The Guest is untrusted at this boundary: every Guest-influenced value
  (syndrome, PC, IPA) is data compared against host-authored expectations;
  nothing derived from the Guest is dereferenced, executed, or used as a Host
  address.
- The Guest/Hypervisor domain split is structural (capture path), so a Guest
  cannot choose its diagnostic domain; the standing review (DV09) audits the
  split.
- The probe-membership guard keeps negative tests honest: a misconfigured
  probe fails setup loudly instead of silently weakening P4-V07 evidence.
- W06 adds no `unsafe`, no new privilege surface, and no Guest-reachable
  interface (the Guest cannot trigger diagnosis directly; only its marked
  triggers, classified by W04, do).
- Standing scope boundaries: no HVC/management error ABI (P5), no vGIC/timer
  fault classes (P6), no device/DMA isolation — pressure to add any here is a
  stage-boundary violation to record.

## 4. Observability model

- Events `diag.exit`, `diag.fault`, `diag.isolation.verdict`, `diag.report`,
  `diag.dump` route through the P0 logging/trace baseline with build identity
  (W01 A8); field lists are the W07/W08-agreed set from workflow step 5.
- The retained `ExitDiagnostic` plus W04's retained frame is the post-stop
  post-mortem source; re-rendering is pure and repeatable.
- The human report (`HV-DIAG` block) and the ledger dump are bounded,
  truncation-annotated, and never carry Guest data content beyond
  addresses/classifications required for diagnosis.
- Verification claims live only in
  `../../verification/p4-w06-fault-isolation-diagnostics-verification.md`;
  design documents and implementation records carry no run evidence.

## 5. Handoff checklist

Before handing W06 work to a reviewer:

- exact changed-file list and implementation-record path
  (`../p4-w06-fault-isolation-diagnostics-record.md`);
- DV01–DV12 statuses with explicit not-run/blocked entries and the waiting
  upstream rows (M1–M6) each blocked item waits on;
- confirmation: zero new `unsafe`; no changes to W04 frame/action policy, W02
  mapper/ledger semantics, or W05 scenario bodies/markers;
- matrix and protocol versions recorded (`IS-T1`, the VG table version it
  was reviewed against) with open items O1/O2 status;
- Specification Investigation items (any QEMU/architecture divergences)
  listed with their resolution state;
- handoff to consumers: `diag.*` event fields and verdict vocabulary to
  [P4-W07](../p4-w07-repeatability-telemetry/README.md); IS-series
  expectations and verdict semantics to
  [P4-W08](../p4-w08-qemu-integration-regression/README.md); factual
  capability/limitation notes to
  [P4-W09](../p4-w09-closeout-p5-handoff/README.md);
- open items carried forward: O1 probe carriage, O2 joint-review
  acknowledgment, P2-ACR-01 unchanged and unresolved.
