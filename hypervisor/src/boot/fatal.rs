//! Single bounded P1 terminal report path; W09 owns phase state and routing.
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use super::console;
use super::fatal_line::Line;
use super::identity::{self, BUILD_IDENTITY};
use super::lifecycle::{FailureReason, InitPhase, LifecyclePosition, TRACKER};
use super::writer::early_write_bytes;
use crate::arch::aarch64::exceptions::{
    ExceptionCategory, ExceptionDisposition, ExceptionFrame, OriginClass, SyndromeClass,
};

const END_MARKER: &str = "ZELYR P1 REPORT END";
static FATAL_PATH_GUARD: AtomicBool = AtomicBool::new(false);
static FATAL_PATH_READY: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
enum Transport {
    Channel,
    Early,
}
impl Transport {
    fn select() -> Self {
        if console::channel_available() {
            Self::Channel
        } else {
            Self::Early
        }
    }
    fn line(self, line: &str) {
        match self {
            Self::Channel => console::console_write_line(line),
            Self::Early => {
                early_write_bytes(line.as_bytes());
                early_write_bytes(b"\r\n");
            }
        }
    }
    fn formatted(self, args: fmt::Arguments<'_>) {
        let mut line = Line::new();
        let _ = line.write_fmt(args);
        self.line(line.as_str());
    }
}

fn acquire_guard() {
    // Single masked P1 boot CPU; the atomic is one-way re-entry exclusion,
    // not cross-CPU publication. Neither guard nor readiness ever resets.
    if FATAL_PATH_GUARD.swap(true, Ordering::Relaxed) {
        bounded_stop();
    }
}
fn bounded_stop() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

fn current_el() -> u64 {
    let value;
    // SAFETY: U-011. W01 established AArch64 EL2. CurrentEL is a closed,
    // side-effect-free read with no memory or stack access. Guard held.
    unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn common(transport: Transport, position: LifecyclePosition) {
    let identity = &BUILD_IDENTITY;
    transport.formatted(format_args!(
        "build=version={} arch={} profile={} rev={} dirty={}",
        identity.project_version,
        identity.target_architecture,
        identity.build_profile,
        identity.source_revision,
        identity.dirty,
    ));
    let el = (current_el() >> 2) & 3;
    transport.formatted(format_args!("cpu=EL{el} el_ok={}", el == 2));
    match position {
        LifecyclePosition::PreBoot => transport.line("ph=unknown(raw=0)"),
        LifecyclePosition::Entered(phase) => {
            transport.formatted(format_args!("ph={}.enter", phase.label()))
        }
        LifecyclePosition::Completed(phase) => {
            transport.formatted(format_args!("ph={}.complete", phase.label()))
        }
        LifecyclePosition::Stable => transport.line("ph=stable"),
        LifecyclePosition::Unknown(raw) => {
            transport.formatted(format_args!("ph=unknown(raw={raw})"))
        }
    }
}

/// W02's single panic handler calls this after capturing its own approximate
/// handler-entry SP/LR. The shared guard contains a panic while reporting.
pub(super) fn report_panic(info: &PanicInfo<'_>, sp0: u64, lr0: u64) -> ! {
    acquire_guard();
    let transport = Transport::select();
    transport.line("ZELYR P1 PANIC kind=P");
    common(transport, TRACKER.position());
    transport.formatted(format_args!("msg={}", info.message()));
    match info.location() {
        Some(location) => transport.formatted(format_args!(
            "loc={}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )),
        None => transport.line("loc=na"),
    }
    transport.formatted(format_args!(
        "sp0={sp0:016x} lr0={lr0:016x} capture=handler-entry"
    ));
    transport.line("syndrome=na fault=na gprs=na");
    transport.line(END_MARKER);
    bounded_stop()
}

/// W05's router calls this only after it claims its distinct entry guard.
pub(crate) fn report_fatal_exception(
    frame: &ExceptionFrame,
    class: SyndromeClass,
    disposition: ExceptionDisposition,
) -> ! {
    acquire_guard();
    let transport = Transport::select();
    transport.line("ZELYR P1 FATAL kind=E");
    common(transport, TRACKER.position());
    let origin = OriginClass::from_slot(frame.origin).map_or("invalid", OriginClass::label);
    let category =
        ExceptionCategory::from_slot(frame.category).map_or("invalid", ExceptionCategory::label);
    transport.formatted(format_args!(
        "org={origin} cat={category} disp={}",
        disposition.label()
    ));
    transport.formatted(format_args!(
        "esr={:016x} cls={} il={}",
        frame.esr,
        class.label(),
        frame.esr & (1 << 25) != 0
    ));
    transport.formatted(format_args!(
        "pc={:016x} spsr={:016x}",
        frame.pc, frame.spsr
    ));
    if frame.far_valid {
        transport.formatted(format_args!("far={:016x}", frame.far));
    } else {
        transport.line("far=na");
    }
    if frame.hpfar_valid {
        transport.formatted(format_args!("hpfar={:016x}", frame.hpfar));
    } else {
        transport.line("hpfar=na");
    }
    for (index, value) in frame.x.iter().enumerate() {
        transport.formatted(format_args!("x{index:02}={value:016x}"));
    }
    transport.formatted(format_args!("sp={:016x}", frame.sp));
    transport.line("msg=na");
    transport.line(END_MARKER);
    bounded_stop()
}

/// W09 calls this only for post-arm Stage1/Stable phase failures.
pub(crate) fn report_fatal_phase(phase: InitPhase, reason: FailureReason) -> ! {
    acquire_guard();
    let transport = Transport::select();
    transport.line("ZELYR P1 FATAL kind=F");
    common(transport, TRACKER.position());
    transport.formatted(format_args!(
        "phase={} reason={}",
        phase.label(),
        reason.message()
    ));
    transport.line("pc=na return=na syndrome=na fault=na gprs=na");
    transport.line(END_MARKER);
    bounded_stop()
}

/// W09's fatal-path phase invokes this once after Console.complete.
pub(crate) fn arm_fatal_path() {
    if FATAL_PATH_READY.load(Ordering::Relaxed)
        || FATAL_PATH_GUARD.load(Ordering::Relaxed)
        || !identity::identity_resolvable()
    {
        readiness_failure();
    }
    // W02's fallback writer has a compile-time nonzero-base assertion.
    FATAL_PATH_READY.store(true, Ordering::Release);
}
pub(crate) fn fatal_path_ready() -> bool {
    FATAL_PATH_READY.load(Ordering::Acquire)
}
pub(crate) fn readiness_failure() -> ! {
    // Do not invoke the renderer whose readiness is being established.
    early_write_bytes(b"ZELYR P1 FATAL readiness fc=FC-INVARIANT inv=fatal_path_not_ready\r\n");
    bounded_stop()
}
