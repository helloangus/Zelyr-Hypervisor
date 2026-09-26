/* W10 R4 harness control, not part of the hypervisor image.
 * Print start-context tokens, omit Stable, then request QEMU's clean
 * semihosting application exit. Only p1-marker-control enables semihosting.
 */
.section .text, "ax"
.global _start
.type _start, %function
_start:
    adr x1, message
    ldr x2, =0x09000000
1:
    ldrb w3, [x1], #1
    cbz w3, 3f
2:
    ldr w4, [x2, #0x18]
    tbnz w4, #5, 2b
    str w3, [x2]
    b 1b
3:
    mov w0, #0x20
    adr x1, exit_args
    hlt #0xf000
4:
    b 4b

message:
    .asciz "ZELYR P1 PHASE entry enter\r\nZELYR P1 PHASE runtime enter\r\n"

.balign 8
exit_args:
    .quad 0x20026
    .quad 0
