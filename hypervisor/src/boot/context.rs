//! Boot-parameter retention (`BootContext`) with its audited
//! once-publication boundary.
//!
//! Contract: P1-W02 design, Rust-runtime contracts §2. The retained values
//! originate from the W01 boot contract's trusted boundary; no downstream
//! consumer may treat them as guest-influenced data in P1.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

// W08 extends the original newtype in the shared boot address module.
pub(crate) use super::address::PhysAddr;

/// Retained boot parameters per the W01 boot contract §4: `x0` is the DTB
/// pointer (T2 guaranteed non-zero), `x1`–`x3` are retained uninterpreted
/// and must not be relied upon by any design.
pub(crate) struct BootContext {
    dtb: Option<PhysAddr>,
    reserved: [u64; 3],
}

impl BootContext {
    const UNPUBLISHED: BootContext = BootContext {
        dtb: None,
        reserved: [0; 3],
    };

    /// Maps the forwarded boot registers. `x0 == 0` cannot occur per the
    /// W01 T2 guarantee; the publisher treats it as the invariant violation
    /// it would be rather than fabricating a context.
    fn from_registers(x0: u64, x1: u64, x2: u64, x3: u64) -> BootContext {
        let dtb = if x0 == 0 {
            None
        } else {
            Some(PhysAddr::new(x0))
        };
        BootContext {
            dtb,
            reserved: [x1, x2, x3],
        }
    }

    /// Read-only accessors; the interior is immutable after publication.
    /// Contracted read API for W03–W09 and the P2 handoff, none of which
    /// exist yet — the scoped allow carries that justification.
    #[allow(dead_code)] // W03–W09/P2 read these per their designs; API fixed now
    pub(crate) fn dtb(&self) -> Option<PhysAddr> {
        self.dtb
    }

    #[allow(dead_code)] // W03–W09/P2 read these per their designs; API fixed now
    pub(crate) fn reserved(&self) -> &[u64; 3] {
        &self.reserved
    }
}

struct BootContextCell {
    published: AtomicBool,
    value: UnsafeCell<BootContext>,
}

// SAFETY: publish-once discipline — `publish_boot_context` is the only
// writer, runs once on the single boot CPU with DAIF masked before any
// consumer exists, and every read is ordered after the published flag by the
// straight-line establishment order (no re-entry, no concurrency in P1).
// Failure if violated: hypervisor-invariant violation, terminal via the
// panic route. Inventory: U-001.
unsafe impl Sync for BootContextCell {}

static BOOT_CONTEXT: BootContextCell = BootContextCell {
    published: AtomicBool::new(false),
    value: UnsafeCell::new(BootContext::UNPUBLISHED),
};

/// Establishment stage 7: build the context from the forwarded registers
/// and publish it exactly once, before Runtime.complete.
pub(crate) fn publish_boot_context(x0: u64, x1: u64, x2: u64, x3: u64) {
    let boot_context = BootContext::from_registers(x0, x1, x2, x3);
    if boot_context.dtb().is_none() {
        panic!("W01 transfer guarantee violated: x0 (DTB pointer) is zero");
    }
    if BOOT_CONTEXT.published.swap(true, Ordering::Relaxed) {
        panic!("BOOT_CONTEXT published twice");
    }
    // SAFETY: publication is the single write of the cell; it runs once on
    // the boot CPU with DAIF masked, strictly before any consumer exists
    // (establishment order, W02 design §2). Relaxed ordering suffices on
    // the single executing CPU; the flag exists to detect a second
    // publication, not to synchronize readers. Failure if the invariant is
    // broken: hypervisor-invariant violation, terminal. Inventory: U-001.
    unsafe {
        *BOOT_CONTEXT.value.get() = boot_context;
    }
}
