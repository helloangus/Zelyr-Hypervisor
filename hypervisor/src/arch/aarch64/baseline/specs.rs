//! Pure W04 baseline values and masked verification (see implementation record).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum ControlId {
    SpSel,
    Daif,
    HcrEl2,
    CptrEl2,
    CpacrEl1,
    MdcrEl2,
    MdscrEl1,
    CnthctlEl2,
    CntkctlEl1,
    CnthpCtlEl2,
    CnthvCtlEl2,
    SctlrEl1,
    SctlrEl2,
    VtcrEl2,
    VttbrEl2,
}
impl ControlId {
    pub const COUNT: usize = 15;
    pub const fn label(self) -> &'static str {
        match self {
            Self::SpSel => "SPSel",
            Self::Daif => "DAIF",
            Self::HcrEl2 => "HCR_EL2",
            Self::CptrEl2 => "CPTR_EL2",
            Self::CpacrEl1 => "CPACR_EL1",
            Self::MdcrEl2 => "MDCR_EL2",
            Self::MdscrEl1 => "MDSCR_EL1",
            Self::CnthctlEl2 => "CNTHCTL_EL2",
            Self::CntkctlEl1 => "CNTKCTL_EL1",
            Self::CnthpCtlEl2 => "CNTHP_CTL_EL2",
            Self::CnthvCtlEl2 => "CNTHV_CTL_EL2",
            Self::SctlrEl1 => "SCTLR_EL1",
            Self::SctlrEl2 => "SCTLR_EL2",
            Self::VtcrEl2 => "VTCR_EL2",
            Self::VttbrEl2 => "VTTBR_EL2",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BaselineCategory {
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
}
impl BaselineCategory {
    pub const ALL: [Self; 8] = [
        Self::C1,
        Self::C2,
        Self::C3,
        Self::C4,
        Self::C5,
        Self::C6,
        Self::C7,
        Self::C8,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaselineStatus {
    NotEstablished,
    Established,
    SkippedAbsent,
}
/// Immutable owned-field specification, not a raw register access API.
#[derive(Clone, Copy, Debug)]
pub struct WriteSpec {
    pub control: ControlId,
    pub category: BaselineCategory,
    pub mask: u64,
    pub value: u64,
    pub full_write: bool,
    pub optional_virtual_timer: bool,
}
impl WriteSpec {
    const fn new(control: ControlId, category: BaselineCategory, mask: u64, value: u64) -> Self {
        Self {
            control,
            category,
            mask,
            value,
            full_write: mask == u64::MAX,
            optional_virtual_timer: false,
        }
    }
    pub const fn apply(self, previous: u64) -> u64 {
        if self.full_write {
            self.value
        } else {
            (previous & !self.mask) | self.value
        }
    }
    pub const fn verify(self, observed: u64) -> bool {
        observed & self.mask == self.value
    }
}
pub const HCR_RW: u64 = 1 << 31;
pub const CPTR_TFP: u64 = 1 << 10;
pub const CPACR_FPEN: u64 = 3 << 20;
pub const MDCR_TRAPS: u64 = (1 << 10) | (1 << 9) | (1 << 8) | (1 << 6) | (1 << 5);
pub const CNTHCTL_EL1_GATES: u64 = 3;
pub const CNTKCTL_EL0_GATES: u64 = 0x3ff;
pub const TIMER_ENABLE_IMASK: u64 = 3;
pub const TIMER_IMASK: u64 = 2;
pub const SCTLR_MCI: u64 = (1 << 12) | (1 << 2) | 1;
pub const VTCR_RES1: u64 = 1 << 31;
pub const DAIF_ALL: u64 = 0x3c0;
/// HCR is written once in C2 and supplies C3 plus C7/C8 shared posture.
pub const SPECS: [WriteSpec; 13] = [
    WriteSpec::new(ControlId::HcrEl2, BaselineCategory::C2, u64::MAX, HCR_RW),
    WriteSpec::new(ControlId::CptrEl2, BaselineCategory::C4, CPTR_TFP, CPTR_TFP),
    WriteSpec::new(ControlId::CpacrEl1, BaselineCategory::C4, CPACR_FPEN, 0),
    WriteSpec::new(
        ControlId::MdcrEl2,
        BaselineCategory::C5,
        MDCR_TRAPS,
        MDCR_TRAPS,
    ),
    WriteSpec::new(ControlId::MdscrEl1, BaselineCategory::C5, u64::MAX, 0),
    WriteSpec::new(
        ControlId::CnthctlEl2,
        BaselineCategory::C6,
        CNTHCTL_EL1_GATES,
        0,
    ),
    WriteSpec::new(
        ControlId::CntkctlEl1,
        BaselineCategory::C6,
        CNTKCTL_EL0_GATES,
        0,
    ),
    WriteSpec {
        full_write: true,
        ..WriteSpec::new(
            ControlId::CnthpCtlEl2,
            BaselineCategory::C6,
            TIMER_ENABLE_IMASK,
            TIMER_IMASK,
        )
    },
    WriteSpec {
        full_write: true,
        optional_virtual_timer: true,
        ..WriteSpec::new(
            ControlId::CnthvCtlEl2,
            BaselineCategory::C6,
            TIMER_ENABLE_IMASK,
            TIMER_IMASK,
        )
    },
    WriteSpec::new(ControlId::SctlrEl1, BaselineCategory::C7, SCTLR_MCI, 0),
    WriteSpec::new(ControlId::SctlrEl2, BaselineCategory::C8, SCTLR_MCI, 0),
    WriteSpec::new(
        ControlId::VtcrEl2,
        BaselineCategory::C8,
        u64::MAX,
        VTCR_RES1,
    ),
    WriteSpec::new(ControlId::VttbrEl2, BaselineCategory::C8, u64::MAX, 0),
];
