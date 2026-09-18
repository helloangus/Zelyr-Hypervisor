//! Build-chain probe for the freestanding AArch64 bare-metal target.
//!
//! This member exists to prove the `no_std` + linking + assembly build path
//! (P0-W03). It defines no EL2 entry, vector table, memory setup, or console:
//! the entry symbol is replaced by the P1 EL2-entry design; the panic handler
//! semantics are owned by P0-W14 (failure classification) and P0-W12 (crash
//! information).
#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// P0-W03 build placeholder, replaced by the P1 EL2-entry design. Parks the
// core in a wait-for-interrupt loop; asserts no CPU mode, exception level,
// MMU state, stack, or memory state, and performs no system-register access.
global_asm!(
    ".globl _start",
    "_start:",
    "1:  wfi",
    "    b 1b",
);

// P0-W03 panic-handler placeholder, succeeded by the P0-W14 (failure
// classification) and P0-W12 (crash information) designs. Last-resort park:
// no allocation, no unwinding, no device access, no payload inspection.
#[panic_handler]
fn baseline_panic(_info: &PanicInfo) -> ! {
    loop {}
}
