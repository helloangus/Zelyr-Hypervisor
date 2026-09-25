//! P1 boot-CPU idle instruction boundary, consumed only after W09 Stable.

pub(crate) fn wait_for_interrupt() {
    // SAFETY: U-015. W01/W04 keep DAIF masked and W09 has reached Stable
    // on the sole boot CPU. WFI has no memory operand or stack effect; on
    // normal completion it returns to the W02 loop. EL3 firmware may trap
    // WFI under SCR_EL3.TWI; P1 neither controls nor promises that policy.
    // A broken EL2/lifecycle premise is terminal FC-INVARIANT.
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}
