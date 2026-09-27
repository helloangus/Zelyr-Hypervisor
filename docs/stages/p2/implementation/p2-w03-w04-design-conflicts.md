# P2-W03/W04 current-baseline design conflicts

**Status:** Original audit retained; documentation contradictions revised.
Detailed-design and runtime foundations remain open per the disposition below.
**Scope:** Admission audit for W03/W04 implementation, not completion evidence.
**Version:** v0.2
**Owner/change context:** P2-W03/W04 request, 2026-09-27.
**Supersedes:** None; records issues without changing existing contracts.

## Current baseline

Audited main commit `de57f0b45b262cb9b079cda70f747b50cb467ec3`.
W01/W02 are implemented in their bounded reference scope; their former
[access conflict](p2-w01-w02-prerequisite-conflict.md) is resolved.
The W03/W04 designs still describe a documentation-only September 18 baseline.
The image bounds, address types, validated DTB and normalized facts now exist.
No bootmap or page allocator implementation is present in the current source.

## Original findings (historical audit at de57f0b)

1. **Writable metadata access has no provider.**
   [W04 A2](p2-w04-physical-page-allocation/01-scope-and-foundations.md)
   requires P1 read/write coverage of allocatable RAM and explicitly says
   absence is a blocked upstream defect, with no improvised identity map.
   [P1's actual contract](../../p1/contracts/host-address-space.md) leaves unused
   RAM unmapped. The [W01 amendment](p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md)
   authorizes only bounded read-only DTB access. Current
   `hypervisor/src/arch/aarch64/stage1/dtb.rs` exposes only an immutable byte
   slice; `hypervisor/src/boot/p2.rs` stops after discovery. Neither supplies
   the exclusive writable metadata storage required by W04.
2. **W03 normalization rules are internally ambiguous.**
   [W03 policy R3/R4](p2-w03-boot-memory-map-ownership/02-architecture-and-state.md)
   says overlapping RAM is fatal but identical RAM merges, without specifying
   the identical-case precedence. R6/R7 similarly conflict for identical ranges
   of the same class with distinct source identities. Its
   [conversion contract](p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md)
   requires every protected extent to be page-aligned. W01 provides exact
   byte extents, not page-rounded protection; W02 also retains unusable facts.
   A current consumer contract must specify rejection versus outward rounding,
   rounding-induced overlap, and fatal handling of unusable protection facts.
   This is not a claim that the canonical QEMU DTB is unaligned.
3. **W04 metadata sizing does not account for its representation.**
   [Sizing rule](p2-w04-physical-page-allocation/01-scope-and-foundations.md)
   includes two bits per frame, list heads and less than 64 bytes of bookkeeping.
   [Architecture](p2-w04-physical-page-allocation/02-architecture-and-state.md)
   additionally requires linked block descriptors in metadata, potentially one
   per free block. The code-contract sketch includes `LIST_NODE_RESERVE`, but its size, capacity
   and units were not defined and the prose formula omitted node storage. It also
   describes a table indexed from the minimum frame, while sizing is per-span;
   treatment of large physical holes needs an explicit indexing contract.
4. **W04 free validation cannot enforce its stated invariant.**
   Invariant I4 requires freeing one original allocation of the matching order;
   the design stores no allocation-boundary/order tags and acknowledges this
   cannot be detected. Two adjacent order-0 allocations and one order-1
   allocation have the same two-bit table states; a table/list audit cannot
   recover that lost history. This conflicts with the required order-mismatch
   detection and W04-DV07 partial-free expectation. The correction needs tags
   or an explicitly revised ownership/API contract, not an implementation
   claiming the existing table proves I4.
5. **Accounting vocabulary disagrees across W04 contracts.**
   The managed domain is defined as sealed allocatable spans excluding
   metadata, while init conservation is `managed = free + reserved_meta`.
   Choose one definition and propagate it to stats, init, W06 and validation;
   metadata cannot be counted both outside and inside the same domain.

## Resolution and remaining work (2026-09-27)

The user authorized documentation reconciliation after the audit. The
[reconciliation record](p2-contract-reconciliation-record.md) links each revised
owner contract. This is a detailed-design repair, not a blanket ADR-required
change and not a new completion claim.

| Finding | Documentation disposition | Remaining work |
|---|---|---|
| Writable metadata provider | P2-W04 owns bounded metadata access; the stale P1 assumption is removed | W04-MAP concrete design, implementation and runtime evidence pending |
| Normalization ambiguity | W03 defines duplicate precedence, one W02 reservation source, non-usable fact rejection and checked outward rounding with source-preserving page union | W03 implementation/tests and storage fit pending |
| Metadata sizing/indexing | Full required sizing terms, worst-case nodes, local indices and normalized-span capacities replace incomplete assumptions | W04-LAYOUT must freeze representation and byte layout before allocator coding |
| Free validation | Original head/order/continuation and allocator identity checks required; non-Copy private handles preserve ownership, including on failure | Concrete identity representation under W04-LAYOUT, then implementation/tests |
| Accounting | Sealed allocatable equals managed equals free+used; metadata is protected and counted separately; init no longer searches for metadata in allocatable | Joint W03/W04 tests and W06/W09 integration pending |

Subsequent audit also corrected W05 storage assumptions and propagated the
remaining W05/W07/W09 admission gates. P2-ACR-01 and the ADR section 18 allocator
freeze question remain visible; this revision does not settle either through
an edit to an accepted ADR.

## Validation boundary

This is a source/contract audit. Documentation translation checking, local-link
checking and `git diff --check` are recorded with this change. No new host,
AArch64, QEMU, fuzz, hardware or allocator tests were run. P2-V05/P2-V06 and
W03/W04 completion remain unclaimed; existing W01/W02 evidence is not rerun
or widened by this record.
