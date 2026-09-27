//! W03: bounded RAM partition and immutable draft -> sealed ownership map.
//! No allocation, physical access, hardware policy, or allocator placement.
mod build;
mod classify;
pub mod ranges;
mod seal;
use super::{discovery::PlatformInfo, intake::Span};
pub use ranges::{PageCount, PhysFrameNum, PhysFrameRange};

pub const MAX_MEMORY_BANKS: usize = 8;
pub const MAX_INPUT_SOURCES: usize = 38;
pub const MAX_ALLOCATABLE_SPANS: usize = MAX_MEMORY_BANKS + MAX_INPUT_SOURCES;
pub const MAX_METADATA_RANGES: usize = MAX_ALLOCATABLE_SPANS;
pub const MAX_SOURCES: usize = MAX_INPUT_SOURCES + MAX_METADATA_RANGES;
pub const MAX_MAP_ENTRIES: usize = 2 * (MAX_MEMORY_BANKS + MAX_SOURCES);
const MAX_CLIPS: usize = MAX_MEMORY_BANKS * MAX_SOURCES;
const _: () = assert!(MAX_SOURCES <= 128);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegionClass {
    Allocatable,
    HypervisorImage,
    HypervisorMetadata,
    ActiveDtb,
    FirmwareReserved,
    BootArtifact,
    SharedProtection,
}
impl RegionClass {
    pub fn is_protected(self) -> bool {
        self != Self::Allocatable
    }
    fn index(self) -> usize {
        self as usize
    }
}
/// Ordinals refer to the original W02 lists, including dropped zero entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceId {
    Ram(usize),
    HypervisorImage,
    ActiveDtb,
    DtbReservation(usize),
    ReservedMemoryNode(usize),
    BootArtifact(usize),
    AllocatorMetadata(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactList {
    Ram,
    Reserved,
    Artifact,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SealViolation {
    Capacity,
    OutsideAllocatable(usize),
    Overlap(usize, usize),
    Audit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapFatal {
    MissingRequiredRange { source: SourceId },
    UnusableFact { list: FactList, ordinal: usize },
    UnalignedRange { source: SourceId },
    RangeOverflow { source: SourceId },
    RamOverlap { a: SourceId, b: SourceId },
    ProtectionConflict { a: SourceId, b: SourceId },
    DuplicateSource { source: SourceId },
    CapacityExhausted { which: &'static str },
    SealRejected { violation: SealViolation },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProtectionFlags {
    pub no_map: bool,
    pub reusable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtectionSource {
    pub id: SourceId,
    pub original: Span,
    pub rounded: PhysFrameRange,
    pub class: RegionClass,
    pub flags: ProtectionFlags,
}
/// Ledger-index set; bounded to 84 sources, never a physical address or handle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceSet(u128);
impl SourceSet {
    pub fn indices(self) -> impl Iterator<Item = usize> {
        (0..MAX_SOURCES).filter(move |i| self.0 & (1u128 << i) != 0)
    }
    fn insert(&mut self, index: usize) {
        self.0 |= 1u128 << index;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapEntry {
    pub range: PhysFrameRange,
    pub class: RegionClass,
    pub sources: SourceSet,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// One source intersected with one normalized RAM-union bank.
/// `bank` indexes the map's `ram_spans()` iterator, not the original W02 list.
pub struct ClipRecord {
    pub protected: SourceId,
    pub bank: usize,
    pub kept: PhysFrameRange,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapAnomaly {
    pub merged_banks: usize,
    pub deduped_banks: usize,
    pub deduped_protections: usize,
    pub dropped_zero_ram: usize,
    pub dropped_zero_protection: usize,
    pub outside_ram_warnings: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassQuery {
    Allocatable,
    Protected(RegionClass),
    OutsideRam,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapSummary {
    pub ram_frames: PageCount,
    pub allocatable_frames: PageCount,
    pub hole_frames: PageCount,
    pub outside_ram_warning_count: usize,
    protected: [PageCount; 7],
}
impl MapSummary {
    pub fn protected_frames(&self, class: RegionClass) -> PageCount {
        self.protected[class.index()]
    }
}
/// Private bounded storage: all live elements are Some; push never truncates.
struct Buffer<T: Copy, const N: usize> {
    items: [Option<T>; N],
    len: usize,
}
impl<T: Copy, const N: usize> Buffer<T, N> {
    const fn new() -> Self {
        Self {
            items: [None; N],
            len: 0,
        }
    }
    fn push(&mut self, value: T, which: &'static str) -> Result<(), MapFatal> {
        if self.len == N {
            return Err(MapFatal::CapacityExhausted { which });
        }
        self.items[self.len] = Some(value);
        self.len += 1;
        Ok(())
    }
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.items[..self.len].iter().flatten()
    }
}
#[derive(Clone, Copy)]
struct Bank {
    range: PhysFrameRange,
    id: SourceId,
}
struct MapData {
    banks: Buffer<Bank, MAX_MEMORY_BANKS>,
    sources: Buffer<ProtectionSource, MAX_SOURCES>,
    entries: Buffer<MapEntry, MAX_MAP_ENTRIES>,
    clips: Buffer<ClipRecord, MAX_CLIPS>,
    anomalies: MapAnomaly,
}
impl MapData {
    fn class_at(&self, frame: PhysFrameNum) -> ClassQuery {
        let entries = &self.entries.items[..self.entries.len];
        let i = entries.partition_point(|entry| entry.is_some_and(|e| e.range.end() <= frame));
        match entries
            .get(i)
            .and_then(Option::as_ref)
            .filter(|e| e.range.contains(frame))
        {
            Some(e) if e.class.is_protected() => ClassQuery::Protected(e.class),
            Some(_) => ClassQuery::Allocatable,
            None => ClassQuery::OutsideRam,
        }
    }
}
pub struct BootMapBuilder;
pub struct UnsealedMemoryMap {
    data: MapData,
}
/// Only successful seal can construct this authority; no Clone or mutation API.
pub struct BootMemoryMap {
    data: MapData,
    summary: MapSummary,
}
impl BootMapBuilder {
    pub fn draft(
        platform: &PlatformInfo,
        active_dtb: Span,
        image: Span,
    ) -> Result<UnsealedMemoryMap, MapFatal> {
        build::draft(platform, active_dtb, image)
    }
}
// Queries return bounded borrowed iterators rather than duplicate cached arrays.
macro_rules! map_queries {
    () => {
        pub fn ram_spans(&self) -> impl Iterator<Item = PhysFrameRange> + '_ {
            self.data.banks.iter().map(|bank| bank.range)
        }
        pub fn entries(&self) -> impl Iterator<Item = &MapEntry> {
            self.data.entries.iter()
        }
        pub fn allocatable_spans(&self) -> impl Iterator<Item = PhysFrameRange> + '_ {
            self.entries()
                .filter(|e| !e.class.is_protected())
                .map(|e| e.range)
        }
        pub fn protected_ranges(&self) -> impl Iterator<Item = &MapEntry> {
            self.entries().filter(|e| e.class.is_protected())
        }
        pub fn source_ledger(&self) -> impl Iterator<Item = &ProtectionSource> {
            self.data.sources.iter()
        }
        pub fn clips(&self) -> impl Iterator<Item = &ClipRecord> {
            self.data.clips.iter()
        }
        pub fn anomalies(&self) -> &MapAnomaly {
            &self.data.anomalies
        }
        pub fn class_at(&self, frame: PhysFrameNum) -> ClassQuery {
            self.data.class_at(frame)
        }
    };
}
impl UnsealedMemoryMap {
    map_queries!();
}
impl BootMemoryMap {
    map_queries!();
    pub fn summary(&self) -> &MapSummary {
        &self.summary
    }
    pub fn metadata_ledger(&self) -> impl Iterator<Item = &ProtectionSource> {
        self.source_ledger()
            .filter(|s| s.class == RegionClass::HypervisorMetadata)
    }
}

#[cfg(test)]
mod tests;
