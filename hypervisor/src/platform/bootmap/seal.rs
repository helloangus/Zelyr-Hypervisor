//! Metadata acceptance and independent coverage/source/accounting audit.
use super::*;
fn rejected(violation: SealViolation) -> MapFatal {
    MapFatal::SealRejected { violation }
}
impl<S: core::borrow::BorrowMut<MapStorage>> UnsealedMemoryMap<S> {
    /// Consume the draft; borrowed backing remains in place on every path.
    /// Failure poisons this storage and publishes no allocation authority.
    pub fn seal(mut self, metadata: &[PhysFrameRange]) -> Result<BootMemoryMap<S>, MapFatal> {
        let data = self.data.borrow_mut();
        if data.state != StorageState::Draft {
            return Err(MapFatal::StorageUnavailable);
        }
        data.state = StorageState::Sealing;
        match seal_data(data, metadata) {
            Ok(summary) => {
                data.state = StorageState::Sealed;
                Ok(BootMemoryMap {
                    data: self.data,
                    summary,
                })
            }
            Err(error) => {
                data.state = StorageState::Failed;
                Err(error)
            }
        }
    }
}
fn seal_data(data: &mut MapData, metadata: &[PhysFrameRange]) -> Result<MapSummary, MapFatal> {
    if metadata.len() > MAX_METADATA_RANGES {
        return Err(rejected(SealViolation::Capacity));
    }
    for (i, range) in metadata.iter().enumerate() {
        if !data
            .entries
            .iter()
            .any(|entry| !entry.class.is_protected() && entry.range.contains_range(*range))
        {
            return Err(rejected(SealViolation::OutsideAllocatable(i)));
        }
        for (j, other) in metadata[..i].iter().enumerate() {
            if range.overlaps(*other) {
                return Err(rejected(SealViolation::Overlap(j, i)));
            }
        }
    }
    for (i, range) in metadata.iter().enumerate() {
        classify::add(
            data,
            range.bytes(),
            SourceId::AllocatorMetadata(i),
            RegionClass::HypervisorMetadata,
            ProtectionFlags::default(),
        )?;
    }
    build::partition(data)?;
    audit(data)
}
/// Walk final entries against the RAM union, independently of the endpoint
/// sweep. Each entry must have exactly every intersecting source, each of which
/// must fully contain it. This proves protected-union equality, not just totals.
pub(super) fn audit(data: &MapData) -> Result<MapSummary, MapFatal> {
    let bad = || rejected(SealViolation::Audit);
    let mut summary = MapSummary {
        outside_ram_warning_count: data.anomalies.outside_ram_warnings,
        ..MapSummary::default()
    };
    let mut entries = data.entries.iter();
    let mut previous_end = None;
    for bank in data.banks.iter() {
        summary.ram_frames = summary
            .ram_frames
            .checked_add(bank.range.count())
            .ok_or_else(bad)?;
        if let Some(end) = previous_end {
            if end > bank.range.first() {
                return Err(bad());
            }
            if let Some(hole) = PhysFrameRange::between(end, bank.range.first()) {
                summary.hole_frames = summary
                    .hole_frames
                    .checked_add(hole.count())
                    .ok_or_else(bad)?;
            }
        }
        previous_end = Some(bank.range.end());
        let mut cursor = bank.range.first();
        while cursor < bank.range.end() {
            let entry = entries.next().ok_or_else(bad)?;
            if entry.range.first() != cursor || !bank.range.contains_range(entry.range) {
                return Err(bad());
            }
            let mut expected = SourceSet::default();
            let mut classes = [false; 7];
            for (i, source) in data.sources.iter().enumerate() {
                if source.rounded.overlaps(entry.range) {
                    if !source.rounded.contains_range(entry.range) {
                        return Err(bad());
                    }
                    expected.insert(i);
                    classes[source.class.index()] = true;
                }
            }
            let class_count = classes.iter().filter(|x| **x).count();
            let class_ok = match class_count {
                0 => entry.class == RegionClass::Allocatable,
                1 => classes[entry.class.index()] && entry.class.is_protected(),
                _ => entry.class == RegionClass::SharedProtection,
            };
            if expected != entry.sources || !class_ok {
                return Err(bad());
            }
            if entry.class.is_protected() {
                let total = &mut summary.protected[entry.class.index()];
                *total = total.checked_add(entry.range.count()).ok_or_else(bad)?;
            } else {
                summary.allocatable_frames = summary
                    .allocatable_frames
                    .checked_add(entry.range.count())
                    .ok_or_else(bad)?;
            }
            cursor = entry.range.end();
        }
    }
    if entries.next().is_some() {
        return Err(bad());
    }
    let mut total = summary.allocatable_frames;
    for count in summary.protected {
        total = total.checked_add(count).ok_or_else(bad)?;
    }
    if total != summary.ram_frames {
        return Err(bad());
    }
    Ok(summary)
}
