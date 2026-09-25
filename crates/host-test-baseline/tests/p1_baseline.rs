//! Production value/mask policy only: no privileged host execution.
#[path = "../../../hypervisor/src/arch/aarch64/baseline/specs.rs"]
pub mod specs;

use specs::*;

#[test]
fn writes_remove_owned_residue_and_preserve_unowned_fields() {
    for spec in SPECS {
        for previous in [0, u64::MAX, 0x5555_5555_5555_5555, 0xaaaa_aaaa_aaaa_aaaa] {
            let written = spec.apply(previous);
            assert!(spec.verify(written), "{}", spec.control.label());
            assert_eq!(written & spec.mask, spec.value);
            if spec.full_write {
                assert_eq!(written, spec.value);
            } else {
                assert_eq!(written & !spec.mask, previous & !spec.mask);
            }
        }
    }
}

#[test]
fn every_owned_bit_readback_mismatch_is_rejected() {
    for spec in SPECS {
        assert_eq!(spec.value & !spec.mask, 0);
        for bit in 0..64 {
            let changed = spec.value ^ (1_u64 << bit);
            assert_eq!(spec.verify(changed), spec.mask & (1_u64 << bit) == 0);
        }
    }
}

#[test]
fn physical_and_optional_virtual_timers_ignore_read_only_istatus() {
    let timers: Vec<_> = SPECS
        .iter()
        .filter(|s| matches!(s.control, ControlId::CnthpCtlEl2 | ControlId::CnthvCtlEl2))
        .collect();
    assert_eq!(timers.len(), 2);
    for spec in timers {
        assert_eq!(spec.apply(u64::MAX), 2);
        assert!(spec.verify(6)); // ISTATUS is read-only and excluded.
        assert!(!spec.verify(3)); // ENABLE must remain clear.
        assert_eq!(
            spec.optional_virtual_timer,
            spec.control == ControlId::CnthvCtlEl2
        );
    }
}

#[test]
fn controls_are_unique_ordered_and_never_write_w02_state() {
    let mut seen = [false; ControlId::COUNT];
    let mut previous = BaselineCategory::C1 as usize;
    for spec in SPECS {
        assert!(!seen[spec.control as usize]);
        seen[spec.control as usize] = true;
        assert!((spec.category as usize) >= previous);
        previous = spec.category as usize;
    }
    assert!(!seen[ControlId::SpSel as usize]);
    assert!(!seen[ControlId::Daif as usize]);
    assert!(seen[2..].iter().all(|v| *v));
    assert_eq!(SPECS[0].control, ControlId::HcrEl2);
    assert_eq!(SPECS[0].value, 1 << 31); // RW only: VM/E2H/routing/traps clear.
}

#[test]
fn stage2_disabled_constant_retains_required_res1() {
    let spec = SPECS
        .iter()
        .find(|s| s.control == ControlId::VtcrEl2)
        .unwrap();
    assert_eq!(spec.apply(u64::MAX), 1 << 31);
    assert!(!spec.verify(0));
}
