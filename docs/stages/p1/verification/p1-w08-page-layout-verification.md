# P1-W08 page-separated image-layout verification record

**Status:** Target layout and pre-MMU boot probe passed; mapped execution pending.
**Date:** 2026-09-25 (Asia/Shanghai).
**Implementation:** [Layout record](../implementation/p1-w08-page-layout-record.md).

The target ELF built successfully. `llvm-nm -n` showed consecutive bounds:
boot code `0x40080000..0x40081000`, vectors
`0x40081000..0x40082000`, text `0x40082000..0x40089000`, rodata
`0x40089000..0x4008b000`, data/BSS `0x4008b000..0x4008c000`, and stack
`0x4008c000..0x4009c000`. `llvm-readelf -S` showed separate page-aligned
sections; BSS and stack were both writable `NOBITS`, and the stack was
exactly 64 KiB. The W02 BSS clear span ends at the stack end.

The W10 draft converter produced image SHA-256
`e05c2747a42825ae3ee04e4cb88cf8c19e764206f4bd2ffd5aa94f0570e928a0`.
One real QEMU `virt`/Cortex-A57 boot reached the known pre-W09 terminal
panic, emitted `cpu=EL2 el_ok=true` and `ZELYR P1 REPORT END`, and the draft
runner classified it as forbidden-marker in 0.24 seconds. The local capture
is under `target/p1-layout/layout-probe-evidence/`. This confirms the new
layout did not prevent the existing pre-MMU path from reaching the W07 panic
report; it does **not** prove Stage-1 translation or post-MMU continuity.

Host tests, host/target Clippy with `-D warnings`, formatting, and diff
checks passed for this change. P1-V13/P1-V14 remain unproven until full W08
activation and W10/W11 execution.
