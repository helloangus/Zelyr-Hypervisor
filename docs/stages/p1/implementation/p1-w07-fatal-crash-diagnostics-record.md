# P1-W07 fatal-report implementation record

**Status:** Mechanism linked and target-built; integrated fault evidence pending.
**Scope:** P1 terminal report model, panic ownership transfer and exception route.
**Version:** v0.1
**Owner/change context:** P1-W07, 2026-09-25.
**Supersedes:** W02's minimal panic body through its recorded extension seam.

The [detailed design](p1-w07-fatal-crash-diagnostics/README.md) and
[implementation reconciliation](p1-w07-fatal-crash-diagnostics/00-implementation-reconciliation.md)
govern this branch. W07 has replaced the body of the binary's sole
`#[panic_handler]`, retaining one registration and a non-resetting single-entry
guard. The handler reads an approximate entry SP/LR pair using two explicit
output registers; it does not claim to recover the original panic call site.
The new fatal module offers panic, classified-exception and phase-failure
entries, a readiness declaration, one transport choice per report, and a
terminal stop. W05's router selects the W07 report only after readiness;
its pre-arm summary remains W05-owned.

All three entries acquire W07's report guard. W05's separate architectural
guard prevents a second vector entry; W07's guard prevents a panic during an
exception report from recursively emitting another report. W05's router
checks that guard before either its pre-arm summary or
post-arm branch, so a vector fault interrupting a W07 report also stops
silently. The report uses fixed 128-byte UTF-8-safe lines with an explicit
truncation suffix and an end
marker. Long panic messages occupy their own line so they cannot truncate
location or SP/LR fields. Exception GPRs use one line each. The chosen
transport is W06 when available, else W02's early writer, and never changes
mid-report. The token classes are `ZELYR P1 PANIC` and `ZELYR P1 FATAL`;
`ZELYR P1 REPORT END` marks terminal output.

New unsafe U-011 comprises only closed `CurrentEL` and handler-entry SP/LR
reads; safe atomics own guard/readiness. No new dependency, feature, external
ABI or public API is intended; crate-local fatal/report seams are new. The
W09-owned `InitPhase`, `LifecyclePosition`, `FailureReason` and `TRACKER`
interfaces are linked from the merged W09 foundation. W07 reads the tracker
without writing or inventing a phase. Until the full W09 sequencer links its
writer and fatal-path phase, a documented `dead_code` allowance is scoped to
the lifecycle module and the three deferred fatal call sites; W09 must remove
these allowances when it connects them. W08 mapping and W11 fault execution
remain separate owners. There are no recovery, storage, Guest or GIC additions.
