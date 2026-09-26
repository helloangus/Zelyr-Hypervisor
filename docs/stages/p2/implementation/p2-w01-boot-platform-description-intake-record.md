# P2-W01 implementation record

**Status:** Implemented within the declared reference boot scope; evidence is
in the [verification record](../verification/p2-w01-boot-platform-description-intake-verification.md).
**Scope:** Boot DTB access, bounded validation and immutable consumer cursor.
**Version:** v0.1
**Owner/change context:** P2-W01, 2026-09-26.
**Supersedes:** The unimplemented A1/A2 assumptions, as specified by the
[current-baseline amendment](p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md).

The P1 pointer-only handoff is consumed after P1 stable. P2 validates and maps
the header, derives the declared size, checks the full rounded extent, and
publishes one immutable `ValidatedBootDtb` only after header, token stream,
strings references and reservation termination pass. Errors contain static
classes and details; untrusted text is never reflected into boot diagnostics.

| Delivered artifact | Role |
|---|---|
| `hypervisor/src/platform/intake.rs` | Pure placement/header/structure/reservation validation, bounded cursor, node handles and borrowed reservation iterator |
| `hypervisor/src/arch/aarch64/stage1/dtb_model.rs` | Checked rounded spans, aperture capacity and RO/XN descriptors |
| `hypervisor/src/arch/aarch64/stage1/dtb.rs` | Claimed-once architecture mapping owner and U-018 byte boundary |
| `hypervisor/src/boot/p2.rs` | Private trusted-envelope construction, diagnostic funnel and caller-owned handoff |
| `crates/host-test-baseline/tests/p2_platform.rs` | Exact source-shared production logic and generated fixtures |

The workspace remains the existing two-member baseline; no parser crate or
new dependency is introduced. The original design's logical modules are
co-located inside intake instead of creating a new crate. Reservations use a
validated borrowed iterator to avoid a 16 KiB copy on the boot stack. Node
names are byte strings with anomaly reporting; header offsets and DT alignment
follow the cited DTSpec. Public handle metadata has getters only; constructors
and raw bytes remain private. These are internal Rust interfaces, not an ABI.

One new unsafe slice operation is U-018; U-012's existing architectural
barriers are re-reviewed for post-enable invalid-to-valid publication. U-013
linker symbols are read only as addresses, preserving their original invariant.
P1's debug table-constructor stack defect was fixed independently in PR #66
before this package was integrated.

W02 consumes only the certified cursor. W03 can consume DTB range and ordered
reservation spans; no allocation ownership is claimed here. The canonical
adapter's trusted envelope is the first 128 MiB of reference RAM. A larger RAM
configuration may place its DTB outside that envelope and is explicitly
rejected; W09 or a future firmware adapter must supply a reviewed enlarged
accessibility premise. W07 fixtures, W09's full matrix, hardware, copy/release,
and permanent mapping APIs remain outside this package. No unresolved design
conflict remains for the declared reference scope.
