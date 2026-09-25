# P1 implementation designs and records

**Status:** Index of proposed designs and future implementation records; not a
completion claim.
**Scope:** P1 stage-local implementation material only.

Read the P1 task book and selected work-package plan before using an
item here.  For code changes, the repository `AGENTS.md` and the mandatory
[Coding Guidelines](../../../development/coding-guidelines.md) still apply.

| Work package | Detailed design / record | Status |
|---|---|---|
| P1-W01 | [Reference boot contract detailed implementation design](p1-w01-reference-boot-contract/README.md) and [implementation record](p1-w01-reference-boot-contract-record.md) | Implemented; verification evidence recorded |
| P1-W02 | [Minimal Rust EL2 runtime detailed implementation design](p1-w02-minimal-rust-el2-runtime/README.md) and [implementation record](p1-w02-minimal-rust-el2-runtime-record.md) | Implemented; verification evidence recorded |
| P1-W03 | [AArch64 capability inventory detailed implementation design](p1-w03-aarch64-capability-inventory/README.md) and [implementation record](p1-w03-aarch64-capability-inventory-record.md) | Mechanism implemented and host-validated; W09–W11 runtime evidence pending |
| P1-W04 | [EL2 architectural-state baseline detailed implementation design](p1-w04-el2-architectural-state-baseline/README.md) and [implementation record](p1-w04-el2-architectural-state-baseline-record.md) | Mechanism implemented and host-validated; W09/W10 runtime evidence pending |
| P1-W05 | [EL2 exception entry baseline detailed implementation design](p1-w05-el2-exception-entry-baseline/README.md) and [implementation record](p1-w05-el2-exception-entry-baseline-record.md) | Mechanism implemented and host-validated; W07/W09/W11 runtime evidence pending |
| P1-W06 | [Early console and bring-up logging detailed implementation design](p1-w06-early-console-logging/README.md) and [implementation record](p1-w06-early-console-logging-record.md) | Mechanism implemented; W09/W10 execution pending |
| P1-W07 | [Fatal crash diagnostics detailed implementation design](p1-w07-fatal-crash-diagnostics/README.md) and [implementation record](p1-w07-fatal-crash-diagnostics-record.md) | Mechanism target-built; fault/QEMU evidence pending |
| P1-W08 | [Host Stage-1 address space detailed implementation design](p1-w08-host-stage1-address-space/README.md) and [table foundation record](p1-w08-stage1-foundation-record.md) | Pure table foundation host-tested; linker/MMU transition pending |
| P1-W09 | [Initialization sequencing detailed implementation design](p1-w09-initialization-sequencing/README.md) and [lifecycle foundation record](p1-w09-lifecycle-foundation-record.md) | Pure type/tracker foundation only; sequencing and P1-V15 pending |
| P1-W10 | [QEMU boot regression detailed implementation design](p1-w10-qemu-boot-regression/README.md) | Proposed design; implementation not claimed |
| P1-W11 | [Negative and fault validation detailed implementation design](p1-w11-negative-fault-validation/README.md) | Proposed design; implementation not claimed |
| P1-W12 | [P1 documentation and P2 handoff detailed implementation design](p1-w12-p1-documentation-handoff/README.md) | Proposed design; implementation not claimed |

Actual command logs and pass/fail evidence belong in `../verification/`, not in
this index or a detailed design.
