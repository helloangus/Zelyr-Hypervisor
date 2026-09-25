//! Boot entry and establishment for the P1 EL2 runtime.
//!
//! Contracts: the P1-W01 pre-transfer tier and rejection reporter are
//! implemented here verbatim (see the W01 entry-validation design); the
//! establishment sequence, the Rust entry, and the seams are the P1-W02
//! design. Physical placement in this module is recorded in the W02
//! implementation record; extraction into architecture crates is a later
//! design decision.

use core::arch::global_asm;
use core::sync::atomic::AtomicU8;

// The W08 table/transition consumer is not linked in this foundation step.
// Full W08 removes this transitional allowance when the typed helpers are used.
#[allow(dead_code)]
pub(crate) mod address;
pub(crate) mod console;
pub(crate) mod context;
pub(crate) mod fatal;
pub(crate) mod fatal_line;
pub(crate) mod identity;
pub(crate) mod lifecycle;
pub(crate) mod panic;
pub(crate) mod writer;

/// Raw reference-UART base: the reference platform's first serial port
/// (PL011). Reference-platform fact documented by the W01 boot contract's
/// layering reconciliation and defined here exactly once (single-source
/// rule): consumed by the assembly rejection reporter below and, through
/// this constant, by the post-transfer early diagnostic writer. No other
/// module may reference it; W06's console owns its own assumption.
const P1_BOOT_UART_BASE: u64 = 0x0900_0000;

/// Boot-stack size for the boot CPU's entire P1 lifetime. Sizing
/// arithmetic (W02 design decision 8): establishment and panic paths use
/// well under 1 KiB of depth against the no-heap, no-reentry regime; 64 KiB
/// leaves the remainder as the stage-wide bound. Resizing is a reviewed
/// design change; overflow behavior is undefined-by-absence (recorded P1
/// limitation for W12).
const P1_BOOT_STACK_SIZE_BYTES: u64 = 64 * 1024;

#[repr(C, align(16))]
struct BootStack([AtomicU8; P1_BOOT_STACK_SIZE_BYTES as usize]);

// SAFETY comment contract: plain zeroed static; contents are owned by the
// executing code after the entry loads SP from its top boundary symbol.
// `link_section` pins the region into W08's dedicated zeroed stack output
// (writable, image-layout resident, non-executable) — without
// it an immutable zeroed static may be emitted into read-only .rodata.
// `non_upper_case_globals` is scoped to this item: the name is the W02
// design's linker-facing contract symbol.
#[allow(non_upper_case_globals)]
#[unsafe(link_section = ".p1_boot_stack")]
#[unsafe(no_mangle)]
static __p1_boot_stack: BootStack =
    BootStack([const { AtomicU8::new(0) }; P1_BOOT_STACK_SIZE_BYTES as usize]);

global_asm!(
    "
    .globl __p1_boot_stack_top
    .set __p1_boot_stack_top, __p1_boot_stack + {stack_size}
    .section .text.boot, \"ax\"
    .globl p1_el2_entry
    .type p1_el2_entry, %function
p1_el2_entry:
    /* W01 Tier A, T1 (first trust decision): exception level must be EL2
     * (CurrentEL.EL == 0b10). Checked before anything else executes. */
    mrs     x16, CurrentEL
    cmp     x16, #0x8
    b.ne    .Lreject_el

    /* W01 Tier A, T2: the DTB pointer delivered in x0 must be present. */
    cbz     x0, .Lreject_dtb

    /* Establishment stage 2: SPSel = 1, SP_EL2 = boot stack top, DAIF
     * all-masked. The top is __p1_boot_stack + size; the size literal comes
     * from the same Rust constant below (single definition). */
    msr     SPSel, #1
    ldr     x16, =__p1_boot_stack + {stack_size}
    mov     sp, x16
    msr     daifset, #0xF

    /* Establishment stage 3: clear BSS. Plain integer stores (no FP/SIMD,
     * no cache maintenance — caches are off per the canonical path). The
     * boot stack region is inside BSS and carries no load-bearing content
     * before first use, so clearing under a live SP is safe here: nothing
     * is pushed or popped until the transfer. x0-x3 are untouched. */
    ldr     x16, =__p1_bss_start
    ldr     x17, =__p1_bss_end
.Lclear_bss:
    cmp     x16, x17
    b.hs    .Lestablished
    str     xzr, [x16], #8
    b       .Lclear_bss

.Lestablished:
    /* Establishment stage 4: transfer. x0-x3 forwarded unmodified. */
    b       el2_rust_entry

    /* W01 rejection boundary: fixed-token line through the raw polling
     * UART write, then the bounded stop. Never returns; no initialization,
     * no stack, no Rust. */
.Lreject_el:
    ldr     x17, =.Lreason_el
    b       .Lreject
.Lreject_dtb:
    ldr     x17, =.Lreason_dtb
.Lreject:
    ldr     x18, =.Lreject_prefix
1:  ldrb    w19, [x18], #1
    cbz     x19, 2f
    bl      .Luart_putc
    b       1b
2:  mov     x18, x17
3:  ldrb    w19, [x18], #1
    cbz     x19, 4f
    bl      .Luart_putc
    b       3b
4:  mov     w19, #0x0d
    bl      .Luart_putc
    mov     w19, #0x0a
    bl      .Luart_putc
    /* Bounded stop: the boot CPU never executes another image
     * instruction; output is best-effort by design elsewhere than the
     * reference platform (W01 R4). */
5:  b       5b

/* Emit one byte from w19 through the polling UART write. Clobbers x20/x21;
 * called only from the rejection path. */
.Luart_putc:
    ldr     x20, ={uart_base}
6:  ldr     w21, [x20, #0x18]       /* FR: wait while TXFF (bit 5) */
    tbnz    w21, #5, 6b
    str     w19, [x20]
    ret

.section .rodata.boot, \"a\"
.Lreject_prefix:
    .ascii \"ZELYR P1 BOOT REJECT reason=\"
    .byte 0
.Lreason_el:
    .ascii \"EL\"
    .byte 0
.Lreason_dtb:
    .ascii \"DTB\"
    .byte 0
    ",
    stack_size = const P1_BOOT_STACK_SIZE_BYTES,
    uart_base = const P1_BOOT_UART_BASE,
);

/// Rust entry of the P1 runtime (`el2_rust_entry` in the W02 design).
///
/// Entered only by the assembly transfer, which guarantees the W01 tier
/// passed and establishment stages 1–3 are complete. Establishes stages 7–8
/// then records the W09 phases, executes their ordered mechanisms and idles.
#[unsafe(no_mangle)]
extern "C" fn el2_rust_entry(x0: u64, x1: u64, x2: u64, x3: u64) -> ! {
    lifecycle::phase_enter(lifecycle::InitPhase::Entry);
    lifecycle::phase_complete(lifecycle::InitPhase::Entry);
    lifecycle::phase_enter(lifecycle::InitPhase::Runtime);

    // Establishment stage 7: build and publish the boot context.
    context::publish_boot_context(x0, x1, x2, x3);

    // Establishment stage 8: assert runtime readiness — identity linkable
    // and the panic route's output path resolvable — before
    // Runtime.complete would be recorded, so later failures are attributable
    // to a routed, reporting runtime (W02 design O5).
    if !identity::identity_resolvable() {
        panic!("runtime readiness failed: build identity does not resolve");
    }
    writer::assert_early_writer_linked();

    lifecycle::phase_complete(lifecycle::InitPhase::Runtime);
    lifecycle::run_init_sequence();
    lifecycle::enter_stable();
    controlled_idle()
}

fn controlled_idle() -> ! {
    loop {
        crate::arch::aarch64::idle::wait_for_interrupt();
    }
}
