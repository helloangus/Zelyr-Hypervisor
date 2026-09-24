//! Pure W03 decoding and report policy. Register references: W03 record.
use core::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactId {
    ExecutionLevel,
    CpuAffinity,
    ArchProfile,
    GicVersion,
    PaRange,
    AsidBits,
    Granule4k,
    Granule16k,
    Granule64k,
    VirtualHostExtensions,
    El2VirtualTimer,
    CounterFrequency,
    El2PhysicalTimer,
}
impl FactId {
    pub const ALL: [Self; 13] = [
        Self::ExecutionLevel,
        Self::CpuAffinity,
        Self::ArchProfile,
        Self::GicVersion,
        Self::PaRange,
        Self::AsidBits,
        Self::Granule4k,
        Self::Granule16k,
        Self::Granule64k,
        Self::VirtualHostExtensions,
        Self::El2VirtualTimer,
        Self::CounterFrequency,
        Self::El2PhysicalTimer,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::ExecutionLevel => "execution-level",
            Self::CpuAffinity => "cpu-affinity",
            Self::ArchProfile => "arch-profile",
            Self::GicVersion => "gic-version",
            Self::PaRange => "pa-range",
            Self::AsidBits => "asid-bits",
            Self::Granule4k => "granule-4k",
            Self::Granule16k => "granule-16k",
            Self::Granule64k => "granule-64k",
            Self::VirtualHostExtensions => "virtual-host-extensions",
            Self::El2VirtualTimer => "el2-virtual-timer",
            Self::CounterFrequency => "counter-frequency",
            Self::El2PhysicalTimer => "el2-physical-timer",
        }
    }
    pub const fn classification(self) -> Classification {
        match self {
            Self::ExecutionLevel | Self::Granule4k | Self::CounterFrequency => {
                Classification::Required
            }
            Self::GicVersion | Self::AsidBits | Self::VirtualHostExtensions => {
                Classification::Future
            }
            _ => Classification::Optional,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classification {
    Required,
    Optional,
    Future,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Present(u64),
    Absent,
    #[allow(dead_code)] // Architecture-access violations are modeled in policy tests.
    Unreadable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactRecord {
    pub id: FactId,
    pub classification: Classification,
    pub observation: Observation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityRejection {
    pub fact: FactId,
    pub reason: &'static str,
}
/// Raw register snapshots, not addresses or access authority.
pub struct RawRegisters {
    pub current_el: u64,
    pub mpidr: u64,
    pub pfr0: u64,
    pub mmfr0: u64,
    pub mmfr1: u64,
    pub cntfrq: u64,
}
#[derive(Clone, Copy)]
pub struct CapabilityReport {
    records: [FactRecord; 13],
}
const fn field(raw: u64, shift: u32) -> u64 {
    (raw >> shift) & 15
}
const fn supported(present: bool) -> Observation {
    if present {
        Observation::Present(1)
    } else {
        Observation::Absent
    }
}
impl CapabilityReport {
    pub fn from_registers(raw: &RawRegisters) -> Self {
        let values = [
            Observation::Present((raw.current_el >> 2) & 3),
            Observation::Present(raw.mpidr & 0xff_41ff_ffff),
            Observation::Present(raw.pfr0 & 0xffff),
            match field(raw.pfr0, 24) {
                n @ (1 | 3) => Observation::Present(n),
                _ => Observation::Absent,
            },
            match field(raw.mmfr0, 0) {
                0 => Observation::Present(32),
                1 => Observation::Present(36),
                2 => Observation::Present(40),
                3 => Observation::Present(42),
                4 => Observation::Present(44),
                5 => Observation::Present(48),
                6 => Observation::Present(52),
                _ => Observation::Absent,
            },
            match field(raw.mmfr0, 4) {
                0 => Observation::Present(8),
                2 => Observation::Present(16),
                _ => Observation::Absent,
            },
            supported(matches!(field(raw.mmfr0, 28), 0 | 1)),
            supported(matches!(field(raw.mmfr0, 20), 1 | 2)),
            supported(field(raw.mmfr0, 24) == 0),
            supported(field(raw.mmfr1, 8) == 1),
            supported(field(raw.mmfr1, 8) == 1),
            match raw.cntfrq & 0xffff_ffff {
                0 => Observation::Absent,
                hz => Observation::Present(hz),
            },
            supported((raw.current_el >> 2) & 3 == 2),
        ];
        Self {
            records: core::array::from_fn(|index| {
                let id = FactId::ALL[index];
                FactRecord {
                    id,
                    classification: id.classification(),
                    observation: values[index],
                }
            }),
        }
    }
    pub fn query(&self, id: FactId) -> FactRecord {
        self.records[id as usize]
    }
    pub fn verify_required(&self) -> Result<(), CapabilityRejection> {
        for record in self.records {
            let (valid, reason) = match record.id {
                FactId::ExecutionLevel => (
                    record.observation == Observation::Present(2),
                    "required: Non-secure EL2 execution",
                ),
                FactId::Granule4k => (
                    record.observation == Observation::Present(1),
                    "required: 4 KiB granule support for P1 mapping work",
                ),
                FactId::CounterFrequency => (
                    matches!(record.observation, Observation::Present(hz) if hz != 0),
                    "required: non-zero system counter frequency",
                ),
                _ => continue,
            };
            if !valid {
                return Err(CapabilityRejection {
                    fact: record.id,
                    reason,
                });
            }
        }
        Ok(())
    }
    /// Structured cpu.capability.fact encoding; sink supplies line framing.
    pub fn render(&self, emit: &mut dyn FnMut(&str)) {
        for record in self.records {
            let mut line = Line {
                bytes: [0; 96],
                len: 0,
            };
            if write!(
                line,
                "cap {}={} ({:?})",
                record.id.label(),
                record.observation,
                record.classification
            )
            .is_err()
            {
                panic!("capability report line capacity invariant");
            }
            // All tokens and numeric formatting are ASCII and fit 96 bytes.
            match core::str::from_utf8(&line.bytes[..line.len]) {
                Ok(text) => emit(text),
                Err(_) => panic!("capability report ASCII invariant"),
            }
        }
    }
}
impl core::fmt::Display for Observation {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Present(value) => write!(formatter, "{value}"),
            Self::Absent => formatter.write_str("Absent"),
            Self::Unreadable => formatter.write_str("Unreadable"),
        }
    }
}
struct Line {
    bytes: [u8; 96],
    len: usize,
}
impl Write for Line {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(core::fmt::Error)?;
        if end > self.bytes.len() {
            return Err(core::fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn raw() -> RawRegisters {
        RawRegisters {
            current_el: 8,
            mpidr: 0x8000_0000,
            pfr0: 0x1111,
            mmfr0: 5,
            mmfr1: 0,
            cntfrq: 62_500_000,
        }
    }
    #[test]
    fn normal_inventory_and_optional_absence() {
        let report = CapabilityReport::from_registers(&raw());
        assert_eq!(report.verify_required(), Ok(()));
        assert_eq!(
            report.query(FactId::PaRange).observation,
            Observation::Present(48)
        );
        assert_eq!(
            report.query(FactId::El2VirtualTimer).observation,
            Observation::Absent
        );
        for (index, id) in FactId::ALL.iter().enumerate() {
            assert_eq!(report.records[index].id, *id);
        }
    }
    #[test]
    fn every_granule_encoding_has_architectural_meaning() {
        for value in 0..16 {
            let mut registers = raw();
            registers.mmfr0 = (value << 28) | (value << 24) | (value << 20);
            let report = CapabilityReport::from_registers(&registers);
            assert_eq!(
                report.query(FactId::Granule4k).observation,
                supported(value <= 1)
            );
            assert_eq!(
                report.query(FactId::Granule16k).observation,
                supported(value == 1 || value == 2)
            );
            assert_eq!(
                report.query(FactId::Granule64k).observation,
                supported(value == 0)
            );
        }
    }
    #[test]
    fn required_absence_is_ordered_and_optional_unreadability_does_not_reject() {
        for bad in [
            Observation::Absent,
            Observation::Unreadable,
            Observation::Present(0),
        ] {
            let mut report = CapabilityReport::from_registers(&raw());
            for record in &mut report.records {
                if record.classification != Classification::Required {
                    record.observation = bad;
                }
            }
            assert_eq!(report.verify_required(), Ok(()));
            for id in [
                FactId::CounterFrequency,
                FactId::Granule4k,
                FactId::ExecutionLevel,
            ] {
                report.records[id as usize].observation = bad;
                assert_eq!(report.verify_required().unwrap_err().fact, id);
            }
        }
    }
    #[test]
    fn reserved_encodings_and_frequency_boundaries() {
        let mut registers = raw();
        registers.mmfr0 = 0xff;
        registers.mmfr1 = 0xf00;
        registers.pfr0 = 0xf00_0000;
        registers.cntfrq = 1 << 32;
        registers.mpidr = u64::MAX;
        let report = CapabilityReport::from_registers(&registers);
        for id in [
            FactId::PaRange,
            FactId::AsidBits,
            FactId::VirtualHostExtensions,
            FactId::El2VirtualTimer,
            FactId::GicVersion,
            FactId::CounterFrequency,
        ] {
            assert_eq!(report.query(id).observation, Observation::Absent);
        }
        assert_eq!(
            report.query(FactId::CpuAffinity).observation,
            Observation::Present(0xff_41ff_ffff)
        );
        registers.cntfrq = 1;
        assert_eq!(
            CapabilityReport::from_registers(&registers).verify_required(),
            Ok(())
        );
    }
    #[test]
    fn complete_bounded_rendering() {
        let mut report = CapabilityReport::from_registers(&raw());
        report.records[FactId::CpuAffinity as usize].observation = Observation::Present(u64::MAX);
        report.records[FactId::GicVersion as usize].observation = Observation::Unreadable;
        let mut count = 0;
        report.render(&mut |line| {
            assert!(line.starts_with(&std::format!("cap {}=", FactId::ALL[count].label())));
            assert!(line.ends_with(')') && line.len() < 96);
            if count == FactId::GicVersion as usize {
                assert_eq!(line, "cap gic-version=Unreadable (Future)");
            }
            count += 1;
        });
        assert_eq!(count, 13);
    }
}
