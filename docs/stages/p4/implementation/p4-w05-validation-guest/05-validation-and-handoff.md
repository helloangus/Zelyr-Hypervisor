# P4-W05 Validation, Error/Security Model, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W05 detailed design](README.md).

## 1. Validation matrix

The Guest is validated as (a) a buildable, maintained asset and (b) the
trigger/marker source for the boundary evidence of P4-V05/V06/V08/V09. The
scenario rows are executed through the W02/W03/W04 paths on the QEMU
reference platform and recorded by
[P4-W08](../p4-w08-qemu-integration-regression/README.md); this matrix
defines what W05's evidence must show.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W05-DV01 | P4-D01 loadable asset | build review + route check | build produces the flat binary; W03 embedding consumes it unchanged | artifact loads and `_start` runs (VG-001 path) | build/load route works; not hypervisor correctness |
| W05-DV02 | P4-D02/D03 EL1 marker + CurrentEL | VG-001 on target | banner/greeting markers; CurrentEL assertion | `Hello from EL1` observable; CurrentEL = EL1; clean completion stop | P4-V05 positive path |
| W05-DV03 | P4-D04 RAM/stack/code execution | VG-002/VG-003 on target | pattern read-back; bounded stack frames | `VG-002:OK`, `VG-003:OK` with no faults | mapped RAM/stack/code work for the Guest |
| W05-DV04 → P4-V06/V07 | unmapped trigger | VG-004 on target | single marked load at the probe address | `Stage2Translation` exit; faulting IPA = probe; EL2 diagnosable; stop `GuestFault` | trigger fidelity and frame correlation (with W04 DV09); not isolation completeness (W02/W06) |
| W05-DV05 → P4-V08 | permission triggers | VG-005/VG-006 on target | marked RO store; marked XN execute | `Stage2Permission` exits distinguishable by access type; immediate effect | permission scenarios distinguishable; not all permission corner cases |
| W05-DV06 → P4-V09 | WFI/WFE defined results | VG-007 (+VG-008 if delivered) | marked WFI/WFE | classified `Wfi`/`Wfe` exit with defined stop; diagnosable | defined results per task book |
| W05-DV07 → P4-V06/V09 | controlled illegal + unknown sync | VG-010 (+VG-011 if delivered) | marked illegal instruction; marked SVC | `IllegalExecution` / `UnknownSync` exits, VM-facing, Host alive | Guest faults stay Guest-facing; not full fault taxonomy (W06) |
| W05-DV08 | Completion/stop protocol | VG-012 on target (+ W07 repeat) | banner → OK → WFI → teardown | `Wfi` stop; teardown clean; repeat run stable | stop protocol; not lifecycle policy (P7+) |
| W05-DV09 | Boot-info defensive validation | validator unit suite + injection | all error classes; truncation/bit-flip properties; on-target corrupted-block run if tooling allows | fail-closed `VG-FAIL:BOOTINFO` on every corruption class | Guest-side defensive parsing |
| W05-DV10 | Scenario-table consistency | cross-consistency review | Guest table vs W04 validation table vs W08 expected-marker set, same version | all three derive from the same `VG-T<n>` version | asset coherence; not scenario correctness itself |
| W05-DV11 | Maintenance boundary | governance review | versioning rules, deferral records (if any), ownership statement present | rules met; any deferral has reason/owner/exit-criterion effect per task book §6 | maintained-asset property |
| W05-DV12 | Guest-scope purity | static review | no HVC calls, no DTB, no virtio, no timer/IRQ dependencies; Guest `unsafe` = `_start` + MMIO writer only | all confirmed; SAFETY notes present | stage-boundary compliance |

Record each as **passed / failed / blocked / not run** with command or
review input, environment, date, and reason. On-target rows prove the QEMU
reference environment only, not real-hardware semantics (W01 A7). Planned
scenarios (VG-008/VG-009/VG-011) are validated only if delivered; otherwise
their deferral records are the evidence of record. Rows are plans until the
verification record exists.

## 2. Error model

- **Guest-internal failures** (boot-info invalid, scenario mismatch, Guest
  panic): fail-closed markers (`VG-FAIL:*`, `VG-PANIC:*`) plus WFI halt —
  always diagnosable, never silent (D7).
- **Scenario-designed faults** (VG-004–VG-006, VG-010, VG-011): expected,
  marked, single-instruction triggers whose outcomes are classified exits —
  these are the test's *success* paths, encoded in the scenario table.
- **Environment failures** (stuck console, missing mapping): detected by
  automation timeouts (W08's determinate-failure requirement), not by Guest
  retries — the Guest has no retry logic by design.
- The Guest has no error path that can hang without a marker: every terminal
  sequence emits before halting (review-enforced).

## 3. Security model

- The Guest is the *untrusted party* in every boundary interaction even
  though it is maintained here: all of its fault behavior is designed to be
  contained (that containment is what W02/W04/W06 prove). The asset never
  uses an uncontained operation outside a marked fault scenario.
- Guest-side defensive posture: boot-info validation (D3), scenario-id
  cross-check, no invented addresses (D4). This keeps the test asset honest
  and prefigures P5's guest-copy discipline without implementing it.
- Standing scope boundaries: no HVC convention (P5 owns the ABI), no
  timer/IRQ scenarios (P6), no scheduler interactions (P7), no DTB/PSCI
  (P8), no virtio (P9). Pressure to add any of these is a stage-boundary
  violation to record.
- Provenance integrity: the scenario table records its reconstruction basis
  and version; the canonical-source gap stays an open question
  ([01 §3](01-scope-and-foundations.md)) until the stage owner resolves it.

## 4. Observability model

The Guest's entire observable surface is the marker stream over the console
page plus the behavior W04 captures in exit frames. Marker grammar and
table versions are the stability contract consumed by W06 (correlation),
W07 (repeat), and W08 (automation). Guest events intentionally do not use
the hypervisor trace baseline — the Guest is on the far side of the
boundary; its output *is* data to the Host's observability stack (W01 A8
applies to the Host side, not the Guest binary).

## 5. Handoff checklist

Before handing W05 work to a reviewer:

- exact changed-file list (Guest crate) and implementation-record path
  (`../p4-w05-validation-guest-record.md`) including protocol/table versions
  and the deferral section (if applicable);
- DV01–DV12 statuses with explicit not-run/blocked entries and the waiting
  dependencies (W02/W03/W04 paths, console mapping);
- Guest `unsafe` list (`_start`, MMIO writer) with SAFETY notes;
- confirmation of stage-boundary purity (no P5+ mechanisms used or
  pre-implemented);
- handoff to consumers: controlled triggers and expected frame facts to
  [P4-W06](../p4-w06-fault-isolation-diagnostics/README.md); marker grammar
  and expected sequences to
  [P4-W08](../p4-w08-qemu-integration-regression/README.md); repeat/stop
  scenario surface to
  [P4-W07](../p4-w07-repeatability-telemetry/README.md); facts for
  [P4-W09](../p4-w09-closeout-p5-handoff/README.md);
- open items: scenario-table provenance confirmation (W01 A9), any
  deferral records, any M-series upstream mismatches and their resolutions.
