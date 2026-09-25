/* W05 U-009: register-only guard-first entry. No stack access until capture.
 * W01/W04 establish EL2, DAIF and no live FP state. TPIDR_EL2 is W05-owned.
 * Guard/frame mappings and this code must be accessible; failures are terminal.
 */
.section .p1_vectors, "ax"
.balign 2048
.globl p1_el2_vector_table
.type p1_el2_vector_table, %object
p1_el2_vector_table:
.macro vector_slot number
    b p1_vector_landing_\number
    .space 124
.endm
vector_slot 0
vector_slot 1
vector_slot 2
vector_slot 3
vector_slot 4
vector_slot 5
vector_slot 6
vector_slot 7
vector_slot 8
vector_slot 9
vector_slot 10
vector_slot 11
vector_slot 12
vector_slot 13
vector_slot 14
vector_slot 15
.size p1_el2_vector_table, . - p1_el2_vector_table

.section .text.p1_exception_landing, "ax"
.macro capture_landing number, origin, category
.type p1_vector_landing_\number, %function
p1_vector_landing_\number:
    msr TPIDR_EL2, x0
    adrp x0, p1_exception_guard
    ldr x0, [x0, :lo12:p1_exception_guard]
    cbz x0, p1_exception_silent_stop
    adrp x0, p1_exception_guard
    str xzr, [x0, :lo12:p1_exception_guard]

    adrp x0, p1_captured_frame
    add x0, x0, :lo12:p1_captured_frame
    stp x1, x2, [x0, #8]
    stp x3, x4, [x0, #24]
    stp x5, x6, [x0, #40]
    stp x7, x8, [x0, #56]
    stp x9, x10, [x0, #72]
    stp x11, x12, [x0, #88]
    stp x13, x14, [x0, #104]
    stp x15, x16, [x0, #120]
    stp x17, x18, [x0, #136]
    stp x19, x20, [x0, #152]
    stp x21, x22, [x0, #168]
    stp x23, x24, [x0, #184]
    stp x25, x26, [x0, #200]
    stp x27, x28, [x0, #216]
    stp x29, x30, [x0, #232]
    mrs x3, TPIDR_EL2
    str x3, [x0]
    mov x3, sp
    str x3, [x0, #{sp_offset}]
    mov w1, #\origin
    mov w2, #\category
    b p1_exception_capture_common
.size p1_vector_landing_\number, . - p1_vector_landing_\number
.endm
capture_landing 0, 0, 0
capture_landing 1, 0, 1
capture_landing 2, 0, 2
capture_landing 3, 0, 3
capture_landing 4, 1, 0
capture_landing 5, 1, 1
capture_landing 6, 1, 2
capture_landing 7, 1, 3
capture_landing 8, 2, 0
capture_landing 9, 2, 1
capture_landing 10, 2, 2
capture_landing 11, 2, 3
capture_landing 12, 3, 0
capture_landing 13, 3, 1
capture_landing 14, 3, 2
capture_landing 15, 3, 3

.type p1_exception_capture_common, %function
p1_exception_capture_common:
    mrs x3, ELR_EL2
    str x3, [x0, #{pc_offset}]
    mrs x3, SPSR_EL2
    str x3, [x0, #{spsr_offset}]
    mrs x3, ESR_EL2
    str x3, [x0, #{esr_offset}]
    str xzr, [x0, #{far_offset}]
    str xzr, [x0, #{hpfar_offset}]
    str wzr, [x0, #{flags_offset}]
    strb w1, [x0, #{origin_offset}]
    strb w2, [x0, #{category_offset}]

    /* All original GPRs and SP are retained. Reclaim the existing stack
     * only for terminal reporting: no live path will ever return. */
    adrp x3, __p1_boot_stack_top
    add x3, x3, :lo12:__p1_boot_stack_top
    mov sp, x3
    b p1_exception_rust_entry
.size p1_exception_capture_common, . - p1_exception_capture_common

.globl p1_exception_silent_stop
.type p1_exception_silent_stop, %function
p1_exception_silent_stop:
    b p1_exception_silent_stop
.size p1_exception_silent_stop, . - p1_exception_silent_stop

/* Assembly is the sole owner before handing a completed frame to Rust.
 * No Rust references or atomics alias these assembly-written bytes. */
.section .bss.p1_exception, "aw", %nobits
.balign 16
.globl p1_exception_guard
.type p1_exception_guard, %object
p1_exception_guard:
    .zero 8
.size p1_exception_guard, 8
.balign 16
.globl p1_captured_frame
.type p1_captured_frame, %object
p1_captured_frame:
    .zero {frame_size}
.size p1_captured_frame, {frame_size}
