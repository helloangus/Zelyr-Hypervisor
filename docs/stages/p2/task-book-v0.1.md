# Zelyr Hypervisor — P2 Stage Task Book v0.2

Chinese readers can use the [Chinese edition](task-book-v0.1.zh-CN.md).

**Stage ID:** P2
**Stage name:** Platform Discovery & Host Memory Foundation
**Status:** Defined planning baseline; implementation and validation are not claimed
**Version:** v0.2
**Owner/change context:** P2-ACR-02 planning amendment, 2026-10-02; root-source reorganization retained
**Supersedes:** the root-level `Rust Type-1 Hypervisor — P2 Stage Task Book v0.2.md` source layout
**Governing documents:** [Architecture baseline ADR](../../adr/adr-000-architecture-baseline-v0.1.md), [documentation index](../../README.md), and [Plan Agent guide](../../development/plan-agent-guidelines.md)

## 1. Purpose and stage boundary

P2 turns the stable AArch64 EL2 environment supplied by P1 into a system that
can derive trustworthy host-platform facts from boot information and safely
manage host physical memory. Its observable stage outcome is a normalized
platform description, a normalized boot physical-memory map, dynamic physical
page and small-object allocation, explicit ordinary-frame Host access, and diagnostics/validation for those
foundations.

P2 does not run a Guest. P3 consumes its CPU inventory, boot-CPU relation,
PSCI facts, platform capabilities, and allocation foundation for SMP work. P4
consumes its host-RAM topology, reserved-region exclusion, page allocation and
ownership-accounting foundation for Stage-2 and Validation Guest work.

### Required

- validate the supplied AArch64 DTB before interpreting it, including the
  location/lifetime of the active DTB and malformed-input diagnostics;
- interpret the DT semantics required by P2 and tolerate safely ignorable
  unknown nodes;
- discover CPU inventory, RAM, reserved memory, GICv3 description, timer,
  PSCI, `/chosen`/console facts, and a capability summary;
- convert raw discovery results to a common, capability-driven platform result
  so Core need not reparse DTB or branch on a board name;
- construct and normalize a boot physical-memory map that excludes the
  hypervisor, active DTB, firmware/DT reserved areas, boot artifacts,
  allocator metadata, and other identified reserved ranges;
- establish page allocation/free, OOM, multi-RAM-region, alignment, debugging
  checks, accounting, and a small-object dynamic-allocation foundation;
- plan the minimal common backing-ownership and mapping-view foundation (W12, §11);
- make the actual normalized result inspectable, provide offline DTB checking,
  and plan host/QEMU negative and integration validation; and
- document the P3/P4 handoff, constraints, known limitations, and evidence
  required before a P2 completion claim.

### Reserved

- extension of page ownership into VM-owned, shared, DMA-pinned, COW,
  ballooned/reclaimed and related later-stage states;
- a later choice to copy and release the original DTB, contiguous-allocation
  interfaces, allocator strategy, metadata placement, and allocator handoff;
- additional platform capability types, device discovery, and support tiers;
- Orange Pi 3B/RK3566 as an offline DTB compatibility fixture only.

### Out of scope

- secondary-CPU startup, PSCI `CPU_ON`, SMP locks, cross-CPU TLB shootdown,
  or any P3 execution mechanism;
- VM/vCPU objects, VMID allocation, GuestAddressSpace, Guest memory mapping,
  Stage-2 translation, Guest EL1 entry, or Validation Guest execution;
- GIC initialization/IRQ injection/vGIC, timer virtualization, PCI discovery
  or assignment, SMMU/IOMMU, virtio, Control Domain, VM configuration,
  ballooning, overcommit, swap/paging, snapshot/COW, or memory hotplug; and
- Orange Pi 3B EL2 runtime bring-up or board-specific Core behavior.

P2 must not freeze a parser implementation, crate/module tree, Rust type/API,
allocator algorithm, metadata layout, lock/concurrency design, or inspection
command layout. Those choices require a later approved detailed design.

## 2. Inputs, constraints, and state

P0 must supply its documented workspace/toolchain, logging and panic baseline,
host-test and QEMU-runner entry points, address/error conventions, unsafe
governance, and ADR/documentation workflow. P1 must demonstrably supply a
stable QEMU `virt` AArch64 EL2 Rust entry, early console and exception
diagnostics, relevant CPU/EL2 registers, supplied DTB or equivalent boot
information, known hypervisor-image physical range, and a stable host execution
environment. Missing P0/P1 inputs are upstream defects; P2 must not work
around them.

The ADR requires AArch64 DTB discovery to become `PlatformInfo`, Core to avoid
board/SoC/QEMU dependencies, and consumers to query capabilities rather than
platform names (ADR-041 through ADR-045). It also requires controlled unsafe,
checked arithmetic for untrusted boundaries, EL2 dynamic allocation,
ownership-oriented memory evolution, diagnostics, and layered validation
(ADR-006, ADR-014, ADR-018, ADR-048, ADR-049).

At stage completion, the intended capability chain is:

```text
boot DTB -> validated intake -> normalized platform facts -> normalized boot map
         -> safe allocatable-page pool -> dynamic allocation and diagnostics
```

This is a required outcome map, not a claim that the chain exists today.

## 3. Architecture-change record

**P2-ACR-01 — owner direction selected; formal ADR integration and delivery pending.**
The original conflict was the ADR's P2 minimum MemoryObject/MemoryRegion versus
P2's exclusion of P4's object system. On 2026-10-02 the owner selected option A
of [ADR-062](../../adr/adr-062-p2-minimal-memory-object-foundation.md): P2 supplies
the minimal common ownership/view foundation, P4 supplies Guest integration.
[W12](plans/p2-w12-minimal-memory-objects.md) and §11 record authorized planning.
The ADR remains Proposed pending formal integration. W12/W11 detailed-design
approval is separately recorded in their design entries; runtime admission does not follow. Historical contrary scope statements are superseded
for this planning direction only; Guest objects/policy remain outside P2.

## 4. Work-package map

| Package | Required outcome and source requirements | Primary validation |
|---|---|---|
| P2-W01 | Validated boot-platform-description intake: DTB presence, range, overlap, structural validation, core DT encoding semantics, and unknown-node handling (P2-A01–A04). | P2-V01, P2-V02 |
| P2-W02 | Capability-driven discovery and normalized platform facts for CPU, RAM, reserved memory, GIC, timer, PSCI, and console/chosen (P2-B01–B08, P2-C01–C03). | P2-V03, P2-V04 |
| P2-W03 | Normalized boot physical-memory map and reserved/ownership foundation, including checked ranges and conflict handling (P2-D01–D08, P2-G01–G03). | P2-V05 |
| P2-W04 | Safe host physical-page allocation/free, alignment, multi-region support, OOM, debug checks, and accounting (P2-E01–E08). | P2-V06 |
| P2-W05 | Dynamic small-object allocation, explicit failure/release behavior, and stress-validation basis (P2-F01–F04). | P2-V07 |
| P2-W06 | Inspection derived from the actual platform/memory/allocation result (P2-H01–H04). | P2-V08 |
| P2-W07 | Offline DTB compatibility checking and QEMU/RK3566 fixture coverage (P2-I01–I05). | P2-V09 |
| P2-W08 | Host-side malformed-DTB, map-conflict, allocator, and determinism regression coverage (P2-J01–J05). | P2-V10 |
| P2-W09 | QEMU `virt` integration coverage across CPU and RAM configurations, accounting and repeated boot (P2-K01–K05). | P2-V11 |
| P2-W10 | P2 platform/ownership contract, P3/P4 handoff, known limitations, and stage-gate evidence map (P2-L01–L05). | P2-V12, P2-V13 |
| P2-W11 | Explicit boot-CPU Host access to ordinary allocated frames, with checked coverage, attributes, lifetime, cleanup and rollback (P2-M01–M05). | P2-V14, P2-V10, P2-V11 |
| P2-W12 | Minimal common backing ownership and mapping-view lifetime foundation (P2-N01–N06); no Guest/translation policy. | P2-V15, P2-V10, P2-V12, P2-V13 |

Each mapped package has exactly one plan in [plans/](plans/README.md). Package
plans describe only bounded outcomes, sequencing, acceptance and handoff; they
are not detailed designs, implementation records, verification evidence, or
completion claims.

## 5. Dependency and execution map

```text
W01 -> W02 -> W03 -> W04 -> W05
  |      |      |      |      |
  |      +----> W06 <---------+
  |      +----> W07 -> W08 ---+
  +----------------------> W08 -> W09 -> W10
                 W06 ---------^
W04 -> W12 -> W11 -> W08/W09/W10
W03/P1 access -> W11
```

W06 consumes stable results from W02–W05. W07 consumes W01–W02 and supplies
fixtures/checker evidence to W08. W12 consumes W04 ownership without hardware mapping; W11 consumes W03/W04/W12
and P1 access evidence.
W08 validates host-side properties of W01–W05, W07, W11 and W12. W09 adds the reference-platform integration evidence after W06/W08.
W10 records the consumer contract and evidence map after all preceding packages.
The dependency graph is directed and contains no cycle.

## 6. Requirement-to-validation traceability

| Requirement groups | Validation IDs | Passing condition |
|---|---|---|
| P2-A01–A04 | P2-V01, P2-V02, P2-V10 | input availability/range errors and malformed structures are rejected with a local diagnostic; valid required encodings are interpreted without unsafe out-of-range behavior. |
| P2-B01–B08, P2-C01–C03 | P2-V03, P2-V04, P2-V09, P2-V11 | reference and fixture inputs produce stable CPU/RAM/reservation/GIC/timer/PSCI/console facts and capability states without Core board-name choices. |
| P2-D01–D08, P2-G01–G03 | P2-V05, P2-V10, P2-V11 | map is sorted/normalized, checked for overflow/conflicts, and excludes every protected range while reserving future ownership extension. |
| P2-E01–E08 | P2-V06, P2-V10, P2-V11 | pages allocate/free across supported regions; OOM and invalid operations are explicit; protected pages are never returned and accounting remains consistent. |
| P2-F01–F04 | P2-V07, P2-V10 | small allocations have explicit failure/release behavior and stress runs preserve allocation/accounting invariants. |
| P2-H01–H04 | P2-V08, P2-V11 | inspection reports the active normalized result and allocator statistics, not an independent hard-coded view. |
| P2-I01–I05 | P2-V09, P2-V10 | offline checker reports P2 readiness for QEMU and RK3566 fixtures, warning for unsupported unrelated devices without claiming runtime board support. |
| P2-J01–J05 | P2-V10 | listed negative, exhaustion, stress, conflict, and determinism cases have reproducible host-side evidence. |
| P2-K01–K05 | P2-V11 | QEMU `virt` results remain stable over required CPU/RAM variants and repeated boot; map accounting is within the documented expected domain. |
| P2-L01–L05 | P2-V12, P2-V13 | consumer contract, known limitations, evidence locations, and every stage gate are reviewable without an implementation-completion assertion. |
| P2-M01–M05 | P2-V14, P2-V10, P2-V11 | Checked allocated-frame Host coverage, attributes, lifetime, cleanup and rollback; see §10 for the added requirements and evidence boundary. |

## 7. Stage validation matrix

| ID | Evidence sought | Success condition |
|---|---|---|
| P2-V01 | Boot-input boundary review/test | DTB absence, location, length, host-access range, and dangerous hypervisor overlap yield an explicit outcome. |
| P2-V02 | DTB structural/encoding tests | magic, total size, blocks, offsets, property sizes, cells and references stay in validated bounds; damaged inputs do not cause uncontrolled failure. |
| P2-V03 | Discovery-result tests | required CPU, RAM, reservation, GIC, timer, PSCI and chosen/console facts are collected when declared. |
| P2-V04 | Normalization/portability review | consumers use normalized facts/capabilities; missing, unsupported, absent and usable states remain distinguishable; Core has no board-name branch. |
| P2-V05 | Boot-map tests | RAM and all protected/owned regions normalize with sorted, checked, non-ambiguous treatment of holes and conflicts. |
| P2-V06 | Page-allocation tests | allocation/free/alignment/multi-region/OOM/debug/accounting behavior is explicit and returns no protected page. |
| P2-V07 | Small-allocation stress evidence | repeated allocation/free, exhaustion and recovery maintain stated allocation invariants. |
| P2-V08 | Inspection review | platform summary, map dump and allocator statistics derive from the active normalized state. |
| P2-V09 | Offline-checker fixture evidence | QEMU `virt` and Orange Pi 3B/RK3566 DTBs receive objective readiness reports; this does not assert Orange Pi EL2 runtime support. |
| P2-V10 | Negative/property regression | malformed DTB, map conflicts/overflow, allocator exhaustion/stress and repeated-discovery determinism meet their package acceptance criteria. |
| P2-V11 | QEMU integration evidence | required QEMU CPU-count/RAM-size configurations and repeated boots meet discovery, map-accounting and inspection expectations. |
| P2-V12 | P3/P4 handoff review | consumers can find supported inputs, limitations and evidence locations without inferring P2's implementation design. |
| P2-V13 | Stage governance review | all required packages map to one plan, all validation IDs have conditions, links/dependencies resolve, and P2-ACR-01/02 remain visible. |
| P2-V14 | Allocated-frame Host mapping validation | Coverage, attributes, ownership/lifetime, rollback and unmap/invalidation-before-reuse meet §10; host-model and target/QEMU evidence are separate. |

Planning this matrix does not supply evidence. Commands, logs, environments and
actual outcomes belong in [verification/](verification/), and implementation
traceability belongs in [implementation/](implementation/).

## 8. Exit criteria and handoff

P2 may be marked complete only when evidence exists for P2-V01 through P2-V14
and all of these conditions hold:

1. QEMU `virt` reliably yields the required platform facts and a normalized
   representation that downstream Core consumers do not obtain by reparsing DTB.
2. Every protected physical range is excluded from allocation for every valid
   allocation/free sequence; this is the hard P2 safety gate.
3. Page and small-object allocation have explicit normal, exhaustion and
   release behavior, with observable accounting.
4. The platform summary, map dump and allocator statistics reflect the active
   normalized state.
5. Host-side negative regression and QEMU integration evidence exists, and the
   offline QEMU/RK3566 fixture check demonstrates semantic portability without
   claiming board runtime support.
6. P3 and P4 can locate the handoff contract and evidence, including known
   limitations and P2-ACR-01.

The P3 handoff exposes CPU inventory, boot-CPU relation, PSCI/capability facts
and allocation availability plus the explicit W11 Host access contract; it does not authorize AP startup. The P4
handoff exposes host RAM topology, protected-range exclusion, allocation/free
and ownership-accounting extension foundation plus W11 Host access; it does not authorize
Stage-2, VM, or Guest work. Known limitations must explicitly include at least
PCI discovery, SMMU/IOMMU, GIC initialization, AP bring-up, and Orange Pi 3B
runtime support as later work.

## 9. Completion review

Before a completion claim, review the validation record, preserved boundary,
dependency map, and handoff with these questions:

- Can consumers distinguish trustworthy discovered facts from unsupported or
  unavailable capability hints?
- Does any allocation path demonstrably exclude all protected pages?
- Are malformed boot inputs and range arithmetic handled as untrusted input?
- Has any P3/P4 or board-specific runtime mechanism entered P2?
- Has P2-ACR-01 been resolved by an authorized source, or retained as an open
  architecture issue?

Any negative answer prevents P2 completion.

## 10. P2-ACR-02 planning amendment (2026-10-02)

The owner selected P2 as the producer of ordinary allocated-frame Host access.
[P2-W11](plans/p2-w11-host-allocated-frame-mapping.md) is the bounded producer;
its detailed design is owner-approved on 2026-10-02; implementation and evidence remain outstanding.
This amendment extends the stage outcome and the maps above. P2-ACR-01's subsequent owner direction and remaining process/delivery gates are
recorded in §11; P1's historical completion scope is unchanged.



| Requirement | Required outcome |
|---|---|
| P2-M01 | Checked HPA/HVA coverage for requested allocated frames; reject protected or outside-domain requests. |
| P2-M02 | Writable, non-executable access and reviewed memory attributes/alias rules. |
| P2-M03 | Allocation ownership and borrow lifetime remain valid for every Host access. |
| P2-M04 | Unmap and required invalidation precede free/reuse; partial failure rolls back transactionally. |
| P2-M05 | Declare the boot-CPU boundary, table-storage/bootstrap prerequisites and consumer-specific evidence for P2-HOST-MAP. |

| Validation | Evidence sought and passing condition |
|---|---|
| P2-V14 | Host-model and target/QEMU records separately demonstrate allocated-frame access for representative P3 local/stack and P4 table/image-buffer uses; checked coverage/attributes/lifetime, protected/outside-domain/invalid-lifetime rejection, alias safety, partial-map rollback and unmap/invalidation before reuse pass. Fixtures do not establish P3/P4 runtime execution or SMP/hardware safety. |

The extended dependency graph is W03/W04 and P1 bootstrap/access evidence →
W11 → W08/W09/W10. W08 adds host negative/lifecycle/rollback coverage under
V10/V14; W09 adds target integration under V11/V14; W10 requires the W11
contract/evidence in V12/V13. W11 executes before these consumers despite its
numeric suffix. W04-MAP remains the independently available metadata mapping
prerequisite for W04; it cannot depend on W11. W05-MAP remains independent;
reuse of W11 needs a reviewed integration and is not a planning dependency.
Thus no allocator/mapping initialization cycle is introduced.

The extended stage exit requires V01–V14 and delivered P2-HOST-MAP evidence.
P3/P4 handoffs include explicit Host coverage, attributes, lifetime and cleanup
limits; physical allocation alone never promises Host access. P3 owns later
SMP synchronization and cross-CPU invalidation. Completion review must check
mapping lifetime/rollback evidence and both ACR records. No permanent identity
map, all-RAM aperture, new ABI, Guest execution, or detailed API is prescribed.

## 11. Owner-authorized minimal memory foundation

The owner selected ADR-062 option A on 2026-10-02 ("可以"). P2-W12 owns the
minimal common MemoryObject/MemoryRegion foundation; P4 owns Guest adapters.
This supplement records planning authorization. W12 detailed-design approval
is separately recorded in its design entry; formal ADR integration and execution
evidence remain distinct. No accepted baseline text is rewritten.

| Requirement | Required outcome |
|---|---|
| P2-N01 | One allocator-derived ordinary-RAM backing owner; actual W04 handle retained, no duplicate frame authority. |
| P2-N02 | Stable object/view identity; bounded capacity, checked exhaustion and stale-identity refusal. |
| P2-N03 | Bounded object-range view with checked permissions/type intent; no architecture descriptors or Guest policy. |
| P2-N04 | In-flight/live/revoking views retain backing; no release until owning backend proves completed cleanup. |
| P2-N05 | Failed construction/map/cleanup preserves ownership or explicitly retains resources; reuse and accounting remain consistent. |
| P2-N06 | Separate common foundation, Host adapter, Guest adapter and later SMP evidence; no implicit runtime admission. |

| Validation | Evidence sought and passing condition |
|---|---|
| P2-V15 | Host tests/review exercise N01–N06: valid/rejected lifecycle, ranges/rights/type errors, stale identity/capacity, partial failure/rollback, failed cleanup retention, token loss, release/reuse and exact allocation accounting. Adapter reviews link W11/P4 authority. Host models do not prove translated access, Guest execution or SMP safety. |

Execution dependency: W04 ownership → W12 common foundation → W11 lifetime
integration → W08/W09/W10. W12 Host modeling does not depend on W11 hardware;
W04-MAP remains an independent boot-retained metadata prerequisite. W05-MAP
and heap ownership are not silently changed. W08 includes V15 in V10;
W09 checks integrated Host behavior under V11/V14; W10 includes W12 contract,
V15 and ADR process status in V12/V13. Extended P2 closure now requires V01–V15,
formal decision integration and evidence for every required capability.

P3 receives only evidenced boot-CPU ownership/lifetime facts and owns any SMP
extension. P4-W02/W03/W10 must reconcile backing/view authority with W12 before
runtime admission; named stage-local handles alone do not establish that link.
External backing, sharing/donation, DMA, COW, hotplug and Guest mechanisms stay
reserved/out of scope. Storage/API/identity representation are Implementation
Choices for approved design; actual translation completion remains backend-owned.
