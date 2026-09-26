# P2-W01/W02 prerequisite conflict: boot DTB access

**Status:** Resolved by the P2-W01 current-baseline amendment and implementation.
**Scope:** Reconcile the P2-W01 detailed design's P1 input assumptions with
the completed P1 handoff before implementing boot DTB access.
**Version:** v0.1
**Owner/change context:** P2-W01/W02 current-state audit, 2026-09-26.
**Supersedes:** None.

## Observed contracts

| Input | P2-W01 design assumption | Current P1 contract and implementation |
|---|---|---|
| DTB extent | [W01 intake boundary §2 A1](p2-w01-boot-platform-description-intake/01-intake-boundary.md#2-assumed-prerequisite-contracts-and-failure-boundaries) requires a stable physical address **and byte length** from P1. | [P1 boot contract](../../p1/contracts/aarch64-boot-contract.md) forwards and retains only a nonzero `x0` pointer. It explicitly performs no size, content, or alignment validation. `BootContext::dtb()` returns only `PhysAddr` in `hypervisor/src/boot/context.rs`. |
| Readable DTB | [W01 intake boundary §2 A2](p2-w01-boot-platform-description-intake/01-intake-boundary.md#2-assumed-prerequisite-contracts-and-failure-boundaries) and [W01 access contract §2.1](p2-w01-boot-platform-description-intake/03-code-contracts-intake.md#21-fdt_accessblob_bytes) require a P1-established readable Stage-1 window covering the DTB. | [P1 Host Stage-1 contract](../../p1/contracts/host-address-space.md) explicitly leaves the DTB unmapped and assigns validated DTB access to P2. The [P1 W08 mapping inventory](../../p1/implementation/p1-w08-host-stage1-address-space/01-architecture-and-state.md#2-region-inventory--ownership) includes only the image and early console. |
| Image range | W01 A3 requires authoritative image bounds for overlap rejection. | P1's linker script and Stage-1 image inventory provide image symbols; this input appears available, subject to a concrete P2 adapter review. |

The [P2 task book §2](../task-book-v0.1.md#2-inputs-constraints-and-state)
requires P1 to supply a DTB or equivalent boot information and a stable host
environment. It does not require P1 to validate its size or map its contents.
It also says missing P0/P1 inputs must be treated as upstream defects, without
an improvised P2 workaround. The accepted P1 handoff names P2 as owner of
validated DTB access. Thus the current W01 detailed design's A1/A2 assumptions
cannot be implemented against the current completed P1 contract.

## Original decision required before boot integration

The owning design must resolve how P2 obtains a bounded, readable header from
the pointer-only handoff, derives and validates the declared `totalsize`, and
establishes a reviewed read-only Stage-1 mapping for the validated extent.
The decision must specify reachable physical ranges, page-table ownership and
capacity, access attributes, ordering and TLB maintenance, unsafe lifetime,
the image-overlap check before any bulk read, and diagnostics for an
unreachable or malformed header. This is a Host Stage-1 and untrusted-input
boundary; the coding guide requires a detailed contract before changing it.

Two possible owners require review against the task book and P1 handoff:

1. Amend P2-W01's detailed design to own the bounded DTB mapping and
   pointer-to-length validation, using an explicitly designed P2 extension of
   the P1 Stage-1 environment.
2. Change the P1 handoff and implementation to supply a stable length and
   readable window, then amend P2-W01's prerequisite table to cite that new
   verified P1 contract.

The initial audit left those alternatives open. The subsequent request to
resolve the problem authorizes route 1, consistent with the P1 handoff's
existing ownership. The [current-baseline amendment](p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md)
specifies the bounded P2 mapping and specification corrections; the
[W01 record](p2-w01-boot-platform-description-intake-record.md) and
[W02 record](p2-w02-platform-discovery-normalization-record.md) link executed
verification. This conflict is resolved for the declared reference boot scope. W02's [normalization entry](p2-w02-platform-discovery-normalization/04-code-contracts-facts.md#42-normalizerun)
requires W01's `ValidatedBootDtb`; the implemented boot path now supplies that boundary. Host-only fixture tests may be developed independently and
can provide structural and discovery evidence, but cannot prove that the boot
image can read its active DTB or complete either package's boot integration.
