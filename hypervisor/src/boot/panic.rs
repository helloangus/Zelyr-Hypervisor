//! W07 owns W02's transferred single panic-handler registration.
use core::panic::PanicInfo;

#[panic_handler]
fn p1_panic(info: &PanicInfo<'_>) -> ! {
    let sp0: u64;
    let lr0: u64;
    // SAFETY: U-011. Closed, read-only handler-entry SP/LR capture at EL2.
    // These approximate handler-entry coordinates, not the panic call-site.
    // There are no memory/stack writes or clobbers.
    unsafe {
        // Explicit outputs prevent register allocation from choosing x30 for
        // the SP result before the original LR has been copied.
        core::arch::asm!("mov x9, sp", "mov x10, x30",
            out("x9") sp0, out("x10") lr0,
            options(nomem, nostack, preserves_flags));
    }
    super::fatal::report_panic(info, sp0, lr0)
}
