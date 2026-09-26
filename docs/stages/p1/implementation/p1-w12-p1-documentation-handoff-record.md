# P1-W12 contract and P2 handoff implementation record

**Status:** W12 contract set merged through PR #57 after local review and online checks; P1 completion not claimed.\
**Scope:** Sources and editorial decisions for the eight P1 contract documents; no runtime evidence or P1 completion claim.\
**Version:** v0.1.\
**Owner/change context:** P1-W12, 2026-09-26.\
**Supersedes:** None.

## Source inventory at the W10/W11-integrated baseline

This inventory was first assembled on the W12 branch based on `main` at
`249e1b3`, then refreshed for W10/W11 and rebased without conflict onto
`main` at `2f821e0`. The W12 document commits are `e5d0e4f` and `93f44e8`,
merged through PR #57 as `02c1d38`; the final
read-only documentation checks are recorded in [W12
verification](../verification/p1-w12-p1-documentation-handoff-verification.md).
An implementation record fixes its package's delivered boundary; a verification
record fixes only the result it actually reports. Earlier records can retain
historical “pending W09” wording even where the later [W09 integrated
record](p1-w09-initialization-sequencing-record.md) supplies that particular
integration evidence. No row upgrades a local check into a stage gate.

| Package | Owning implementation source and assembled boundary | Verification source and remaining item |
|---|---|---|
| W01 | [Boot design](p1-w01-reference-boot-contract/01-boot-contract.md) and [record](p1-w01-reference-boot-contract-record.md): one ARM64 Image recipe, pre-transfer EL2/nonzero-DTB checks, retained raw DTB. | [W01 verification](../verification/p1-w01-reference-boot-contract-verification.md): review passed; [W11 NC1](../verification/p1-w11-negative-fault-validation-verification.md) adds an EL2-disabled paired rejection, not a general firmware matrix. |
| W02 | [Runtime design](p1-w02-minimal-rust-el2-runtime/README.md) and [record](p1-w02-minimal-rust-el2-runtime-record.md): stack/BSS, panic/identity, boot-context transfer and controlled idle; W09 owns the later phase sequence. | [W02 verification](../verification/p1-w02-minimal-rust-el2-runtime-verification.md): local review and informative probes; [W10](../verification/p1-w10-qemu-boot-regression-verification.md) records 100 same-image boots reaching Stable, not all hidden-state properties. |
| W03 | [Capability design](p1-w03-aarch64-capability-inventory/README.md) and [record](p1-w03-aarch64-capability-inventory-record.md): typed fact/classification report and required/optional distinction. | [W03 verification](../verification/p1-w03-aarch64-capability-inventory-verification.md): host evidence; integrated report observed by [W09](../verification/p1-w09-initialization-sequencing-verification.md); [W11 NC2](../verification/p1-w11-negative-fault-validation-verification.md) paired required-fact sample rejection is not absent CPU hardware. |
| W04 | [EL2 baseline design](p1-w04-el2-architectural-state-baseline/README.md) and [record](p1-w04-el2-architectural-state-baseline-record.md): explicit controls, guarded writes and readback. | [W04 verification](../verification/p1-w04-el2-architectural-state-baseline-verification.md): local checks; W09 observed one integrated boot and W10 observed 100 Stable boots, without per-cycle architectural readback. |
| W05 | [Vector design](p1-w05-el2-exception-entry-baseline/README.md) and [record](p1-w05-el2-exception-entry-baseline-record.md): 16 vector slots, bounded frame and category/origin classification. | [W05 verification](../verification/p1-w05-el2-exception-entry-baseline-verification.md): static/host checks; [W11 NC3/NC5](../verification/p1-w11-negative-fault-validation-verification.md) execute synchronous paths; NC6 unexpected category remains blocked. |
| W06 | [Console design](p1-w06-early-console-logging/README.md) and [record](p1-w06-early-console-logging-record.md): reference PL011 and structured phase transport. | [W06 verification](../verification/p1-w06-early-console-logging-verification.md): mechanism checks; W09 observed full replay/live markers once; W10 recorded start/Stable tokens 100 times; [W11](../verification/p1-w11-negative-fault-validation-verification.md) captured phase-attributed NC2–NC5 failures, including post-MMU NC5. |
| W07 | [Fatal design](p1-w07-fatal-crash-diagnostics/README.md) and [record](p1-w07-fatal-crash-diagnostics-record.md): bounded, non-recursive panic/exception/phase terminal reports. | [W07 verification](../verification/p1-w07-fatal-crash-diagnostics-verification.md): earlier panic probe; [W11 NC2–NC5](../verification/p1-w11-negative-fault-validation-verification.md) paired verdicts check relevant fields, one END and no later marker, but not recursive physical-fault stress. |
| W08 | [Mapping/transition design](p1-w08-host-stage1-address-space/README.md), [table](p1-w08-stage1-foundation-record.md), [layout](p1-w08-page-layout-record.md), [activation](p1-w08-mmu-activation-record.md): fixed classes, page-separated image and one-way Stage-1 activation; identity VA=PA is temporary. | [W08 activation verification](../verification/p1-w08-mmu-activation-verification.md) and [W09](../verification/p1-w09-initialization-sequencing-verification.md): model/layout and one mapped boot; [W11 NC5/S2](../verification/p1-w11-negative-fault-validation-verification.md) adds paired post-MMU fault and no-W+X linked review. |
| W09 | [Lifecycle design](p1-w09-initialization-sequencing/README.md), [foundation](p1-w09-lifecycle-foundation-record.md), [integration record](p1-w09-initialization-sequencing-record.md): ordered phase tracker, deferred-marker replay, failure routes and Stable handoff. | [W09 verification](../verification/p1-w09-initialization-sequencing-verification.md): one full ordered boot; [W10](../verification/p1-w10-qemu-boot-regression-verification.md) adds 100 start/Stable observations; [W11](../verification/p1-w11-negative-fault-validation-verification.md) adds NC2–NC5 failure-phase attribution, not per-cycle intermediate-phase audit. |
| W10 | [Regression design](p1-w10-qemu-boot-regression/README.md) and [merged record](p1-w10-qemu-boot-regression-record.md): one runner entry, bounded verdict grammar, evidence retention and R4 marker-only control. | [W10 verification and NC4 addendum](../verification/p1-w10-qemu-boot-regression-verification.md): R1–R6 recorded with NC4 replacing provisional NC2 R2; same-image 100/100 run supports P1-V17 on its named reference host; P1-V16 locally supported. |
| W11 | [Fault design](p1-w11-negative-fault-validation/README.md) and [record with integrated addendum](p1-w11-negative-fault-validation-record.md): default-off NC2–NC5 variants, fixed NC1 no-EL2 profile and exact verdict layer, no second QEMU launcher. | [W11 verification](../verification/p1-w11-negative-fault-validation-verification.md): NC1–NC5 pairs and S1–S6 source/linked-image review passed locally; NC6 has no accepted genuine unexpected-vector proof and blocks P1-V18/stage completion. |

## W12 artifacts and assembly boundary

The eight documents in [contracts/](../contracts/README.md) are the only
contract-set outputs. The stage-local directory is the [W12 detailed-design
placement](p1-w12-p1-documentation-handoff/README.md); repository-wide
taxonomy ratification remains a coordination item, not a new class decision.
The [evidence map](../contracts/stage-gate-evidence-map.md) distinguishes
existing records, open execution gates, and the future completion-report
location. P2 consumer names come from the [P2 task book](../../p2/task-book-v0.1.md)
and [plan index](../../p2/plans/README.md); W12 freezes no P2 API.

No conflicting accepted technical source was resolved editorially in
this assembly. The material open finding is W11 NC6: the task-book unexpected
vector criterion has no accepted reproducible Stable-state injection within
P1's no-GIC/IRQ boundary. It remains an explicit block on P1-V18 and P1
completion. Both W10 and W11 merged results are reflected above. The NC6
blocker is not an editorial contradiction to resolve inside W12; it is an
unmet execution criterion requiring a future approved event source within
the stage boundary or an explicit governance decision outside this package.

## Change report and review handoff

W12 changes documentation only. It adds no `unsafe`, production code, ABI,
public Rust API, external dependency, ADR decision, or P2 mechanism. The
document-set, source, link, coverage, governance and no-claim review outcomes
belong in [W12 verification](../verification/p1-w12-p1-documentation-handoff-verification.md).
That review can establish P1-V20/P1-V21 documentation conditions; it cannot
establish the missing execution evidence for other stage gates.

## 2026-09-26 governance addendum — ADR-061

The source inventory and W12 review above describe the v0.1 acceptance
baseline at PR #57. [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md)
and [P1 task book v0.2](../task-book-v0.2.md) subsequently revise P1-V18 to
NC3–NC5 and transfer unexecuted NC6 to P6-W12/P6-V29. The current
[evidence map](../contracts/stage-gate-evidence-map.md) and
[limitations](../contracts/known-limitations.md) carry that change. This
addendum does not create runtime evidence or declare P1 complete.
