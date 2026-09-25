//! W11 NC2 validation-image injection. This module is absent from default builds.

/// Replace only ID_AA64MMFR0_EL1.TGran4 with its unsupported encoding.
/// The production decoder and required-capability policy consume the result.
pub(super) const fn inject_nc2_sample(raw: u64) -> u64 {
    const TGRAN4_MASK: u64 = 0xf << 28;
    (raw & !TGRAN4_MASK) | TGRAN4_MASK
}
