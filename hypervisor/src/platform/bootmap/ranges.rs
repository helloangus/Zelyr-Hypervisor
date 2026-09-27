//! Checked physical frame arithmetic; no mapping or physical access authority.
use super::{MapFatal, SourceId};
use crate::{
    boot::address::{ByteSize, PhysAddr},
    platform::intake::Span,
};

pub const PAGE_BYTES: u64 = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrameNum(u64);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageCount(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysFrameRange {
    first: PhysFrameNum,
    end: PhysFrameNum,
}
impl PhysFrameNum {
    pub fn from_address(address: PhysAddr) -> Option<Self> {
        address
            .raw()
            .is_multiple_of(PAGE_BYTES)
            .then_some(Self(address.raw() / PAGE_BYTES))
    }
    pub fn address(self) -> PhysAddr {
        PhysAddr::new(self.0 * PAGE_BYTES)
    }
}
impl PageCount {
    pub const fn new(pages: u64) -> Self {
        Self(pages)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
}
impl PhysFrameRange {
    pub fn new(first: PhysFrameNum, count: PageCount) -> Option<Self> {
        let end = first.0.checked_add(count.0)?;
        // The exclusive byte end must also be representable.
        if count.0 == 0 || end > u64::MAX / PAGE_BYTES {
            return None;
        }
        Some(Self {
            first,
            end: PhysFrameNum(end),
        })
    }
    pub fn first(self) -> PhysFrameNum {
        self.first
    }
    pub fn end(self) -> PhysFrameNum {
        self.end
    }
    pub fn count(self) -> PageCount {
        PageCount(self.end.0 - self.first.0)
    }
    pub fn contains(self, frame: PhysFrameNum) -> bool {
        self.first <= frame && frame < self.end
    }
    pub fn contains_range(self, other: Self) -> bool {
        self.first <= other.first && other.end <= self.end
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.first < other.end && other.first < self.end
    }
    pub fn adjacent(self, other: Self) -> bool {
        self.end == other.first || other.end == self.first
    }
    pub fn intersect(self, other: Self) -> Option<Self> {
        Self::between(self.first.max(other.first), self.end.min(other.end))
    }
    pub(super) fn between(first: PhysFrameNum, end: PhysFrameNum) -> Option<Self> {
        Self::new(first, PageCount(end.0.checked_sub(first.0)?))
    }
    pub fn bytes(self) -> Span {
        Span {
            base: self.first.address(),
            len: ByteSize(self.count().0 * PAGE_BYTES),
        }
    }
}
fn convert(span: Span, source: SourceId, round: bool) -> Result<PhysFrameRange, MapFatal> {
    if span.len.0 == 0 {
        return Err(MapFatal::MissingRequiredRange { source });
    }
    let end = span.end().ok_or(MapFatal::RangeOverflow { source })?.raw();
    let start = span.base.raw();
    let (start, end) = if round {
        let end = end
            .checked_add((PAGE_BYTES - end % PAGE_BYTES) % PAGE_BYTES)
            .ok_or(MapFatal::RangeOverflow { source })?;
        (start - start % PAGE_BYTES, end)
    } else {
        if !start.is_multiple_of(PAGE_BYTES) || !span.len.0.is_multiple_of(PAGE_BYTES) {
            return Err(MapFatal::UnalignedRange { source });
        }
        (start, end)
    };
    PhysFrameRange::new(
        PhysFrameNum(start / PAGE_BYTES),
        PageCount((end - start) / PAGE_BYTES),
    )
    .ok_or(MapFatal::RangeOverflow { source })
}
pub fn ram_to_frames(span: Span, source: SourceId) -> Result<PhysFrameRange, MapFatal> {
    convert(span, source, false)
}
pub fn protect_to_frames(span: Span, source: SourceId) -> Result<PhysFrameRange, MapFatal> {
    convert(span, source, true)
}
