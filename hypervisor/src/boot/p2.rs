//! Reference boot adapter: trusted minimum RAM envelope, not discovered RAM.
use super::{
    address::{ByteSize, PhysAddr},
    console,
};
use crate::{
    arch::aarch64::stage1::dtb::DtbWindow,
    platform::{discovery, intake::Span},
};
pub(crate) struct BootReadEnvelope {
    coverage: Span,
}
impl BootReadEnvelope {
    pub(crate) fn range(&self) -> Span {
        self.coverage
    }
}

fn stop(class: &str, detail: &str) -> ! {
    use core::fmt::Write;
    let mut line = super::fatal_line::Line::new();
    let _ = write!(line, "ZELYR P2 REJECT reason={class} detail={detail}");
    console::transport_line(line.as_str());
    loop {
        crate::arch::aarch64::idle::wait_for_interrupt();
    }
}
pub(super) fn run(pointer: PhysAddr) -> ! {
    use core::fmt::Write;
    let mut line = super::fatal_line::Line::new();
    let _ = write!(line, "ZELYR P2 INTAKE pointer={:#x}", pointer.raw());
    console::transport_line(line.as_str());
    let coverage = Span {
        base: PhysAddr::new(0x4000_0000),
        len: ByteSize(128 * 1024 * 1024),
    };
    let window = match DtbWindow::open(pointer, BootReadEnvelope { coverage }) {
        Ok(v) => v,
        Err(e) => stop(e.class(), e.detail()),
    };
    let dtb = match window.validate() {
        Ok(v) => v,
        Err(e) => stop(e.class(), e.detail()),
    };
    let mut line = super::fatal_line::Line::new();
    let _ = write!(
        line,
        "ZELYR P2 INTAKE complete bytes={} version={} nodes={} props={} reservations={}",
        dtb.range().len.0,
        dtb.version(),
        dtb.node_count(),
        dtb.properties(),
        dtb.reservations().count()
    );
    console::transport_line(line.as_str());
    let facts = match discovery::normalize(&dtb) {
        Ok(v) => v,
        Err(e) => stop(e.class(), e.class()),
    };
    let mut line = super::fatal_line::Line::new();
    let _ = write!(
        line,
        "ZELYR P2 DISCOVERY complete cpus={} banks={} boot={}",
        facts.cpus().len(),
        facts.banks().len(),
        facts.boot_cpu()
    );
    console::transport_line(line.as_str());
    let mut line = super::fatal_line::Line::new();
    let _ = write!(
        line,
        "ZELYR P2 FACTS gic={} timer={} psci={}",
        facts.gic().label(),
        facts.timer().label(),
        facts.psci().label()
    );
    console::transport_line(line.as_str());
    loop {
        core::hint::black_box((&window, &dtb, &facts));
        crate::arch::aarch64::idle::wait_for_interrupt();
    }
}
