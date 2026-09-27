//! Reference boot adapter: trusted minimum RAM envelope, not discovered RAM.
use super::{
    address::{ByteSize, PhysAddr, VirtAddr},
    console,
};
use crate::{
    arch::aarch64::stage1::dtb::DtbWindow,
    platform::{
        bootmap::{BootMapBuilder, MapStorage},
        discovery,
        intake::{Span, ValidatedBootDtb},
    },
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
    memory_map(&window, &dtb, &facts)
}

/// W03 owns this non-returning frame and its backing until restart. Keeping it
/// separate prevents its array from overlapping W02's normalization call stack.
#[inline(never)]
fn memory_map(
    window: &DtbWindow,
    dtb: &ValidatedBootDtb<'_>,
    facts: &discovery::PlatformInfo,
) -> ! {
    use core::fmt::Write;
    // Const initialization emits no runtime full-MapStorage constructor or
    // by-value map return. The W03 pinned-binary stack audit covers this frame.
    let mut storage = const { MapStorage::new() };
    let storage_extent = Span {
        base: VirtAddr::new(core::ptr::from_ref(&storage) as u64).p1_identity_physical(),
        len: ByteSize(core::mem::size_of::<MapStorage>() as u64),
    };
    let image = window.image_range();
    if !image.contains(storage_extent) {
        stop("MapStoragePlacement", "outside-image");
    }
    let draft = match BootMapBuilder::draft_in(&mut storage, facts, dtb.range(), image) {
        Ok(map) => map,
        Err(error) => map_stop(error),
    };
    console::transport_line("ZELYR P2 MAP draft ready");
    // W04 inserts its metadata planner here. No allocator exists in this
    // binary, so there are no allocator metadata extents to reserve yet.
    let map = match draft.seal(&[]) {
        Ok(map) => map,
        Err(error) => map_stop(error),
    };
    let summary = map.summary();
    let mut line = super::fatal_line::Line::new();
    let _ = write!(
        line,
        "ZELYR P2 MAP sealed ram={} allocatable={} protected={} metadata=0 allocator=absent",
        summary.ram_frames.get(),
        summary.allocatable_frames.get(),
        summary.ram_frames.get() - summary.allocatable_frames.get()
    );
    console::transport_line(line.as_str());
    let mut line = super::fatal_line::Line::new();
    let _ = write!(
        line,
        "ZELYR P2 MAP bounds image={:#x}+{:#x} dtb={:#x}+{:#x} storage={}",
        image.base.raw(),
        image.len.0,
        dtb.range().base.raw(),
        dtb.range().len.0,
        storage_extent.len.0
    );
    console::transport_line(line.as_str());
    let mut line = super::fatal_line::Line::new();
    let _ = write!(
        line,
        "ZELYR P2 MAP records banks={} sources={} entries={} clips={} outside={}",
        map.ram_spans().count(),
        map.source_ledger().count(),
        map.entries().count(),
        map.clips().count(),
        map.anomalies().outside_ram_warnings
    );
    console::transport_line(line.as_str());
    loop {
        // Retain the window/validated input, facts and map/backing lifetimes.
        core::hint::black_box((window, dtb, facts, &map));
        crate::arch::aarch64::idle::wait_for_interrupt();
    }
}
fn map_stop(error: crate::platform::bootmap::MapFatal) -> ! {
    use core::fmt::Write;
    let mut line = super::fatal_line::Line::new();
    let _ = write!(line, "ZELYR P2 MAP REJECT {error:?}");
    console::transport_line(line.as_str());
    loop {
        crate::arch::aarch64::idle::wait_for_interrupt();
    }
}
