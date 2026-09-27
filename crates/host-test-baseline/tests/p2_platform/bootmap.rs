//! W03 tests reuse the real W01 -> W02 pipeline, with independent frame oracles.
use super::*;
use platform::bootmap::{ranges::*, *};
fn frame(address: u64) -> PhysFrameNum {
    PhysFrameNum::from_address(PhysAddr::new(address)).unwrap()
}
fn fr(address: u64, pages: u64) -> PhysFrameRange {
    PhysFrameRange::new(frame(address), PageCount::new(pages)).unwrap()
}
fn draft(
    tree: &Tree,
    reservations: &[(u64, u64)],
    image: Span,
) -> Result<UnsealedMemoryMap, MapFatal> {
    let bytes = blob(tree, reservations);
    let dtb = validate(&bytes).unwrap();
    let facts = discovery::normalize(&dtb).unwrap();
    BootMapBuilder::draft(&facts, dtb.range(), image)
}
fn image() -> Span {
    span(0x40080000, 0x100000)
}
fn reservations(nodes: Vec<Tree>) -> Tree {
    let mut parent = Tree::new("reserved-memory")
        .prop("#address-cells", words(&[2]))
        .prop("#size-cells", words(&[2]))
        .prop("ranges", vec![]);
    parent.children = nodes;
    parent
}
fn error(result: Result<UnsealedMemoryMap, MapFatal>) -> MapFatal {
    result.err().unwrap()
}
#[test]
fn checked_frame_conversion_covers_alignment_zero_and_both_overflows() {
    let id = SourceId::Ram(2);
    for s in [span(1, 4096), span(4096, 4095)] {
        assert_eq!(
            ram_to_frames(s, id),
            Err(MapFatal::UnalignedRange { source: id })
        );
    }
    for s in [span(u64::MAX, 2), span(u64::MAX - 4, 4)] {
        assert_eq!(
            protect_to_frames(s, id),
            Err(MapFatal::RangeOverflow { source: id })
        );
    }
    assert_eq!(
        ram_to_frames(span(0, 0), id),
        Err(MapFatal::MissingRequiredRange { source: id })
    );
    assert_eq!(protect_to_frames(span(4095, 2), id).unwrap(), fr(0, 2));
    assert_eq!(
        protect_to_frames(span(u64::MAX - 8191, 4096), id)
            .unwrap()
            .count(),
        PageCount::new(1)
    );
    assert!(PhysFrameRange::new(frame(0), PageCount::new(0)).is_none());
    assert!(PhysFrameRange::new(frame(4096), PageCount::new(u64::MAX)).is_none());
    assert!(PhysFrameNum::from_address(PhysAddr::new(1)).is_none());
    assert!(fr(0, 2).contains_range(fr(4096, 1)));
    assert!(fr(0, 1).adjacent(fr(4096, 1)));
    assert!(!fr(0, 1).overlaps(fr(4096, 1)));
    assert_eq!(fr(0, 2).intersect(fr(4096, 2)), Some(fr(4096, 1)));
}
#[test]
fn all_protection_sources_and_exact_dtb_extent_survive_real_pipeline() {
    let tree = root().child(reservations(vec![
        Tree::new("keep")
            .prop("reg", pairs(&[0x43000003, 9]))
            .prop("no-map", vec![])
            .prop("reusable", vec![]),
    ]));
    let bytes = blob(&tree, &[(0x45000001, 7)]);
    let dtb = validate(&bytes).unwrap();
    let facts = discovery::normalize(&dtb).unwrap();
    let map = BootMapBuilder::draft(&facts, dtb.range(), image())
        .unwrap()
        .seal(&[])
        .unwrap();
    for (address, class) in [
        (0x40080000, RegionClass::HypervisorImage),
        (0x42000000, RegionClass::BootArtifact),
        (0x43000000, RegionClass::FirmwareReserved),
        (0x44000000, RegionClass::ActiveDtb),
        (0x45000000, RegionClass::FirmwareReserved),
    ] {
        assert_eq!(map.class_at(frame(address)), ClassQuery::Protected(class));
    }
    let sources: Vec<_> = map.source_ledger().collect();
    assert_eq!(sources.len(), 5);
    assert_eq!(
        sources
            .iter()
            .find(|s| s.id == SourceId::ActiveDtb)
            .unwrap()
            .original,
        dtb.range()
    );
    let node = sources
        .iter()
        .find(|s| matches!(s.id, SourceId::ReservedMemoryNode(_)))
        .unwrap();
    assert!(node.flags.no_map && node.flags.reusable);
    assert_eq!(map.clips().count(), 5);
}
#[test]
fn image_and_dtb_are_required_and_protection_outside_ram_is_retained() {
    assert_eq!(
        error(draft(&root(), &[], span(0, 0))),
        MapFatal::MissingRequiredRange {
            source: SourceId::HypervisorImage
        }
    );
    let bytes = blob(&root(), &[]);
    let dtb = validate(&bytes).unwrap();
    let facts = discovery::normalize(&dtb).unwrap();
    assert_eq!(
        error(BootMapBuilder::draft(&facts, span(0, 0), image())),
        MapFatal::MissingRequiredRange {
            source: SourceId::ActiveDtb
        }
    );
    for (s, warnings) in [
        (span(0x3ffff000, 8192), 1),
        (span(0, 1), 1),
        (span(0x40000000, 1), 0),
        (span(0x47ffffff, 1), 0),
    ] {
        let map = draft(&root(), &[], s).unwrap().seal(&[]).unwrap();
        assert_eq!(map.anomalies().outside_ram_warnings, warnings);
        assert_eq!(map.source_ledger().next().unwrap().original, s);
        assert_eq!(map.class_at(frame(0)), ClassQuery::OutsideRam);
    }
}
#[test]
fn exact_duplicates_precede_adjacency_and_keep_both_provenances() {
    let mut tree = root().child(reservations(vec![
        Tree::new("same").prop("reg", pairs(&[0x45000000, 8])),
    ]));
    child(&mut tree, "memory@40000000").set(
        "reg",
        pairs(&[
            0x40000000, 0x4000000, 0x44000000, 0x4000000, 0x40000000, 0x4000000,
        ]),
    );
    let map = draft(&tree, &[(0x45000000, 8), (0x45000000, 8)], image())
        .unwrap()
        .seal(&[])
        .unwrap();
    assert_eq!(map.anomalies().deduped_banks, 1);
    assert_eq!(map.anomalies().merged_banks, 1);
    assert_eq!(map.anomalies().deduped_protections, 2);
    assert_eq!(map.summary().ram_frames.get(), 32768);
    let entry = map
        .entries()
        .find(|e| e.range.contains(frame(0x45000000)))
        .unwrap();
    assert_eq!(entry.class, RegionClass::FirmwareReserved);
    assert_eq!(entry.sources.indices().count(), 3);
    assert_eq!(
        map.summary()
            .protected_frames(RegionClass::FirmwareReserved)
            .get(),
        1
    );
}
#[test]
fn ownership_overlap_and_different_flags_are_fatal_with_source_ids() {
    assert_eq!(
        error(draft(&root(), &[(0x40080000, 1)], image())),
        MapFatal::ProtectionConflict {
            a: SourceId::HypervisorImage,
            b: SourceId::DtbReservation(0)
        }
    );
    let tree = root().child(reservations(vec![
        Tree::new("same")
            .prop("reg", pairs(&[0x45000000, 8]))
            .prop("no-map", vec![]),
    ]));
    assert!(matches!(
        error(draft(&tree, &[(0x45000000, 8)], image())),
        MapFatal::ProtectionConflict {
            a: SourceId::DtbReservation(0),
            b: SourceId::ReservedMemoryNode(1)
        }
    ));
    assert!(matches!(
        error(draft(
            &root(),
            &[(0x45000000, 16), (0x45000008, 16)],
            image()
        )),
        MapFatal::ProtectionConflict { .. }
    ));
}
#[test]
fn page_only_sharing_is_union_without_ownership_precedence() {
    let mut tree = root();
    child(&mut tree, "chosen").set("linux,initrd-start", pairs(&[0x45000008]));
    child(&mut tree, "chosen").set("linux,initrd-end", pairs(&[0x45000010]));
    let map = draft(&tree, &[(0x45000000, 8)], image())
        .unwrap()
        .seal(&[])
        .unwrap();
    assert_eq!(
        map.class_at(frame(0x45000000)),
        ClassQuery::Protected(RegionClass::SharedProtection)
    );
    assert_eq!(
        map.summary()
            .protected_frames(RegionClass::SharedProtection)
            .get(),
        1
    );
    assert_eq!(
        map.summary()
            .protected_frames(RegionClass::FirmwareReserved)
            .get(),
        0
    );
    assert_eq!(
        map.summary()
            .protected_frames(RegionClass::BootArtifact)
            .get(),
        0
    );
    assert_eq!(
        map.entries()
            .find(|e| e.range.contains(frame(0x45000000)))
            .unwrap()
            .sources
            .indices()
            .count(),
        2
    );
}
#[test]
fn ram_partial_overlap_and_alignment_reject_instead_of_shrinking() {
    let mut tree = root();
    child(&mut tree, "memory@40000000")
        .set("reg", pairs(&[0x40000000, 0x8000000, 0x41000000, 0x1000]));
    assert_eq!(
        error(draft(&tree, &[], image())),
        MapFatal::RamOverlap {
            a: SourceId::Ram(0),
            b: SourceId::Ram(1)
        }
    );
    for values in [[0x40000001, 0x1000], [0x40000000, 0x1001]] {
        child(&mut tree, "memory@40000000").set("reg", pairs(&values));
        assert_eq!(
            error(draft(&tree, &[], image())),
            MapFatal::UnalignedRange {
                source: SourceId::Ram(0)
            }
        );
    }
}
#[test]
fn present_unusable_facts_are_not_filtered_out() {
    let mut tree = root();
    child(&mut tree, "chosen").set("linux,initrd-end", pairs(&[0x42000000]));
    assert_eq!(
        error(draft(&tree, &[], image())),
        MapFatal::UnusableFact {
            list: FactList::Artifact,
            ordinal: 0
        }
    );
    let tree = root().child(reservations(vec![Tree::new("missing-reg")]));
    assert_eq!(
        error(draft(&tree, &[], image())),
        MapFatal::UnusableFact {
            list: FactList::Reserved,
            ordinal: 0
        }
    );
    let tree = root().child(Tree::new("memory@bad").prop("reg", vec![0]));
    assert_eq!(
        error(draft(&tree, &[], image())),
        MapFatal::UnusableFact {
            list: FactList::Ram,
            ordinal: 1
        }
    );
}
#[test]
fn upstream_zero_omissions_are_not_misattributed_to_w03() {
    let mut tree = root();
    child(&mut tree, "memory@40000000").set("reg", pairs(&[0, 0, 0x40000000, 0x8000000]));
    let bytes = blob(&tree, &[(0x45000000, 0)]);
    let dtb = validate(&bytes).unwrap();
    let facts = discovery::normalize(&dtb).unwrap();
    assert!(facts.counters().zero_ranges > 0);
    let map = BootMapBuilder::draft(&facts, dtb.range(), image()).unwrap();
    assert_eq!(map.anomalies().dropped_zero_ram, 0);
    assert_eq!(map.anomalies().dropped_zero_protection, 0);
}
#[test]
fn metadata_is_protected_and_invalid_plans_never_produce_authority() {
    let map = draft(&root(), &[], image())
        .unwrap()
        .seal(&[fr(0x40180000, 2), fr(0x46000000, 1)])
        .unwrap();
    assert_eq!(map.metadata_ledger().count(), 2);
    assert_eq!(
        map.class_at(frame(0x40180000)),
        ClassQuery::Protected(RegionClass::HypervisorMetadata)
    );
    assert_eq!(
        map.summary()
            .protected_frames(RegionClass::HypervisorMetadata)
            .get(),
        3
    );
    for addr in [0, 0x40080000, 0x44000000, 0x48000000] {
        assert_eq!(
            draft(&root(), &[], image())
                .unwrap()
                .seal(&[fr(addr, 1)])
                .err(),
            Some(MapFatal::SealRejected {
                violation: SealViolation::OutsideAllocatable(0)
            })
        );
    }
    assert_eq!(
        draft(&root(), &[], image())
            .unwrap()
            .seal(&[fr(0x45000000, 2), fr(0x45001000, 1)])
            .err(),
        Some(MapFatal::SealRejected {
            violation: SealViolation::Overlap(0, 1)
        })
    );
    assert_eq!(
        draft(&root(), &[], image())
            .unwrap()
            .seal(&[fr(0x45000000, 1); MAX_METADATA_RANGES + 1])
            .err(),
        Some(MapFatal::SealRejected {
            violation: SealViolation::Capacity
        })
    );
    // A plan spanning a protected hole fails even with allocatable endpoints.
    assert!(
        draft(&root(), &[], image())
            .unwrap()
            .seal(&[fr(0x41fff000, 3)])
            .is_err()
    );
}
#[test]
fn sparse_banks_clip_without_filling_holes_and_report_hole_totals() {
    let mut tree = root();
    child(&mut tree, "memory@40000000")
        .set("reg", pairs(&[0x100000000, 0x2000, 0x200000000, 0x3000]));
    let map = draft(&tree, &[(0x100001000, 0x100000000)], image())
        .unwrap()
        .seal(&[])
        .unwrap();
    assert_eq!(map.summary().ram_frames.get(), 5);
    assert_eq!(map.summary().hole_frames.get(), 0x100000000 / 4096 - 2);
    assert_eq!(
        map.summary()
            .protected_frames(RegionClass::FirmwareReserved)
            .get(),
        2
    );
    assert_eq!(
        map.clips()
            .filter(|c| c.protected == SourceId::DtbReservation(0))
            .count(),
        2
    );
    assert_eq!(map.class_at(frame(0x180000000)), ClassQuery::OutsideRam);
}
#[test]
fn independent_seeded_frame_oracle_checks_partition_ledger_clips_and_seal() {
    let mut seed = 0x5eed_u64;
    for iteration in 0..160 {
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            seed
        };
        let mut tree = root();
        tree.children.retain(|n| n.name != "chosen");
        let banks = [
            (0x50000000, 1 + next() % 48),
            (0x50040000 + (next() % 8) * 4096, 1 + next() % 64),
        ];
        child(&mut tree, "memory@40000000").set(
            "reg",
            pairs(&[banks[1].0, banks[1].1 * 4096, banks[0].0, banks[0].1 * 4096]),
        );
        let mut rsv = Vec::new();
        // Disjoint byte slots may round to the same page or land in RAM holes.
        for slot in 0..24u64 {
            let base = 0x50000000 + slot * 16384 + next() % 8192;
            let len = 1 + next() % 7000;
            rsv.push((base, len));
        }
        let map = draft(&tree, &rsv, span(0x60000000, 1)).unwrap();
        let available: Vec<_> = map.allocatable_spans().collect();
        let metadata: Vec<_> = available
            .iter()
            .take(5)
            .map(|r| PhysFrameRange::new(r.first(), PageCount::new(1)).unwrap())
            .collect();
        let map = map.seal(&metadata).unwrap();
        let mut free = 0;
        let mut protected = 0;
        for page in 0..140 {
            let addr = 0x50000000 + page * 4096;
            let inside = banks
                .iter()
                .any(|(base, count)| *base <= addr && addr < base + count * 4096);
            let firmware = rsv
                .iter()
                .any(|(base, len)| *base < addr + 4096 && addr < base + len);
            let meta = metadata.iter().any(|r| r.contains(frame(addr)));
            let expected = if !inside {
                ClassQuery::OutsideRam
            } else if firmware {
                ClassQuery::Protected(RegionClass::FirmwareReserved)
            } else if meta {
                ClassQuery::Protected(RegionClass::HypervisorMetadata)
            } else {
                ClassQuery::Allocatable
            };
            assert_eq!(
                map.class_at(frame(addr)),
                expected,
                "case {iteration} page {page}"
            );
            assert_eq!(
                map.allocatable_spans().any(|r| r.contains(frame(addr))),
                expected == ClassQuery::Allocatable
            );
            if inside {
                if firmware || meta {
                    protected += 1;
                } else {
                    free += 1;
                }
            }
        }
        assert_eq!(map.summary().ram_frames.get(), free + protected);
        assert_eq!(map.summary().allocatable_frames.get(), free);
        let entries: Vec<_> = map.entries().collect();
        for pair in entries.windows(2) {
            assert!(pair[0].range.end() <= pair[1].range.first());
        }
        for e in entries {
            let sources: Vec<_> = map
                .source_ledger()
                .enumerate()
                .filter(|(_, s)| s.rounded.overlaps(e.range))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(e.sources.indices().collect::<Vec<_>>(), sources);
        }
        for clip in map.clips() {
            let source = map
                .source_ledger()
                .find(|s| s.id == clip.protected)
                .unwrap();
            assert!(source.rounded.contains_range(clip.kept));
            assert_eq!(
                map.ram_spans()
                    .nth(clip.bank)
                    .unwrap()
                    .intersect(source.rounded),
                Some(clip.kept)
            );
            assert!(map.protected_ranges().any(|e| e.range.overlaps(clip.kept)));
        }
        let again = draft(&tree, &rsv, span(0x60000000, 1))
            .unwrap()
            .seal(&metadata)
            .unwrap();
        assert_eq!(
            map.entries().collect::<Vec<_>>(),
            again.entries().collect::<Vec<_>>()
        );
    }
}
#[test]
fn full_input_and_metadata_capacities_remain_bounded() {
    let mut tree = root();
    let banks: Vec<_> = (0..8)
        .flat_map(|i| [0x50000000 + i * 0x100000, 0x80000])
        .collect();
    child(&mut tree, "memory@40000000").set("reg", pairs(&banks));
    let rsv: Vec<_> = (0..32).map(|i| (0x50001000 + i * 0x2000, 1)).collect();
    let draft = draft(&tree, &rsv, image()).unwrap();
    assert!(draft.allocatable_spans().count() > 8);
    let metadata: Vec<_> = (0..MAX_METADATA_RANGES)
        .map(|i| fr(0x50100000 + i as u64 * 8192, 1))
        .collect();
    let map = draft.seal(&metadata).unwrap();
    assert_eq!(map.metadata_ledger().count(), MAX_METADATA_RANGES);
    assert!(map.entries().count() <= MAX_MAP_ENTRIES);
    assert!(map.source_ledger().count() <= MAX_SOURCES);
    eprintln!(
        "W03 storage bytes: draft={} sealed={} entry={} source={} clip={}",
        std::mem::size_of::<UnsealedMemoryMap>(),
        std::mem::size_of::<BootMemoryMap>(),
        std::mem::size_of::<MapEntry>(),
        std::mem::size_of::<ProtectionSource>(),
        std::mem::size_of::<ClipRecord>()
    );
}

#[test]
fn borrowed_storage_matches_owned_map_without_moving_backing() {
    let bytes = blob(&root(), &[(0x45000000, 17)]);
    let dtb = validate(&bytes).unwrap();
    let facts = discovery::normalize(&dtb).unwrap();
    let mut storage = const { MapStorage::new() };
    let address = core::ptr::from_ref(&storage);
    let metadata = [fr(0x46000000, 2)];
    {
        let map = BootMapBuilder::draft_in(&mut storage, &facts, dtb.range(), image())
            .unwrap()
            .seal(&metadata)
            .unwrap();
        let owned = BootMapBuilder::draft(&facts, dtb.range(), image())
            .unwrap()
            .seal(&metadata)
            .unwrap();
        assert_eq!(
            map.entries().collect::<Vec<_>>(),
            owned.entries().collect::<Vec<_>>()
        );
        assert_eq!(
            map.source_ledger().collect::<Vec<_>>(),
            owned.source_ledger().collect::<Vec<_>>()
        );
        assert_eq!(
            map.clips().collect::<Vec<_>>(),
            owned.clips().collect::<Vec<_>>()
        );
        assert_eq!(map.summary(), owned.summary());
        assert!(map.clips().count() <= MAX_CLIPS);
    }
    assert_eq!(core::ptr::from_ref(&storage), address);
    assert!(matches!(
        BootMapBuilder::draft_in(&mut storage, &facts, dtb.range(), image()),
        Err(MapFatal::StorageUnavailable)
    ));
    assert_eq!(
        core::mem::size_of::<UnsealedMemoryMap<&mut MapStorage>>(),
        8
    );
    assert_eq!(core::mem::size_of::<BootMemoryMap<&mut MapStorage>>(), 96);
}
#[test]
fn failed_or_dropped_draft_storage_cannot_publish_a_second_map() {
    let bytes = blob(&root(), &[]);
    let dtb = validate(&bytes).unwrap();
    let facts = discovery::normalize(&dtb).unwrap();
    for case in 0..3 {
        let mut storage = const { MapStorage::new() };
        match case {
            0 => assert!(
                BootMapBuilder::draft_in(&mut storage, &facts, dtb.range(), span(0, 0)).is_err()
            ),
            1 => {
                let draft =
                    BootMapBuilder::draft_in(&mut storage, &facts, dtb.range(), image()).unwrap();
                assert!(draft.seal(&[fr(0, 1)]).is_err());
            }
            _ => {
                let _draft =
                    BootMapBuilder::draft_in(&mut storage, &facts, dtb.range(), image()).unwrap();
            }
        }
        assert!(matches!(
            BootMapBuilder::draft_in(&mut storage, &facts, dtb.range(), image()),
            Err(MapFatal::StorageUnavailable)
        ));
    }
}
