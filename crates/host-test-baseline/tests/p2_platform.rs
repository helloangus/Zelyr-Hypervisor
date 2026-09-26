//! Exact production P2 code; fixture allocations belong only to the host harness.
#[path = "../../../hypervisor/src/boot/address.rs"]
pub mod address;
pub mod boot {
    pub use crate::address;
}
#[path = "../../../hypervisor/src/arch/aarch64/stage1/dtb_model.rs"]
pub mod dtb_model;
#[path = "../../../hypervisor/src/platform/mod.rs"]
pub mod platform;
use address::{ByteSize, PhysAddr};
use platform::{
    discovery::{self, Error, Fact, Problem, Reason},
    intake::{self, Error as IntakeError, Span},
};

#[derive(Clone)]
struct Tree {
    name: String,
    props: Vec<(String, Vec<u8>)>,
    children: Vec<Tree>,
}
impl Tree {
    fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            props: Vec::new(),
            children: Vec::new(),
        }
    }
    fn prop(mut self, name: &str, value: impl Into<Vec<u8>>) -> Self {
        self.props.push((name.into(), value.into()));
        self
    }
    fn child(mut self, node: Tree) -> Self {
        self.children.push(node);
        self
    }
    fn set(&mut self, name: &str, value: Vec<u8>) {
        if let Some(p) = self.props.iter_mut().find(|p| p.0 == name) {
            p.1 = value;
        } else {
            self.props.push((name.into(), value));
        }
    }
}
fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}
fn pairs(values: &[u64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}
fn pad(v: &mut Vec<u8>) {
    while !v.len().is_multiple_of(4) {
        v.push(0);
    }
}
fn emit(node: &Tree, structure: &mut Vec<u8>, strings: &mut Vec<u8>) {
    structure.extend(words(&[1]));
    structure.extend(node.name.as_bytes());
    structure.push(0);
    pad(structure);
    for (name, value) in &node.props {
        let off = strings.len();
        strings.extend(name.as_bytes());
        strings.push(0);
        structure.extend(words(&[3, value.len() as u32, off as u32]));
        structure.extend(value);
        pad(structure);
    }
    for child in &node.children {
        emit(child, structure, strings);
    }
    structure.extend(words(&[2]));
}
fn blob(tree: &Tree, reservations: &[(u64, u64)]) -> Vec<u8> {
    let mut structure = Vec::new();
    let mut strings = Vec::new();
    emit(tree, &mut structure, &mut strings);
    structure.extend(words(&[9]));
    let mut rsv = Vec::new();
    for (a, b) in reservations {
        rsv.extend(pairs(&[*a, *b]));
    }
    rsv.extend([0; 16]);
    let off_struct = 40 + rsv.len();
    let off_strings = off_struct + structure.len();
    let total = off_strings + strings.len();
    let mut out = words(&[
        0xd00dfeed,
        total as u32,
        off_struct as u32,
        off_strings as u32,
        40,
        17,
        16,
        0,
        strings.len() as u32,
        structure.len() as u32,
    ]);
    out.extend(rsv);
    out.extend(structure);
    out.extend(strings);
    out
}
fn root() -> Tree {
    Tree::new("")
        .prop("#address-cells", words(&[2]))
        .prop("#size-cells", words(&[2]))
        .prop("interrupt-parent", words(&[1]))
        .child(
            Tree::new("cpus")
                .prop("#address-cells", words(&[2]))
                .prop("#size-cells", words(&[0]))
                .child(
                    Tree::new("cpu@0")
                        .prop("device_type", b"cpu\0".to_vec())
                        .prop("reg", pairs(&[0]))
                        .prop("enable-method", b"psci\0".to_vec()),
                )
                .child(Tree::new("cpu-map")),
        )
        .child(Tree::new("memory@40000000").prop("reg", pairs(&[0x40000000, 0x8000000])))
        .child(
            Tree::new("intc@8000000")
                .prop("compatible", b"arm,gic-v3\0".to_vec())
                .prop("reg", pairs(&[0x8000000, 0x10000, 0x80a0000, 0xf60000]))
                .prop("#interrupt-cells", words(&[3]))
                .prop("phandle", words(&[1])),
        )
        .child(
            Tree::new("timer")
                .prop("compatible", b"arm,armv8-timer\0".to_vec())
                .prop(
                    "interrupts",
                    words(&[1, 13, 4, 1, 14, 4, 1, 11, 4, 1, 10, 4]),
                ),
        )
        .child(
            Tree::new("psci")
                .prop("compatible", b"arm,psci-1.0\0arm,psci-0.2\0".to_vec())
                .prop("method", b"hvc\0".to_vec()),
        )
        .child(Tree::new("uart@9000000"))
        .child(Tree::new("aliases").prop("serial0", b"/uart@9000000\0".to_vec()))
        .child(
            Tree::new("chosen")
                .prop("stdout-path", b"serial0:115200n8\0".to_vec())
                .prop("bootargs", b"example\0".to_vec())
                .prop("linux,initrd-start", pairs(&[0x42000000]))
                .prop("linux,initrd-end", pairs(&[0x42001000])),
        )
}
fn span(base: u64, len: u64) -> Span {
    Span {
        base: PhysAddr::new(base),
        len: ByteSize(len),
    }
}
fn validate(bytes: &[u8]) -> Result<intake::ValidatedBootDtb<'_>, IntakeError> {
    intake::validate(
        bytes,
        PhysAddr::new(0x44000000),
        span(0x40000000, 0x8000000),
        span(0x40080000, 0x100000),
    )
}
fn set_word(bytes: &mut [u8], off: usize, value: u32) {
    bytes[off..off + 4].copy_from_slice(&value.to_be_bytes());
}
fn child<'a>(tree: &'a mut Tree, name: &str) -> &'a mut Tree {
    tree.children.iter_mut().find(|n| n.name == name).unwrap()
}
#[test]
fn valid_cursor_reservations_and_facts_are_deterministic() {
    let bytes = blob(&root(), &[(0, 4096), (0x47000000, 4096)]);
    let dtb = validate(&bytes).unwrap();
    assert_eq!(dtb.reservations().count(), 2);
    assert!(dtb.anomalies() >= 1);
    assert_eq!(
        dtb.find(b"/cpus/cpu@0").unwrap().property(b"reg"),
        Some(pairs(&[0]).as_slice())
    );
    let a = discovery::normalize(&dtb).unwrap();
    let b = discovery::normalize(&dtb).unwrap();
    assert!(a == b);
    assert_eq!(a.cpus().len(), 1);
    assert_eq!(a.boot_cpu(), 0);
    assert_eq!(a.banks().len(), 1);
    assert_eq!(a.reserved().len(), 2);
    assert!(a.gic().usable().is_some());
    assert_eq!(a.timer().usable().unwrap().interrupt_count, 4);
    assert_eq!(
        a.psci().usable().unwrap().method,
        discovery::PsciMethod::Hvc
    );
    assert!(a.console().usable().unwrap().resolved_node.is_some());
    assert_eq!(a.artifacts().len(), 1);
    assert!(matches!(a.capabilities().pci, Fact::NotDiscovered));
    assert!(matches!(a.capabilities().smmu, Fact::NotDiscovered));
}
#[test]
fn placement_checks_precede_header_reads() {
    let invalid = [0u8; 40];
    let coverage = span(0x40000000, 0x8000000);
    let image = span(0x40080000, 0x10000);
    for (address, error) in [
        (0, IntakeError::Absent),
        (0x44000001, IntakeError::Alignment),
        (0x48000000, IntakeError::Unreachable),
        (0x40080000, IntakeError::ImageOverlap),
    ] {
        assert_eq!(
            intake::validate(&invalid, PhysAddr::new(address), coverage, image).unwrap_err(),
            error
        );
    }
    assert_eq!(
        intake::placement(span(u64::MAX - 7, 40), span(0, u64::MAX), image),
        Err(IntakeError::Unreachable)
    );
}
#[test]
fn header_fields_and_block_overlap_rejected() {
    let original = blob(&root(), &[]);
    for (offset, value, class) in [
        (0, 0, "DtbHeaderInvalid"),
        (4, 40, "DtbSizeInvalid"),
        (8, u32::MAX, "DtbHeaderInvalid"),
        (20, 16, "DtbHeaderInvalid"),
        (24, 18, "DtbHeaderInvalid"),
        (36, u32::MAX, "DtbHeaderInvalid"),
    ] {
        let mut b = original.clone();
        set_word(&mut b, offset, value);
        assert_eq!(validate(&b).unwrap_err().class(), class);
    }
    let mut b = original.clone();
    let structure = u32::from_be_bytes(b[8..12].try_into().unwrap());
    set_word(&mut b, 12, structure);
    assert!(validate(&b).is_err());
    for end in 0..original.len() {
        assert!(validate(&original[..end]).is_err());
    }
}
#[test]
fn structural_root_balance_names_and_limits() {
    for tree in [
        Tree::new("bad-root"),
        Tree::new("").child(Tree::new("")),
        Tree::new("").child(Tree::new(&"x".repeat(257))),
    ] {
        assert!(matches!(
            validate(&blob(&tree, &[])),
            Err(IntakeError::Structure(_))
        ));
    }
    let mut deep = Tree::new("leaf");
    for _ in 0..32 {
        deep = Tree::new("n").child(deep);
    }
    assert!(validate(&blob(&Tree::new("").child(deep), &[])).is_err());
    let mut many = Tree::new("");
    for _ in 0..4096 {
        many.children.push(Tree::new("n"));
    }
    assert!(validate(&blob(&many, &[])).is_err());
    let mut properties = Tree::new("");
    for _ in 0..16385 {
        properties.props.push(("p".into(), vec![]));
    }
    assert!(validate(&blob(&properties, &[])).is_err());
    let mut b = blob(&Tree::new(""), &[]);
    set_word(&mut b, 56, 9);
    assert!(validate(&b).is_err());
}
#[test]
fn reservation_overflow_overlap_and_both_capacity_limits() {
    assert!(matches!(
        validate(&blob(&root(), &[(u64::MAX, 2)])),
        Err(IntakeError::Reservation(_))
    ));
    let b = blob(&root(), &vec![(0x47000000, 4096); 1025]);
    assert!(matches!(validate(&b), Err(IntakeError::Reservation(_))));
    let b = blob(&root(), &vec![(0x47000000, 4096); 33]);
    let dtb = validate(&b).unwrap();
    assert!(matches!(discovery::normalize(&dtb), Err(Error::Capacity)));
    let mut b = blob(&root(), &[]);
    b[40..56].fill(1);
    assert!(matches!(validate(&b), Err(IntakeError::Reservation(_))));
}
#[test]
fn mutation_sweep_never_panics_or_escapes_cursor_bounds() {
    let original = blob(&root(), &[(0x46000000, 8192)]);
    for i in 0..original.len() {
        for bit in [1, 0x80] {
            let mut b = original.clone();
            b[i] ^= bit;
            if let Ok(dtb) = validate(&b) {
                for node in dtb.nodes() {
                    let _ = node.property(b"reg");
                    let _ = node.parent();
                }
                let _ = discovery::normalize(&dtb);
            }
        }
    }
}
#[test]
fn cpu_status_bad_records_boot_match_and_capacity() {
    let mut single = root();
    let cpus = child(&mut single, "cpus");
    cpus.set("#address-cells", words(&[1]));
    cpus.children[0].set("reg", words(&[0]));
    cpus.children.push(
        Tree::new("cpu@2")
            .prop("reg", words(&[2]))
            .prop("status", b"unknown\0".to_vec()),
    );
    let bytes = blob(&single, &[]);
    let info = discovery::normalize(&validate(&bytes).unwrap()).unwrap();
    assert_eq!(info.cpus().len(), 2);
    assert_eq!(
        info.cpus().iter().last().unwrap().usable().unwrap().status,
        discovery::CpuStatus::Unknown
    );
    let mut tree = root();
    let cpus = child(&mut tree, "cpus");
    cpus.children.push(
        Tree::new("cpu@1")
            .prop("reg", pairs(&[1]))
            .prop("status", b"disabled\0".to_vec()),
    );
    cpus.children.push(Tree::new("cpu@2"));
    let b = blob(&tree, &[]);
    let dtb = validate(&b).unwrap();
    let info = discovery::normalize(&dtb).unwrap();
    assert_eq!(info.cpus().len(), 3);
    assert!(matches!(info.capabilities().cpus, Fact::Usable(1)));
    assert!(matches!(
        info.cpus().iter().last(),
        Some(Fact::Unusable(Problem::Missing))
    ));
    let mut b = b.clone();
    set_word(&mut b, 28, 1);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::BootCpuUnmatched)
    ));
    let mut tree = root();
    for i in 1..17 {
        child(&mut tree, "cpus")
            .children
            .push(Tree::new(&format!("cpu@{i}")).prop("reg", pairs(&[i])));
    }
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::Capacity)
    ));
    let mut tree = root();
    child(&mut tree, "cpus").children.clear();
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::CpuInventoryEmpty)
    ));
}
#[test]
fn memory_and_reservation_errors_remain_visible() {
    let mut tree = root();
    child(&mut tree, "memory@40000000").set("reg", pairs(&[0x40000000, 0, 0x41000000, 0x100000]));
    tree.children.push(
        Tree::new("reserved-memory")
            .prop("#address-cells", words(&[2]))
            .prop("#size-cells", words(&[2]))
            .prop("ranges", vec![])
            .child(
                Tree::new("valid")
                    .prop("reg", pairs(&[0x42000000, 8192]))
                    .prop("no-map", vec![])
                    .prop("reusable", vec![]),
            )
            .child(Tree::new("bad")),
    );
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert_eq!(info.counters().zero_ranges, 1);
    assert_eq!(info.reserved().len(), 2);
    assert!(
        info.reserved()
            .iter()
            .next()
            .unwrap()
            .usable()
            .unwrap()
            .no_map
    );
    assert!(matches!(
        info.reserved().iter().last(),
        Some(Fact::Unusable(Problem::Missing))
    ));
    child(&mut tree, "memory@40000000").set("reg", pairs(&[u64::MAX, 2]));
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::NoMemoryBanks)
    ));
}
#[test]
fn singleton_states_are_distinct_and_capabilities_project_them() {
    for (binding, expected) in [
        (
            b"arm,gic-400\0".as_slice(),
            Fact::Unsupported(Reason::GicV2),
        ),
        (b"unrecognized\0", Fact::Absent),
    ] {
        let mut tree = root();
        child(&mut tree, "intc@8000000").set("compatible", binding.to_vec());
        let b = blob(&tree, &[]);
        let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
        assert!(info.capabilities().gic == expected);
    }
    let mut tree = root();
    child(&mut tree, "intc@8000000").set("redistributor-stride", pairs(&[1 << 32]));
    child(&mut tree, "psci").set("method", b"bad\0".to_vec());
    child(&mut tree, "timer").set("interrupts", vec![]);
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert_eq!(info.gic().usable().unwrap().stride.0, 1 << 32);
    assert!(matches!(info.psci(), Fact::Unusable(Problem::Method)));
    assert!(matches!(info.timer(), Fact::Unusable(Problem::Interrupts)));
    child(&mut tree, "intc@8000000").set("status", b"disabled\0".to_vec());
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert!(matches!(info.gic(), Fact::Absent));
    assert_eq!(info.counters().disabled_singletons, 1);
    assert_eq!(info.gic_status().usable().unwrap().bytes(), b"disabled");
    child(&mut tree, "intc@8000000").set("status", b"fail\0".to_vec());
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert!(matches!(info.gic(), Fact::Unusable(Problem::Disabled)));
}
#[test]
fn chosen_alias_invalid_artifact_and_string_capacity() {
    let mut tree = root();
    child(&mut tree, "chosen").set("stdout-path", b"missing\0".to_vec());
    child(&mut tree, "chosen").set("linux,initrd-end", pairs(&[0x42000000]));
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert!(!info.console().usable().unwrap().resolved_node.is_some());
    assert!(matches!(
        info.artifacts().iter().next(),
        Some(Fact::Unusable(Problem::Range))
    ));
    let mut long = vec![b'x'; 257];
    long.push(0);
    child(&mut tree, "chosen").set("bootargs", long);
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::Capacity)
    ));
}
#[test]
fn default_cells_are_two_address_one_size_and_widths_checked() {
    let mut tree = root();
    tree.props.retain(|p| !p.0.ends_with("-cells"));
    child(&mut tree, "memory@40000000").set("reg", words(&[0, 0x40000000, 0x8000000]));
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert_eq!(
        info.banks().iter().next().unwrap().usable().unwrap().len.0,
        0x8000000
    );
    tree.set("#address-cells", words(&[3]));
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::NoMemoryBanks)
    ));
    assert!(!discovery::compatible(b"arm,gic-v3", b"arm,gic-v3"));
    assert!(!discovery::compatible(b"arm,gic-v30\0", b"arm,gic-v3"));
}
#[test]
fn aperture_rounding_capacity_and_permissions() {
    use dtb_model::Plan;
    let coverage = span(0x40000000, 0x8000000);
    let image = span(0x40080000, 0x100000);
    let p = Plan::new(span(0x44000ff8, 8 * 1024 * 1024), coverage, image).unwrap();
    assert_eq!(p.pages, 2049);
    assert_eq!(p.virtual_start().raw(), 0x80000ff8);
    let descriptor = p.descriptor(0).unwrap();
    assert_ne!(descriptor & (1 << 54), 0);
    assert_ne!(descriptor & (1 << 7), 0);
    assert_eq!((descriptor >> 2) & 7, 1);
    assert!(p.descriptor(p.pages).is_none());
    assert_eq!(
        Plan::new(
            span(0x40080000 - 32, 40),
            coverage,
            span(0x40080008, 0x100000)
        ),
        Err(IntakeError::ImageOverlap)
    );
    assert_eq!(
        Plan::new(span(0x47fffff8, 40), coverage, image),
        Err(IntakeError::Unreachable)
    );
}

#[test]
fn node_name_anomaly_and_cpu_size_cells_are_not_silent() {
    let mut b = blob(&root().child(Tree::new("foreign")), &[]);
    let pos = b.windows(8).position(|v| v == b"foreign\0").unwrap();
    b[pos] = 0xff;
    let dtb = validate(&b).unwrap();
    assert!(dtb.anomalies() > 0);
    assert!(discovery::normalize(&dtb).unwrap().counters().skipped_nodes > 0);
    let mut tree = root();
    child(&mut tree, "cpus").set("#size-cells", words(&[1]));
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::CpuInventoryEmpty)
    ));
}
#[test]
fn singleton_versions_missing_properties_and_duplicate_policy() {
    for (binding, method, expected) in [
        (
            b"arm,psci-0.2\0".as_slice(),
            Some(b"smc\0".as_slice()),
            "usable",
        ),
        (b"arm,psci\0", None, "unsupported"),
        (b"arm,psci-1.0\0", None, "unusable"),
    ] {
        let mut tree = root();
        let psci = child(&mut tree, "psci");
        psci.set("compatible", binding.to_vec());
        psci.props.retain(|p| p.0 != "method");
        if let Some(method) = method {
            psci.set("method", method.to_vec());
        }
        let b = blob(&tree, &[]);
        assert_eq!(
            discovery::normalize(&validate(&b).unwrap())
                .unwrap()
                .psci()
                .label(),
            expected
        );
    }
    let mut tree = root();
    let duplicate = child(&mut tree, "timer").clone();
    tree.children.push(duplicate);
    let b = blob(&tree, &[]);
    assert_eq!(
        discovery::normalize(&validate(&b).unwrap())
            .unwrap()
            .counters()
            .duplicate_singletons,
        1
    );
    tree.children
        .retain(|n| !["intc@8000000", "timer", "psci", "chosen"].contains(&n.name.as_str()));
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    for state in [
        info.gic().summary(),
        info.timer().summary(),
        info.psci().summary(),
        info.console().summary(),
    ] {
        assert!(matches!(state, Fact::Absent));
    }
}
#[test]
fn gic_and_timer_invalid_details_preserve_states() {
    for (property, value) in [
        ("redistributor-stride", pairs(&[0])),
        ("#interrupt-cells", words(&[4])),
        ("reg", vec![]),
    ] {
        let mut tree = root();
        child(&mut tree, "intc@8000000").set(property, value);
        let b = blob(&tree, &[]);
        assert!(matches!(
            discovery::normalize(&validate(&b).unwrap()).unwrap().gic(),
            Fact::Unusable(_)
        ));
    }
    let mut tree = root();
    child(&mut tree, "intc@8000000").set("#redistributor-regions", words(&[3]));
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap()),
        Err(Error::Capacity)
    ));
    let mut tree = root();
    tree.set("interrupt-parent", words(&[99]));
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap())
            .unwrap()
            .timer(),
        Fact::Unusable(Problem::Interrupts)
    ));
    child(&mut tree, "timer").set("compatible", b"arm,armv7-timer\0".to_vec());
    let b = blob(&tree, &[]);
    assert!(matches!(
        discovery::normalize(&validate(&b).unwrap())
            .unwrap()
            .timer(),
        Fact::Unsupported(Reason::TimerV7)
    ));
}
#[test]
fn chosen_direct_alias_chain_bad_strings_and_inverted_initrd() {
    let mut tree = root();
    child(&mut tree, "chosen").set("stdout-path", b"/uart@9000000:115200\0".to_vec());
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert_eq!(
        info.console()
            .usable()
            .unwrap()
            .resolved_node
            .as_ref()
            .unwrap()
            .bytes(),
        b"/uart@9000000"
    );
    child(&mut tree, "chosen").set("stdout-path", b"serial0\0".to_vec());
    child(&mut tree, "aliases").set("serial0", b"serial1\0".to_vec());
    child(&mut tree, "chosen").set("bootargs", b"no-nul".to_vec());
    child(&mut tree, "chosen").set("linux,initrd-end", pairs(&[0x41000000]));
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert!(info.console().usable().unwrap().resolved_node.is_none());
    assert!(matches!(info.bootargs(), Fact::Unusable(Problem::Encoding)));
    assert!(matches!(
        info.artifacts().iter().next(),
        Some(Fact::Unusable(Problem::Range))
    ));
}
#[test]
fn one_cell_multibank_and_bad_reservation_translation() {
    let mut tree = root();
    tree.set("#address-cells", words(&[1]));
    tree.set("#size-cells", words(&[1]));
    child(&mut tree, "memory@40000000").set(
        "reg",
        words(&[0x40000000, 0x1000000, 0x42000000, 0x1000000]),
    );
    child(&mut tree, "intc@8000000").set("reg", words(&[0x8000000, 0x10000, 0x80a0000, 0xf60000]));
    tree.children.push(
        Tree::new("reserved-memory")
            .prop("#address-cells", words(&[1]))
            .prop("#size-cells", words(&[1]))
            .prop("ranges", vec![1])
            .child(Tree::new("r").prop("reg", words(&[0x47000000, 4096]))),
    );
    let b = blob(&tree, &[]);
    let info = discovery::normalize(&validate(&b).unwrap()).unwrap();
    assert_eq!(info.banks().len(), 2);
    assert!(matches!(
        info.reserved().iter().next(),
        Some(Fact::Unusable(Problem::Range))
    ));
}

#[test]
fn cursor_properties_and_subtree_skip_preserve_sibling_boundaries() {
    let bytes = blob(&root(), &[]);
    let dtb = validate(&bytes).unwrap();
    let mut nodes = dtb.nodes();
    let root = nodes.next().unwrap();
    assert_eq!(root.properties().count(), 3);
    let mut properties = root.properties();
    for _ in 0..3 {
        assert!(properties.next().is_some());
    }
    assert!(properties.next().is_none());
    assert!(properties.next().is_none());
    assert_eq!(nodes.next().unwrap().name, b"cpus");
    nodes.skip_subtree();
    assert_eq!(nodes.next().unwrap().name, b"memory@40000000");
    nodes.skip_subtree();
    assert_eq!(nodes.next().unwrap().name, b"intc@8000000");
    let mut nodes = dtb.nodes();
    nodes.next();
    nodes.skip_subtree();
    assert!(nodes.next().is_none());
}
