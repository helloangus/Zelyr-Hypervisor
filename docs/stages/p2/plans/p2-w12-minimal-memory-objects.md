# P2-W12 — Minimal memory ownership and mapping views

Chinese readers can use the [Chinese edition](p2-w12-minimal-memory-objects.zh-CN.md).

**Status:** Authorized work package; detailed design owner-approved on 2026-10-02; implementation and validation pending
**Version:** v0.1
**Owner/change context:** P2-ACR-01 / ADR-062 option A selected by the project owner on 2026-10-02 ("可以")
**Parent:** [P2 task book](../task-book-v0.1.md#11-owner-authorized-minimal-memory-foundation)
**Prerequisites and consumers:** [P2 plan index](README.md)
**Decision record:** [ADR-062](../../../adr/adr-062-p2-minimal-memory-object-foundation.md); owner direction recorded, formal ADR integration pending

## Goal

Provide the minimal architecture-neutral MemoryObject backing-ownership and
MemoryRegion mapping-view lifetime foundation needed by Host and Guest mapping
consumers, preserving the accepted ADR distinction between owned backing and
address-space views.

## Scope

Required P2-N01–N06 / P2-V15: allocator-derived ordinary RAM ownership; stable
checked identities and bounded capacity; object-range/permission/memory-type
view validation; retention throughout reservation/publication/live mapping and
revocation; backend-specific completed cleanup before release; failure retention
and exact allocation accounting; separate Host/P4 adapter handoffs.

W04 is the only physical allocation/free authority. W12 must retain its actual
ownership handles instead of introducing an independent frame-ownership ledger.
The region is a view, never a second backing owner. No raw HPA or plain boolean
may prove ownership or completed invalidation. The first implementation is
boot-CPU-only, with explicit exclusive execution and lifetime constraints.

W12 depends on W04 ownership and typed-address inputs, not on actual W11 mapping.
An injected backend permits Host lifetime testing before W11's hardware adapter
exists. W11 remains the Host translation authority; P4-W02/W10 remain the Guest
translation authorities; P4-W03 owns Guest content/loading. Their final adapters
must consume the same object/view authority without duplicating ownership.

Reserved: external backing, sharing/donation, DMA/IOMMU, COW, hotplug and later
P3-reviewed concurrent use. Compatibility with those extensions is required;
their policies or mechanisms are not delivered by this package.

## Out of scope

Host/Guest descriptor encoding or register operations; W04 metadata bootstrap;
VM/vCPU/GuestAddressSpace and VMID mechanisms; public management/capability ABI;
scheduler policy; SMP locks or cross-CPU shootdown; raw-address ownership imports;
new module layouts, Rust signatures or storage algorithms frozen by this plan.

## Work sequence

1. Inspect W04 ownership identities, move/error/free semantics, address types,
   W11 lifetime/rollback needs and P4 backing/view handoff. Record missing inputs
   and the ADR decision/integration status without inventing an approved API.
2. Produce a reviewed detailed design for object/view states, authoritative
   ownership, bounded storage, identity exhaustion, failed construction,
   retention/release and backend-completion authority; obtain coding admission.
3. Implement the approved pure ownership/view foundation using injected
   storage/backend seams, preserving actual W04 allocation ownership and counts.
4. Verify successful and rejected lifecycles, invalid/overflowing ranges,
   rights/type mismatches, stale identities, capacity exhaustion, partial-map
   failure, failed cleanup, leaked tokens and repeated release/reuse.
5. Reconcile the W11 Host adapter and P4-W02/W03/W10 consumer designs. Distinguish
   simulated completion from each backend's architecture, execution and
   quiescence evidence; no raw-address or duplicated-ledger workaround.
6. Hand W08/W09/W10 and P3/P4 the implemented contract, evidence, integration
   requirements and limitations. Keep hardware/SMP/Guest capability gates open
   until their owning-stage evidence exists.

## Acceptance and closure

P2-V15 requires evidence that each backing has one authoritative owner; a region
is bounded and cannot outlive it; live/in-flight/failed-cleanup views retain
backing; only reviewed backend completion allows revocation and release; caller
errors preserve ownership; reuse never accepts stale identities; exhaustion and
accounting are explicit and correct. Missing or forgotten tokens must retain
resources rather than silently release hardware-visible backing.

Host-model tests prove the modeled lifetime/ownership behavior only. W11's
V14 and W09's V11 separately establish Host mapped access. P4 owns actual Guest
translation/retirement proof, P3 owns any SMP extension. V12/V13 handoff review
must distinguish those boundaries. A plan, an empty type or successful synthetic
callback does not satisfy an execution row.

## Handoff

W11 consumes common backing/view lifetime authority while owning Host map and
revoke operations. P4-W03 integrates Guest backing without owning translation;
P4-W02/W10 consume bounded views and keep mappings/context/retirement authority.
W08 receives the failure/lifetime scenarios; W09 tests integrated Host use; W10
publishes capability-specific evidence. W04-MAP remains outside this dependency
chain to avoid bootstrap recursion. W05 heap integration is not silently added.

Detailed design and implementation records belong in ../implementation/; actual
verification belongs in ../verification/. Identity/storage/API representation
are Implementation Choices, backend completion is a producer-owned contract,
architecture ordering is Specification Investigation, and any remaining ADR
conflict is ADR Required. This plan does not claim formal ADR integration.

The [detailed design](../implementation/p2-w12-minimal-memory-objects/README.md) is owner-approved on 2026-10-02; implementation and runtime evidence remain separate.
