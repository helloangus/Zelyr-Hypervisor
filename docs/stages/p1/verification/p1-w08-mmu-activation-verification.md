# P1-W08 Stage-1 activation mechanism verification record

**Status:** Compilation/model validation passed; executed MMU transition pending.
**Date:** 2026-09-25 (Asia/Shanghai).
**Implementation:** [W08 activation record](../implementation/p1-w08-mmu-activation-record.md).

On branch `p1/w08-mmu-activation`, after the independent AP[1] review fix:

- `cargo fmt --all` passed.
- `cargo test --workspace --exclude hypervisor` passed: 27 host tests in total,
  including six W08 source-shared tests. The W08 class test explicitly checks
  non-VHE EL2 AP[1] bit 6 for every mapping class.
- Host `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`
  passed.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor` passed.
- Target `cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings`
  passed.
- `git diff --check` passed.

The independent architecture review identified an omitted AP[1] RES1 bit in
the first implementation; the code and test above incorporate the correction.
The reviewer also checked MAIR/TCR/XN and barrier/control ordering against
Arm-maintained reference definitions. Final review disposition is recorded in
the unsafe inventory; this paragraph alone is not an acceptance claim.

W09 does not yet call W08. The linker may discard this dormant path, so the
current boot ELF cannot establish final `.rodata` sentinel placement, the
physical static table address, emitted sentinel accesses, or generated stack
peak. No QEMU boot with SCTLR.M set, physical table readback, negative fault,
or hardware trace was collected in this change. W09 integration must make the
path reachable; W10/W11 then collect real P1-V13/P1-V14 and fault evidence.
This checkpoint does not close the full W08 acceptance criteria.
