//! Explicit P1 EL2 baseline; W09 owns phase invocation.
mod specs;
use super::capabilities::{self, FactId, Observation};
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
pub(crate) use specs::{BaselineCategory, BaselineStatus, ControlId};
use specs::{DAIF_ALL, SPECS};

/// Safe atomics express the designed single-publication cell without unsafe.
struct RecordedControl {
    status: AtomicU8,
    value: AtomicU64,
}
impl RecordedControl {
    const NEW: Self = Self {
        status: AtomicU8::new(0),
        value: AtomicU64::new(0),
    };
    fn publish(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
        self.status.store(1, Ordering::Release);
    }
    fn status(&self) -> BaselineStatus {
        match self.status.load(Ordering::Acquire) {
            1 => BaselineStatus::Established,
            2 => BaselineStatus::SkippedAbsent,
            _ => BaselineStatus::NotEstablished,
        }
    }
}
struct BaselineDeclaration {
    started: AtomicBool,
    categories: [AtomicBool; 8],
    controls: [RecordedControl; ControlId::COUNT],
}
static EL2_BASELINE: BaselineDeclaration = BaselineDeclaration {
    started: AtomicBool::new(false),
    categories: [const { AtomicBool::new(false) }; 8],
    controls: [const { RecordedControl::NEW }; ControlId::COUNT],
};

/// Category readiness: optional controls are classified separately.
#[allow(dead_code)] // W05/W08/W09 consume after phase integration.
pub(crate) fn baseline_status(category: BaselineCategory) -> BaselineStatus {
    if EL2_BASELINE.categories[category as usize].load(Ordering::Acquire) {
        BaselineStatus::Established
    } else {
        BaselineStatus::NotEstablished
    }
}
/// Recorded owned fields, never a live hardware reread.
#[allow(dead_code)] // W05/W08 assert premises through this contract.
pub(crate) fn baseline_value(control: ControlId) -> Option<u64> {
    let record = &EL2_BASELINE.controls[control as usize];
    if record.status() == BaselineStatus::Established {
        Some(record.value.load(Ordering::Relaxed))
    } else {
        None
    }
}
#[allow(dead_code)] // W10/W12 observe the optional-timer skip explicitly.
pub(crate) fn control_status(control: ControlId) -> BaselineStatus {
    EL2_BASELINE.controls[control as usize].status()
}

/// Once on boot CPU at EL2 with W03 published. Never allocates or retries.
#[allow(dead_code)] // W09 exclusively owns invocation; no premature boot wiring.
pub(crate) fn establish_el2_baseline() {
    if EL2_BASELINE.started.swap(true, Ordering::Relaxed) {
        panic!("el2-baseline repeated establishment");
    }
    for fact in [
        FactId::ExecutionLevel,
        FactId::ArchProfile,
        FactId::El2PhysicalTimer,
        FactId::CounterFrequency,
        FactId::PaRange,
        FactId::Granule4k,
    ] {
        if !matches!(
            capabilities::query(fact).observation,
            Observation::Present(_)
        ) {
            panic!("el2-baseline missing fact={}", fact.label());
        }
    }
    // Context only: VHE absence does not prevent this non-VHE baseline.
    let _ = capabilities::query(FactId::VirtualHostExtensions);
    let virtual_timer = match capabilities::query(FactId::El2VirtualTimer).observation {
        Observation::Present(_) => true,
        Observation::Absent => false,
        Observation::Unreadable => panic!("el2-baseline unreadable virtual timer fact"),
    };
    verify_assert(ControlId::SpSel, 1, 1);
    verify_assert(ControlId::Daif, DAIF_ALL, DAIF_ALL);
    mark_category(BaselineCategory::C1);
    for category in BaselineCategory::ALL.into_iter().skip(1) {
        for spec in SPECS.iter().filter(|s| s.category == category) {
            let record = &EL2_BASELINE.controls[spec.control as usize];
            if spec.optional_virtual_timer && !virtual_timer {
                record.status.store(2, Ordering::Release);
                continue;
            }
            let previous = if spec.full_write {
                0
            } else {
                read_sysreg(spec.control)
            };
            write_sysreg(spec.control, spec.apply(previous));
            let observed = read_sysreg(spec.control);
            if !spec.verify(observed) {
                fail_baseline(spec.control, spec.value, observed & spec.mask);
            }
            record.publish(observed & spec.mask);
        }
        // C3 consumes the single HCR write from C2; C7/C8 reuse RW/VM.
        mark_category(category);
    }
}
fn mark_category(category: BaselineCategory) {
    EL2_BASELINE.categories[category as usize].store(true, Ordering::Release);
}
fn verify_assert(control: ControlId, mask: u64, expected: u64) {
    let observed = read_sysreg(control) & mask;
    if observed != expected {
        fail_baseline(control, expected, observed);
    }
    EL2_BASELINE.controls[control as usize].publish(observed);
}
fn fail_baseline(control: ControlId, expected: u64, observed: u64) -> ! {
    panic!(
        "el2-baseline {} expected={:#x} observed={:#x}",
        control.label(),
        expected,
        observed
    );
}

fn read_sysreg(control: ControlId) -> u64 {
    let value;
    // SAFETY: W01 establishes EL2. The closed set is used only during W04;
    // CNTHV is guarded by W03 existence. No memory/stack effect; an access
    // trap violates the firmware/ISA contract (FC-INVARIANT). U-005.
    unsafe {
        match control {
            ControlId::SpSel => {
                core::arch::asm!("mrs {}, SPSel", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::Daif => {
                core::arch::asm!("mrs {}, DAIF", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::HcrEl2 => {
                core::arch::asm!("mrs {}, HCR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::CptrEl2 => {
                core::arch::asm!("mrs {}, CPTR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::CpacrEl1 => {
                core::arch::asm!("mrs {}, CPACR_EL1", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::MdcrEl2 => {
                core::arch::asm!("mrs {}, MDCR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::MdscrEl1 => {
                core::arch::asm!("mrs {}, MDSCR_EL1", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::CnthctlEl2 => {
                core::arch::asm!("mrs {}, CNTHCTL_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::CntkctlEl1 => {
                core::arch::asm!("mrs {}, CNTKCTL_EL1", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::CnthpCtlEl2 => {
                core::arch::asm!("mrs {}, CNTHP_CTL_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::CnthvCtlEl2 => {
                core::arch::asm!("mrs {}, S3_4_C14_C3_1", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::SctlrEl1 => {
                core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::SctlrEl2 => {
                core::arch::asm!("mrs {}, SCTLR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::VtcrEl2 => {
                core::arch::asm!("mrs {}, VTCR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
            ControlId::VttbrEl2 => {
                core::arch::asm!("mrs {}, VTTBR_EL2", out(reg) value, options(nomem, nostack, preserves_flags))
            }
        }
    }
    value
}
fn write_sysreg(control: ControlId, value: u64) {
    // SAFETY: SPECS select only owned controls after EL2/W03/C1 prerequisites.
    // Masks preserve other fields; full constants honor RES1. Each write has
    // ISB before dependent accesses. No nomem hides control effects from the
    // compiler. Violated premises are FC-INVARIANT, terminal. U-006.
    unsafe {
        match control {
            ControlId::SpSel | ControlId::Daif => {
                panic!("el2-baseline cannot write W02-owned state")
            }
            ControlId::HcrEl2 => {
                core::arch::asm!("msr HCR_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::CptrEl2 => {
                core::arch::asm!("msr CPTR_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::CpacrEl1 => {
                core::arch::asm!("msr CPACR_EL1, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::MdcrEl2 => {
                core::arch::asm!("msr MDCR_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::MdscrEl1 => {
                core::arch::asm!("msr MDSCR_EL1, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::CnthctlEl2 => {
                core::arch::asm!("msr CNTHCTL_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::CntkctlEl1 => {
                core::arch::asm!("msr CNTKCTL_EL1, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::CnthpCtlEl2 => {
                core::arch::asm!("msr CNTHP_CTL_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::CnthvCtlEl2 => {
                core::arch::asm!("msr S3_4_C14_C3_1, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::SctlrEl1 => {
                core::arch::asm!("msr SCTLR_EL1, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::SctlrEl2 => {
                core::arch::asm!("msr SCTLR_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::VtcrEl2 => {
                core::arch::asm!("msr VTCR_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
            ControlId::VttbrEl2 => {
                core::arch::asm!("msr VTTBR_EL2, {}", "isb", in(reg) value, options(nostack, preserves_flags))
            }
        }
    }
}
