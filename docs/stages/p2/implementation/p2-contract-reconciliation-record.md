# P2 contract reconciliation record

**Status:** Documentation revision authorized by the user's accepted audit plan;
not allocator implementation or package-completion evidence.\
**Scope:** Current baseline and W03–W10 producer/consumer design consistency.\
**Version:** v0.1\
**Owner/change context:** P2 documentation reconciliation, 2026-09-27.\
**Supersedes:** Stale assumptions and contradictory clauses identified below;
no accepted ADR, task-book outcome or historical execution evidence is replaced.

## Baseline and authority

Inspected `main@ecae09f`: Cargo workspace and host-test member exist; the target
is `aarch64-unknown-none-softfloat`; P1 exposes linker image bounds and its fixed
Stage-1 environment; W01 owns a bounded read-only DTB aperture; W02 supplies
`PlatformInfo` with `Fact<T>` entries. The current boot stops after discovery.

The [W01 amendment](p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md)
and [W02 record](p2-w02-platform-discovery-normalization-record.md) already
explain their implemented departures from September 18 assumptions. Their
verification records are retained unchanged. A proposed downstream design or
host fixture is not evidence that a missing runtime foundation exists.

## Changes by owner

| Owner | Corrected contract | Authoritative home |
|---|---|---|
| W03 | W02 is the sole reservation source; present non-usable facts fail closed; exact byte conflict checks precede outward rounding; duplicate exceptions precede overlap errors; page union retains every source | [input policy](p2-w03-boot-memory-map-ownership/01-scope-and-foundations.md), [range policy and bounds](p2-w03-boot-memory-map-ownership/02-architecture-and-state.md) |
| W03 | Distinguish original byte ledger, normalized page classes and metadata ledger; count overlapping rounded sources once; 46 span and 184 map-entry bounds follow current input capacities | [code contracts](p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md) |
| W04 | Initialize from sealed allocatable directly; metadata ledger supplies separate exclusive storage; complete layout budget, local indexing, original head/order validation and non-Copy owner-specific handles | [code contracts](p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md) |
| W04 | `managed = sealed allocatable = free + used`; metadata is protected and separate; maximum order 18 means 1 GiB, not 256 MiB | [stats contract](p2-w04-physical-page-allocation/03-code-contracts-pagealloc.md#7-allocationstats) |
| W05 | Fixed in-Heap directory, zero initial page usage, every slab class reserves header slots, full-block budget preflight; mapping ownership separate from physical allocation | [architecture](p2-w05-dynamic-small-allocation/02-architecture-and-state.md) |
| W06 | Consume source getters and normalized region capacities; compare RAM union rather than raw duplicate bank sum; apply W04 equations | [inspection contracts](p2-w06-platform-memory-inspection/03-code-contracts-inspection.md) |
| W07 | Real W01 entry has placement parameters, no offline bypass/max-size override; retain honest offline proof boundary | [adapter gate](p2-w07-offline-dtb-compatibility/01-scope-and-foundations.md#3-offline-input-model) |
| W08 | Align input/map oracles and add single-ingestion, non-usable fact, rounded page sharing, sparse/many-region, release-identity and full-state failure checks | [map matrix](p2-w08-host-robustness-regression/02-matrices-input-and-map.md), [allocator matrix](p2-w08-host-robustness-regression/03-matrices-allocator-stress-determinism.md) |
| W09/W10 | Preserve large-RAM goals with explicit access prerequisite; propagate equations, identity validation and actual evidence limits | [QEMU matrix](p2-w09-qemu-integration-regression/02-configuration-matrix.md), [handoff](p2-w10-p3-p4-handoff-contract/01-consumer-contract.md) |

No stage task book or bounded plan is changed into a detailed design. No
`MemoryObject`/`MemoryRegion`, Guest mechanism or SMP synchronization is added.
Buddy remains the existing proposal; the ADR section 18 pending freeze remains
visible. Ordinary detailed-design corrections are not new ADR decisions.

## Remaining admission gates

| Gate | Owner | Required before coding/integration | Current status |
|---|---|---|---|
| W04-LAYOUT | W04 | Freeze byte layout/widths, stable owner identity, complete size bounds and transactional list/tag edits; prove worst-case node capacity | Detailed design pending; allocator coding not ready |
| W04-MAP | W04 architecture adapter | Freeze bounded sealed-metadata RW/XN aperture, table storage, attributes, barriers/TLBs, aliases, lifetime and failure handling | Detailed design and runtime implementation pending |
| W05-MAP | W05 backing adapter | Retain W04 handles through mapping/borrows; unmap and invalidate before free; specify rollback | Detailed design and implementation pending |
| W05-GLOBAL | W05 | Freeze global registration/lifetime, interior mutability, exclusive access, reentrancy and pinned-target failure handling | Adapter design and implementation pending |
| W07-ADAPTER | W07 with W01 if its interface must change | Reuse actual validator and distinguish synthetic placement from runtime evidence | Adapter design pending; no fictional `intake_offline` API |
| W09-DTB | W09 recipe and W01 access owner | Trusted placement/access for 512 MiB and 2 GiB success cells, independently verified | Required cells retained; prerequisite pending |

W03 pure map contracts are reconciled for subsequent implementation review;
its own code/tests, bounded storage fit and joint W04 integration do not exist.
W04 cannot be called implementation-ready until W04-LAYOUT closes, or live until
W04-MAP also closes. W05–W10 remain proposed downstream designs. P2-ACR-01
remains independently **ADR Required** and blocks memory-object work only.

## Validation and delivery boundary

Changed artifacts are documentation only. New unsafe, code ABI/public API and
dependency changes: none. Logical design interfaces changed as listed above;
no actual Rust interface is added. Local documentation checks and the PR's
required checks accompany this revision; their exact results are recorded in
the [documentation verification](../verification/p2-contract-reconciliation-verification.md).
No new host, AArch64, QEMU, allocator, fuzz or hardware execution is claimed by
this record. Existing W01/W02 evidence is neither rerun nor widened.
