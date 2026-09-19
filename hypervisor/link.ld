/*
 * Zelyr hypervisor boot-image layout (P1-W02).
 *
 * Owned by the P1-W02 detailed implementation design; wired through the
 * member build script under the P0 build-target baseline's reserved linking
 * extension position. The load address is the reference boot path's
 * post-header body address: the canonical recipe loads the derived ARM64
 * boot image (64-byte Image header included) at 0x40080000, so the linked
 * body starts at 0x40080040. BSS bounds are consumed by the entry
 * assembly's clear loop; the boot stack lives in .bss as a static.
 */
ENTRY(p1_el2_entry)

SECTIONS
{
  . = 0x40080040;

  .text : {
    KEEP(*(.text.boot))
    *(.text .text.*)
  }

  .rodata : ALIGN(16) {
    *(.rodata.boot)
    *(.rodata .rodata.*)
  }

  .data : ALIGN(16) {
    *(.data .data.*)
    *(.got .got.*)
    *(.data.rel.ro .data.rel.ro.*)
  }

  .bss : ALIGN(16) {
    __p1_bss_start = .;
    *(.bss .bss.*)
    *(.sbss .sbss.*)
    *(COMMON)
    . = ALIGN(16);
    __p1_bss_end = .;
  }

  /DISCARD/ : {
    *(.eh_frame .eh_frame.*)
    *(.eh_frame_hdr)
    *(.note .note.*)
    *(.comment)
    *(.interp)
  }
}
