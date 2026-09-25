# P1-W08 page-separated image-layout implementation record

**Status:** Linker/layout foundation; Stage-1 activation not implemented.
**Scope:** Required page boundaries for the static W08 mapping inventory.
**Version:** v0.1
**Owner/change context:** P1-W08 linker foundation, 2026-09-25.
**Supersedes:** W02's 16-byte class-alignment layout through the recorded W08
extension seam; the W01 ARM64 Image entry contract is unchanged.

The [W08 preflight amendment](p1-w08-host-stage1-address-space/00-preflight-amendment.md)
requires page-separated classes. `hypervisor/link.ld` now exports distinct
4 KiB-aligned bounds for the first boot-code page, the dedicated vector page,
the rest of executable text, read-only data, writable data/BSS, and the
64 KiB boot stack. The first boot-code page begins at `0x40080000`; actual
instructions still begin at `0x40080040` after the W01 Image header. Linker
assertions reject boot-code overflow, incorrect stack size, and escape from
the fixed 2 MiB image L3 window.

The stack is in a separate `NOLOAD` output section, contiguous with the BSS
clear range. Its safe `AtomicU8` backing makes the linker section writable
without a `static mut` or a new raw-pointer boundary; assembly still owns
the entire stack after W02's establishment. The BSS clear end remains after
the stack. These symbols are internal inventory inputs, not a promise that
identity mapping persists beyond P1.

No new `unsafe` operation, external ABI, public API, dependency, allocator,
Guest Stage-2 or MMU-enable behavior is introduced. Full W08 must still link
the table model, map these exact bounds, program MAIR/TTBR/TCR/SCTLR, verify
post-MMU memory and collect continuity evidence.
