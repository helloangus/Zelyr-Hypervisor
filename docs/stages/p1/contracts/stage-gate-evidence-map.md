# P1 stage-gate evidence map and completion checklist

**Status:** Proposed evidence-location map; revised P1 completion review remains pending.\
**Scope:** P1-V01–V21 and seven exit conditions, not a completion report.\
**Version:** v0.2.\
**Owner/change context:** P1-W12 review of the [amended task book](../task-book-v0.2.md) under [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md), 2026-09-26.\
**Supersedes:** v0.1 of this evidence map.

The [task-book success conditions](../task-book-v0.2.md#6-stage-validation-matrix)
remain authoritative. “Partial” below denotes only what the linked record
actually reports. W10 and W11 are included from merged `main` at `2f821e0`.
W11's earlier foundation and exploratory sections are dated snapshots; its
W10-integrated acceptance review and W10's NC4 addendum own the current
NC1–NC5, R2 and S1–S6 statuses. Local QEMU/source checks are not real-board
proof or a substitute for NC6. Historical W11 findings predate ADR-061 and
remain unchanged; the revised P1 gate excludes NC6 execution, now P6-V29.

| Gate | Evidence location | Current assessment / missing proof |
|---|---|---|
| P1-V01 | [W01](../verification/p1-w01-reference-boot-contract-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md), [W10](../verification/p1-w10-qemu-boot-regression-verification.md) | Canonical R1 passed and the same image reached Stable in 100/100 reference boots; unsupported-entry coverage is a separate V02 gate. |
| P1-V02 | [W01](../verification/p1-w01-reference-boot-contract-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | NC1 EL2-disabled reference profile rejected the identical default image 2/2 with `reason=EL` before Runtime/Stable; other unsupported loader/firmware paths are not inferred. |
| P1-V03 | [W02](../verification/p1-w02-minimal-rust-el2-runtime-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md) | Partial: establishment review and one integrated ordered path. |
| P1-V04 | [W02](../verification/p1-w02-minimal-rust-el2-runtime-verification.md), [W10](../verification/p1-w10-qemu-boot-regression-verification.md) | Repeat-boot observation: 100/100 reached Stable on one image/reference host; hidden initial-register independence remains bounded by W02 source review rather than a varied-firmware test. |
| P1-V05 | [W03](../verification/p1-w03-aarch64-capability-inventory-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md) | Host classification tests and one integrated boot-CPU fact report recorded; no varied-CPU or real-hardware classification claim. |
| P1-V06 | [W03](../verification/p1-w03-aarch64-capability-inventory-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | NC2 selected required-granule sample rejected 2/2 at `capabilities.enter`; W03 host policy tests cover optional/required distinction. This is not CPU-hardware absence. |
| P1-V07 | [W04](../verification/p1-w04-el2-architectural-state-baseline-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md), [W10](../verification/p1-w10-qemu-boot-regression-verification.md) | Partial: source/host review and one integrated baseline observation; 100 Stable boots do not include per-cycle control-register readback. |
| P1-V08 | [W05](../verification/p1-w05-el2-exception-entry-baseline-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | **Structurally supported locally:** 16-slot static/host coverage and NC3/NC5 synchronous execution; this does not prove genuine asynchronous delivery, which is P6-V29. |
| P1-V09 | [W05](../verification/p1-w05-el2-exception-entry-baseline-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | NC3 and NC5 paired synchronous faults expose ESR, phase and location/FAR validity, then terminate; other exception classes are not inferred. |
| P1-V10 | [W06](../verification/p1-w06-early-console-logging-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md), [W10](../verification/p1-w10-qemu-boot-regression-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | One full replay/live-marker capture, start/Stable tokens in 100 boots, and phase-attributed NC2–NC5 fatal captures including post-MMU NC5; pre-vector gap remains declared. |
| P1-V11 | [W07](../verification/p1-w07-fatal-crash-diagnostics-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | NC2/NC4 panic and NC3/NC5 exception pairs checked build, CPU/EL, phase, relevant PC/return/syndrome/FAR and register fields; NC6 category remains unproven. |
| P1-V12 | [W07](../verification/p1-w07-fatal-crash-diagnostics-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | Partial: bounded/non-recursive mechanism review and NC2–NC5 paired single-END/no-later-marker terminal captures; recursive physical-fault stress was not run. |
| P1-V13 | [W08](../verification/p1-w08-mmu-activation-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | W08 closed mapping model and linked placement plus W11 S2 default-image section review show no W+X; temporary VA=PA is not a permanent ABI. |
| P1-V14 | [W08](../verification/p1-w08-mmu-activation-verification.md), [W09](../verification/p1-w09-initialization-sequencing-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | Reference-QEMU mapped boot and NC5 paired post-MMU data-abort report, ESR.EC `0x25`, FAR `0x5000_0000`, phase `stage1.complete`; no real-hardware cache/TLB claim. |
| P1-V15 | [W09](../verification/p1-w09-initialization-sequencing-verification.md), [W10](../verification/p1-w10-qemu-boot-regression-verification.md) | Partial: source/host lifecycle review and one full ordered boot; 100 start/Stable repetitions do not inspect every intermediate transition. |
| P1-V16 | [W10 and NC4 addendum](../verification/p1-w10-qemu-boot-regression-verification.md), [W11](../verification/p1-w11-negative-fault-validation-verification.md) | **Supported locally:** R1–R6 controls recorded; NC4 intentional-panic image replaced historical provisional NC2 R2, yielding expected `FAIL-PANIC` with retained capture. |
| P1-V17 | [W10](../verification/p1-w10-qemu-boot-regression-verification.md) | **Supported for the recorded image/environment:** accepted `r100-accepted` run returned 100/100 sequential PASS, exactly one Stable and no forbidden class per cycle; local raw evidence must remain available. |
| P1-V18 | [W11](../verification/p1-w11-negative-fault-validation-verification.md) | **Supported locally under ADR-061's revised scope:** NC3–NC5 paired cases passed. NC1/NC2 support V02/V06. Historical NC6 remains unexecuted and is required for P6-V29; no synchronous proxy counts. |
| P1-V19 | [W11](../verification/p1-w11-negative-fault-validation-verification.md), [unsafe inventory](../../../security/unsafe-inventory.md) | **Supported locally:** S1–S6 source/linked-image review passed on W10-integrated baseline, including U-016/U-017 static review and default trigger containment; not hardware security proof. |
| P1-V20 | [W12 review](../verification/p1-w12-p1-documentation-handoff-verification.md) | **Supported locally:** eight-document consistency/content/P2-consumability review passed; PR #57 checks passed. Underlying runtime gates remain separate. |
| P1-V21 | [W12 review](../verification/p1-w12-p1-documentation-handoff-verification.md) | **Supported locally:** plan/graph/link, evidence-map and no-claim review passed; PR #57 checks passed. Map completion is not stage completion. |

## Seven exit conditions

| [Task-book §7](../task-book-v0.2.md#7-exit-criteria-and-handoff) condition | Required gate evidence | Current evidence status and remaining boundary |
|---|---|---|
| 1. Stable Non-secure EL2 Rust entry | V01, V03, V04, V15 | **Locally supported:** W09 ordered boot and W10 same-image 100/100 Stable boots; this is reference-QEMU evidence, not a real-board claim. |
| 2. Capabilities and EL2 baseline | V05–V07 | **Locally supported with scope limit:** W03 classification, W09 integrated report, W11 paired injected NC2 and W04 control-state review; no varied-CPU/hardware-absence or per-cycle register-readback claim. |
| 3. Vectors, console and fatal diagnostics before/after MMU | V08–V12, V14, V18 | **Locally supported with scope limit:** structural vector coverage, NC3/NC5 and marker/report paths have local evidence. Genuine asynchronous delivery remains unproved and is a P6-V29 gate. |
| 4. Mapping classes and no permanent identity promise | V13, V14, V19 | **Locally supported with scope limit:** W08 classes, W11 S2 no-W+X review and paired NC5; no permanent identity ABI or real-hardware cache/TLB claim. |
| 5. Automated verdict and 100 clean boots | V16, V17 | **Locally supported:** R1–R6 including NC4 R2 and the accepted one-image 100/100 reference run; raw evidence is local, not a durable CI artifact. |
| 6. Unsupported entry, sync fault, panic and post-MMU fault; structural unexpected-vector coverage | V02, V06, V08, V09, V11, V12, V18 | **Locally supported with scope limit:** NC1–NC5 pairs and structural vector review are recorded. NC6 genuine unexpected-vector execution remains absent and is required by P6-V29. |
| 7. Contracts, limitations, reporting and P2 handoff | V19–V21 | **Locally supported:** W11 S1–S6 and W12 V20/V21 documentation review passed within their stated limits; PR #57 checks passed. The ADR-061 transfer must be reflected in the completion review and handoff. |

## Completion review questions and decision rule

Before any P1 completion report, the reviewer records in a future
`../verification/p1-completion-report.md` which entry assumptions were
eliminated, which reference assumptions remain, whether every fatal route
retains phase/location/syndrome as applicable, whether the Host Stage-1
identity window is treated only as temporary, and whether Guest/SMP/GIC/
discovery/allocator/board-runtime mechanisms stayed outside P1. Each answer
must cite the final integrated package evidence. That completion-report path
is a **future location**, not an existing artifact or authority to claim
success. All revised V01–V21 and all seven exit conditions require evidence.
ADR-061 removes NC6 from P1-V18 but does not itself constitute a P1 completion
decision. P6-V29 remains open until genuine asynchronous evidence is accepted.
