# P2-W02 implementation record

**Status:** Implemented; bounded host and reference-QEMU evidence is in the
[verification record](../verification/p2-w02-platform-discovery-normalization-verification.md).
**Scope:** CPU, memory, reservation, GIC/timer/PSCI, chosen and capability facts.
**Version:** v0.1
**Owner/change context:** P2-W02, 2026-09-26.
**Supersedes:** Implementation absence; the owning design remains subject to
the [specification corrections](p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md).

`hypervisor/src/platform/discovery.rs` supplies pure bounded decoders and one
`normalize` entry. Only W01 cursor access is used. `PlatformInfo` has private
fields and read-only getters; it owns copied facts, not raw DTB references.
`Fact<T>` preserves NotDiscovered, Absent, Unsupported, Unusable and Usable.
Capabilities project those states; PCI, SMMU and ACPI remain NotDiscovered.

CPU records retain affinity, status and enable method; boot CPU must match an
enabled usable entry. RAM and both reservation sources retain DT order. Invalid
individual records remain visible; zero-size ranges are counted. Empty usable
CPU/RAM sets, boot-CPU mismatch and fixed storage exhaustion are fatal.
Reservations cannot silently disappear into an incomplete allocator input.

GICv3 regions/stride, raw timer interrupt count via interrupt-parent, PSCI
version/conduit, stdout path with verified single-hop alias target, bootargs,
and initrd ranges are captured. Singleton status annotations are retained;
disabled/reserved devices are Absent and other non-operational statuses are
Unusable. Unsupported translated bus addresses remain unusable rather than
being mistaken for physical addresses. Additional singleton candidates and
skipped unknown nodes are counted.

The original design's record sketch is represented with bounded `List<Fact<T>>`
where malformed records must survive. Reservation sources and flags remain
part of usable records. Console resolution retains an optional copied target
path. Determinism compares Rust fields, not padding bytes. The host-only fixture
builder may allocate; production code has no allocation, unsafe, board-name
branch, external dependency, device access or serialization ABI.

Tests reside in `crates/host-test-baseline/tests/p2_platform.rs`; the boot adapter
in `hypervisor/src/boot/p2.rs` retains the W01 owner/handle and the normalized
result while idle. W03 and W06 receive facts, not a second parsing path. W07,
W09 and W10 still own real fixture coverage, full regression and formal P3/P4
handoff; this record does not claim those packages complete.
