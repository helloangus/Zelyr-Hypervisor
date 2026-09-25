# P1-W06 Early Console — Implementation Record

**Status:** Mechanism implemented on `p1/w06-early-console`; W09 integration and W10/W11 execution pending.  
**Date:** 2026-09-25 (Asia/Shanghai).  
**Plan:** [P1-W06](../plans/p1-w06-early-console-logging.md).  
**Design:** [W06 detailed design](p1-w06-early-console-logging/README.md) and [reconciliation](p1-w06-early-console-logging/05-implementation-reconciliation.md).  
**Verification:** [W06 verification record](../verification/p1-w06-early-console-logging-verification.md).

W02 supplies a boot-only panic writer, identity and terminal route. W03
supplies capability lines. W05/W08/W09 exist as designs, so W06 provides
their named mechanisms and leaves their execution to their owners.

The new `hypervisor/src/boot/console.rs` owns the fixed reference UART
window, minimal UARTEN/TXE readback init, polled bytes, 128-byte bounded
line formatting, start-line identity and a monotone availability flag.
`hypervisor/src/boot/mod.rs` links the module without changing W02's
entry or terminal seam. Comments in `hypervisor/src/main.rs` and
`hypervisor/src/boot/writer.rs` now describe the actual code state. The
[event registry](../../../development/trace-event-namespace.md) declares
W06's two phase encodings, while W09 retains event emission ownership.
The design reconciliation records distinct diagnostic meanings on one
physical transport. The P1 implementation index, this record, the
verification record and the [unsafe inventory](../../../security/unsafe-inventory.md)
change with the implementation.

`CONSOLE_REGION_BASE=0x0900_0000`, size 4 KiB, and required W08 mapping
Device-nGnRE/RW/XN are the selected reference facts. UARTCR's current bits
are preserved; if UARTEN/TXE are absent they are set and read back. A second
bring-up panics as an invariant violation. The start line is the sole
init-time writer before availability; later producers must be phase-gated.
The line cap is 128 bytes; both formatted and producer lines reserve the
12-byte suffix, keep at most 116 bytes at a UTF-8 boundary on overflow, and
add ` [truncated]` before CRLF. Formatted lines have a nonpanicking UTF-8
fallback; W09's closed phase labels fit the bound. The marker spelling
is `ZELYR P1 PHASE <label> <enter|complete>`; W09 supplies labels and
ordering, W10 fixes its own stable token before regression runs.

W07 receives `console_write_line` and `channel_available`; W08 receives
the console region and attribute requirement; W09 receives
`bring_up_early_console`, `format_marker` and `transport_line`.
The output poll has no software timeout, so W10's runner supplies the
external observation bound. P2 discovery owns replacing the fixed address.

One new `unsafe` boundary, U-010, wraps closed volatile MMIO access with
nearby SAFETY arguments; independent soundness review is complete. No dependency,
feature, public external API, ABI, heap, lock, UART probe or second UART
was added. There are no reachable TODO/FIXME placeholders. Evidence and
unrun tests are stated in the separate verification record.
