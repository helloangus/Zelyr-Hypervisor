# P1 implementation designs and records

**Status:** Index of P1 designs, implementation records and local verification;
not a stage-completion claim.
**Scope:** P1 stage-local implementation material only.

Read the P1 task book and selected work-package plan before using an
item here.  For code changes, the repository `AGENTS.md` and the mandatory
[Coding Guidelines](../../../development/coding-guidelines.md) still apply.

| Work package | Detailed design / record | Status |
|---|---|---|
| P1-W01 | [Reference boot contract detailed implementation design](p1-w01-reference-boot-contract/README.md) and [implementation record](p1-w01-reference-boot-contract-record.md) | Implemented; verification evidence recorded |
| P1-W02 | [Minimal Rust EL2 runtime detailed implementation design](p1-w02-minimal-rust-el2-runtime/README.md) and [implementation record](p1-w02-minimal-rust-el2-runtime-record.md) | Implemented; verification evidence recorded |
| P1-W03 | [AArch64 capability inventory detailed implementation design](p1-w03-aarch64-capability-inventory/README.md) and [implementation record](p1-w03-aarch64-capability-inventory-record.md) | Mechanism implemented; W11 NC2 injected-fact pair passed locally, not CPU hardware absence |
| P1-W04 | [EL2 architectural-state baseline detailed implementation design](p1-w04-el2-architectural-state-baseline/README.md) and [implementation record](p1-w04-el2-architectural-state-baseline-record.md) | Mechanism implemented; W09 integrated boot and W10 100 Stable boots observed, without per-cycle register readback |
| P1-W05 | [EL2 exception entry baseline detailed implementation design](p1-w05-el2-exception-entry-baseline/README.md) and [implementation record](p1-w05-el2-exception-entry-baseline-record.md) | Structural vector coverage and W11 NC3/NC5 synchronous pairs locally supported; NC6 execution remains open at P6-V29 under ADR-061 |
| P1-W06 | [Early console and bring-up logging detailed implementation design](p1-w06-early-console-logging/README.md) and [implementation record](p1-w06-early-console-logging-record.md) | W09 replay/live markers, W10 start/Stable repetition and W11 fault-phase captures recorded locally |
| P1-W07 | [Fatal crash diagnostics detailed implementation design](p1-w07-fatal-crash-diagnostics/README.md) and [implementation record](p1-w07-fatal-crash-diagnostics-record.md) | W11 NC2–NC5 paired panic/synchronous fatal reports passed locally; NC6 category remains open |
| P1-W08 | [Host Stage-1 address space detailed implementation design](p1-w08-host-stage1-address-space/README.md), [table foundation](p1-w08-stage1-foundation-record.md), [page layout](p1-w08-page-layout-record.md), and [activation mechanism](p1-w08-mmu-activation-record.md) | W09 reached Stage1.complete; W11 NC5 post-MMU pair and no-W+X review passed locally; hardware remains unproven |
| P1-W09 | [Initialization sequencing detailed implementation design](p1-w09-initialization-sequencing/README.md), [foundation record](p1-w09-lifecycle-foundation-record.md), [integration record](p1-w09-initialization-sequencing-record.md) and [verification](../verification/p1-w09-initialization-sequencing-verification.md) | One ordered boot, W10 100 start/Stable observations and W11 NC2–NC5 failure-phase attribution recorded locally |
| P1-W10 | [QEMU boot regression detailed implementation design](p1-w10-qemu-boot-regression/README.md), [implementation record](p1-w10-qemu-boot-regression-record.md) and [verification](../verification/p1-w10-qemu-boot-regression-verification.md) | R1, R3–R6, strict 100-cycle evidence and W11 NC4-based R2 control recorded; P1-V16/V17 reference-QEMU evidence, not hardware |
| P1-W11 | [Negative and fault validation detailed implementation design](p1-w11-negative-fault-validation/README.md), [implementation record](p1-w11-negative-fault-validation-record.md) and [local verification](../verification/p1-w11-negative-fault-validation-verification.md) | NC1–NC5 pairs and S1–S6 review passed locally; ADR-061 revises P1-V18 to NC3–NC5 and transfers still-open NC6 execution to P6-V29. The L7 completion decision is in the [report](../verification/p1-completion-report.md) |
| P1-W12 | [P1 documentation and P2 handoff detailed implementation design](p1-w12-p1-documentation-handoff/README.md), [contract set](../contracts/README.md), [implementation record](p1-w12-p1-documentation-handoff-record.md) and [verification](../verification/p1-w12-p1-documentation-handoff-verification.md) | W12 documentation review and PR #57 checks passed; P1-V20/V21 locally supported, not stage completion |

Actual command logs and pass/fail evidence belong in `../verification/`, not in
this index or a detailed design.

The [W08 constant-initialization correction](../verification/p1-w08-const-initialization-verification.md)
records the debug-build stack issue discovered during P2 integration and its
bounded correction and validation.
