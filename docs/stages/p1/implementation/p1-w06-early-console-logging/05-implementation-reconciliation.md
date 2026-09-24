# P1-W06 Current-State and Channel Reconciliation

**Status:** Proposed detailed-design amendment; no validation claim.  
**Scope:** P0 diagnostic taxonomy and the console publication boundary.  
**Version:** v0.2  
**Owner/change context:** P1-W06 current-state audit, 2026-09-25.  
**Supersedes:** README decision 1's combined-channel wording and channel-contract §2's blanket availability precondition.

W02 currently has a panic-only writer and no console phase. W03 has a
structured capability fact producer. W05/W09/W08 have design contracts but
no executing consumers at this branch point. The delivered
[diagnostics baseline](../../../../development/diagnostics-baseline.md)
requires distinct human, structured trace and fatal meanings. W06 supplies
one physical PL011 byte transport for these producers; sharing bytes does not
merge their channel semantics.

| Required outcome | Foundation and owner | Evidence |
|---|---|---|
| Human start message with identity | W06 init and bounded start line | W06-DV02 |
| Machine-readable phase events | W06 framing, W09 emission; canonical `boot.phase.enter` / `boot.phase.complete` names in the [event registry](../../../../development/trace-event-namespace.md) | W06-DV02 and W10 |
| Guarded fatal reporting | W07 content through W06 transport, W02 fallback before publication | W06-DV03 and W11 |
| Post-MMU output | W08 maps the whole UART window Device-nGnRE/RW/XN | W06-DV05 through W10 |

These are Required. A generic trace backend, runtime filter engine and
P2 platform discovery are Reserved. New drivers, allocation and
interrupt-driven I/O are Out of Scope. The start line is an Info-class
human message; phase markers are low-frequency structured boot facts;
W03's `cpu.capability.fact` remains W03-owned; W07 owns fatal content.
The fixed line encodings are P1 internal regression inputs, not external ABI.

`bring_up_early_console()` alone may call the private polled byte primitive
after successful `console_init()` and before `CHANNEL_AVAILABLE` publication
to emit the start line. All other producers require availability. A second
bring-up is FC-INVARIANT. Readback failure leaves availability false and
follows W09's console failure route once wired.

The fixed PL011 window is `0x0900_0000..0x0900_1000` in the
[QEMU virt map](https://github.com/qemu/qemu/blob/v8.2.2/hw/arm/virt.c).
The minimal init preserves existing UARTCR bits, enables UARTEN and TXE if
needed, and reads them back. [Arm's PL011 manual](https://documentation-service.arm.com/static/5e8e36c2fd977155116a90b5)
defines the control and TXFF bits. P1 relies on the reference boot's
clock/divisor configuration; it does not program baud rate. A stuck
transmitter remains an explicit W10 runner-timeout limitation.

W06-DV01–DV04 review these contracts and the implementation. W06-DV05
requires executed W10 evidence before and after W08's MMU transition.
Actual results belong only in the verification record.
