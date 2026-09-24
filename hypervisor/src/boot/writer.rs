//! Early diagnostic writer: raw polling writes to the reference UART.
//!
//! Contract: P1-W02 design, panic/identity contracts §3. The
//! panic route (and, through W07's
//! extension seam, its report body) is the caller. W06's channel supersedes
//! this writer for markers and phase output and defines its own
//! reference-console assumption — no sharing of the constant below.

use core::ptr;

use super::P1_BOOT_UART_BASE;

/// PL011 flag-register offset from the base and its transmit-FIFO-full bit.
/// Architecture facts of the reference UART, distinct from the
/// single-source base constant.
const P1_BOOT_UART_FR_OFFSET: u64 = 0x18;
const P1_BOOT_UART_FR_TXFF: u32 = 1 << 5;

// Readiness assertion (establishment stage 8): the early writer's constant
// resolves at compile time — a zero base would mean the boot entry module
// lost its single-source definition.
const _: () = assert!(P1_BOOT_UART_BASE != 0);

/// Assert, without mutating machine state, that the writer's output path is
/// linked. Kept as an explicit establishment step so stage 8 is reviewable.
pub(crate) fn assert_early_writer_linked() {
    // The const assertion above proves the constant resolves; nothing to
    // probe at run time — no device initialization is authorized in P1.
}

/// Write every byte through the raw reference-UART polling write. No error
/// path: a transmitter stuck busy spins in the poll, and the callers'
/// bounded-stop discipline bounds the outcome (the panic route never
/// continues because output failed).
pub(crate) fn early_write_bytes(bytes: &[u8]) {
    // SAFETY: volatile MMIO access to the reference UART's flag and data
    // registers at the single-source boot constant; single-consumer rule
    // (only the panic route writes after transfer); polling loop and store
    // are the whole effect, no state change otherwise. A stuck transmitter
    // spins here; the caller's terminal-stop contract bounds the outcome.
    // Inventory: U-002.
    unsafe {
        let data_register = P1_BOOT_UART_BASE as *mut u32;
        let flag_register = (P1_BOOT_UART_BASE + P1_BOOT_UART_FR_OFFSET) as *mut u32;
        for &byte in bytes {
            while ptr::read_volatile(flag_register) & P1_BOOT_UART_FR_TXFF != 0 {
                core::hint::spin_loop();
            }
            ptr::write_volatile(data_register, u32::from(byte));
        }
    }
}
