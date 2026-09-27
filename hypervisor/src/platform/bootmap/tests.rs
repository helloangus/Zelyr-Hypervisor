//! Internal safety-boundary unit tests; no public fixture bypass.
use super::*;
use crate::{
    boot::address::{ByteSize, PhysAddr},
    platform::discovery::{Fact, Problem, Reason},
};
fn span(base: u64, len: u64) -> Span {
    Span {
        base: PhysAddr::new(base),
        len: ByteSize(len),
    }
}
fn empty() -> MapData {
    MapStorage::new()
}

#[test]
fn every_nonusable_fact_state_fails_closed_in_every_list() {
    for fact in [
        Fact::<Span>::Absent,
        Fact::NotDiscovered,
        Fact::Unsupported(Reason::Binding),
        Fact::Unusable(Problem::Range),
    ] {
        for list in [FactList::Ram, FactList::Reserved, FactList::Artifact] {
            assert_eq!(
                classify::usable(&fact, list, 7),
                Err(MapFatal::UnusableFact { list, ordinal: 7 })
            );
        }
    }
}
#[test]
fn zero_protection_is_counted_and_reused_identity_is_rejected() {
    let mut data = empty();
    let id = SourceId::DtbReservation(0);
    classify::add(
        &mut data,
        span(u64::MAX, 0),
        id,
        RegionClass::FirmwareReserved,
        ProtectionFlags::default(),
    )
    .unwrap();
    assert_eq!(data.anomalies.dropped_zero_protection, 1);
    assert_eq!(data.sources.len, 0);
    classify::add(
        &mut data,
        span(0, 1),
        id,
        RegionClass::FirmwareReserved,
        ProtectionFlags::default(),
    )
    .unwrap();
    assert_eq!(
        classify::add(
            &mut data,
            span(0, 1),
            id,
            RegionClass::FirmwareReserved,
            ProtectionFlags::default()
        ),
        Err(MapFatal::DuplicateSource { source: id })
    );
}
#[test]
fn bounded_storage_never_silently_truncates() {
    let mut buffer = Buffer::<u8, 2>::new();
    buffer.push(1, "fixture").unwrap();
    buffer.push(2, "fixture").unwrap();
    assert_eq!(
        buffer.push(3, "fixture"),
        Err(MapFatal::CapacityExhausted { which: "fixture" })
    );
    assert_eq!(buffer.iter().copied().collect::<std::vec::Vec<_>>(), [1, 2]);
}
#[test]
fn seal_audit_rejects_class_source_coverage_and_boundary_corruption() {
    let mut data = empty();
    let id = SourceId::Ram(0);
    data.banks
        .push(
            Bank {
                range: ranges::ram_to_frames(span(0, 4096 * 4), id).unwrap(),
                id,
            },
            "banks",
        )
        .unwrap();
    classify::add(
        &mut data,
        span(4096, 1),
        SourceId::ActiveDtb,
        RegionClass::ActiveDtb,
        ProtectionFlags::default(),
    )
    .unwrap();
    build::partition(&mut data).unwrap();
    assert!(seal::audit(&data).is_ok());
    let original = data.entries.items;
    for mutation in 0..4 {
        data.entries.items = original;
        let entry = data.entries.items[1].as_mut().unwrap();
        match mutation {
            0 => entry.class = RegionClass::Allocatable,
            1 => entry.sources = SourceSet::default(),
            2 => entry.range = data.banks.iter().next().unwrap().range,
            _ => data.entries.items[0] = None,
        }
        assert_eq!(
            seal::audit(&data).err(),
            Some(MapFatal::SealRejected {
                violation: SealViolation::Audit
            })
        );
    }
}
#[test]
fn all_84_source_slots_and_8_banks_preserve_capacity_and_provenance() {
    let mut data = empty();
    for bank in 0..MAX_MEMORY_BANKS {
        let id = SourceId::Ram(bank);
        data.banks
            .push(
                Bank {
                    range: ranges::ram_to_frames(span(bank as u64 * 0x100000, 0x80000), id)
                        .unwrap(),
                    id,
                },
                "banks",
            )
            .unwrap();
    }
    for i in 0..MAX_SOURCES {
        let id = SourceId::AllocatorMetadata(i);
        classify::add(
            &mut data,
            span(i as u64 * 4096, 1),
            id,
            RegionClass::HypervisorMetadata,
            ProtectionFlags::default(),
        )
        .unwrap();
    }
    assert_eq!(
        classify::add(
            &mut data,
            span(0x70000, 1),
            SourceId::ActiveDtb,
            RegionClass::ActiveDtb,
            ProtectionFlags::default()
        ),
        Err(MapFatal::CapacityExhausted {
            which: "source ledger"
        })
    );
    build::partition(&mut data).unwrap();
    assert!(seal::audit(&data).is_ok());
    assert_eq!(data.sources.len, MAX_SOURCES);
    assert_eq!(
        data.entries
            .iter()
            .filter(|e| e.class.is_protected())
            .count(),
        MAX_SOURCES
    );
    assert!(data.entries.len <= MAX_MAP_ENTRIES);
    assert!(
        data.entries
            .iter()
            .any(|e| e.sources.indices().any(|i| i == 83))
    );
}
#[test]
fn classification_default_is_protected_and_accounting_does_not_add_owners() {
    for class in [
        RegionClass::HypervisorImage,
        RegionClass::HypervisorMetadata,
        RegionClass::ActiveDtb,
        RegionClass::FirmwareReserved,
        RegionClass::BootArtifact,
        RegionClass::SharedProtection,
    ] {
        assert!(class.is_protected());
    }
    assert!(!RegionClass::Allocatable.is_protected());
}

#[test]
fn zero_ram_empty_domain_and_capacity_are_checked_before_publication() {
    let mut data = empty();
    assert_eq!(
        build::collect_ram(&mut data, [Fact::Usable(span(u64::MAX, 0))].iter()),
        Err(MapFatal::MissingRequiredRange {
            source: SourceId::Ram(0)
        })
    );
    assert_eq!(data.anomalies.dropped_zero_ram, 1);
    let mut data = empty();
    build::collect_ram(
        &mut data,
        [Fact::Usable(span(u64::MAX, 0)), Fact::Usable(span(0, 4096))].iter(),
    )
    .unwrap();
    assert_eq!(data.anomalies.dropped_zero_ram, 1);
    let mut data = empty();
    let facts: [_; 9] = core::array::from_fn(|i| Fact::Usable(span(i as u64 * 8192, 4096)));
    assert_eq!(
        build::collect_ram(&mut data, facts.iter()),
        Err(MapFatal::CapacityExhausted { which: "RAM banks" })
    );
    assert_eq!(
        build::collect_ram(
            &mut empty(),
            [Fact::Usable(span(u64::MAX - 4095, 4096))].iter()
        ),
        Err(MapFatal::RangeOverflow {
            source: SourceId::Ram(0)
        })
    );
}
