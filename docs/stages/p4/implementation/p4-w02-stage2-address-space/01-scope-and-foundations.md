# P4-W02 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W02 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-A01–A07)

- Address-space lifecycle: create (allocate root table, assign VMID), quiescent
  mutation, activation, deactivation on destroy, page return.
- Map: Guest IPA page range → HPA frame range with permissions and memory type.
- Unmap: remove mappings with defined not-mapped semantics.
- Protect: change read/write/execute permission and memory type of existing
  mappings.
- Query: report mapped/unmapped, flags, and translating HPA for one IPA.
- Activate/deactivate: install the space as the current Stage-2 context for
  Guest execution on the current pCPU.
- Invalidate: make prior mutations visible to the current execution path
  (current pCPU, current VMID) before Guest (re-)entry.

### Reserved (must not be precluded; not implemented in P4)

- Block/large descriptors and level collapse/expand (re-entry: a later
  performance design; P4 keeps descriptor-type handling extensible in the
  writer/walker).
- Cross-pCPU invalidation over the P3 transport (re-entry: first multi-vCPU or
  multi-pCPU Guest design; the seam is [02 §6.3](02-architecture-and-state.md)).
- Non-identity Guest IPA placement (re-entry: P8 machine-layout design; P4's
  temporary identity placement is a P4-W03 test fact).
- A published, stable mapper API for later stages (re-entry: P5+ management
  surfaces; W09 records only implemented facts).
- VMID recycling/generation reuse (re-entry: a later VMID lifecycle design).

### Out of Scope

Dirty/access tracking, COW, balloon/reclaim, overcommit, IOMMU/SMMU, Guest
SMP (multi-vCPU execution), scheduler interactions, a multi-VM machine ABI,
the ADR-level `MemoryObject`/`MemoryRegion` object model (P2-ACR-01, remains
`ADR Required` and unresolved; this design defines only the stage-local P4
handles it needs and claims no ADR-level object system), and Host Stage-1
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
| M3 | Allocation accounting supports an ownership-state extension (managed → Guest/Stage-2 use) without P4 defining the ADR object model | P2-W10 plan, W01 R12; P2-ACR-01 open | W03 marks Guest RAM pages; W02 records table-page use; both release through the allocator | if no extension point exists, W02 records the gap (W01 gap list) and stages page release as free-on-destroy only, without claiming ownership accounting |
| M4 | Guest RAM frames are handed to Stage-2 by [P4-W03](../p4-w03-guest-memory-image/README.md) as allocator-derived ranges; the mapper is not the ownership authority | P4-W03 design, P4 task book P4-B01 | map() preconditions; no loader logic in W02 | if W03 changes the hand-over shape, the map() input contract is renegotiated in both designs, not patched |
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

- **P2-ACR-01 (ADR Required, unresolved):** the ADR roadmap bullet for P2 and
  the P2 task book conflict over minimal `MemoryObject`/`MemoryRegion`. W02's
  mapping model needs "a set of owned pages behind an address-space mapping."
  To avoid resolving the ADR question locally, W02 models this as the
  stage-local type `MappingGrant` (a P4 test-stage transfer record produced by
  W03's Guest RAM, consumed by `map`), explicitly not named or shaped as the
  ADR `MemoryObject`. This keeps P2-ACR-01 visible (README excluded
  interfaces; [05 §3](05-validation-and-handoff.md)) and keeps the future
  object model free to supersede the P4 type without an ABI break, because
  P4 asserts no API stability for it.
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
| D3 | `GuestAddressSpace` owns root table + VMID + mapping ledger; construction and destruction are explicit; no global registry of address spaces in P4 | ADR-018 core-object requirement with P4-scope minimalism; avoids a Manager-style global | Plan Agent guardrails (no implicit registry owners) |
| D4 | VMID: minimal allocator handing distinct VMIDs from the architectural width (read from the P1 capability inventory); exhaustion is a permanent create-failure; no recycling in P4 | task book excludes VMID-recycling optimization; uniqueness avoids cross-VM TLB aliasing risks even though P4 runs one Guest | task book §1; ADR-018; AArch64 VMID semantics (pinned spec revision) |
| D5 | Single-writer concurrency: one address-space lock serializes map/unmap/protect/destroy/activate for the space; lock is never held while the Guest runs; guest execution relies on quiescent state plus invalidation, not on held locks | bounds lock scopes (Coding Guidelines); the guest cannot take host locks; keeps the world-switch free of lock coupling | P3-W06 semantics (assumed); Coding Guidelines lock rules |
| D6 | Mutations while active are legal and use break-before-make plus current-path invalidation; P4 defines no deferred/async invalidation | P4-A07 requires current-path consistency without stale translations; async schemes are final-policy scope (P7+) | task book P4-A07; ADR §6 |
| D7 | Invalidation seam: W02 defines an internal invalidation backend (enum of operations: all-current, range-current) implemented as current-pCPU-only in P4; the P3 transport plugs into the same seam later without changing mapper call sites | task book Reserved: transport available for later shootdown; must not preclude multi-pCPU | task book §1 Reserved; P3-W08 scope boundary |
| D8 | `MappingGrant` stage-local transfer type (see §3) carries frame range + intended flags from W03 to `map`; W02 rejects zero-length, misaligned, or overflow inputs as VM-facing errors | keeps P2-ACR-01 unresolved; untrusted/checked-input discipline at the host-side API too | ADR §19 (checked arithmetic); P2-ACR-01 record |
| D9 | Query results are read-only snapshots of the mapping ledger, not a live hardware walk; a hardware-walk primitive is Reserved | diagnostic read must not race descriptor mutation (D5 lock); hardware walk adds AT-instruction unsafe for zero P4 need | Coding Guidelines (bounded unsafe); W06 diagnostic need (plan) |
| D10 | All W02 errors are VM-facing recoverable values (`Stage2Error`); internal invariant violations (for example, allocator returning a protected page, if ever detectable) escalate through the P0 failure-classification contract as fatal | guest-caused faults must never panic the host; invariant breaks must never be swallowed | ADR §19; P0-W14 (assumed, W01 R19) |
