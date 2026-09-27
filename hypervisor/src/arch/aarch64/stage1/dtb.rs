//! P2 owns this one-shot post-P1 DTB aperture. No general mapping API.
use super::{
    STAGE1_TABLES,
    dtb_model::{Plan, TABLE_PAGES},
    regs,
};
use crate::{
    boot::address::{ByteSize, PhysAddr, VirtAddr},
    platform::intake::{self, Error, Span},
};
use core::{
    ptr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
#[repr(C, align(4096))]
struct ApertureTables([AtomicU64; TABLE_PAGES * 512]);
static TABLES: ApertureTables = ApertureTables([const { AtomicU64::new(0) }; TABLE_PAGES * 512]);
static CLAIMED: AtomicBool = AtomicBool::new(false);
pub(crate) struct DtbWindow {
    physical: PhysAddr,
    virtual_address: VirtAddr,
    len: usize,
    coverage: Span,
    image: Span,
}
fn synchronize() {
    regs::dsb_sy();
    regs::tlbi_alle2();
    regs::dsb_sy();
    regs::isb();
}
fn populate(plan: Plan) -> Result<(), Error> {
    for page in 0..plan.pages {
        let value = plan.descriptor(page).ok_or(Error::Unreachable)?;
        let entry = &TABLES.0[512 + page];
        match entry.compare_exchange(0, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {}
            Err(previous) if previous == value => {}
            Err(_) => return Err(Error::Unreachable),
        }
    }
    Ok(())
}
impl DtbWindow {
    pub(crate) fn open(
        physical: PhysAddr,
        envelope: crate::boot::p2::BootReadEnvelope,
    ) -> Result<Self, Error> {
        let coverage = envelope.range();
        let image = Span {
            base: PhysAddr::new(ptr::addr_of!(super::__p1_boot_code_start) as u64),
            len: ByteSize(
                (ptr::addr_of!(super::__p1_stack_end) as u64)
                    .checked_sub(ptr::addr_of!(super::__p1_boot_code_start) as u64)
                    .ok_or(Error::Unreachable)?,
            ),
        };
        let header_plan = Plan::new(
            Span {
                base: physical,
                len: ByteSize(40),
            },
            coverage,
            image,
        )?;
        if !super::STARTED.load(Ordering::Relaxed) {
            return Err(Error::Access("stage1-not-started"));
        }
        if regs::sctlr_read() & 1 == 0 {
            return Err(Error::Access("mmu-off"));
        }
        if CLAIMED.swap(true, Ordering::Relaxed) {
            return Err(Error::Access("aperture-repeated"));
        }
        let base = ptr::addr_of!(TABLES) as u64;
        let extent = Span {
            base: PhysAddr::new(base),
            len: ByteSize((TABLE_PAGES * 4096) as u64),
        };
        if base & 4095 != 0 || !image.contains(extent) {
            return Err(Error::Access("table-placement"));
        }
        populate(header_plan)?;
        for page in 0..5 {
            TABLES.0[page].store((base + (page as u64 + 1) * 4096) | 3, Ordering::Relaxed);
        }
        // Complete child tables before the only invalid-to-valid L1 publication.
        regs::dsb_sy();
        STAGE1_TABLES.0[2]
            .compare_exchange(0, base | 3, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| Error::Access("L1-collision"))?;
        synchronize();
        let mut owner = Self {
            physical,
            virtual_address: header_plan.virtual_start(),
            len: 40,
            coverage,
            image,
        };
        let length = intake::declared_size(owner.bytes())?;
        let full = Plan::new(
            Span {
                base: physical,
                len: ByteSize(length as u64),
            },
            coverage,
            image,
        )?;
        populate(full)?;
        synchronize();
        owner.len = length;
        Ok(owner)
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        // SAFETY: U-018. open() proves rounded trusted coverage and image
        // exclusion before populating RO/XN pages and ordering their use.
        // The sole owner never remaps; the canonical loader's DTB bytes have
        // no writers or DMA, remain initialized and immutable until restart.
        // The borrow cannot outlive this owner. False loader/EL2 premises are
        // terminal FC-PLATFORM/FC-INVARIANT, not recoverable guest failures.
        unsafe { core::slice::from_raw_parts(self.virtual_address.raw() as *const u8, self.len) }
    }
    pub(crate) fn validate(&self) -> Result<intake::ValidatedBootDtb<'_>, Error> {
        intake::validate(self.bytes(), self.physical, self.coverage, self.image)
    }
}
