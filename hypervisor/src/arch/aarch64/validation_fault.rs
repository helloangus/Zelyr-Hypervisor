//! Default-off, build-selected W11 intentional fault instructions.
//! This module is absent from the ordinary image.

#[cfg(feature = "p1-w11-nc5")]
use crate::boot::address::VirtAddr;

/// Closed build-time scenario identity; no external or runtime selector.
pub(crate) enum ScenarioId {
    #[cfg(feature = "p1-w11-nc3")]
    Nc3,
    #[cfg(feature = "p1-w11-nc4")]
    Nc4,
    #[cfg(feature = "p1-w11-nc5")]
    Nc5,
}

/// Called once at the W09 phase hook selected at build time. The intended
/// exception or panic is terminal; unexpected architectural continuation
/// takes the panic route with an explicit failed-trigger reason.
pub(crate) fn fault_scenario(id: ScenarioId) -> ! {
    match id {
        #[cfg(feature = "p1-w11-nc3")]
        ScenarioId::Nc3 => {
            // SAFETY: U-016. W09 calls this only after W05 vectors and W07
            // fatal reporting are armed on the masked boot CPU. `.inst 0`
            // is permanently undefined in AArch64 and should synchronously
            // trap; failure of that architectural premise is FC-INVARIANT
            // and the fallback panic below is terminal.
            unsafe {
                core::arch::asm!(".inst 0", options(nostack, preserves_flags));
            }
            panic!("P1-W11 NC3 undefined instruction returned");
        }
        #[cfg(feature = "p1-w11-nc4")]
        ScenarioId::Nc4 => panic!("P1-W11 NC4 intentional panic"),
        #[cfg(feature = "p1-w11-nc5")]
        ScenarioId::Nc5 => {
            // W08's verified image window is at 0x4000_0000 and console
            // window at 0x0800_0000. The 0x5000_0000 L2 entry is absent.
            // Keep the address typed until this architecture instruction.
            let unmapped = VirtAddr::new(0x5000_0000);
            // SAFETY: U-017. W09 calls this only after W08 successfully
            // enabled Stage-1, with W05 vectors and W07 fatal reporting
            // armed. This one load intentionally targets an unmapped L2
            // entry; it is not a Rust pointer dereference. Expected outcome
            // is a terminal EL2 translation fault; an unexpectedly mapped
            // address is FC-INVARIANT and reaches the fallback panic.
            unsafe {
                core::arch::asm!(
                    "ldr {value}, [{address}]",
                    value = out(reg) _,
                    address = in(reg) unmapped.raw(),
                    options(nostack, preserves_flags),
                );
            }
            panic!("P1-W11 NC5 unmapped load returned");
        }
    }
}
