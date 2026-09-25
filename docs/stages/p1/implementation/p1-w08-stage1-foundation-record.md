# P1-W08 Stage-1 table foundation implementation record

**Status:** Pure table/address foundation only; MMU transition not implemented.
**Scope:** Typed addresses, three-level fixed table model and host tests.
**Version:** v0.1
**Owner/change context:** P1-W08 foundation, 2026-09-25.
**Supersedes:** None.

The [W08 detailed design](p1-w08-host-stage1-address-space/README.md) and
[preflight amendment](p1-w08-host-stage1-address-space/00-preflight-amendment.md)
govern this work. `boot/address.rs` moves W02's crate-local `PhysAddr` from
`context.rs` into a shared boot module and adds `VirtAddr`, `ByteSize`, and
checked addition/distance. `arch/aarch64/stage1/model.rs` defines six closed
mapping classes, five page-aligned table pages (L1, two L2, two L3), the
39-bit 4 KiB index geometry, descriptor encoding, and a bidirectional
inventory/table verification walk. It does not touch hardware. Table building
clears first, validates the entire supplied inventory before filling, and
rejects overlap, unaligned or out-of-window regions and invalid table bases.

The page model is source-shared into five host tests. The host inventory in
those tests is synthetic; it is **not** an assertion about current linker
bounds. The model source is deliberately not linked into the target boot path
until W08 delivers the reviewed linker inventory, premise checks, MAIR and
translation-control programming, and post-MMU sentinels. A scoped
`dead_code` allowance on the new address module is transitional and must be
removed by the full W08 change. The W02 boot-context address semantics are
unchanged; no external ABI or public API changes.

No new `unsafe`, dependency, allocator, dynamic mapping API, Guest Stage-2,
SMP, GIC or permanent identity-map promise is added. This checkpoint does not
claim P1-V13/P1-V14, mapped execution, console/vector continuity or QEMU
success.
