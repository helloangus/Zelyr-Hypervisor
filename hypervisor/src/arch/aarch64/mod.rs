//! AArch64 boot CPU mechanisms.
pub(crate) mod baseline;
pub(crate) mod capabilities;
pub(crate) mod exceptions;
pub(crate) mod idle;
#[cfg(not(any(feature = "p1-w11-nc3", feature = "p1-w11-nc4")))]
pub(crate) mod stage1;
#[cfg(any(feature = "p1-w11-nc3", feature = "p1-w11-nc4", feature = "p1-w11-nc5"))]
pub(crate) mod validation_fault;
