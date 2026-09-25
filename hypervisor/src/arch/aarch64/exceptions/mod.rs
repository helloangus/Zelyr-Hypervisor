//! W05 EL2 vectors, guarded fatal capture, and pre-arm diagnostic route.
mod model;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
pub(crate) use model::{
    ExceptionCategory, ExceptionDisposition, ExceptionFrame, OriginClass, SyndromeClass,
};
use model::{classify, far_valid, syndrome_class};

// SAFETY: U-009. The sixteen branch-only slots enter a guard-first assembly
// protocol, capture original GPRs before scratch use, then switch to the
// existing aligned stack for a terminal Rust call. W01/W04 establish EL2,
// masked DAIF and no live FP state; frame/guard/code mappings must be valid.
// Violating these premises is FC-INVARIANT, not a recoverable exception.
core::arch::global_asm!(
    include_str!("entry.S"),
    frame_size = const model::FRAME_SIZE,
    sp_offset = const model::SP_OFFSET,
    pc_offset = const model::PC_OFFSET,
    spsr_offset = const model::SPSR_OFFSET,
    esr_offset = const model::ESR_OFFSET,
    far_offset = const model::FAR_OFFSET,
    hpfar_offset = const model::HPFAR_OFFSET,
    flags_offset = const model::FLAGS_OFFSET,
    origin_offset = const model::ORIGIN_OFFSET,
    category_offset = const model::CATEGORY_OFFSET,
);

// SAFETY: U-009. Symbols are uniquely defined in entry.S and kept by the
// image linker script. Guard bytes have no Rust references and are accessed
// only before installation or inside assembly under the entry protocol.
unsafe extern "C" {
    static p1_el2_vector_table: u8;
    static p1_exception_guard: core::cell::UnsafeCell<u64>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VectorStatus {
    Unestablished,
    Established,
}
#[derive(Clone, Copy)]
struct VectorBase(u64);
static INSTALL_STARTED: AtomicBool = AtomicBool::new(false);
static VECTOR_BASE: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)] // W06/W07/W09 consume this declaration after integration.
pub(crate) fn vector_status() -> VectorStatus {
    if VECTOR_BASE.load(Ordering::Acquire) == 0 {
        VectorStatus::Unestablished
    } else {
        VectorStatus::Established
    }
}
#[allow(dead_code)] // W08 consumes retained vector identity; no live sysreg read.
pub(crate) fn vector_base() -> Option<u64> {
    match VECTOR_BASE.load(Ordering::Acquire) {
        0 => None,
        base => Some(base),
    }
}

/// W09 exceptions phase, after W04 declares C1–C4.
#[allow(dead_code)] // W09 owns the actual phase call and removes this allowance.
pub(crate) fn install_el2_exception_entry() {
    assert_preconditions();
    if INSTALL_STARTED.swap(true, Ordering::Relaxed) {
        panic!("vector install repeated");
    }
    let base = VectorBase(core::ptr::addr_of!(p1_el2_vector_table) as u64);
    if base.0 & 0xfff != 0 {
        fail_vector(VectorError {
            control: VectorControl::Vbar,
            expected: 0,
            observed: base.0 & 0xfff,
        });
    }
    // SAFETY: U-009. First install on the masked boot CPU, before VBAR is
    // live; no exception path yet accesses this assembly-owned guard.
    // A second install is rejected above. FC-INVARIANT if violated.
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of!(p1_exception_guard)
                .cast_mut()
                .cast::<u64>(),
            1,
        );
    }
    vbar_write(base);
    // VBAR is a context-synchronizing control: the write must become
    // effective before a dependent read or readiness publication.
    exception_barrier_isb();
    let observed = vbar_read();
    if observed != base.0 {
        fail_vector(VectorError {
            control: VectorControl::Vbar,
            expected: base.0,
            observed,
        });
    }
    VECTOR_BASE.store(base.0, Ordering::Release);
}

fn assert_preconditions() {
    use super::baseline::{BaselineCategory, BaselineStatus, baseline_status};
    for (category, control) in [
        (BaselineCategory::C1, VectorControl::SpSel),
        (BaselineCategory::C2, VectorControl::HcrRouting),
        (BaselineCategory::C3, VectorControl::HcrTraps),
        (BaselineCategory::C4, VectorControl::CptrFp),
    ] {
        if baseline_status(category) != BaselineStatus::Established {
            fail_vector(VectorError {
                control,
                expected: 1,
                observed: 0,
            });
        }
    }
}

#[derive(Debug)]
enum VectorControl {
    Vbar,
    SpSel,
    HcrRouting,
    HcrTraps,
    CptrFp,
}
struct VectorError {
    control: VectorControl,
    expected: u64,
    observed: u64,
}
fn fail_vector(error: VectorError) -> ! {
    panic!(
        "vector-reject {:?} expected={:016x} observed={:016x}",
        error.control, error.expected, error.observed
    );
}

fn vbar_write(base: VectorBase) {
    // SAFETY: U-008. EL2 established; link and runtime checks establish
    // VBAR alignment and mapped executable table. No memory/stack effects.
    // Caller is the sole once-install writer. FC-INVARIANT if violated.
    unsafe {
        core::arch::asm!("msr VBAR_EL2, {}", in(reg) base.0, options(nostack, preserves_flags));
    }
}
fn vbar_read() -> u64 {
    let value;
    // SAFETY: U-008. W01 established EL2; VBAR read is side-effect-free.
    // Only install reads it for verification. FC-INVARIANT if violated.
    unsafe {
        core::arch::asm!("mrs {}, VBAR_EL2", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}
fn exception_barrier_isb() {
    // SAFETY: U-008. Context synchronization after VBAR write before phase
    // publication; no stack use. FC-INVARIANT if the caller order changes.
    unsafe {
        core::arch::asm!("isb", options(nostack, preserves_flags));
    }
}

/// # Safety
/// Called only by guarded entry.S with a complete, uniquely owned static
/// frame and the mapped/aligned existing boot stack. Never returns.
// SAFETY: U-009. Unique assembly symbol and signature match the branch ABI.
#[unsafe(no_mangle)]
unsafe extern "C" fn p1_exception_rust_entry(frame: *mut ExceptionFrame) -> ! {
    // SAFETY: U-009. Assembly initialized all fields (including valid bool
    // bytes), claimed the one guard, and will never write again. Recursive
    // entry stops before capture. Pointer is valid/aligned; FC-INVARIANT.
    let frame = unsafe { &mut *frame };
    if let Some(category) = ExceptionCategory::from_slot(frame.category) {
        frame.far_valid = far_valid(frame.esr, category);
        if frame.far_valid {
            // SAFETY: U-008. We are in guarded EL2 synchronous capture;
            // the architectural category/FnV check establishes FAR validity.
            // No stack/memory side effects; FC-INVARIANT if violated.
            unsafe {
                core::arch::asm!("mrs {}, FAR_EL2", out(reg) frame.far, options(nomem, nostack, preserves_flags));
            }
        }
    }
    route_classified(frame)
}

/// W07 owns the post-arm report; W05 retains the pre-arm summary.
pub(crate) fn route_classified(frame: &ExceptionFrame) -> ! {
    let origin = OriginClass::from_slot(frame.origin);
    let category = ExceptionCategory::from_slot(frame.category);
    let disposition = match (origin, category) {
        (Some(origin), Some(category)) => classify(origin, category),
        _ => ExceptionDisposition::UnexpectedEvent,
    };
    let class = match category {
        Some(ExceptionCategory::Synchronous) => syndrome_class(frame.esr),
        Some(ExceptionCategory::SError) => SyndromeClass::SError,
        _ => SyndromeClass::Unknown,
    };
    if crate::boot::fatal::fatal_path_ready() {
        crate::boot::fatal::report_fatal_exception(frame, class, disposition);
    }
    emit_pre_arm_summary(frame, class, disposition);
    loop {
        core::hint::spin_loop();
    }
}

fn emit_pre_arm_summary(
    frame: &ExceptionFrame,
    class: SyndromeClass,
    disposition: ExceptionDisposition,
) {
    let mut line = SummaryBuffer {
        bytes: [0; 512],
        len: 0,
    };
    let category =
        ExceptionCategory::from_slot(frame.category).map_or("invalid", ExceptionCategory::label);
    let origin = OriginClass::from_slot(frame.origin).map_or("invalid", OriginClass::label);
    let _ = write!(
        line,
        "ZELYR P1 FATAL exc cat={category} org={origin} class={} disposition={} esr={:016x} pc={:016x} far=",
        class.label(),
        disposition.label(),
        frame.esr,
        frame.pc
    );
    if frame.far_valid {
        let _ = write!(line, "{:016x}", frame.far);
    } else {
        let _ = line.write_str("na");
    }
    // W09 replaces unavailable with its exception-safe tracker snapshot.
    let identity = &crate::boot::identity::BUILD_IDENTITY;
    let _ = write!(
        line,
        " ph=unavailable fc=FC-INVARIANT inv=el2_exception_no_safe_resume site=vector version={} arch={} profile={} rev={} dirty={}\r\n",
        identity.project_version,
        identity.target_architecture,
        identity.build_profile,
        identity.source_revision,
        identity.dirty
    );
    crate::boot::writer::early_write_bytes(&line.bytes[..line.len]);
}
struct SummaryBuffer {
    bytes: [u8; 512],
    len: usize,
}
impl Write for SummaryBuffer {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let taken = text.len().min(self.bytes.len() - self.len);
        self.bytes[self.len..self.len + taken].copy_from_slice(&text.as_bytes()[..taken]);
        self.len += taken;
        Ok(())
    }
}
