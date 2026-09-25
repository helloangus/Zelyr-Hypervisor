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

  .text.boot : {
    __p1_boot_code_start = 0x40080000;
    KEEP(*(.text.boot))
    ASSERT(. <= 0x40081000, "W08 boot text exceeds first page")
    . = ALIGN(4096);
    __p1_boot_code_end = .;
  }

  .p1_vectors : ALIGN(4096) {
    __p1_vectors_start = .;
    KEEP(*(.p1_vectors))
    ASSERT(. - __p1_vectors_start == 2048, "W05 vector table must have sixteen slots")
    . = __p1_vectors_start + 4096;
    __p1_vectors_end = .;
  }

  .text : ALIGN(4096) {
    __p1_text_start = .;
    *(.text .text.*)
    . = ALIGN(4096);
    __p1_text_end = .;
  }

  .rodata : ALIGN(4096) {
    __p1_rodata_start = .;
    *(.rodata.boot)
    *(.rodata .rodata.*)
    . = ALIGN(4096);
    __p1_rodata_end = .;
  }

  .data : ALIGN(4096) {
    __p1_data_start = .;
    *(.data .data.*)
    *(.got .got.*)
    *(.data.rel.ro .data.rel.ro.*)
  }

  .bss : ALIGN(4096) {
    __p1_bss_start = .;
    *(.bss .bss.*)
    *(.sbss .sbss.*)
    *(COMMON)
    . = ALIGN(4096);
    __p1_data_end = .;
  }

  .p1_boot_stack (NOLOAD) : ALIGN(4096) {
    __p1_stack_start = .;
    KEEP(*(.p1_boot_stack))
    . = ALIGN(4096);
    __p1_stack_end = .;
    __p1_bss_end = .;
  }

  ASSERT(__p1_stack_end - __p1_stack_start == 65536, "W08 boot stack must be 64 KiB")
  ASSERT(__p1_stack_end <= 0x40200000, "W08 image exceeds one L3 window")

  /DISCARD/ : {
    *(.eh_frame .eh_frame.*)
    *(.eh_frame_hdr)
    *(.note .note.*)
    *(.comment)
    *(.interp)
  }
}
