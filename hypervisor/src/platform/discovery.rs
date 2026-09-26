//! W02: deterministic facts from the W01 cursor; no hardware or allocation.
use super::intake::{Node, Span, ValidatedBootDtb};
use crate::boot::address::{ByteSize, PhysAddr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    Missing,
    Encoding,
    Cells,
    Range,
    Method,
    Interrupts,
    Duplicate,
    Disabled,
    Capacity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    CellWidth,
    GicV2,
    TimerV7,
    PsciV01,
    Binding,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fact<T> {
    NotDiscovered,
    Absent,
    Unsupported(Reason),
    Unusable(Problem),
    Usable(T),
}
impl<T> Fact<T> {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotDiscovered => "not-discovered",
            Self::Absent => "absent",
            Self::Unsupported(_) => "unsupported",
            Self::Unusable(_) => "unusable",
            Self::Usable(_) => "usable",
        }
    }
    pub fn usable(&self) -> Option<&T> {
        if let Self::Usable(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn summary(&self) -> Fact<()> {
        match self {
            Self::NotDiscovered => Fact::NotDiscovered,
            Self::Absent => Fact::Absent,
            Self::Unsupported(r) => Fact::Unsupported(*r),
            Self::Unusable(p) => Fact::Unusable(*p),
            Self::Usable(_) => Fact::Usable(()),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct List<T: Copy, const N: usize> {
    items: [Option<T>; N],
    used: usize,
}
impl<T: Copy, const N: usize> List<T, N> {
    const fn new() -> Self {
        Self {
            items: [None; N],
            used: 0,
        }
    }
    fn push(&mut self, value: T) -> Result<(), Error> {
        if self.used == N {
            return Err(Error::Capacity);
        }
        self.items[self.used] = Some(value);
        self.used += 1;
        Ok(())
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items[..self.used].iter().flatten()
    }
    pub fn len(&self) -> usize {
        self.used
    }
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Capacity,
    CpuInventoryEmpty,
    BootCpuUnmatched,
    NoMemoryBanks,
}
impl Error {
    pub fn class(self) -> &'static str {
        match self {
            Self::Capacity => "CapacityExhausted",
            Self::CpuInventoryEmpty => "CpuInventoryEmpty",
            Self::BootCpuUnmatched => "BootCpuUnmatched",
            Self::NoMemoryBanks => "NoMemoryBanks",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuStatus {
    Enabled,
    Disabled,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnableMethod {
    Psci,
    SpinTable,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cpu {
    pub affinity: u64,
    pub status: CpuStatus,
    pub method: EnableMethod,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationSource {
    Header,
    Node,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub span: Span,
    pub source: ReservationSource,
    pub no_map: bool,
    pub reusable: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Gic {
    pub distributor: Span,
    pub redistributors: List<Span, 2>,
    pub stride: ByteSize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timer {
    pub always_on: bool,
    pub interrupt_count: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PsciMethod {
    Smc,
    Hvc,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PsciVersion {
    V02,
    V10,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Psci {
    pub method: PsciMethod,
    pub version: PsciVersion,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Text {
    bytes: [u8; 256],
    len: usize,
}
impl Text {
    fn copy(value: &[u8]) -> Result<Self, Error> {
        if value.len() > 256 {
            return Err(Error::Capacity);
        }
        let mut result = Self {
            bytes: [0; 256],
            len: value.len(),
        };
        result.bytes[..value.len()].copy_from_slice(value);
        Ok(result)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Console {
    pub path: Text,
    pub resolved_node: Option<Text>,
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Counters {
    pub zero_ranges: usize,
    pub duplicate_singletons: usize,
    pub skipped_nodes: usize,
    pub disabled_singletons: usize,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub cpus: Fact<usize>,
    pub gic: Fact<()>,
    pub timer: Fact<()>,
    pub psci: Fact<()>,
    pub console: Fact<()>,
    pub pci: Fact<()>,
    pub smmu: Fact<()>,
    pub acpi: Fact<()>,
}
#[derive(PartialEq, Eq)]
pub struct PlatformInfo {
    cpus: List<Fact<Cpu>, 16>,
    boot_cpu: usize,
    banks: List<Fact<Span>, 8>,
    reserved: List<Fact<Reservation>, 32>,
    gic_status: Fact<Text>,
    timer_status: Fact<Text>,
    psci_status: Fact<Text>,
    gic: Fact<Gic>,
    timer: Fact<Timer>,
    psci: Fact<Psci>,
    console: Fact<Console>,
    bootargs: Fact<Text>,
    artifacts: List<Fact<Span>, 4>,
    capabilities: Capabilities,
    counters: Counters,
}
impl PlatformInfo {
    pub fn cpus(&self) -> &List<Fact<Cpu>, 16> {
        &self.cpus
    }
    pub fn boot_cpu(&self) -> usize {
        self.boot_cpu
    }
    pub fn banks(&self) -> &List<Fact<Span>, 8> {
        &self.banks
    }
    pub fn reserved(&self) -> &List<Fact<Reservation>, 32> {
        &self.reserved
    }
    pub fn gic_status(&self) -> &Fact<Text> {
        &self.gic_status
    }
    pub fn timer_status(&self) -> &Fact<Text> {
        &self.timer_status
    }
    pub fn psci_status(&self) -> &Fact<Text> {
        &self.psci_status
    }
    pub fn gic(&self) -> &Fact<Gic> {
        &self.gic
    }
    pub fn timer(&self) -> &Fact<Timer> {
        &self.timer
    }
    pub fn psci(&self) -> &Fact<Psci> {
        &self.psci
    }
    pub fn console(&self) -> &Fact<Console> {
        &self.console
    }
    pub fn bootargs(&self) -> &Fact<Text> {
        &self.bootargs
    }
    pub fn artifacts(&self) -> &List<Fact<Span>, 4> {
        &self.artifacts
    }
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
    pub fn counters(&self) -> Counters {
        self.counters
    }
}
fn string(value: &[u8]) -> Result<&[u8], Problem> {
    if value.last() != Some(&0) || value[..value.len().saturating_sub(1)].contains(&0) {
        return Err(Problem::Encoding);
    }
    Ok(&value[..value.len() - 1])
}
fn prop_string<'a>(node: Node<'a>, name: &[u8]) -> Result<&'a [u8], Problem> {
    string(node.property(name).ok_or(Problem::Missing)?)
}
fn number(value: &[u8]) -> Result<u64, Problem> {
    match value.len() {
        4 => Ok(u32::from_be_bytes(value.try_into().map_err(|_| Problem::Encoding)?) as u64),
        8 => Ok(u64::from_be_bytes(
            value.try_into().map_err(|_| Problem::Encoding)?,
        )),
        _ => Err(Problem::Encoding),
    }
}
fn word(node: Node<'_>, name: &[u8]) -> Result<u32, Problem> {
    let bytes = node.property(name).ok_or(Problem::Missing)?;
    Ok(u32::from_be_bytes(
        bytes.try_into().map_err(|_| Problem::Encoding)?,
    ))
}
pub fn compatible(value: &[u8], wanted: &[u8]) -> bool {
    value.last() == Some(&0)
        && value[..value.len() - 1]
            .split(|b| *b == 0)
            .any(|part| part == wanted)
}
fn matches(node: Node<'_>, binding: &[u8]) -> bool {
    node.property(b"compatible")
        .is_some_and(|v| compatible(v, binding))
}
fn unit(name: &[u8], base: &[u8]) -> bool {
    name == base
        || name
            .strip_prefix(base)
            .is_some_and(|v| v.first() == Some(&b'@'))
}
#[derive(Clone, Copy)]
struct Cells {
    address: usize,
    size: usize,
}
fn cells(node: Node<'_>, cpu: bool) -> Result<Cells, Problem> {
    let get = |name, default| match node.property(name) {
        Some(_) => word(node, name),
        None => Ok(default),
    };
    let address = get(b"#address-cells", if cpu { 1 } else { 2 })? as usize;
    let size = get(b"#size-cells", if cpu { 0 } else { 1 })? as usize;
    if !(1..=2).contains(&address) || size > 2 || (!cpu && size == 0) || (cpu && size != 0) {
        return Err(Problem::Cells);
    }
    Ok(Cells { address, size })
}
fn regs(
    node: Node<'_>,
    cpu: bool,
) -> Result<impl Iterator<Item = Result<Span, Problem>> + '_, Problem> {
    let parent = node.parent().ok_or(Problem::Cells)?;
    let cells = cells(parent, cpu)?;
    // Physical interpretation is authorized only for root children and the
    // root reserved-memory container. Other bus translations need a design.
    if !cpu
        && !parent.is_root()
        && !(parent.name == b"reserved-memory"
            && parent.parent().is_some_and(|p| p.is_root())
            && parent.property(b"ranges") == Some(&[][..]))
    {
        return Err(Problem::Range);
    }
    let value = node.property(b"reg").ok_or(Problem::Missing)?;
    let width = (cells.address + cells.size) * 4;
    if value.is_empty() || value.len() % width != 0 {
        return Err(Problem::Encoding);
    }
    Ok(value.chunks_exact(width).map(move |entry| {
        let base = number(&entry[..cells.address * 4])?;
        let len = if cells.size == 0 {
            0
        } else {
            number(&entry[cells.address * 4..])?
        };
        base.checked_add(len).ok_or(Problem::Range)?;
        Ok(Span {
            base: PhysAddr::new(base),
            len: ByteSize(len),
        })
    }))
}
fn record<T>(result: Result<T, Problem>) -> Fact<T> {
    match result {
        Ok(v) => Fact::Usable(v),
        Err(Problem::Cells) => Fact::Unsupported(Reason::CellWidth),
        Err(p) => Fact::Unusable(p),
    }
}
fn with_status<T>(
    node: Node<'_>,
    fact: Fact<T>,
    counters: &mut Counters,
) -> Result<(Fact<T>, Fact<Text>), Error> {
    let Some(raw) = node.property(b"status") else {
        return Ok((fact, Fact::Absent));
    };
    let value = match string(raw) {
        Ok(v) => v,
        Err(p) => return Ok((Fact::Unusable(p), Fact::Unusable(p))),
    };
    let status = Fact::Usable(Text::copy(value)?);
    let fact = match value {
        b"ok" | b"okay" => fact,
        b"disabled" | b"reserved" => {
            counters.disabled_singletons += 1;
            Fact::Absent
        }
        _ => Fact::Unusable(Problem::Disabled),
    };
    Ok((fact, status))
}

fn cpu(node: Node<'_>) -> Result<Cpu, Problem> {
    let mut entries = regs(node, true)?;
    let reg = entries.next().ok_or(Problem::Missing)??;
    if entries.next().is_some() || reg.base.raw() & !0xff00_ffffff != 0 {
        return Err(Problem::Encoding);
    }
    let status = match node.property(b"status") {
        None => CpuStatus::Enabled,
        Some(v) => match string(v)? {
            b"ok" | b"okay" => CpuStatus::Enabled,
            b"disabled" => CpuStatus::Disabled,
            _ => CpuStatus::Unknown,
        },
    };
    let method = match prop_string(node, b"enable-method") {
        Ok(b"psci") => EnableMethod::Psci,
        Ok(b"spin-table") => EnableMethod::SpinTable,
        _ => EnableMethod::Unknown,
    };
    Ok(Cpu {
        affinity: reg.base.raw(),
        status,
        method,
    })
}
fn gic(node: Node<'_>) -> Result<Gic, Problem> {
    if word(node, b"#interrupt-cells")? != 3 {
        return Err(Problem::Interrupts);
    }
    let count = match node.property(b"#redistributor-regions") {
        None => 1,
        Some(_) => word(node, b"#redistributor-regions")?,
    };
    if count == 0 || count > 2 {
        return Err(Problem::Capacity);
    }
    let stride = match node.property(b"redistributor-stride") {
        None => 128 * 1024,
        Some(v) if v.len() == 8 => number(v)?,
        Some(_) => return Err(Problem::Encoding),
    };
    if stride == 0 || stride % 65536 != 0 {
        return Err(Problem::Range);
    }
    let mut entries = regs(node, false)?;
    let distributor = entries.next().ok_or(Problem::Missing)??;
    if distributor.len.0 == 0 {
        return Err(Problem::Range);
    }
    let mut redistributors = List::new();
    for entry in entries {
        let span = entry?;
        if span.len.0 == 0 {
            return Err(Problem::Range);
        }
        redistributors.push(span).map_err(|_| Problem::Capacity)?;
    }
    if redistributors.len() != count as usize {
        return Err(Problem::Range);
    }
    Ok(Gic {
        distributor,
        redistributors,
        stride: ByteSize(stride),
    })
}
fn timer(node: Node<'_>, dtb: &ValidatedBootDtb<'_>) -> Result<Timer, Problem> {
    let mut cursor = Some(node);
    let mut parent = None;
    while let Some(n) = cursor {
        if n.property(b"interrupt-parent").is_some() {
            parent = Some(word(n, b"interrupt-parent")?);
            break;
        }
        cursor = n.parent();
    }
    let id = parent.ok_or(Problem::Interrupts)?;
    if id == 0 || id == u32::MAX {
        return Err(Problem::Interrupts);
    }
    let mut parents = dtb
        .nodes()
        .filter(|n| word(*n, b"phandle") == Ok(id) || word(*n, b"linux,phandle") == Ok(id));
    let controller = parents.next().ok_or(Problem::Interrupts)?;
    if parents.next().is_some() {
        return Err(Problem::Duplicate);
    }
    let width = word(controller, b"#interrupt-cells")? as usize;
    if width != 3 {
        return Err(Problem::Interrupts);
    }
    let interrupts = node.property(b"interrupts").ok_or(Problem::Missing)?;
    if interrupts.len() != 4 * width * 4 {
        return Err(Problem::Interrupts);
    }
    Ok(Timer {
        always_on: node.property(b"always-on").is_some(),
        interrupt_count: 4,
    })
}
fn psci(node: Node<'_>) -> Fact<Psci> {
    let version = if matches(node, b"arm,psci-1.0") {
        PsciVersion::V10
    } else if matches(node, b"arm,psci-0.2") {
        PsciVersion::V02
    } else {
        return Fact::Unsupported(if matches(node, b"arm,psci") {
            Reason::PsciV01
        } else {
            Reason::Binding
        });
    };
    record((|| {
        let method = match prop_string(node, b"method")? {
            b"smc" => PsciMethod::Smc,
            b"hvc" => PsciMethod::Hvc,
            _ => return Err(Problem::Method),
        };
        Ok(Psci { method, version })
    })())
}
fn chosen(dtb: &ValidatedBootDtb<'_>, info: &mut PlatformInfo) -> Result<(), Error> {
    let Some(node) = dtb.find(b"/chosen") else {
        return Ok(());
    };
    if let Some(value) = node.property(b"bootargs") {
        info.bootargs = match string(value) {
            Ok(v) => Fact::Usable(Text::copy(v)?),
            Err(p) => Fact::Unusable(p),
        };
    }
    if let Some(value) = node.property(b"stdout-path") {
        info.console = match string(value) {
            Err(p) => Fact::Unusable(p),
            Ok(value) => {
                let path = value.split(|b| *b == b':').next().unwrap_or_default();
                let target = if path.starts_with(b"/") {
                    Some(path)
                } else {
                    dtb.find(b"/aliases")
                        .and_then(|n| n.property(path))
                        .and_then(|v| string(v).ok())
                        .filter(|v| v.starts_with(b"/"))
                };
                Fact::Usable(Console {
                    path: Text::copy(path)?,
                    resolved_node: match target.filter(|p| dtb.find(p).is_some()) {
                        Some(p) => Some(Text::copy(p)?),
                        None => None,
                    },
                })
            }
        };
    }
    let start = node.property(b"linux,initrd-start");
    let end = node.property(b"linux,initrd-end");
    if start.is_some() || end.is_some() {
        info.artifacts.push(record((|| {
            let start = number(start.ok_or(Problem::Missing)?)?;
            let end = number(end.ok_or(Problem::Missing)?)?;
            let len = end
                .checked_sub(start)
                .filter(|v| *v != 0)
                .ok_or(Problem::Range)?;
            Ok(Span {
                base: PhysAddr::new(start),
                len: ByteSize(len),
            })
        })()))?;
    }
    Ok(())
}
pub fn normalize(dtb: &ValidatedBootDtb<'_>) -> Result<PlatformInfo, Error> {
    let mut info = PlatformInfo {
        cpus: List::new(),
        boot_cpu: 0,
        banks: List::new(),
        reserved: List::new(),
        gic_status: Fact::Absent,
        timer_status: Fact::Absent,
        psci_status: Fact::Absent,
        gic: Fact::Absent,
        timer: Fact::Absent,
        psci: Fact::Absent,
        console: Fact::Absent,
        bootargs: Fact::Absent,
        artifacts: List::new(),
        capabilities: Capabilities {
            cpus: Fact::Absent,
            gic: Fact::Absent,
            timer: Fact::Absent,
            psci: Fact::Absent,
            console: Fact::Absent,
            pci: Fact::NotDiscovered,
            smmu: Fact::NotDiscovered,
            acpi: Fact::NotDiscovered,
        },
        counters: Counters::default(),
    };
    if let Some(cpus) = dtb.find(b"/cpus") {
        for node in cpus.children() {
            if !unit(node.name, b"cpu") && prop_string(node, b"device_type") != Ok(&b"cpu"[..]) {
                continue;
            }
            let mut entry = record(cpu(node));
            if let Fact::Usable(value) = entry
                && info
                    .cpus
                    .iter()
                    .filter_map(Fact::usable)
                    .any(|c| c.affinity == value.affinity)
            {
                entry = Fact::Unusable(Problem::Duplicate);
            }
            info.cpus.push(entry)?;
        }
    }
    let enabled = info
        .cpus
        .iter()
        .filter_map(Fact::usable)
        .filter(|c| c.status == CpuStatus::Enabled)
        .count();
    if enabled == 0 {
        return Err(Error::CpuInventoryEmpty);
    }
    info.boot_cpu = info
        .cpus
        .iter()
        .position(|v| {
            v.usable().is_some_and(|c| {
                c.affinity == dtb.boot_cpu() as u64 && c.status == CpuStatus::Enabled
            })
        })
        .ok_or(Error::BootCpuUnmatched)?;
    for node in dtb
        .nodes()
        .filter(|n| unit(n.name, b"memory") && n.parent().is_some_and(|p| p.is_root()))
    {
        match regs(node, false) {
            Err(p) => info.banks.push(record(Err(p)))?,
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(span) if span.len.0 == 0 => info.counters.zero_ranges += 1,
                        other => info.banks.push(record(other))?,
                    }
                }
            }
        }
    }
    if info.banks.iter().all(|v| v.usable().is_none()) {
        return Err(Error::NoMemoryBanks);
    }
    for span in dtb.reservations() {
        if span.len.0 == 0 {
            info.counters.zero_ranges += 1;
            continue;
        }
        info.reserved.push(Fact::Usable(Reservation {
            span,
            source: ReservationSource::Header,
            no_map: false,
            reusable: false,
        }))?;
    }
    if let Some(parent) = dtb.find(b"/reserved-memory") {
        for node in parent.children() {
            match regs(node, false) {
                Err(p) => info.reserved.push(record(Err(p)))?,
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(span) if span.len.0 == 0 => info.counters.zero_ranges += 1,
                            other => info.reserved.push(record(other.map(|span| Reservation {
                                span,
                                source: ReservationSource::Node,
                                no_map: node.property(b"no-map").is_some(),
                                reusable: node.property(b"reusable").is_some(),
                            })))?,
                        }
                    }
                }
            }
        }
    }
    let mut gic_seen = false;
    let mut timer_seen = false;
    let mut psci_seen = false;
    for node in dtb.nodes() {
        if matches(node, b"arm,gic-v3") {
            if gic_seen {
                info.counters.duplicate_singletons += 1;
                continue;
            }
            gic_seen = true;
            (info.gic, info.gic_status) = with_status(node, record(gic(node)), &mut info.counters)?;
        } else if [
            b"arm,gic-400".as_slice(),
            b"arm,cortex-a15-gic",
            b"arm,cortex-a9-gic",
        ]
        .iter()
        .any(|v| matches(node, v))
            && !gic_seen
        {
            (info.gic, info.gic_status) =
                with_status(node, Fact::Unsupported(Reason::GicV2), &mut info.counters)?;
        } else if matches(node, b"arm,armv8-timer") {
            if timer_seen {
                info.counters.duplicate_singletons += 1;
                continue;
            }
            timer_seen = true;
            (info.timer, info.timer_status) =
                with_status(node, record(timer(node, dtb)), &mut info.counters)?;
        } else if matches(node, b"arm,armv7-timer") && !timer_seen {
            (info.timer, info.timer_status) =
                with_status(node, Fact::Unsupported(Reason::TimerV7), &mut info.counters)?;
        } else if unit(node.name, b"psci") {
            if psci_seen {
                info.counters.duplicate_singletons += 1;
                continue;
            }
            psci_seen = true;
            (info.psci, info.psci_status) = with_status(node, psci(node), &mut info.counters)?;
        }
    }
    if matches!(info.gic, Fact::Unusable(Problem::Capacity)) {
        return Err(Error::Capacity);
    }
    chosen(dtb, &mut info)?;
    let cpus_id = dtb.find(b"/cpus").map(Node::id);
    let reserved_id = dtb.find(b"/reserved-memory").map(Node::id);
    let console_id = info
        .console
        .usable()
        .and_then(|c| c.resolved_node.as_ref())
        .and_then(|p| dtb.find(p.bytes()))
        .map(Node::id);
    info.counters.skipped_nodes = dtb
        .nodes()
        .filter(|n| {
            let structural = n.is_root()
                || [
                    b"cpus".as_slice(),
                    b"chosen",
                    b"aliases",
                    b"reserved-memory",
                ]
                .contains(&n.name);
            let cpu = n.parent_id().is_some()
                && n.parent_id() == cpus_id
                && (unit(n.name, b"cpu") || prop_string(*n, b"device_type") == Ok(&b"cpu"[..]));
            let reserved = reserved_id.is_some() && n.parent_id() == reserved_id;
            let singleton = [
                b"arm,gic-v3".as_slice(),
                b"arm,gic-400",
                b"arm,cortex-a15-gic",
                b"arm,cortex-a9-gic",
                b"arm,armv8-timer",
                b"arm,armv7-timer",
            ]
            .iter()
            .any(|binding| matches(*n, binding));
            !(structural
                || cpu
                || reserved
                || singleton
                || unit(n.name, b"memory")
                || unit(n.name, b"psci")
                || Some(n.id()) == console_id)
        })
        .count();
    info.capabilities.cpus = Fact::Usable(enabled);
    info.capabilities.gic = info.gic.summary();
    info.capabilities.timer = info.timer.summary();
    info.capabilities.psci = info.psci.summary();
    info.capabilities.console = info.console.summary();
    Ok(info)
}
