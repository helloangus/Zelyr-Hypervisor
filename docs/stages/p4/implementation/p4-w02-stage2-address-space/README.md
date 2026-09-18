# P4-W02 Stage-2 Address-Space Correctness — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** One Guest's independently managed AArch64 Stage-2 address space with
lifecycle, map, unmap, protect, query, activation, and current-path TLB
consistency, as required by [P4-W02](../../plans/p4-w02-stage2-address-space.md)
(P4-A01–A07).  
**Owner/change context:** P4-W02 implementation handoff; this design owns the
Stage-2 object model, descriptor representation, VMID policy, and TLB
semantics for P4.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W02. It converts the bounded
work-package plan into concrete logical modules, object lifecycles, and
function/type contracts for the Stage-2 capability boundary, with pseudocode
rather than production code. It deliberately does **not** design the Guest
memory image or loader ([P4-W03](../p4-w03-guest-memory-image/README.md)),
the vCPU world-switch ([P4-W04](../p4-w04-vcpu-entry-exit/README.md)), fault
classification and diagnostics ([P4-W06](../p4-w06-fault-isolation-diagnostics/README.md)),
or the final generic `ExitReason` API, cross-pCPU shootdown policy, dirty
tracking, or a multi-VM machine ABI.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — goal-to-baseline
  ledger, assumed upstream contracts with failure boundaries, scope
  classification, and the resolved design decisions with their authority.
  Load first; every other file depends on it.
- [02-architecture-and-state.md](02-architecture-and-state.md) — logical
  modules, core objects, ownership, the address-space and mutation state
  machines, the concurrency model, and the Stage-2 descriptor/TLB/barrier
  semantics. Load for steps in the architecture and lifecycle work areas.
- [03-code-contracts-stage2-core.md](03-code-contracts-stage2-core.md) — the
  full function/type contract templates with pseudocode for the address-space
  object, mapping operations, query, activation, invalidation, and the
  `unsafe` descriptor/sysreg boundaries. Load for the code-contract work area.
- [04-implementation-workflow.md](04-implementation-workflow.md) — the ordered
  implementation steps with acceptance and failure handling.
- [05-validation-and-handoff.md](05-validation-and-handoff.md) — the
  validation matrix (P4-V02, P4-V07, P4-V08 inputs), error/security/
  observability model, and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P4 task book, the
P4-W02 plan, and the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md)
entry review result). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W02 plan](../../plans/p4-w02-stage2-address-space.md) → this design →
Coding Guidelines. Binding constraints include:

- ADR-018 and ADR §6: the Stage-2 address space is a core object with
  map/unmap/protect/translate semantics, break-before-make, TLB invalidation,
  and concurrency handling. P4 delivers the P4-scoped subset; ADR-level page
  metadata, share counts, and dirty tracking stay out.
- ADR-007 and ADR §19: Guest input is untrusted; Guest-caused faults are
  recoverable VM-facing events. Stage-2 is the enforcement mechanism that
  makes this true for memory.
- ADR-041–ADR-045 and ADR §19: architecture mechanism lives in the Arch
  layer; Core must not see descriptor bits, `VTTBR_EL2`/`VTCR_EL2`, or VMID
  encodings.
- Task book §1: page-table/VMID representation, mapper algorithms, final
  concurrent shootdown policy, large pages, dirty tracking, COW, migration,
  and a formal multi-VM machine ABI are out of W02 scope; "VMID-recycling
  optimization" is explicitly listed out of scope for P4.
- Task book §1 Reserved: the P3 cross-CPU transport is available for a later
  shootdown; P4 proves current-path consistency and must not preclude
  multi-pCPU handling.
- Task book §2: P4 consumes P2 memory ownership and P3's future cross-CPU
  transport without choosing their internals.

Classification: the modules, objects, contracts, and validation scenarios in
[02](02-architecture-and-state.md) and [03](03-code-contracts-stage2-core.md)
are **Required** for P4-A01–A07. Non-identity Guest IPA relocation, block/large
descriptors, cross-pCPU shootdown over the P3 transport, hardware
Access-flag-based dirty tracking, and a published mapper API for later stages
are **Reserved** with recorded re-entry points. Dirty tracking, COW, balloon,
memory overcommit, IOMMU/SMMU, Guest SMP, and any multi-VM lifecycle are
**Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-A01 independent address-space lifecycle (create/destroy) | [architecture](02-architecture-and-state.md) §3; [contracts](03-code-contracts-stage2-core.md) §3.1, §3.8 | P4-V02 lifecycle evidence |
| P4-A02 map Guest IPA → Host physical | [contracts](03-code-contracts-stage2-core.md) §3.2 | P4-V02; P4-V07 (mapped case) |
| P4-A03 unmap | [contracts](03-code-contracts-stage2-core.md) §3.3 | P4-V02; P4-V07 (unmapped case) |
| P4-A04 read/write/execute permission control | [contracts](03-code-contracts-stage2-core.md) §3.2, §3.4; [architecture](02-architecture-and-state.md) §5 | P4-V02; P4-V08 |
| P4-A05 diagnostic query of a mapping | [contracts](03-code-contracts-stage2-core.md) §3.5 | P4-V02; consumed by P4-W06 diagnostics |
| P4-A06 installation for Guest execution (active context) | [contracts](03-code-contracts-stage2-core.md) §3.6; [architecture](02-architecture-and-state.md) §4 | P4-V02; consumed by P4-W04 entry |
| P4-A07 current-path TLB consistency after mutation | [architecture](02-architecture-and-state.md) §6; [contracts](03-code-contracts-stage2-core.md) §3.7 | P4-V02, P4-V07, P4-V08 ("does not rely on stale translations") |
| P2 ownership / P3 transport assumptions fixed | [foundations](01-scope-and-foundations.md) §2–§3 | W01 row compatibility; failure boundaries recorded |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs` at
`4e631ee`): documentation-only repository. There is no Cargo workspace, no
crate, no Rust source, no AArch64 target definition, and no Stage-2 or memory
code. P0–P3 packages are planned; only P0-W01 has a verification record.
`docs/stages/p4/implementation/` contains only this design set (plus the
parallel W06–W09 design effort). Every foundation W02 needs is therefore an
assumed contract from an upstream plan, not an observable artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P4-A01: an independent address space can be created and destroyed | No code exists anywhere | Stage-2 table memory obtained only through the P2 page allocator (assumed contract, R09) and a P4-owned address-space object | "Independent" means the space owns its tables and releases them on destroy; there is no other source of pages | P2-W04 plan (assumed); W02 owns the object | P4-V02 create/destroy evidence |
| P4-A02–A04: map/unmap/protect work correctly | No descriptor code exists; descriptor format is an Implementation Choice per task book §8 | P4-owned descriptor representation and walker for 4 KiB granules ([03](03-code-contracts-stage2-core.md) §2) | map/unmap/protect are descriptor mutations plus invalidation; no upstream provides them | W02 | P4-V02 mutation evidence; P4-V07/V08 negatives |
| P4-A05: query reports mapping state for diagnostics | Nothing exists | Query API returning mapping/flags/HPA per IPA ([03 §3.5](03-code-contracts-stage2-core.md)) | W06 needs IPA/access context for fault diagnosis | W02; consumed by P4-W06 | P4-V02 query evidence |
| P4-A06: installation for Guest execution | `VTTBR_EL2`/`VTCR_EL2`/VMID policy undocumented anywhere | Activation contract + VMID allocator ([03 §3.6](03-code-contracts-stage2-core.md); [02 §4](02-architecture-and-state.md)) | Guest translation cannot start without an installed context | W02; consumed by P4-W04 | P4-V02 activation evidence; P4-V04 via W04 |
| P4-A07: current-path consistency, no stale translations | Nothing exists; P3 transport is planned-only | BBM ordering + current-path invalidation operation with a transport seam ([02 §6](02-architecture-and-state.md); [03 §3.7](03-code-contracts-stage2-core.md)) | P4-V07/V08 passing explicitly requires that required capability does not rely on stale translations | W02; transport seam reserved for P3 delivery | P4-V07/V08 negative evidence |
| Mapper never violates Host memory safety | P2 protected-range exclusion is planned, not delivered (W01 R08/R09) | Assumed-contract dependency: allocator never returns protected pages; W02 adds no second allocator | Stage-2 tables and mapped frames must come from managed pages only | P2-W04 plan (assumed); failure boundary in [01 §2](01-scope-and-foundations.md) | W01 row status; P2-V06 evidence when delivered |
| Multi-pCPU future is not precluded | P3 SMP is planned, not delivered (W01 R13–R17) | Single-writer P4 concurrency model with an explicit invalidation seam ([02 §6](02-architecture-and-state.md)) | A design that hardcodes uniprocessor translation state would contradict ADR-015 | W02 model; P3 transport (assumed) | design review; P4 does not prove shootdown |

No row above requires a decision this design is not authorized to make; the
descriptor representation, VMID policy, and locking model are Implementation
Choices the task book assigns to this design (task book §8), resolved in
[01 §4](01-scope-and-foundations.md).

## Resolved design decisions and their authority

Summarized here; full rationale and authority citations in
[01 §4](01-scope-and-foundations.md):

1. **Granule and level coverage:** 4 KiB granule, Stage-2 starting at a single
   root level sized for the temporary Guest IPA range; block descriptors are
   Reserved. Task book §1 (large-page optimization out of scope).
2. **Descriptor representation:** P4-owned 64-bit Stage-2 descriptor encoding
   in the Arch layer, exact field layout per the Arm A-profile architecture
   (verified against the pinned architecture reference at implementation).
   ADR-006/ADR-041; Coding Guidelines (hardware boundary rules).
3. **Address-space object:** one `GuestAddressSpace` owns its root table and
   VMID; pages back it exclusively; destroy unmaps, invalidates, and returns
   pages. ADR-018; ADR §4 object model.
4. **VMID policy:** minimal non-recycling allocator sized from the P1
   capability inventory; VMID exhaustion is a permanent, reported failure for
   that space. Task book §1 (VMID-recycling optimization out of scope).
5. **Concurrency:** P4 uses a single-writer model serialized by an
   address-space lock; mutations are legal while active (with BBM); cross-pCPU
   shootdown is behind an invalidation seam that P4 implements as
   current-path-only. Task book §1 Reserved (must not preclude multi-pCPU).
6. **Ownership trust boundary:** the mapper trusts that mapped frames come
   from the P2 allocator (which never returns protected pages) and does not
   re-derive platform memory facts; Guest RAM construction and its ownership
   hand-over belong to [P4-W03](../p4-w03-guest-memory-image/README.md).
   Task book §2; P2-W10 handoff scope.
7. **Placement:** Stage-2 mechanism belongs to the AArch64 Arch layer, with
   only arch-neutral vocabulary (IPA/HPA newtypes from the P0 address-type
   baseline, mapping-flag value object) visible to Core. Crate placement
   follows the P0 workspace design (assumed, W01 R18/R19); this design fixes
   logical placement, not file paths. ADR §13, ADR-041–ADR-045.

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): the ledger
   above, the assumed P2/P3/P0/P1 contracts with failure boundaries, and the
   resolved decisions (D1–D10).
2. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   module map, object ownership, the address-space and VMID state machines,
   and the Stage-2 ordering/barrier rules that the contracts rely on.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md),
   loading [03-code-contracts-stage2-core.md](03-code-contracts-stage2-core.md)
   for each contract you are implementing; every `unsafe` boundary there
   carries mandatory SAFETY-justification requirements.
4. Record validation in
   `../../verification/p4-w02-stage2-address-space-verification.md` and
   implementation facts in `../p4-w02-stage2-address-space-record.md` only
   when work starts; neither this design nor the records may claim W02
   complete.

## Explicitly excluded interfaces

Not designed or authorized by W02: the final generic `ExitReason` software
API (P4-W06/W07 own the P4 classification surfaces; the ADR-level API comes
with the Arch exit contract), any hypercall or management ABI (P5), the
`MemoryObject`/`MemoryRegion` ADR-level object model (P2-ACR-01, unresolved),
a public mapper API for later stages (W09 records only implemented facts),
block-descriptor and huge-page paths, cross-pCPU shootdown policy, dirty/access
tracking, IOMMU/SMMU interactions, and any QEMU-conditional behavior in Core.

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W04** receives the activation boundary ([03 §3.6](03-code-contracts-stage2-core.md)):
  the world-switch installs/switches contexts only through it, and the
  register-ownership contract keeps `VTTBR_EL2`/`VTCR_EL2`/VMID host-owned.
- **P4-W06** receives the query and negative-access basis ([03 §3.5](03-code-contracts-stage2-core.md)):
  translation/permission fault diagnosis reads mapping state from this API,
  not from its own descriptor walk.
- **P4-W03** receives the mapping-input contract: frames and flags it may pass
  and the ownership hand-over expectation (designed jointly in
  [01 §3](01-scope-and-foundations.md) assumption M4).
- **P4-W07/W08** receive the observable Stage-2 mutation events (event names
  routed through the P0 trace namespace, W01 A8) used for repeatability and
  automation evidence.
- **P4-W09** receives only factual capability/limitation records; no frozen
  mapper API.
- **P5** inherits the evidenced Stage-2 isolation boundary only; it owns all
  capability/handle semantics on top.
