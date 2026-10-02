# P2 contract reconciliation record

**Status:** Documentation revision authorized by the user's accepted audit plan;
not allocator implementation or package-completion evidence.\
**Scope:** Current baseline and W03–W10 producer/consumer design consistency.\
**Version:** v0.2\
**Owner/change context:** P2 documentation reconciliation, 2026-09-27;
AUD-003 owner direction recorded on 2026-10-02.\
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

### P2-ACR-02 — ordinary allocated-frame Host mapping (AUD-003)

**Owner decision (2026-10-02):** the project owner agreed that P2 is
accountable for producing Host Stage-1 access to ordinary allocated frames
consumed by P3/P4. P1's fixed bootstrap mappings retain their recorded scope.
This resolves the stage-ownership direction. The planning amendment assigns
[P2-W11](../plans/p2-w11-host-allocated-frame-mapping.md) as producer;
[proposed detailed design](p2-w11-host-allocated-frame-mapping/README.md) now
records geometry, architecture and ownership gates. Approval, implementation
and evidence remain pending.
P2-W10 tracks this prerequisite in its P3/P4 handoff; W10 itself is not the
mapping implementation. Neither W04-MAP's metadata-only aperture nor
W05-MAP's heap backing adapter supplies this broader contract.

The `P2-HOST-MAP` gate requires a named bounded producer package and an
approved design before affected consumer implementation. Its contract must
specify requested frame coverage, typed HPA/HVA conversion, RW/XN and memory
attributes, aliases, page-table storage and bootstrap access, ownership and
borrow lifetime, unmap/invalidation-before-free, and transactional failure
rollback. P2 supplies the initial boot-CPU access boundary; P3 owns any
subsequent SMP synchronization and cross-CPU invalidation protocol. This
decision adds no identity-map or all-RAM access promise.

Acceptance requires actual allocated-frame access for P3 local areas/stacks,
P4 Stage-2 tables, and P4 image-loading buffers; protected/out-of-range and
invalid-lifetime rejection; partial-map failure with complete rollback; and
unmap/invalidation-before-reuse evidence. Record host-model, target/QEMU, and
SMP coverage separately. All affected consumers remain blocked while the
required producer contract or applicable evidence is missing. No accepted
ADR is changed, and P2-ACR-01 remains independently ADR Required.

| Gate | Owner | Required before coding/integration | Current status |
|---|---|---|---|
| W04-LAYOUT | W04 | Freeze layout, identity, sizing and transactional free contracts during the future W04 run | Unimplemented; prior local work excluded by owner |
| W04-MAP | W04 architecture adapter | Establish bounded metadata access, architecture and final capacity evidence | Unimplemented; future W04 work |
| W05-MAP | W05 backing adapter | Retain W04 handles through mapping/borrows; unmap and invalidate before free; specify rollback | Detailed design and implementation pending |
| W05-GLOBAL | W05 | Freeze global registration/lifetime, interior mutability, exclusive access, reentrancy and pinned-target failure handling | Adapter design and implementation pending |
| P2-HOST-MAP | P2-W11; W10 tracks handoff | Deliver P2-ACR-02 frame-mapping contract and consumer-specific evidence | W11 proposed design recorded; geometry and W12 ownership specified; architecture/fit review, approval, implementation and evidence pending |
| W07-ADAPTER | W07 with W01 if its interface must change | Reuse actual validator and distinguish synthetic placement from runtime evidence | Adapter design pending; no fictional `intake_offline` API |
| W09-DTB | W09 recipe and W01 access owner | Trusted placement/access for 512 MiB and 2 GiB success cells, independently verified | Required cells retained; prerequisite pending |

W03 pure map contracts are reconciled for subsequent implementation review;
its own code/tests, bounded storage fit and joint W04 integration do not exist.
W04 is unimplemented; its layout and mapping require a fresh work-package run. W04
cannot be called live until W04-MAP closes and actual target evidence exists. W05–W10 remain proposed downstream designs. P2-ACR-01
remains independently **ADR Required** and blocks memory-object work only.

## Validation and delivery boundary

Changed artifacts are documentation only. New unsafe, code ABI/public API and
dependency changes: none. Logical design interfaces changed as listed above;
no actual Rust interface is added. Local documentation checks and the PR's
required checks accompany this revision; their exact results are recorded in
the [documentation verification](../verification/p2-contract-reconciliation-verification.md).
No new host, AArch64, QEMU, allocator, fuzz or hardware execution is claimed by
this record. Existing W01/W02 evidence is neither rerun nor widened.

## W04 excluded from this submission (2026-10-02)

The owner directed that W04 be treated as not completed and restarted later.
The local implementation, supplemental design and delivery/verification artifacts
are excluded; original W04 design remains a future-work baseline. W04-LAYOUT,
W04-MAP and actual allocation contracts require fresh admission/implementation
and evidence before downstream execution. W11/W12 describe required producer
interfaces, not already available Rust APIs.

## P2-ACR-01 proposal ready for owner decision (2026-10-02)

[ADR-062](../../../adr/adr-062-p2-minimal-memory-object-foundation.md) proposes
keeping the accepted P2 assignment for the minimal common object/view foundation,
with a bounded W12 follow-up and explicit W11/P4 adapters. It is Proposed, not
selected or accepted; P2-ACR-01 remains ADR Required. P4-W10 scope approval is
not approval of this object-model boundary. Existing physical allocator evidence
is preserved at its original scope; no new implementation is admitted here.

## P2-ACR-01 owner direction recorded (2026-10-02)

The project owner answered "可以" to ADR-062 option A: P2 supplies the common
minimum object/view foundation and P4 integrates Guest use. This supersedes the
preceding undecided-direction status. [W12](../plans/p2-w12-minimal-memory-objects.md)
is planned with N01–N06/V15; W11-OBJECT and P4 adapter reconciliation are explicit.
The ADR remains Proposed pending formal integration under the standing local
no-merge instruction; no completed PR is invented. Detailed-design admission,
implementation and evidence remain distinct gates; do not ask again for the
same owner direction. W04 is unavailable and must be redone as directed by the owner.

## W12/W11/P4 design reconciliation completed (2026-10-02)

[W12 design](p2-w12-minimal-memory-objects/README.md) supplies N01–N06;
[W11 v0.2](p2-w11-host-allocated-frame-mapping/README.md) consumes its retained
regions and specifies bounded VA/table geometry. P4-W02/W03 bodies now use this
ownership and load/retire ordering; P4-W10 supplies installed/resident/retirement
contracts and P7-W02 supplies the companion pre-entry abort design. These are
completed proposed documents, not completed production mechanisms. Formal
ADR/design approval, architecture/fit reviews and runtime evidence remain open.

## Owner approval of the memory/Stage-2 designs (2026-10-02)

The owner explicitly confirmed “我确认批准”. W12 v0.1, W11 v0.2, P4-W02/W03
v0.2, P4-W10 v0.1 and the P7-W02 pre-entry-abort companion v0.1 are approved;
see the [bounded approval record](../../../testing/documentation-audit/design-approval.md#owner-approval).
Earlier design-approval-pending statements for this set are superseded. ADR-062
formal lifecycle, W04-MAP, architecture/fit checks, producer implementation and
runtime evidence remain independently tracked. No unrelated stage is approved.
