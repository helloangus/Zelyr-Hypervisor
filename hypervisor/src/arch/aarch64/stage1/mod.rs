//! P1-W08's single-shot EL2 Stage-1 transition mechanism.
//! W09 owns invocation and terminal routing of the returned Stage1Error.
mod model;
mod regs;

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::baseline::{self, BaselineCategory, BaselineStatus, ControlId};
use super::capabilities::{self, FactId, Observation};
use super::exceptions::{self, VectorStatus};
use crate::boot::address::{ByteSize, PhysAddr, VirtAddr};
use crate::boot::{console, fatal};
use model::{MAIR, MappingClass, PAGE_BYTES, Region, Stage1Error, Stage1Step, Tables};

const SCTLR_MCI: u64 = (1 << 12) | (1 << 2) | 1;
const RO_SENTINEL_VALUE: u64 = 0x5a45_4c59_5252_4f31;
static RO_SENTINEL: u64 = RO_SENTINEL_VALUE;
static RW_SENTINEL: AtomicU64 = AtomicU64::new(0);
static STARTED: AtomicBool = AtomicBool::new(false);

/// Five contiguous page-sized tables with a 4 KiB-aligned L1 base. Relaxed
/// atomic stores provide safe interior writes on the single masked boot CPU;
/// DSB SY orders them before the hardware walker is enabled. The source
/// model is first built and verified on the bounded 64 KiB boot stack.
#[repr(C, align(4096))]
struct StaticTables([AtomicU64; 5 * 512]);
impl StaticTables {
    const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; 5 * 512])
    }
}
const _: () = assert!(core::mem::size_of::<StaticTables>() == 5 * 4096);
static STAGE1_TABLES: StaticTables = StaticTables::new();

// SAFETY: U-013. W08's linker script uniquely defines these page-aligned
// bounds. Only symbol addresses are taken, never their u8 contents. The
// subsequent inventory check rejects bad order/alignment; a false linker
// premise is terminal FC-INVARIANT.
unsafe extern "C" {
    static __p1_boot_code_start: u8;
    static __p1_boot_code_end: u8;
    static __p1_vectors_start: u8;
    static __p1_vectors_end: u8;
    static __p1_text_start: u8;
    static __p1_text_end: u8;
    static __p1_rodata_start: u8;
    static __p1_rodata_end: u8;
    static __p1_data_start: u8;
    static __p1_data_end: u8;
    static __p1_stack_start: u8;
    static __p1_stack_end: u8;
}

fn fail(step: Stage1Step, detail: &'static str) -> Stage1Error {
    Stage1Error::new(step, detail)
}

fn inventory() -> [Region; 7] {
    // The linker supplies boundaries, not runtime-discovered addresses.
    // `addr_of!` forms raw addresses without reading the extern objects.
    let region = |name, start: *const u8, end: *const u8, class| Region {
        name,
        start: VirtAddr::new(start as u64),
        end: VirtAddr::new(end as u64),
        class,
    };
    [
        region(
            "boot-code",
            ptr::addr_of!(__p1_boot_code_start),
            ptr::addr_of!(__p1_boot_code_end),
            MappingClass::CodeRx,
        ),
        region(
            "vectors",
            ptr::addr_of!(__p1_vectors_start),
            ptr::addr_of!(__p1_vectors_end),
            MappingClass::Vectors,
        ),
        region(
            "text",
            ptr::addr_of!(__p1_text_start),
            ptr::addr_of!(__p1_text_end),
            MappingClass::CodeRx,
        ),
        region(
            "rodata",
            ptr::addr_of!(__p1_rodata_start),
            ptr::addr_of!(__p1_rodata_end),
            MappingClass::RoData,
        ),
        region(
            "data",
            ptr::addr_of!(__p1_data_start),
            ptr::addr_of!(__p1_data_end),
            MappingClass::DataRw,
        ),
        region(
            "stack",
            ptr::addr_of!(__p1_stack_start),
            ptr::addr_of!(__p1_stack_end),
            MappingClass::BootStack,
        ),
        Region {
            name: "console",
            start: VirtAddr::new(console::CONSOLE_REGION_BASE),
            end: VirtAddr::new(console::CONSOLE_REGION_BASE + console::CONSOLE_REGION_SIZE),
            class: MappingClass::ConsoleMmio,
        },
    ]
}

fn premise() -> Result<u64, Stage1Error> {
    if baseline::baseline_status(BaselineCategory::C8) != BaselineStatus::Established {
        return Err(fail(Stage1Step::Premise, "C8 not established"));
    }
    let sctlr = baseline::baseline_value(ControlId::SctlrEl2)
        .ok_or_else(|| fail(Stage1Step::Premise, "SCTLR undeclared"))?;
    if sctlr & SCTLR_MCI != 0 {
        return Err(fail(Stage1Step::Premise, "SCTLR M/C/I already on"));
    }
    let hcr = baseline::baseline_value(ControlId::HcrEl2)
        .ok_or_else(|| fail(Stage1Step::Premise, "HCR undeclared"))?;
    if hcr & 1 != 0 {
        return Err(fail(Stage1Step::Premise, "HCR VM already on"));
    }
    if exceptions::vector_status() != VectorStatus::Established {
        return Err(fail(Stage1Step::Premise, "vectors not established"));
    }
    if !fatal::fatal_path_ready() || !console::channel_available() {
        return Err(fail(Stage1Step::Premise, "fatal or console not ready"));
    }
    if capabilities::query(FactId::Granule4k).observation != Observation::Present(1) {
        return Err(fail(Stage1Step::Premise, "4K granule absent"));
    }
    match capabilities::query(FactId::PaRange).observation {
        Observation::Present(bits) => Ok(bits),
        _ => Err(fail(Stage1Step::Premise, "PA range absent")),
    }
}

/// One fallible mechanism call. W09 owns the only caller and turns every Err
/// into a terminal Stage1 report before it can record Stage1.complete.
pub(crate) fn enable_host_stage1() -> Result<(), Stage1Error> {
    if STARTED.swap(true, Ordering::Relaxed) {
        return Err(fail(Stage1Step::Premise, "Stage1 repeated"));
    }
    let pa_bits = premise()?;
    let regions = inventory();
    if exceptions::vector_base() != Some(regions[1].start.raw()) {
        return Err(fail(Stage1Step::Premise, "vector base mismatch"));
    }
    let table_base = PhysAddr::new(ptr::addr_of!(STAGE1_TABLES) as u64);
    let table_end = table_base
        .checked_add(ByteSize(5 * PAGE_BYTES))
        .ok_or_else(|| fail(Stage1Step::Build, "table end overflow"))?;
    if table_base.raw() < regions[4].start.raw() || table_end.raw() > regions[4].end.raw() {
        return Err(fail(Stage1Step::Build, "tables outside data"));
    }

    // Const evaluation avoids nested 4 KiB-aligned constructor temporaries
    // overflowing the 64 KiB boot stack in unoptimized builds.
    let mut prepared = const { Tables::new() };
    prepared.build(&regions, table_base)?;
    prepared.verify(&regions, table_base)?;
    for page in 0..5 {
        let words = prepared
            .page_words(page)
            .ok_or_else(|| fail(Stage1Step::Build, "table page index"))?;
        for (index, word) in words.iter().enumerate() {
            STAGE1_TABLES.0[page * 512 + index].store(*word, Ordering::Relaxed);
        }
    }
    for page in 0..5 {
        let words = prepared
            .page_words(page)
            .ok_or_else(|| fail(Stage1Step::Verify, "table page index"))?;
        for (index, word) in words.iter().enumerate() {
            if STAGE1_TABLES.0[page * 512 + index].load(Ordering::Relaxed) != *word {
                return Err(fail(Stage1Step::Verify, "table copy mismatch"));
            }
        }
    }

    let tcr = model::tcr_for_pa_bits(pa_bits)?;
    regs::dsb_sy();
    regs::tlbi_alle2();
    regs::dsb_sy();
    regs::isb();
    regs::ic_iallu();
    regs::dsb_sy();
    regs::isb();
    regs::mair_write(MAIR);
    regs::ttbr0_write(table_base.raw());
    regs::tcr_write(tcr);
    regs::isb();
    if regs::mair_read() != MAIR
        || regs::ttbr0_read() & !0xfff != table_base.raw()
        || regs::tcr_read() != tcr
    {
        return Err(fail(Stage1Step::Program, "translation register readback"));
    }
    regs::sctlr_write(regs::sctlr_read() | SCTLR_MCI);
    regs::isb();
    post_mmu_checks()?;
    Ok(())
}

fn post_mmu_checks() -> Result<(), Stage1Error> {
    if !STARTED.load(Ordering::Relaxed) {
        return Err(fail(Stage1Step::PostVerify, "startup flag clobbered"));
    }
    if regs::sctlr_read() & SCTLR_MCI != SCTLR_MCI {
        return Err(fail(Stage1Step::PostVerify, "SCTLR M/C/I readback"));
    }
    // SAFETY: U-014. RO_SENTINEL is a valid, aligned in-image immutable u64.
    // The verified RoData mapping covers it before translation is enabled;
    // volatile forces an actual memory read. A false mapping premise faults
    // through W05/W07 and is terminal FC-INVARIANT.
    let observed = unsafe { ptr::read_volatile(&RO_SENTINEL) };
    if observed != RO_SENTINEL_VALUE {
        return Err(fail(Stage1Step::PostVerify, "rodata sentinel"));
    }
    RW_SENTINEL.store(RO_SENTINEL_VALUE, Ordering::Relaxed);
    if RW_SENTINEL.load(Ordering::Relaxed) != RO_SENTINEL_VALUE {
        return Err(fail(Stage1Step::PostVerify, "data sentinel"));
    }
    Ok(())
}
