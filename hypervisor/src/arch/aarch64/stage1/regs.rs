//! Closed EL2 Stage-1 register/instruction boundary (inventory U-012).

pub(super) fn dsb_sy() {
    // SAFETY: U-012. W09 invokes W08 once on the masked EL2 boot CPU;
    // DSB SY orders completed table writes before translation use. A
    // violated execution premise is terminal FC-INVARIANT.
    unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)) }
}
pub(super) fn isb() {
    // SAFETY: U-012. The W01-established EL2 boot CPU executes this context
    // synchronization barrier after control writes. FC-INVARIANT otherwise.
    unsafe { core::arch::asm!("isb", options(nostack, preserves_flags)) }
}
pub(super) fn tlbi_alle2() {
    // SAFETY: U-012. The masked boot CPU owns EL2 translations; invalidating
    // all EL2 stage-1 entries before the one-time enable is closed and
    // ordered by adjacent DSB/ISB. FC-INVARIANT if the premise fails.
    unsafe { core::arch::asm!("tlbi alle2", options(nostack, preserves_flags)) }
}
pub(super) fn ic_iallu() {
    // SAFETY: U-012. This whole-I-cache invalidation precedes SCTLR.I on the
    // single masked EL2 boot CPU; adjacent barriers complete the sequence.
    // A broken hardware/EL premise is terminal FC-INVARIANT.
    unsafe { core::arch::asm!("ic iallu", options(nostack, preserves_flags)) }
}

pub(super) fn mair_write(value: u64) {
    // SAFETY: U-012. MAIR_EL2 is W08-owned in the Stage1 phase, after the
    // reviewed class table is built and before SCTLR.M. The asm declares
    // its input and memory side effects. FC-INVARIANT if called out of phase.
    unsafe {
        core::arch::asm!("msr MAIR_EL2, {}", in(reg) value, options(nostack, preserves_flags))
    }
}
pub(super) fn mair_read() -> u64 {
    let value;
    // SAFETY: U-012. Side-effect-free MAIR_EL2 read at established EL2;
    // explicit output, no memory/stack effect. FC-INVARIANT otherwise.
    unsafe {
        core::arch::asm!("mrs {}, MAIR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
    }
    value
}
pub(super) fn ttbr0_write(value: u64) {
    // SAFETY: U-012. W08 supplies an aligned, in-image L1 table PA before
    // MMU enable; W09 calls once with DAIF masked. Input is explicit and
    // compiler memory reordering is inhibited. FC-INVARIANT otherwise.
    unsafe {
        core::arch::asm!("msr TTBR0_EL2, {}", in(reg) value, options(nostack, preserves_flags))
    }
}
pub(super) fn ttbr0_read() -> u64 {
    let value;
    // SAFETY: U-012. Closed, side-effect-free EL2 register read at the W01
    // EL2 execution premise. FC-INVARIANT if that premise is broken.
    unsafe {
        core::arch::asm!("mrs {}, TTBR0_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
    }
    value
}
pub(super) fn tcr_write(value: u64) {
    // SAFETY: U-012. W03's PA-range/4K facts and W08's fixed 39-bit model
    // produce this TCR before translation is enabled; explicit input and
    // compiler memory barrier. FC-INVARIANT on a false premise.
    unsafe { core::arch::asm!("msr TCR_EL2, {}", in(reg) value, options(nostack, preserves_flags)) }
}
pub(super) fn tcr_read() -> u64 {
    let value;
    // SAFETY: U-012. Closed EL2 TCR read for readback after ISB at the
    // established W01 EL2 level. FC-INVARIANT otherwise.
    unsafe {
        core::arch::asm!("mrs {}, TCR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
    }
    value
}
pub(super) fn sctlr_write(value: u64) {
    // SAFETY: U-012. W08 alone supersedes W04's SCTLR M/C/I bits after
    // verified tables, MAIR/TTBR/TCR and barriers; W01 EL2, DAIF masked.
    // Input is explicit and compiler memory reordering inhibited. A broken
    // ordering/execution premise is terminal FC-INVARIANT.
    unsafe {
        core::arch::asm!("msr SCTLR_EL2, {}", in(reg) value, options(nostack, preserves_flags))
    }
}
pub(super) fn sctlr_read() -> u64 {
    let value;
    // SAFETY: U-012. Closed EL2 SCTLR read at W01-established EL2;
    // side-effect-free explicit output. FC-INVARIANT otherwise.
    unsafe {
        core::arch::asm!("mrs {}, SCTLR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
    }
    value
}
