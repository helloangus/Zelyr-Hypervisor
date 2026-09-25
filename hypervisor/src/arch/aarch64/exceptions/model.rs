//! Pure P1 exception vocabulary, validity and terminal-disposition policy.
//! Reference: Arm DDI 0601 ID032522, ESR_EL2/FAR_EL2.

/// Assembly layout: all register values are diagnostic data, never pointers.
#[repr(C, align(16))]
pub struct ExceptionFrame {
    pub x: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub spsr: u64,
    pub esr: u64,
    pub far: u64,
    pub hpfar: u64,
    pub far_valid: bool,
    pub hpfar_valid: bool,
    pub origin: u8,
    pub category: u8,
}

// These layout constants are passed to global_asm; assertions protect the
// single literal GPR storage stride and make drift a compile-time failure.
pub const FRAME_SIZE: usize = core::mem::size_of::<ExceptionFrame>();
pub const SP_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, sp);
pub const PC_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, pc);
pub const SPSR_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, spsr);
pub const ESR_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, esr);
pub const FAR_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, far);
pub const HPFAR_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, hpfar);
pub const FLAGS_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, far_valid);
pub const ORIGIN_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, origin);
pub const CATEGORY_OFFSET: usize = core::mem::offset_of!(ExceptionFrame, category);
const _: () = {
    assert!(core::mem::offset_of!(ExceptionFrame, x) == 0);
    assert!(SP_OFFSET == 31 * 8);
    assert!(core::mem::offset_of!(ExceptionFrame, hpfar_valid) == FLAGS_OFFSET + 1);
    assert!(ORIGIN_OFFSET == FLAGS_OFFSET + 2);
    assert!(CATEGORY_OFFSET == FLAGS_OFFSET + 3);
    assert!(FRAME_SIZE == 304);
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginClass {
    CurrentElSp0,
    CurrentElSpx,
    LowerElA64,
    LowerElA32,
}
impl OriginClass {
    pub const fn from_slot(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::CurrentElSp0),
            1 => Some(Self::CurrentElSpx),
            2 => Some(Self::LowerElA64),
            3 => Some(Self::LowerElA32),
            _ => None,
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::CurrentElSp0 => "current-el-sp0",
            Self::CurrentElSpx => "current-el-spx",
            Self::LowerElA64 => "lower-el-a64",
            Self::LowerElA32 => "lower-el-a32",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExceptionCategory {
    Synchronous,
    Irq,
    Fiq,
    SError,
}
impl ExceptionCategory {
    pub const fn from_slot(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Synchronous),
            1 => Some(Self::Irq),
            2 => Some(Self::Fiq),
            3 => Some(Self::SError),
            _ => None,
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Synchronous => "sync",
            Self::Irq => "irq",
            Self::Fiq => "fiq",
            Self::SError => "serror",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExceptionDisposition {
    FatalSyndrome,
    UnexpectedEvent,
    #[allow(dead_code)] // P6 owns any first recoverable disposition and return path.
    Recoverable,
}
impl ExceptionDisposition {
    pub const fn label(self) -> &'static str {
        match self {
            Self::FatalSyndrome => "fatal-syndrome",
            Self::UnexpectedEvent => "unexpected-event",
            Self::Recoverable => "recoverable",
        }
    }
}
pub const fn classify(origin: OriginClass, category: ExceptionCategory) -> ExceptionDisposition {
    match (origin, category) {
        (OriginClass::CurrentElSpx, ExceptionCategory::Synchronous | ExceptionCategory::SError) => {
            ExceptionDisposition::FatalSyndrome
        }
        _ => ExceptionDisposition::UnexpectedEvent,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbortKind {
    Translation,
    Alignment,
    Permission,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapKind {
    Wfx,
    SystemRegister,
    LdcStc,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugKind {
    Breakpoint,
    Step,
    Watchpoint,
    Brk,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyndromeClass {
    Unknown,
    TrappedInstruction(TrapKind),
    IllegalState,
    InstructionAbort,
    DataAbort(AbortKind),
    SpAlignment,
    PcAlignment,
    Debug(DebugKind),
    SError,
    GuestClassOnly,
}
impl SyndromeClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::TrappedInstruction(TrapKind::Wfx) => "trap-wfx",
            Self::TrappedInstruction(TrapKind::SystemRegister) => "trap-sysreg",
            Self::TrappedInstruction(TrapKind::LdcStc) => "trap-ldc-stc",
            Self::IllegalState => "illegal-state",
            Self::InstructionAbort => "instruction-abort",
            Self::DataAbort(AbortKind::Translation) => "data-abort-translation",
            Self::DataAbort(AbortKind::Alignment) => "data-abort-alignment",
            Self::DataAbort(AbortKind::Permission) => "data-abort-permission",
            Self::DataAbort(AbortKind::Other) => "data-abort-other",
            Self::SpAlignment => "sp-alignment",
            Self::PcAlignment => "pc-alignment",
            Self::Debug(DebugKind::Breakpoint) => "breakpoint",
            Self::Debug(DebugKind::Step) => "software-step",
            Self::Debug(DebugKind::Watchpoint) => "watchpoint",
            Self::Debug(DebugKind::Brk) => "brk",
            Self::SError => "serror",
            Self::GuestClassOnly => "lower-el-only",
        }
    }
}
pub const fn syndrome_class(esr: u64) -> SyndromeClass {
    match (esr >> 26) & 0x3f {
        0x01 => SyndromeClass::TrappedInstruction(TrapKind::Wfx),
        0x06 => SyndromeClass::TrappedInstruction(TrapKind::LdcStc),
        0x18 => SyndromeClass::TrappedInstruction(TrapKind::SystemRegister),
        0x0e => SyndromeClass::IllegalState,
        0x21 => SyndromeClass::InstructionAbort,
        0x22 => SyndromeClass::PcAlignment,
        0x25 => SyndromeClass::DataAbort(match esr & 0x3f {
            0x04..=0x07 => AbortKind::Translation,
            0x0c..=0x0f => AbortKind::Permission,
            0x21 => AbortKind::Alignment,
            _ => AbortKind::Other,
        }),
        0x26 => SyndromeClass::SpAlignment,
        0x2f => SyndromeClass::SError,
        0x31 => SyndromeClass::Debug(DebugKind::Breakpoint),
        0x33 => SyndromeClass::Debug(DebugKind::Step),
        0x35 => SyndromeClass::Debug(DebugKind::Watchpoint),
        0x3c => SyndromeClass::Debug(DebugKind::Brk),
        0x11..=0x13 | 0x20 | 0x24 | 0x30 | 0x32 | 0x34 | 0x38 | 0x3a => {
            SyndromeClass::GuestClassOnly
        }
        _ => SyndromeClass::Unknown,
    }
}
/// Gate FAR reads by the event category and architecture-defined validity.
pub const fn far_valid(esr: u64, category: ExceptionCategory) -> bool {
    if !matches!(category, ExceptionCategory::Synchronous) {
        return false;
    }
    match syndrome_class(esr) {
        SyndromeClass::InstructionAbort | SyndromeClass::DataAbort(_) => {
            !(esr & 0x3f == 0x10 && esr & (1 << 10) != 0)
        }
        SyndromeClass::PcAlignment => true,
        SyndromeClass::Debug(DebugKind::Watchpoint) => esr & (1 << 10) == 0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_slot_is_terminal_and_only_current_spx_has_fatal_syndrome() {
        for o in 0..4 {
            for c in 0..4 {
                let origin = OriginClass::from_slot(o).unwrap();
                let category = ExceptionCategory::from_slot(c).unwrap();
                let expected = if o == 1 && (c == 0 || c == 3) {
                    ExceptionDisposition::FatalSyndrome
                } else {
                    ExceptionDisposition::UnexpectedEvent
                };
                assert_eq!(classify(origin, category), expected);
            }
        }
        assert_eq!(OriginClass::from_slot(4), None);
        assert_eq!(ExceptionCategory::from_slot(255), None);
    }
    #[test]
    fn far_validity_rejects_sp_alignment_and_external_abort_fnv() {
        let sync = ExceptionCategory::Synchronous;
        assert!(!far_valid(0x26 << 26, sync));
        assert!(far_valid(0x22 << 26, sync));
        for ec in [0x21, 0x25] {
            assert!(far_valid((ec << 26) | 0x10, sync));
            assert!(!far_valid((ec << 26) | 0x10 | (1 << 10), sync));
            assert!(far_valid((ec << 26) | 0x04, sync));
        }
        assert!(far_valid(0x35 << 26, sync));
        assert!(!far_valid((0x35 << 26) | (1 << 10), sync));
    }
    #[test]
    fn asynchronous_categories_never_reuse_stale_far_validity() {
        for ec in 0..64 {
            for category in [
                ExceptionCategory::Irq,
                ExceptionCategory::Fiq,
                ExceptionCategory::SError,
            ] {
                assert!(!far_valid(ec << 26, category));
            }
        }
    }
    #[test]
    fn syndrome_table_is_total_and_data_abort_groups_are_bounded() {
        for ec in 0..64 {
            assert!(!syndrome_class(ec << 26).label().is_empty());
        }
        for status in 0..64 {
            let kind = match status {
                4..=7 => AbortKind::Translation,
                12..=15 => AbortKind::Permission,
                33 => AbortKind::Alignment,
                _ => AbortKind::Other,
            };
            assert_eq!(
                syndrome_class((0x25 << 26) | status),
                SyndromeClass::DataAbort(kind)
            );
        }
        assert_eq!(syndrome_class(u64::MAX), SyndromeClass::Unknown);
    }
}
