# P1 AArch64 boot contract

**Status:** Proposed assembly of the current W01/W02 boundaries; executed coverage is partial.\
**Scope:** Canonical QEMU `virt` boot-CPU entry, rejection and supplied DTB; no discovery or guest boot.\
**Version:** v0.1.\
**Owner/change context:** P1-W12 assembly of [W01's owning contract](../implementation/p1-w01-reference-boot-contract/01-boot-contract.md) and [implementation selection](../implementation/p1-w01-reference-boot-contract-record.md), 2026-09-25.\
**Supersedes:** None.

The single canonical recipe is `qemu-system-aarch64 -machine
virt,virtualization=on -cpu cortex-a57 -smp 1 -m 128M -nographic -kernel
hypervisor-boot.img`. The image is the 64-byte ARM64 Image header plus the
ELF's extracted raw body, loaded at the reference image base `0x40080000`.
The [W10 converter](../implementation/p1-w10-qemu-boot-regression-record.md)
materializes that image; the P0 ELF remains the build artifact. QEMU 8.2.2 was
the selection environment, not a guarantee for all emulator versions.

At entry, the [W01 E1–E8 table](../implementation/p1-w01-reference-boot-contract/01-boot-contract.md#3-entry-state-contract-what-the-image-may-rely-on-after-the-pre-transfer-tier)
distinguishes checked facts from canonical-path assumptions. Before stack,
BSS or Rust transfer, T1 requires `CurrentEL == EL2` and T2 requires a
nonzero `x0` DTB pointer. Failure emits one `ZELYR P1 BOOT REJECT
reason=EL|DTB` line and stops; no normal runtime marker follows. AArch64
execution, Non-secure state, and disabled MMU/caches are canonical loader
assumptions, not additional pre-transfer checks. The boot CPU alone is in
scope; `x1`–`x3` have no load-bearing meaning.

The DTB pointer is retained without dereference, size/content/alignment
validation or Host Stage-1 mapping. `-m 128M` is a recipe minimum selected
for the image, DTB and 64 KiB stack with slack; P1 does not measure usable
memory at runtime. [P2-W01](../../p2/plans/p2-w01-boot-platform-description-intake.md)
owns validating the active DTB and its lifetime. Firmware/`booti` and real
board delivery are reserved, not supported P1 paths.

[W01 verification](../verification/p1-w01-reference-boot-contract-verification.md)
records contract review but defers the full executable rejection matrix.
[W09 verification](../verification/p1-w09-initialization-sequencing-verification.md)
records one integrated boot and one validation-image NC2 rejection.
[W10 verification](../verification/p1-w10-qemu-boot-regression-verification.md)
records canonical R1 and a same-image 100/100 reference run. [W11 NC1
verification](../verification/p1-w11-negative-fault-validation-verification.md)
separately records two EL2-disabled boots of the identical default image,
both rejected with `reason=EL` before Runtime/Stable. This is not a general
firmware or unsupported-loader matrix.
