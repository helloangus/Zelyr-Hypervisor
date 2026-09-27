//! Original-byte ownership validation precedes page-granular union.
use super::*;
use crate::platform::discovery::{Fact, ReservationSource};
pub(super) fn usable<T>(fact: &Fact<T>, list: FactList, ordinal: usize) -> Result<&T, MapFatal> {
    match fact {
        Fact::Usable(value) => Ok(value),
        _ => Err(MapFatal::UnusableFact { list, ordinal }),
    }
}
pub(super) fn add(
    data: &mut MapData,
    original: Span,
    id: SourceId,
    class: RegionClass,
    flags: ProtectionFlags,
) -> Result<(), MapFatal> {
    if original.len.0 == 0 {
        data.anomalies.dropped_zero_protection += 1;
        return Ok(());
    }
    original
        .end()
        .ok_or(MapFatal::RangeOverflow { source: id })?;
    let mut duplicate = false;
    for old in data.sources.iter() {
        if old.id == id {
            return Err(MapFatal::DuplicateSource { source: id });
        }
        if old.original == original && old.class == class && old.flags == flags {
            duplicate = true;
        } else if old.original.overlaps(original) {
            return Err(MapFatal::ProtectionConflict { a: old.id, b: id });
        }
    }
    let rounded = ranges::protect_to_frames(original, id)?;
    data.sources.push(
        ProtectionSource {
            id,
            original,
            rounded,
            class,
            flags,
        },
        "source ledger",
    )?;
    data.anomalies.deduped_protections += usize::from(duplicate);
    Ok(())
}
pub(super) fn assemble(
    data: &mut MapData,
    platform: &PlatformInfo,
    dtb: Span,
    image: Span,
) -> Result<(), MapFatal> {
    for (span, id, class) in [
        (
            image,
            SourceId::HypervisorImage,
            RegionClass::HypervisorImage,
        ),
        (dtb, SourceId::ActiveDtb, RegionClass::ActiveDtb),
    ] {
        if span.len.0 == 0 {
            return Err(MapFatal::MissingRequiredRange { source: id });
        }
        add(data, span, id, class, ProtectionFlags::default())?;
    }
    for (i, fact) in platform.reserved().iter().enumerate() {
        let r = usable(fact, FactList::Reserved, i)?;
        let id = match r.source {
            ReservationSource::Header => SourceId::DtbReservation(i),
            ReservationSource::Node => SourceId::ReservedMemoryNode(i),
        };
        add(
            data,
            r.span,
            id,
            RegionClass::FirmwareReserved,
            ProtectionFlags {
                no_map: r.no_map,
                reusable: r.reusable,
            },
        )?;
    }
    for (i, fact) in platform.artifacts().iter().enumerate() {
        add(
            data,
            *usable(fact, FactList::Artifact, i)?,
            SourceId::BootArtifact(i),
            RegionClass::BootArtifact,
            ProtectionFlags::default(),
        )?;
    }
    if data.sources.len > MAX_INPUT_SOURCES {
        return Err(MapFatal::CapacityExhausted {
            which: "input sources",
        });
    }
    Ok(())
}
