//! Pure P2 aperture bounds and descriptor model.
use crate::boot::address::{ByteSize, PhysAddr, VirtAddr};
use crate::platform::intake::{self, Error, Span};
pub const APERTURE: u64 = 0x8000_0000;
pub const TABLE_PAGES: usize = 6;
pub const LEAF_CAPACITY: usize = 5 * 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub physical_page: PhysAddr,
    pub offset: usize,
    pub pages: usize,
}
impl Plan {
    pub fn new(span: Span, coverage: Span, image: Span) -> Result<Self, Error> {
        intake::placement(span, coverage, image)?;
        let start = span.base.raw() & !4095;
        let end = span
            .end()
            .ok_or(Error::Unreachable)?
            .raw()
            .checked_add(4095)
            .ok_or(Error::Unreachable)?
            & !4095;
        let rounded = Span {
            base: PhysAddr::new(start),
            len: ByteSize(end - start),
        };
        if !coverage.contains(rounded) {
            return Err(Error::Unreachable);
        }
        if rounded.overlaps(image) {
            return Err(Error::ImageOverlap);
        }
        let pages = usize::try_from((end - start) / 4096).map_err(|_| Error::Size)?;
        if pages > LEAF_CAPACITY {
            return Err(Error::Size);
        }
        Ok(Self {
            physical_page: rounded.base,
            offset: (span.base.raw() - start) as usize,
            pages,
        })
    }
    pub fn virtual_start(self) -> VirtAddr {
        VirtAddr::new(APERTURE + self.offset as u64)
    }
    pub fn descriptor(self, page: usize) -> Option<u64> {
        if page >= self.pages {
            return None;
        }
        let physical = self
            .physical_page
            .checked_add(ByteSize((page as u64).checked_mul(4096)?))?;
        if physical.raw() >= 1 << 48 {
            return None;
        }
        // AF, EL2 AP[1] RES1, RO, XN, existing MAIR[1] Normal WB.
        Some(physical.raw() | 3 | (1 << 10) | (1 << 6) | (1 << 7) | (1 << 54) | (1 << 2))
    }
}
