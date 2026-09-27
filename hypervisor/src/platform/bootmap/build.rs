//! Bounded endpoint sweep: only discovered RAM is ever partitioned.
use super::*;
pub(super) fn populate(
    data: &mut MapData,
    platform: &PlatformInfo,
    dtb: Span,
    image: Span,
) -> Result<(), MapFatal> {
    if data.state != StorageState::Fresh {
        return Err(MapFatal::StorageUnavailable);
    }
    data.state = StorageState::Building;
    let result = populate_fresh(data, platform, dtb, image);
    data.state = if result.is_ok() {
        StorageState::Draft
    } else {
        StorageState::Failed
    };
    result
}
fn populate_fresh(
    data: &mut MapData,
    platform: &PlatformInfo,
    dtb: Span,
    image: Span,
) -> Result<(), MapFatal> {
    classify::assemble(data, platform, dtb, image)?;
    collect_ram(data, platform.banks().iter())?;
    partition(data)?;
    super::seal::audit(data)?;
    Ok(())
}
pub(super) fn collect_ram<'a>(
    data: &mut MapData,
    facts: impl Iterator<Item = &'a crate::platform::discovery::Fact<Span>>,
) -> Result<(), MapFatal> {
    for (i, fact) in facts.enumerate() {
        let span = *classify::usable(fact, FactList::Ram, i)?;
        if span.len.0 == 0 {
            data.anomalies.dropped_zero_ram += 1;
            continue;
        }
        let id = SourceId::Ram(i);
        let range = ranges::ram_to_frames(span, id)?;
        if data.banks.iter().any(|b| b.range == range) {
            data.anomalies.deduped_banks += 1;
            continue;
        }
        for bank in data.banks.iter() {
            if bank.range.overlaps(range) {
                return Err(MapFatal::RamOverlap { a: bank.id, b: id });
            }
        }
        data.banks.push(Bank { range, id }, "RAM banks")?;
    }
    if data.banks.len == 0 {
        return Err(MapFatal::MissingRequiredRange {
            source: SourceId::Ram(0),
        });
    }
    // At most eight banks: bounded insertion sort avoids a general sorter's
    // recursive call graph in the boot-stack budget.
    for i in 1..data.banks.len {
        let mut j = i;
        while j > 0
            && data.banks.items[j].map(|b| b.range.first())
                < data.banks.items[j - 1].map(|b| b.range.first())
        {
            data.banks.items.swap(j, j - 1);
            j -= 1;
        }
    }
    let mut merged: Buffer<Bank, MAX_MEMORY_BANKS> = Buffer::new();
    for bank in data.banks.iter() {
        if let Some(Some(last)) = merged.items[..merged.len].last_mut()
            && last.range.end() == bank.range.first()
        {
            last.range = PhysFrameRange::between(last.range.first(), bank.range.end())
                .ok_or(MapFatal::RangeOverflow { source: bank.id })?;
            data.anomalies.merged_banks += 1;
            continue;
        }
        merged.push(*bank, "RAM union")?;
    }
    data.banks = merged;
    Ok(())
}
pub(super) fn partition(data: &mut MapData) -> Result<(), MapFatal> {
    data.entries.len = 0;
    data.anomalies.outside_ram_warnings = 0;
    for source in data.sources.iter() {
        let mut covered = PageCount::default();
        for b in data.banks.iter() {
            if let Some(kept) = b.range.intersect(source.rounded) {
                covered = covered
                    .checked_add(kept.count())
                    .ok_or(MapFatal::RangeOverflow { source: source.id })?;
            }
        }
        if covered != source.rounded.count() {
            data.anomalies.outside_ram_warnings += 1;
        }
    }
    for bank in data.banks.iter() {
        let mut first = bank.range.first();
        while first < bank.range.end() {
            let mut end = bank.range.end();
            let mut sources = SourceSet::default();
            let mut class = RegionClass::Allocatable;
            for (i, source) in data.sources.iter().enumerate() {
                for boundary in [source.rounded.first(), source.rounded.end()] {
                    if boundary > first {
                        end = end.min(boundary);
                    }
                }
                if source.rounded.contains(first) {
                    sources.insert(i);
                    class = if class == RegionClass::Allocatable {
                        source.class
                    } else if class == source.class {
                        class
                    } else {
                        RegionClass::SharedProtection
                    };
                }
            }
            let range = PhysFrameRange::between(first, end)
                .ok_or(MapFatal::RangeOverflow { source: bank.id })?;
            data.entries.push(
                MapEntry {
                    range,
                    class,
                    sources,
                },
                "map entries",
            )?;
            first = end;
        }
    }
    Ok(())
}
