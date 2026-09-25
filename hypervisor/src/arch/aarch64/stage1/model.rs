//! Pure three-level, five-page P1 translation tables; no hardware accesses.
use crate::boot::address::{ByteSize, PhysAddr, VirtAddr};

pub const PAGE_BYTES: u64 = 4096;
const WINDOW_BYTES: u64 = 2 * 1024 * 1024;
const ADDRESS_MASK: u64 = 0x0000_ffff_ffff_f000;
const VALID_PAGE: u64 = 3;
const AF: u64 = 1 << 10;
// Non-VHE EL2 uses one VA range: AP[1] is RES1 in every page descriptor.
const EL2_AP_ONE_VA_RANGE: u64 = 1 << 6;
const READ_ONLY: u64 = 1 << 7;
const XN: u64 = 1 << 54;
pub const MAIR: u64 = 0x04_ff_ee;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingClass {
    CodeRx,
    RoData,
    DataRw,
    BootStack,
    Vectors,
    ConsoleMmio,
}
impl MappingClass {
    pub const fn attributes(self) -> u64 {
        let (index, ro, xn) = match self {
            Self::CodeRx | Self::Vectors => (0, true, false),
            Self::RoData => (0, true, true),
            Self::DataRw | Self::BootStack => (1, false, true),
            Self::ConsoleMmio => (2, false, true),
        };
        VALID_PAGE
            | AF
            | EL2_AP_ONE_VA_RANGE
            | (index << 2)
            | if ro { READ_ONLY } else { 0 }
            | if xn { XN } else { 0 }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub name: &'static str,
    pub start: VirtAddr,
    pub end: VirtAddr,
    pub class: MappingClass,
}
impl Region {
    fn contains(self, address: VirtAddr) -> bool {
        address >= self.start && address < self.end
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage1Step {
    Premise,
    Build,
    Verify,
    Program,
    PostVerify,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stage1Error {
    pub step: Stage1Step,
    pub detail: &'static str,
}
impl Stage1Error {
    pub const fn new(step: Stage1Step, detail: &'static str) -> Self {
        Self { step, detail }
    }
    /// Closed, allocation-free handoff to W09's static FailureReason.
    /// Every production error has a step-qualified token; unfamiliar input
    /// (possible in host model tests) remains an explicit invariant label.
    pub fn static_message(self) -> &'static str {
        match self.step {
            Stage1Step::Premise => match self.detail {
                "C8 not established" => "premise:C8-not-established",
                "SCTLR undeclared" => "premise:SCTLR-undeclared",
                "SCTLR M/C/I already on" => "premise:SCTLR-MCI-on",
                "HCR undeclared" => "premise:HCR-undeclared",
                "HCR VM already on" => "premise:HCR-VM-on",
                "vectors not established" => "premise:vectors-unestablished",
                "vector base mismatch" => "premise:vector-base-mismatch",
                "fatal or console not ready" => "premise:diagnostics-unready",
                "4K granule absent" => "premise:granule-4k-absent",
                "PA range absent" => "premise:pa-range-absent",
                "Stage1 repeated" => "premise:Stage1-repeated",
                "PA width" => "premise:PA-width-unsupported",
                _ => "premise:unrecognized",
            },
            Stage1Step::Build => match self.detail {
                "table end overflow" => "build:table-end-overflow",
                "tables outside data" => "build:tables-outside-data",
                "page overflow" => "build:page-overflow",
                "table overflow" => "build:table-overflow",
                "table address" => "build:table-address",
                "no image" => "build:no-image",
                "no console" => "build:no-console",
                "L1 collision" => "build:L1-collision",
                "window overflow" => "build:window-overflow",
                "window capacity" => "build:window-capacity",
                "overlap" => "build:overlap",
                "table offset" => "build:table-offset",
                "table page index" => "build:table-page-index",
                "boot-code" => "build:boot-code-bounds",
                "vectors" => "build:vector-bounds",
                "text" => "build:text-bounds",
                "rodata" => "build:rodata-bounds",
                "data" => "build:data-bounds",
                "stack" => "build:stack-bounds",
                "console" => "build:console-bounds",
                _ => "build:unrecognized",
            },
            Stage1Step::Verify => match self.detail {
                "descriptor/inventory mismatch" => "verify:descriptor-inventory-mismatch",
                "table page index" => "verify:table-page-index",
                "table copy mismatch" => "verify:table-copy-mismatch",
                _ => "verify:unrecognized",
            },
            Stage1Step::Program => "program:translation-register-readback",
            Stage1Step::PostVerify => match self.detail {
                "SCTLR M/C/I readback" => "postverify:SCTLR-MCI-readback",
                "rodata sentinel" => "postverify:rodata-sentinel",
                "data sentinel" => "postverify:data-sentinel",
                _ => "postverify:unrecognized",
            },
        }
    }
}
fn invalid(detail: &'static str) -> Stage1Error {
    Stage1Error::new(Stage1Step::Build, detail)
}

/// Typed fixed-table index. Bit extraction is internal, not address arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TableIndex(usize);
impl TableIndex {
    fn for_level(address: VirtAddr, shift: u32) -> Self {
        Self(((address.raw() >> shift) & 511) as usize)
    }
}

#[repr(C, align(4096))]
pub struct Table([u64; 512]);
impl Table {
    const fn zeroed() -> Self {
        Self([0; 512])
    }
}
#[repr(C, align(4096))]
pub struct Tables {
    l1: Table,
    image_l2: Table,
    console_l2: Table,
    image_l3: Table,
    console_l3: Table,
}
const _: () = assert!(core::mem::size_of::<Table>() == 4096);
const _: () = assert!(core::mem::size_of::<Tables>() == 5 * 4096);
impl Tables {
    pub const fn new() -> Self {
        Self {
            l1: Table::zeroed(),
            image_l2: Table::zeroed(),
            console_l2: Table::zeroed(),
            image_l3: Table::zeroed(),
            console_l3: Table::zeroed(),
        }
    }
    /// The five physical pages in the descriptor order used by table links.
    /// W08 copies these words into its aligned static atomic backing only
    /// after the model has passed complete verification.
    pub fn page_words(&self, page: usize) -> Option<&[u64; 512]> {
        match page {
            0 => Some(&self.l1.0),
            1 => Some(&self.image_l2.0),
            2 => Some(&self.console_l2.0),
            3 => Some(&self.image_l3.0),
            4 => Some(&self.console_l3.0),
            _ => None,
        }
    }
    fn clear(&mut self) {
        self.l1.0.fill(0);
        self.image_l2.0.fill(0);
        self.console_l2.0.fill(0);
        self.image_l3.0.fill(0);
        self.console_l3.0.fill(0);
    }
    /// Validate completely before filling: every build error leaves zero tables.
    pub fn build(&mut self, regions: &[Region], base: PhysAddr) -> Result<(), Stage1Error> {
        self.clear();
        let layout = Layout::validate(regions, base)?;
        self.l1.0[TableIndex::for_level(layout.image, 30).0] = table_descriptor(base, 1)?;
        self.l1.0[TableIndex::for_level(layout.console, 30).0] = table_descriptor(base, 2)?;
        self.image_l2.0[TableIndex::for_level(layout.image, 21).0] = table_descriptor(base, 3)?;
        self.console_l2.0[TableIndex::for_level(layout.console, 21).0] = table_descriptor(base, 4)?;
        for region in regions {
            let mut page = region.start;
            while page < region.end {
                let entries = if region.class == MappingClass::ConsoleMmio {
                    &mut self.console_l3.0
                } else {
                    &mut self.image_l3.0
                };
                entries[TableIndex::for_level(page, 12).0] =
                    page.p1_identity_physical().raw() | region.class.attributes();
                page = page
                    .checked_add(ByteSize(PAGE_BYTES))
                    .ok_or_else(|| invalid("page overflow"))?;
            }
        }
        Ok(())
    }
    /// Every descriptor is inspected, including invalid holes and both directions
    /// of inventory closure. Arbitrary descriptor pointers are never dereferenced.
    pub fn verify(&self, regions: &[Region], base: PhysAddr) -> Result<(), Stage1Error> {
        let layout = Layout::validate(regions, base)
            .map_err(|error| Stage1Error::new(Stage1Step::Verify, error.detail))?;
        let mismatch = || Stage1Error::new(Stage1Step::Verify, "descriptor/inventory mismatch");
        for i in 0..512 {
            let expected_l1 = if i == TableIndex::for_level(layout.image, 30).0 {
                table_descriptor(base, 1)?
            } else if i == TableIndex::for_level(layout.console, 30).0 {
                table_descriptor(base, 2)?
            } else {
                0
            };
            let expected_image_l2 = if i == TableIndex::for_level(layout.image, 21).0 {
                table_descriptor(base, 3)?
            } else {
                0
            };
            let expected_console_l2 = if i == TableIndex::for_level(layout.console, 21).0 {
                table_descriptor(base, 4)?
            } else {
                0
            };
            if self.l1.0[i] != expected_l1
                || self.image_l2.0[i] != expected_image_l2
                || self.console_l2.0[i] != expected_console_l2
            {
                return Err(mismatch());
            }
            for (window, entries) in [
                (layout.image, &self.image_l3.0),
                (layout.console, &self.console_l3.0),
            ] {
                let page = window
                    .checked_add(ByteSize((i as u64) * PAGE_BYTES))
                    .ok_or_else(mismatch)?;
                let expected = regions.iter().find(|r| r.contains(page));
                match expected {
                    None if entries[i] == 0 => {}
                    Some(region) => {
                        let actual = entries[i];
                        if actual & ADDRESS_MASK != page.p1_identity_physical().raw()
                            || actual & !ADDRESS_MASK != region.class.attributes()
                        {
                            return Err(mismatch());
                        }
                    }
                    _ => return Err(mismatch()),
                }
            }
        }
        Ok(())
    }
}
impl Default for Tables {
    fn default() -> Self {
        Self::new()
    }
}

struct Layout {
    image: VirtAddr,
    console: VirtAddr,
}
impl Layout {
    fn validate(regions: &[Region], base: PhysAddr) -> Result<Self, Stage1Error> {
        let table_end = base
            .checked_add(ByteSize(5 * PAGE_BYTES))
            .ok_or_else(|| invalid("table overflow"))?;
        if base.raw() & 4095 != 0 || table_end.raw() > (1_u64 << 48) {
            return Err(invalid("table address"));
        }
        let image = regions
            .iter()
            .find(|r| r.class != MappingClass::ConsoleMmio)
            .ok_or_else(|| invalid("no image"))?;
        let console = regions
            .iter()
            .find(|r| r.class == MappingClass::ConsoleMmio)
            .ok_or_else(|| invalid("no console"))?;
        let result = Self {
            image: VirtAddr::new(image.start.raw() & !(WINDOW_BYTES - 1)),
            console: VirtAddr::new(console.start.raw() & !(WINDOW_BYTES - 1)),
        };
        if TableIndex::for_level(result.image, 30) == TableIndex::for_level(result.console, 30) {
            return Err(invalid("L1 collision"));
        }
        for (i, region) in regions.iter().enumerate() {
            if !region.start.is_page_aligned()
                || !region.end.is_page_aligned()
                || region.start >= region.end
                || region.end.raw() > (1_u64 << 39)
            {
                return Err(invalid(region.name));
            }
            let window = if region.class == MappingClass::ConsoleMmio {
                result.console
            } else {
                result.image
            };
            let limit = window
                .checked_add(ByteSize(WINDOW_BYTES))
                .ok_or_else(|| invalid("window overflow"))?;
            if region.start < window || region.end > limit {
                return Err(invalid("window capacity"));
            }
            for other in &regions[..i] {
                if region.start < other.end && other.start < region.end {
                    return Err(invalid("overlap"));
                }
            }
        }
        Ok(result)
    }
}
fn table_descriptor(base: PhysAddr, page: u64) -> Result<u64, Stage1Error> {
    let offset = page
        .checked_mul(PAGE_BYTES)
        .ok_or_else(|| invalid("table offset"))?;
    Ok(base
        .checked_add(ByteSize(offset))
        .ok_or_else(|| invalid("table address"))?
        .raw()
        | 3)
}

/// Non-VHE TCR_EL2: 39-bit VA, 4 KiB, WB walks, non-shareable, RES1.
pub fn tcr_for_pa_bits(bits: u64) -> Result<u64, Stage1Error> {
    let ps = match bits {
        32 => 0,
        36 => 1,
        40 => 2,
        42 => 3,
        44 => 4,
        48 | 52 => 5,
        _ => return Err(Stage1Error::new(Stage1Step::Premise, "PA width")),
    };
    // P1 descriptors are 48-bit; a 52-bit implementation safely uses PS=48.
    Ok((1 << 31) | (1 << 23) | (ps << 16) | (1 << 10) | (1 << 8) | 25)
}
