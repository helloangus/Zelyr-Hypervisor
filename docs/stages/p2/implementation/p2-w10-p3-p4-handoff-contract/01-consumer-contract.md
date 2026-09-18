# P2-W10 Consumer Contract Content

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W10 detailed design](README.md).  
This file fixes the required content of the published record
(`p2-w10-p3-p4-handoff-contract-record.md`). Sections below are the
record's skeleton and substance; the implementing agent authors the record
from it, filling statuses only from real records.

## 1. Record header

Status, scope, owner/change context, supersedes: none, and the standing
statement: "This record describes P2's intended consumer contracts and
evidence map. It is not a completion claim; P2 status is whatever the
evidence map in [02](02-evidence-map-and-gates.md) truthfully shows."
Governing documents listed by pointer (ADR baseline, P2 task book, P2 plan
index).

## 2. Deliverable catalog

Each entry: producer (package + authoritative design path), one-paragraph
semantic contract, validation IDs, consumers (named plan IDs). The
paragraphs summarize; the linked designs remain the only normative
sources.

| ID | Deliverable | Producer | Validation | Consumers |
|---|---|---|---|---|
| D-01 | Normalized platform facts: `PlatformInfo` — CPU inventory with boot-CPU relation, RAM banks, `/reserved-memory` ranges, boot artifacts, GIC/timer/PSCI/console facts; five-state `FactState` model (`NotDiscovered/Absent/Unsupported/Unusable/Usable`); determinism guarantee; no re-derivation from DTB by consumers | W02 → [design](../p2-w02-platform-discovery-normalization/README.md) | P2-V03, P2-V04 | P3 (p3-w01, p3-w02), P4 (p4-w01, p4-w02), W03/W06/W07 |
| D-02 | Capability query surface: `PlatformCapabilities` with honest `NotDiscovered` for PCI/SMMU/ACPI; consumers decide by capability, never platform name | W02 → same | P2-V04 | P3 (p3-w01, p3-w02), P4 (p4-w01) |
| D-03 | Boot memory map: sealed immutable `BootMemoryMap`; RAM partition sorted/disjoint; protected ranges with `ProtectedSourceId` ledger and `RegionClass` (default-protected extension rule); `MapSummary` equation `ram = allocatable + protected` exact | W03 → [design](../p2-w03-boot-memory-map-ownership/README.md) | P2-V05 | P4 (p4-w02, p4-w03), W04/W06/W09 |
| D-04 | Page allocation: typed order/count allocation and exact free; hard gate — no protected frame returnable for any valid sequence; `AllocationStats` accounting; **single-owner, lock-free boot-phase concurrency boundary; P3 owns all locking/per-CPU design** | W04 → [design](../p2-w04-physical-page-allocation/README.md) | P2-V06 | P3 (p3-w04, p3-w06), P4 (p4-w02, p4-w03), W05/W06 |
| D-05 | Dynamic small allocation: fixed size-class slab heap over W04; typed fallible API + `GlobalAlloc` adapter (target permitting); heap budget containment; **single-core ownership boundary; P3 owns locking**; W01–W03 boot arrays documented temporary | W05 → [design](../p2-w05-dynamic-small-allocation/README.md) | P2-V07 | P3 (p3-w04, p3-w06), P4 (p4-w02–p4-w05), W06 |
| D-06 | Inspection: read-only `InspectionReport` over W02–W05 live state with consistency checks C1–C4 and deterministic render; a review input, **not a control API** | W06 → [design](../p2-w06-platform-memory-inspection/README.md) | P2-V08 | P3/P4 reviewers (via this record), W08/W09 |
| D-07 | Offline compatibility corpus: fixture format, QEMU `virt` and RK3566 fixtures with binding expectations; readiness verdicts carry the fixed no-support disclaimer | W07 → [design](../p2-w07-offline-dtb-compatibility/README.md) | P2-V09 | Platform planners (P15 direction), W08, W09 |
| D-08 | Host robustness regression: scenario matrices S1xx–S5xx, seeded reproducibility, evidence-row schema at `docs/stages/p2/verification/p2-w08-host-robustness-regression-verification.md` | W08 → [design](../p2-w08-host-robustness-regression/README.md) | P2-V10 | W09, P3 (p3-w12 pattern), P4 (p4-w08 pattern) |
| D-09 | QEMU integration evidence: configuration matrix `virt-cpu{1,2,4}-memB`, `-mem512`, `-mem2g-hs`; per-run record schema; **QEMU results are reference-platform evidence, never hardware proof** | W09 → [design](../p2-w09-qemu-integration-regression/README.md) | P2-V11 | P3 (p3-w13), P4 (p4-w08), P2 completion review |

## 3. P3 handoff statement (record section)

Supported inputs to P3 — and nothing more:

- CPU inventory (count, per-CPU enable status, enable-method as recorded)
  and the boot-CPU relation (D-01);
- PSCI version class and method; capability facts (D-01/D-02);
- allocation availability: page allocator and heap ready with stated
  concurrency boundary (D-04/D-05);
- reference-platform boot facts and the runner usage baseline (D-09).

**Non-authorization (verbatim in substance):** this handoff does not
authorize AP startup, PSCI `CPU_ON` execution, SMP locks, or any P3
execution mechanism. P3 owns its own designs (p3-w01…p3-w15); P2 supplies
inputs and contracts only.

## 4. P4 handoff statement (record section)

Supported inputs to P4 — and nothing more:

- host RAM topology: banks, protected ranges and their sources,
  allocatable spans, accounting totals (D-03);
- protected-range exclusion as a structural guarantee of the allocation
  domain, with the hard-gate evidence chain (D-03/D-04/D-08);
- allocation/free and ownership-accounting extension foundation:
  `RegionClass`/`ProtectedSourceId` extension surface designed to carry
  later ownership states (D-03/D-04; ADR-018 direction);
- dynamic allocation for VM/vCPU object designs (D-05).

**Non-authorization (verbatim in substance):** this handoff does not
authorize Stage-2 translation, GuestAddressSpace, VM/vCPU objects, guest
memory mapping, EL1 entry, or any Guest execution. `MemoryObject`/
`MemoryRegion` are **not** delivered (P2-ACR-01); P4 must not assume them.

## 5. Known limitations (record section)

Mandatory (task book §8): PCI discovery; SMMU/IOMMU; GIC initialization;
AP bring-up; Orange Pi 3B runtime support — all later-stage work.

Design residuals recorded by the producing designs, restated as
limitations:

- DTB consumed in place; no copy/release policy (W01/W03 Reserved);
- singleton facts use first-in-DT-order; cell widths 1–2 only; PSCI 0.1
  unsupported; capacities fixed (`MAX_*` stage constants) with fatal
  capacity handling (W01/W02);
- protected-source under-declaration inherited from firmware descriptions
  is detected only as far as described ranges conflict (W03 residual);
- order-mismatched free detection has a documented limit; >`MAX_ORDER`
  contiguous requests are typed failures (W04 residuals);
- no realloc/grow-in-place; alignment above page size rejected (W05);
- inspection is a snapshot at composition time, not continuous monitoring
  (W06);
- offline readiness is descriptive only — no board-support claim of any
  kind (W07);
- host regression bounds, not exhausts, the input space; QEMU evidence
  does not extend to hardware (W08/W09).

**P2-ACR-01 (unresolved, quoted):** the ADR roadmap bullet and the source
task book conflict on P2 defining minimal `MemoryObject`/`MemoryRegion`
structures; P2 defines and implements neither. Resolution requires an ADR
clarification or superseding stage requirement (task book §3). The record
must restate this verbatim-in-substance and mark it **ADR Required**.

## 6. Change-note rule

The record carries a change-note section; any post-authoring edit adds a
dated note describing what changed and why. Silent edits void the record's
audit value for P3/P4 stage reviews.
