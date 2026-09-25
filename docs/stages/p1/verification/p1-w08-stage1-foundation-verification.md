# P1-W08 Stage-1 table foundation verification record

**Status:** Pure model checks passed; integrated W08 evidence pending.
**Date:** 2026-09-25 (Asia/Shanghai).
**Implementation:** [Foundation record](../implementation/p1-w08-stage1-foundation-record.md).

`cargo test --workspace --exclude hypervisor` passed 23 host tests, including
five source-shared W08 tests: complete split-code inventory roundtrip,
missing/extra/incorrect descriptors, overlap/alignment/window/table-base
rejection, no writable-executable class, and checked arithmetic/TCR input
bounds. Host and target Clippy with `-D warnings`, target build,
`cargo fmt --all -- --check`, and `git diff --check` passed. The target
build compiles the address types but not the unlinked page-table model.

No linker-layout/page-separation validation, physical table readback,
MAIR/TCR/TTBR/SCTLR activation, post-MMU sentinel, vector/console continuity,
NC5 fault or real QEMU mapped boot has run. These are required for the full
W08 package and remain pending; this foundation is not P1-V13/P1-V14 proof.
