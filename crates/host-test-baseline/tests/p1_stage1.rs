//! Exact production address/table model; no host privileged execution.
#[path = "../../../hypervisor/src/boot/address.rs"]
pub mod address;
pub mod boot {
    pub use crate::address;
}
#[path = "../../../hypervisor/src/arch/aarch64/stage1/model.rs"]
pub mod model;
use address::{ByteSize, PhysAddr, VirtAddr};
use model::{MappingClass as C, Region, Tables};

fn regions() -> [Region; 7] {
    [
        Region {
            name: "boot",
            start: VirtAddr::new(0x40080000),
            end: VirtAddr::new(0x40081000),
            class: C::CodeRx,
        },
        Region {
            name: "vectors",
            start: VirtAddr::new(0x40081000),
            end: VirtAddr::new(0x40082000),
            class: C::Vectors,
        },
        Region {
            name: "text",
            start: VirtAddr::new(0x40082000),
            end: VirtAddr::new(0x40090000),
            class: C::CodeRx,
        },
        Region {
            name: "rodata",
            start: VirtAddr::new(0x40090000),
            end: VirtAddr::new(0x40092000),
            class: C::RoData,
        },
        Region {
            name: "data",
            start: VirtAddr::new(0x40092000),
            end: VirtAddr::new(0x400a0000),
            class: C::DataRw,
        },
        Region {
            name: "stack",
            start: VirtAddr::new(0x400a0000),
            end: VirtAddr::new(0x400b0000),
            class: C::BootStack,
        },
        Region {
            name: "console",
            start: VirtAddr::new(0x09000000),
            end: VirtAddr::new(0x09001000),
            class: C::ConsoleMmio,
        },
    ]
}
#[test]
fn complete_split_code_inventory_roundtrips() {
    let mut t = Tables::new();
    let r = regions();
    let base = PhysAddr::new(0x40094000);
    t.build(&r, base).unwrap();
    t.verify(&r, base).unwrap();
    let mut changed = r;
    changed[2].class = C::DataRw;
    assert!(t.verify(&changed, base).is_err());
    assert!(t.verify(&r, PhysAddr::new(0x40095000)).is_err());
}
#[test]
fn extra_missing_and_wrong_attribute_inventory_rejected() {
    let r = regions();
    let base = PhysAddr::new(0x40094000);
    let mut t = Tables::new();
    t.build(&r, base).unwrap();
    let mut changed = r;
    changed[2].end = VirtAddr::new(0x4008f000);
    assert!(t.verify(&changed, base).is_err());
    changed = r;
    changed[5].end = VirtAddr::new(0x400b1000);
    assert!(t.verify(&changed, base).is_err());
}
#[test]
fn rejects_overlap_alignment_window_and_table_address() {
    let base = PhysAddr::new(0x40094000);
    let mut t = Tables::new();
    let mut r = regions();
    r[2].start = r[1].start;
    assert!(t.build(&r, base).is_err());
    r = regions();
    r[2].end = VirtAddr::new(0x40090001);
    assert!(t.build(&r, base).is_err());
    r = regions();
    r[5].end = VirtAddr::new(0x40201000);
    assert!(t.build(&r, base).is_err());
    assert!(t.build(&regions(), PhysAddr::new(u64::MAX)).is_err());
    assert!(t.build(&regions(), PhysAddr::new(1)).is_err());
    assert!(t.build(&[], base).is_err());
}
#[test]
fn mappings_never_combine_write_and_execute() {
    for c in [
        C::CodeRx,
        C::Vectors,
        C::RoData,
        C::DataRw,
        C::BootStack,
        C::ConsoleMmio,
    ] {
        let a = c.attributes();
        assert!(a & (1 << 7) != 0 || a & (1 << 54) != 0);
        assert_eq!(a & 3, 3);
        assert_ne!(a & (1 << 10), 0);
    }
    assert_eq!((C::ConsoleMmio.attributes() >> 2) & 7, 2);
    assert_eq!(model::MAIR, 0x04ffee);
}
#[test]
fn typed_arithmetic_and_tcr_bounds() {
    assert!(PhysAddr::new(u64::MAX).checked_add(ByteSize(1)).is_none());
    assert!(VirtAddr::new(u64::MAX).checked_add(ByteSize(1)).is_none());
    assert!(VirtAddr::new(0).distance_from(VirtAddr::new(1)).is_none());
    for bits in [32, 36, 40, 42, 44, 48, 52] {
        let tcr = model::tcr_for_pa_bits(bits).unwrap();
        assert_eq!(tcr & 63, 25);
        assert_ne!(tcr & (1 << 31), 0);
        assert_ne!(tcr & (1 << 23), 0);
    }
    assert!(model::tcr_for_pa_bits(0).is_err());
}
