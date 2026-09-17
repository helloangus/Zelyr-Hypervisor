# Zelyr Hypervisor — P2 Stage Task Book v0.1

**Stage ID:** P2
**Stage name:** Platform Discovery & Host Memory Foundation
**Status:** Defined planning baseline; implementation and validation are not claimed
**Owner/change context:** P2 planning reorganization from the root-source task book
**Supersedes:** the root-level `Rust Type-1 Hypervisor — P2 Stage Task Book v0.1.md` source layout
**Governing documents:** [Architecture baseline ADR](../../adr/adr-000-architecture-baseline-v0.1.md), [documentation index](../../README.md), and [Plan Agent guide](../../development/plan-agent-guidelines.md)

## 1. Purpose and stage boundary

P2 turns the stable AArch64 EL2 environment supplied by P1 into a system that
can derive trustworthy host-platform facts from boot information and safely
manage host physical memory. Its observable stage outcome is a normalized
platform description, a normalized boot physical-memory map, dynamic physical
page and small-object allocation, and diagnostics/validation for those
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

**P2-ACR-01 — ADR Required.** The ADR's roadmap P2 bullet says to define
minimal `MemoryObject`/`MemoryRegion` structures, while the source P2 task
book says P2 must not design P4's object system and only reserve extension
space. These sources conflict under the documented authority order. P2 plans
therefore neither define nor implement those objects. An ADR clarification or
superseding stage requirement is required before that work is scheduled.

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
```

W06 consumes stable results from W02–W05. W07 consumes W01–W02 and supplies
fixtures/checker evidence to W08. W08 validates host-side properties of W01–W05
and W07. W09 adds the reference-platform integration evidence after W06/W08.
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
| P2-V13 | Stage governance review | all required packages map to one plan, all validation IDs have conditions, links/dependencies resolve, and P2-ACR-01 remains visible. |

Planning this matrix does not supply evidence. Commands, logs, environments and
actual outcomes belong in [verification/](verification/), and implementation
traceability belongs in [implementation/](implementation/).

## 8. Exit criteria and handoff

P2 may be marked complete only when evidence exists for P2-V01 through P2-V13
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
and allocation availability only; it does not authorize AP startup. The P4
handoff exposes host RAM topology, protected-range exclusion, allocation/free
and ownership-accounting extension foundation only; it does not authorize
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
