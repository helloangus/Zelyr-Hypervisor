//! AArch64 boot CPU mechanisms.
pub(crate) mod baseline;
pub(crate) mod capabilities;
pub(crate) mod exceptions;
// W09 owns the only invocation; compile the W08 mechanism before integration.
#[allow(dead_code)]
pub(crate) mod stage1;
