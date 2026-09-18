//! P0-W08 placeholder host-test member.
//!
//! This member exists only to prove that the repository's host-test execution
//! entry executes and reports truthfully (see
//! `docs/testing/host-test-baseline.md`). Its single test asserts the entry's
//! own health and encodes no product logic; it is never counted as product
//! coverage and is expected to be superseded by the first real host-tested
//! logic.

#[cfg(test)]
mod tests {
    /// The entry-health placeholder (P0-W08): the canonical host-test entry
    /// reaches this member, compiles it for the host target, runs its tests,
    /// and reports the result truthfully. Superseded by the first real
    /// host-tested logic; retirement is a recorded minor change.
    #[test]
    fn host_entry_executes_and_reports() {
        // Entry health = the harness executes this test inside the member's
        // package context and reports the outcome truthfully.
        assert_eq!(
            std::env::var("CARGO_PKG_NAME").as_deref(),
            Ok("host-test-baseline")
        );
    }
}
