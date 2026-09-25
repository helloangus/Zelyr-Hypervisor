//! Source-shared NC2 injection and unchanged W03 decoder/policy tests.
#[path = "../../../hypervisor/src/arch/aarch64/capabilities/facts.rs"]
mod facts;
#[path = "../../../hypervisor/src/arch/aarch64/capabilities/validation_nc2.rs"]
mod validation_nc2;

use facts::{CapabilityReport, FactId, Observation, RawRegisters};

#[test]
fn nc2_changes_only_tgran4_across_every_encoding() {
    let base = 0xa5c3_0123_9876_5432_u64;
    let mask = 0xf_u64 << 28;
    for nibble in 0..16_u64 {
        let raw = (base & !mask) | (nibble << 28);
        let injected = validation_nc2::inject_nc2_sample(raw);
        assert_eq!(injected & mask, mask);
        assert_eq!(injected & !mask, raw & !mask);
        assert_eq!(injected, validation_nc2::inject_nc2_sample(injected));
    }
}

#[test]
fn nc2_reaches_existing_required_rejection_without_other_fact_changes() {
    let mut raw = RawRegisters {
        current_el: 8,
        mpidr: 0x8000_0000,
        pfr0: 0x1111,
        mmfr0: 5,
        mmfr1: 0,
        cntfrq: 62_500_000,
    };
    let normal = CapabilityReport::from_registers(&raw);
    assert_eq!(normal.verify_required(), Ok(()));
    raw.mmfr0 = validation_nc2::inject_nc2_sample(raw.mmfr0);
    let injected = CapabilityReport::from_registers(&raw);
    let rejection = injected.verify_required().expect_err("NC2 must reject");
    assert_eq!(rejection.fact, FactId::Granule4k);
    assert_eq!(
        injected.query(FactId::Granule4k).observation,
        Observation::Absent
    );
    for id in FactId::ALL {
        if id != FactId::Granule4k {
            assert_eq!(injected.query(id), normal.query(id), "{}", id.label());
        }
    }
}
