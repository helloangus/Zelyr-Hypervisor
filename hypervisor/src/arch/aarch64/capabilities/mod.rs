//! W03 phase entry and once-published report. W09 owns wiring.
mod facts;
#[cfg(feature = "p1-w11-nc2")]
mod validation_nc2;
pub(crate) use facts::{FactId, FactRecord};
// W04 needs the observation vocabulary when its consumer lands.
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};
#[allow(unused_imports)]
pub(crate) use facts::Observation;
use facts::{CapabilityReport, RawRegisters};

struct ReportCell {
    // Unpublished -> publishing -> published; no retry or reset.
    state: AtomicU8,
    value: UnsafeCell<Option<CapabilityReport>>,
}
// SAFETY: compare_exchange admits one writer. Readers acquire state 2
// only after release publication; storage remains immutable thereafter.
// Early/repeated access panics before touching storage. FC-INVARIANT; U-004.
unsafe impl Sync for ReportCell {}
static CAPABILITIES: ReportCell = ReportCell {
    state: AtomicU8::new(0),
    value: UnsafeCell::new(None),
};
impl ReportCell {
    fn publish(&self, report: CapabilityReport) {
        if self
            .state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            panic!("CAPABILITIES published twice");
        }
        // SAFETY: this writer uniquely owns state 1; no reader may borrow
        // until state 2. The cell is valid/aligned. FC-INVARIANT; U-004.
        unsafe {
            *self.value.get() = Some(report);
        }
        self.state.store(2, Ordering::Release);
    }
    fn get(&self) -> &CapabilityReport {
        if self.state.load(Ordering::Acquire) != 2 {
            panic!("CAPABILITIES queried before publication");
        }
        // SAFETY: acquire sees complete publication; no future writes.
        // Borrow lifetime is bounded by self. FC-INVARIANT; U-004.
        match unsafe { &*self.value.get() } {
            Some(report) => report,
            None => panic!("CAPABILITIES publication invariant"),
        }
    }
}
/// After W02 establishment, on the boot CPU with DAIF masked.
#[allow(dead_code)] // W09 owns phase wiring; remove when W09 integrates.
pub(crate) fn build_capability_report() {
    let mmfr0 = read_id_aa64mmfr0();
    #[cfg(feature = "p1-w11-nc2")]
    let mmfr0 = validation_nc2::inject_nc2_sample(mmfr0);
    let report = CapabilityReport::from_registers(&RawRegisters {
        current_el: read_current_el(),
        mpidr: read_mpidr(),
        pfr0: read_id_aa64pfr0(),
        mmfr0,
        mmfr1: read_id_aa64mmfr1(),
        cntfrq: read_cntfrq(),
    });
    if let Err(rejection) = report.verify_required() {
        panic!(
            "cap-reject fact={} {}",
            rejection.fact.label(),
            rejection.reason
        );
    }
    CAPABILITIES.publish(report);
}
#[allow(dead_code)] // W04 consumes this after W09 publishes.
pub(crate) fn query(id: FactId) -> FactRecord {
    CAPABILITIES.get().query(id)
}
#[allow(dead_code)] // W09 console phase supplies W06 transport exactly once.
pub(crate) fn render_report(emit: &mut dyn FnMut(&str)) {
    CAPABILITIES.get().render(emit);
}

fn read_current_el() -> u64 {
    let value;
    // SAFETY: CurrentEL is readable at W01-established EL2 under
    // the trusted firmware contract; no memory/stack side effects.
    // A firmware trap violates that contract (FC-INVARIANT); U-003.
    unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn read_mpidr() -> u64 {
    let value;
    // SAFETY: MPIDR_EL1 is readable at W01-established EL2 under
    // the trusted firmware contract; no memory/stack side effects.
    // A firmware trap violates that contract (FC-INVARIANT); U-003.
    unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn read_id_aa64pfr0() -> u64 {
    let value;
    // SAFETY: ID_AA64PFR0_EL1 is readable at W01-established EL2 under
    // the trusted firmware contract; no memory/stack side effects.
    // A firmware trap violates that contract (FC-INVARIANT); U-003.
    unsafe {
        core::arch::asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn read_id_aa64mmfr0() -> u64 {
    let value;
    // SAFETY: ID_AA64MMFR0_EL1 is readable at W01-established EL2 under
    // the trusted firmware contract; no memory/stack side effects.
    // A firmware trap violates that contract (FC-INVARIANT); U-003.
    unsafe {
        core::arch::asm!("mrs {}, ID_AA64MMFR0_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn read_id_aa64mmfr1() -> u64 {
    let value;
    // SAFETY: ID_AA64MMFR1_EL1 is readable at W01-established EL2 under
    // the trusted firmware contract; no memory/stack side effects.
    // A firmware trap violates that contract (FC-INVARIANT); U-003.
    unsafe {
        core::arch::asm!("mrs {}, ID_AA64MMFR1_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn read_cntfrq() -> u64 {
    let value;
    // SAFETY: CNTFRQ_EL0 is readable at W01-established EL2 under
    // the trusted firmware contract; no memory/stack side effects.
    // A firmware trap violates that contract (FC-INVARIANT); U-003.
    unsafe {
        core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}
