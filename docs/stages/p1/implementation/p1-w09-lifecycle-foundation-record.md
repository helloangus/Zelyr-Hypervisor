# P1-W09 lifecycle foundation implementation record

**Status:** Coherent type/tracker foundation only; W09 sequencing not complete.
**Scope:** Internal phase vocabulary, legal encoding, tracker and reason carrier.
**Version:** v0.1
**Owner/change context:** P1-W09 foundation, 2026-09-25.
**Supersedes:** None.

`hypervisor/src/boot/lifecycle.rs` defines the nine labels, the eight
production phases, total position decoder, single-word tracker, and static
failure reason. Legal encodings are 0 through 17. Each transition performs
one exact compare-exchange; a duplicate or out-of-order call returns the
observed position and leaves the word unchanged. Stable has a separate
16→17 transition. `Unknown(usize)` preserves invalid raw values. All atomics
use Relaxed ordering because P1 has one masked boot CPU, the tracker carries
only its own value, and readers require no cross-CPU publication.

The [foundation reconciliation](p1-w09-initialization-sequencing/04-foundation-reconciliation.md)
records the narrow changes from the original design. The module is not yet
linked into the target boot path: W07 will link its tracker read and the full
W09 package will own all writes, marker materialization, adapters and stable
idle. Source-sharing the file into `crates/host-test-baseline/tests/` verifies
the same pure implementation without introducing a second runtime tracker.

No new `unsafe`, dependency, feature, external ABI or public API is added.
The crate-local lifecycle types are new internal interfaces. No Guest, SMP,
GIC, allocator, Stage-2 or recovery code is introduced. This record does not
claim P1-V15 or a stable QEMU boot.
