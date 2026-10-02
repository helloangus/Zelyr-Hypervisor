# P4-W02 Scope, Foundations, and Resolved Decisions

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [P4-W02 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-A01–A07)

- Address-space lifecycle: create (allocate root table, assign VMID), quiescent
  mutation, activation, explicit detachment before destroy, page return.
- Map: Guest IPA page range → HPA frame range with permissions and memory type.
- Unmap: remove mappings with defined not-mapped semantics.
- Protect: change read/write/execute permission (memory-type changes are rejected in this profile) of existing
  mappings.
- Query: report mapped/unmapped, flags, and translating HPA for one IPA.
- Activate/deactivate: install the space as the current Stage-2 context for
  Guest execution on the current pCPU.
- Invalidate: make prior mutations visible to the current execution path
  (current pCPU, current VMID) before Guest (re-)entry.

### Reserved in base W02 (extension belongs to W10)

- Block/large descriptors and level collapse/expand (re-entry: a later
  performance design; P4 keeps descriptor-type handling extensible in the
  writer/walker).
- Cross-pCPU invalidation over the P3 transport (re-entry: first multi-vCPU or
  multi-pCPU Guest design; the seam is [02 §6.3](02-architecture-and-state.md)).
- General machine-layout policy (P8); fixed temporary IPA already translates
  independently allocated HPA under the revised W03 contract.
- A published, stable mapper API for later stages (re-entry: P5+ management
  surfaces; W09 records only implemented facts).
- VMID recycling/generation reuse (re-entry: a later VMID lifecycle design).

### Out of Scope

Dirty/access tracking, COW, balloon/reclaim, overcommit, IOMMU/SMMU, Guest
SMP (multi-vCPU execution), scheduler interactions, a multi-VM machine ABI,
implementation of the common object model (owned by P2-W12; W02 consumes it),
and Host Stage-1
changes (P1-W08 contract stays intact; Stage-2 never edits Host mappings).

## 2. Assumed upstream contracts and failure boundaries

W02 consumes the following as assumed contracts per the
[P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry review. If
an upstream delivers differently than assumed, the W02 assumption becomes a
recorded conflict (`Architecture Change Request` or `ADR Required` per W01 §4)
and the affected W02 step stops; W02 never patches around an upstream change
silently.

| ID | Assumed contract | Source (plan path) | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | Address/identifier newtypes exist for HPA and Guest-physical (IPA/GPA) with checked arithmetic; P4 adds no raw-`usize` address APIs | P0-W15 plan (`docs/stages/p0/plans/p0-w15-address-identifier-type-safety.md`), W01 R19 | typed `HostPhysAddr`, `GuestPhysAddr` values usable in Arch code | if no typed addresses exist, W02 blocks (its API shape is a Coding-Guidelines requirement, not a free choice) |
| M2 | Page allocator: allocate/free of 4 KiB pages from non-protected RAM, explicit OOM, accounting | P2-W04 plan, W01 R09 | root/table frames and mapped frames come only from this allocator; protected pages are unreachable by construction | if protected-range exclusion is not demonstrable, Guest-isolation evidence (P4-V07) is unsound; W02 stops before Guest-exposure steps |
| M3 | W12 retains W04 backing and every pending/live/revoking region | P2-W12 detailed design; ADR-062 owner option A | W02 holds leases and table object capabilities, never duplicate backing ownership | Missing producer blocks real mapping; no plain-alloc/free degradation |
| M4 | W03 supplies non-Copy W12 Guest regions bound to SpaceId; console uses a separate exact-window authority | W03 revised design and W12 | map validates identity, range, permissions and memory type | Raw physical range is not authority; reject foreign/stale input before mutation |
| M5 | EL2 exception environment can take and diagnose synchronous exceptions while Guest runs; EL2 baseline registers are host-owned | P1-W04/W05 plans, W01 R02/R03 | Stage-2 faults produce a routed, capturable exception; the EL1 baseline restore contract exists for W04 | if routing/baseline assumptions break, Guest isolation evidence is unsound; W02 stops at mutation-level evidence (P4-V02 only) |
| M6 | CPU-local context is discoverable via the P3-declared per-CPU mechanism; secondary pCPUs exist but P4 runs one Guest path | P3-W04/W14 plans, W01 R14 | invalidation seam targets "current pCPU" through that mechanism; no global current-CPU state | if the discovery mechanism differs, only the seam's implementation adapts; the seam's interface is W02-owned |
| M7 | Logging/trace baseline and event namespace exist | P0-W12/W13 plans, W01 R20/A8 | Stage-2 events emit through it with stage-local names | if absent, events degrade to the P0 logging fallback; never ad-hoc prints in Arch code |
| M8 | Toolchain/target baseline builds AArch64 bare-metal Rust with the pinned stable toolchain | P0-W02/W03 plans, W01 R18 | `unsafe`-bounded sysreg/descriptor code compiles as designed | toolchain changes follow P0-W02 contract rules; W02 does not adopt unstable features |

Assumptions M1, M2, M5, M6, M8 are entry conditions for the *corresponding
acceptance evidence*, not all for coding start: descriptor/locker/query logic
with unit-level host tests can proceed against typed-address and allocator
interfaces defined as assumed signatures, but P4-V07/V08 isolation claims are
blocked until M2/M5 evidence exists. The workflow ([04](04-implementation-workflow.md))
marks where each boundary applies.

## 3. Authority analysis for contested areas

- **P2-ACR-01:** Owner selected ADR-062 option A. P2-W12 owns the common
  object/region foundation; W02 is its Stage-2 adapter. Formal ADR integration
  and production evidence are pending; no competing P4 backing owner is allowed.
- **VMID semantics:** ADR-018 requires Stage-2 as core object; the ADR does
  not fix VMID allocation. VMID is an Arch-layer mechanism detail (ADR-041
  layering); allocation policy for P4 is stage-local freedom (D4), with
  recycling explicitly out of scope by the task book.
- **Descriptor bit-level encoding:** owned by the architecture specification,
  not by this design. This design fixes the *representation strategy*
  (P4-owned accessors over 64-bit descriptors, no raw struct reinterpretation
  per Coding Guidelines ABI rules); the bit layout follows the pinned Arm ARM
  revision and is verified at implementation ([03 §2](03-code-contracts-stage2-core.md)).
  QEMU-vs-architecture divergences are Specification Investigation items
  (W01 A7), recorded, not encoded as Core semantics.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | 4 KiB granule only; root level chosen at create time from the requested IPA span; no block descriptors in P4 | smallest reviewable correct subset; task book excludes large-page optimization; extensibility kept in descriptor type handling | task book §1 Out of scope / §8 Implementation Choice |
| D2 | Descriptor representation = P4-owned accessor type over `u64` words (field get/set with reserved-bit handling), never a `#[repr(C)]` struct cast or raw memory reinterpretation | Coding Guidelines: external hardware formats need explicit representation; reserved bits must be handled; unsafe minimized | ADR-006; Coding Guidelines (hardware boundary and ABI rules) |
| D3 | Space owns table-object control capabilities and Guest region leases; W12 owns actual backing handles; no implicit Drop reclamation | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
| D4 | Use validated 8-bit profile, reserve 0, consume VMIDs 1–255 monotonically; never recycle, including failed creation after mint | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
| D5 | Short state lock reserves/commits Frozen transactions; hardware work and waits occur outside the lock; zero Guest execution leases required | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
| D6 | Installed-but-idle mutation uses BBM and resident-history invalidation; running Guest mutation returns Busy | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
| D7 | Base backend proves local VMID retirement and restores the installed context; W10 extends it to every resident CPU | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
| D8 | MappingGrant wraps a non-Copy W12 region bound to SpaceId, or the separate exact base-console authority; no raw HPA import | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
| D9 | Query results are read-only snapshots of the mapping ledger, not a live hardware walk; a hardware-walk primitive is Reserved | diagnostic read must not race descriptor mutation (D5 lock); hardware walk adds AT-instruction unsafe for zero P4 need | Coding Guidelines (bounded unsafe); W06 diagnostic need (plan) |
| D10 | Pre-publication rejection is recoverable; unknown publication/completion retains resources and may require fail-stop, never unconditional unchanged-on-error | Preserves ownership, completion and bounded stage scope | W12, revised W02 contracts and W10 extension |
