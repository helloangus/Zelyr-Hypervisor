# Zelyr Hypervisor — P3 Stage Task Book v0.1

**Stage ID:** P3
**Stage name:** Host SMP and per-CPU foundation
**Status:** Implementation-planning baseline; work and evidence are not claimed
**Scope:** AArch64 host-physical-CPU SMP foundation on QEMU `virt`
**Owner/change context:** P3 planning set derived from the P3 stage source task book
**Upstream:** P0 engineering baseline; P1 AArch64 EL2 runtime; P2 platform discovery and memory infrastructure
**Downstream:** P4 Stage-2 and first Rust Validation Guest
**Supersedes:** Root-level `Rust Type-1 Hypervisor — P3 Stage Task Book v0.1.md`, relocated into the governed stage layout by this document

## 1. Purpose and authoritative constraints

P3 removes host-side single-CPU assumptions before guest virtualization begins. It establishes a diagnosable, concurrent EL2 execution environment in which physical CPUs can be discovered, brought online, identified locally, coordinated, notified, and audited safely. P3 concerns Host SMP only. Physical CPUs remain distinct from future guest vCPUs.

The Architecture Baseline governs this task book. ADR-002 keeps the first implementation AArch64-first; ADR-003 names QEMU `virt` the reference platform and Orange Pi 3B a later hardware target; ADR-006 constrains low-level Rust and `unsafe`; ADR-015 requires early SMP; ADR-032 keeps GICv3 the interrupt baseline; ADR-041 through ADR-045 require platform layering and capability-driven behavior; ADR-048 and ADR-049 require structured telemetry and layered validation. The ADR P3 roadmap requires secondary-CPU start, independent per-CPU state, boot rendezvous, basic synchronization, cross-CPU notification, a TLB mailbox framework, and 2/4/8-CPU QEMU validation.

This book does not define crate boundaries, Rust APIs, data layouts, lock algorithms, PSCI/GIC register sequences, or a detailed CPU-state implementation. Those decisions require an approved P3 detailed design. A prerequisite-contract or frozen-interface conflict is an Architecture Change Request or ADR Required issue, never a silent redesign.

## 2. Entry conditions and scope classification

### Required predecessor contracts

P3 implementation may start only when reviewed predecessor deliverables and evidence provide:

| Source | Required input |
|---|---|
| P0 | repeatable AArch64/QEMU entry, quality and CI rules, unsafe/dependency governance, diagnostics/telemetry governance, portability and stage-workflow rules |
| P1 | stable Non-secure EL2 entry and Rust runtime, exception vectors and fatal diagnostics, CPU-feature inventory, known EL2 baseline, EL2 Stage-1 execution environment |
| P2 | DTB-derived `PlatformInfo` expressing CPU topology and platform capabilities, CPU-start input, boot memory map/reserved ranges, safe physical allocation and heap/object-allocation baseline |

`docs/stages/p2/` supplies the P2 planning set, but it does not supply implementation or verification evidence. The P2 inputs above remain conditions for implementation, not completion claims. A P3 design must inspect the actual P2 contracts, unresolved architecture items, and evidence before code work.

### Required work

- Classify present, possible, online, and failed physical CPUs without assuming contiguous hardware identities or equating present with online.
- Start expected secondary CPUs into EL2, report success, failure, and timeout, and make physical-CPU lifecycle state diagnosable.
- Provide independent stack, logical identity, local runtime/exception/interrupt state, notification state, TLB-request reception state, and telemetry basis for every online CPU.
- Coordinate one-time global initialization, per-CPU initialization, secondary readiness, and a testable SMP-ready condition.
- Establish synchronization semantics sufficient for shared allocator/registry/state access and interrupt-sensitive contexts, including lock-order and atomic-ordering rules.
- Establish a minimal cross-CPU event primitive and a future TLB-invalidation transport with targeting, acknowledgement/completion, invalid/offline behavior, concurrency expectations, and diagnosable failure.
- Audit P0–P2 mutable infrastructure for SMP safety; make CPU lifecycle and cross-CPU activity observable; require repeatable stress/failure and QEMU 1/2/4/8-CPU regression evidence.
- Deliver the P4-facing SMP contract and P3 bring-up, per-CPU, synchronization, and test-matrix documentation.

### Reserved for later stages

- P4 chooses Stage-2 address-space/VMID/IPA semantics and the concrete TLB invalidation operation; P3 supplies only transport and completion foundations.
- P4 may use CPU-local storage for current-vCPU, VMID/TLB-generation cache, and later VM work after its detailed design.
- P6 owns virtual GIC, List Register, guest PPI/SPI, and maintenance-interrupt virtualization. P7 owns vCPU scheduling and affinity policy. P8 owns guest secondary CPUs, virtual PSCI/SGI, and Linux guest SMP. P15 owns full RK3566/Orange Pi 3B bring-up and hardware SMP regression.

### Out of scope

P3 does not create a VM, guest RAM, guest loader, Guest EL1 execution, Stage-2 page tables, VMIDs, Stage-2 faults, guest TLB shootdown semantics, vCPU scheduling, Guest SMP, complete GIC virtualization, runtime host CPU hotplug, a general message queue/RPC layer, or Orange Pi 3B BSP adaptation. It must not encode QEMU or board-name branches in generic Core, permanently bind runtime Core behavior to CPU0, or treat host SMP primitives as guest-facing interfaces.

## 3. Work-package map

| ID | Required outcome | Source task group | Primary validation |
|---|---|---|---|
| P3-W01 | Platform CPU topology is classified into stable host physical-CPU inputs and logical identities. | T01 | P3-V01 |
| P3-W02 | Expected secondary CPUs reach a diagnosable EL2 bring-up result. | T02 | P3-V02 |
| P3-W03 | Physical CPU lifecycle and online eligibility have explicit, observable authority. | T03 | P3-V03 |
| P3-W04 | Every online physical CPU has independent execution and CPU-local state foundations. | T04 | P3-V04 |
| P3-W05 | Boot-time global/per-CPU ordering and SMP-ready rendezvous are defined and verifiable. | T05 | P3-V05 |
| P3-W06 | Shared-state synchronization semantics, constraints, and contention acceptance are established. | T06 | P3-V06 |
| P3-W07 | A minimal, safe physical-CPU notification primitive is available. | T07 | P3-V07 |
| P3-W08 | A future TLB-invalidation transport can target, acknowledge, and diagnose physical CPUs. | T08 | P3-V08 |
| P3-W09 | Exception and interrupt diagnostic foundations are correct per CPU. | T09 | P3-V09 |
| P3-W10 | P0–P2 infrastructure has an explicit SMP-safety classification and remediation boundary. | T10 | P3-V10 |
| P3-W11 | SMP lifecycle, events, contention, and timing have governed observable outputs. | T11 | P3-V11 |
| P3-W12 | Concurrency, failure, and exceptional paths have repeatable stress-test evidence requirements. | T12 | P3-V12 |
| P3-W13 | QEMU 1/2/4/8-CPU regression has a repeatable matrix and cold-boot coverage. | T13 | P3-V13 |
| P3-W14 | P4 receives an explicit, bounded Host SMP contract. | T14 | P3-V14 |
| P3-W15 | P3 documentation, traceability, and closure review package are complete. | T15 | P3-V15 |

See [plans/README.md](plans/README.md) for the acyclic dependency and consumer map. Each work package has exactly one plan in `plans/`.

## 4. Requirement-to-validation traceability

| Requirement group | Validation IDs |
|---|---|
| CPU topology, identity, disabled/unavailable CPU handling | P3-V01, P3-V03, P3-V13 |
| Secondary entry, timeout/failure, all expected CPUs online | P3-V02, P3-V03, P3-V13 |
| Per-CPU stacks, local state, exception/interrupt isolation | P3-V04, P3-V09 |
| Boot rendezvous and readiness ordering | P3-V05, P3-V12, P3-V13 |
| Synchronization, atomic/IRQ constraints, shared allocator/registry safety | P3-V06, P3-V10, P3-V12 |
| Cross-CPU events and TLB transport | P3-V07, P3-V08, P3-V12, P3-V13 |
| Audit, lifecycle/event telemetry, diagnostic quality | P3-V09, P3-V10, P3-V11 |
| Stress, failed-secondary/offline-target behavior, repeatability | P3-V12, P3-V13 |
| P4 contract and documentation/closure | P3-V14, P3-V15 |

## 5. Validation matrix

| ID | Evidence sought | Passing condition and limit |
|---|---|---|
| P3-V01 | Platform/topology review across 1/2/4/8 CPUs | Stable hardware-to-logical CPU mapping, boot CPU identification, and exclusion of unavailable CPUs; does not prove hotplug. |
| P3-V02 | Secondary bring-up evidence | Each expected secondary reaches the documented lifecycle result or yields CPU/phase-specific failure or timeout diagnostics. |
| P3-V03 | Lifecycle review and online-set evidence | No CPU is usable before local initialization or twice online; failed CPUs stay outside the online set. |
| P3-V04 | Per-CPU isolation review/test | Online CPUs use distinct execution/local diagnostic state and no implicit global-current-CPU/context remains. |
| P3-V05 | Repeated boot rendezvous evidence | Global initialization is once-only, local initialization is per CPU, and SMP-ready is reached only after defined readiness. |
| P3-V06 | Synchronization review plus contention evidence | Shared-state access obeys documented atomic/lock/IRQ rules and completes without detected corruption or unexplained deadlock. |
| P3-V07 | Targeted, self, concurrent, invalid/offline notification evidence | Arrival/type accounting and target behavior are defined and recoverable; does not prove a general RPC facility. |
| P3-V08 | TLB transport request/acknowledgement evidence | Target/mask handling, acknowledgement, completion, timeout, invalid/offline exclusion, and concurrent requests are diagnosable; no Stage-2 TLBI semantics are asserted. |
| P3-V09 | Exception/fatal-path evidence on boot and secondary CPUs | CPU identity and local diagnostic context remain correct during supported exceptional paths. |
| P3-V10 | P0–P2 shared-state audit record | Every reviewed mutable state is classified immutable-after-boot, CPU-local, atomic, lock-protected, or boot-only, with unresolved items blocking closure. |
| P3-V11 | Telemetry/log review | Lifecycle, notification, shootdown transport, contention, and boot-synchronization observability include meaningful CPU attribution. |
| P3-V12 | Allocator, notification, atomic, barrier, concurrent-log, and failure-path stress evidence | Repeated stress has explainable accounting and no corruption, permanent hang, or undiagnosed failure in the documented test limits. |
| P3-V13 | QEMU regression matrix and repeated cold boots | 1/2/4/8 CPU matrix meets declared boot/online/event/stress/allocator criteria over repeated cold boots; QEMU does not prove hardware correctness. |
| P3-V14 | P4 consumer review | A P4 planner can rely on current pCPU identity, CPU-local state, shared-state protection, cross-CPU completion, and TLB transport without redesigning P3. |
| P3-V15 | Stage-document review | Required/planned/reserved/out-of-scope boundaries, links, plan coverage, evidence locations, and exit traceability are coherent without a completion assertion. |

## 6. Exit criteria and P4 handoff

P3 can close only with real evidence for P3-V01 through P3-V15 and all of the following:

1. QEMU `virt` boot and Host SMP regression cover 1, 2, 4, and 8 physical CPUs; expected CPUs either become online or yield precise failure diagnostics.
2. Every online CPU has independent execution/local state and CPU-attributed exception diagnostics.
3. Boot rendezvous, synchronization, notification, and TLB transport complete within their documented acceptance/failure behavior; shared allocator and infrastructure audits expose no unresolved single-core assumption.
4. Stress and repeated cold-boot regressions provide the declared evidence; unsupported hardware behavior is not inferred from QEMU results.
5. P3 documentation records CPU discovery, bring-up, lifecycle, state classification, synchronization constraints, test matrix, known limits, and P4 handoff.

The P4 handoff contains reviewed P3 plans, implementation and verification records, CPU topology/identity contract, physical-CPU lifecycle and per-CPU-state contract, boot synchronization and shared-state rules, cross-CPU notification and TLB-transport contract, SMP audit/telemetry contracts, QEMU regression evidence, and unresolved issues. P4 may depend on these foundations but must separately design Stage-2, VM, Guest EL1, and concrete TLB semantics.

## 7. Completion review

The closing review asks: is Host SMP free of undocumented single-core assumptions; are CPU lifecycle and failure authority explicit; does every shared mutable state have a declared synchronization classification; can diagnostics identify CPU, state, and phase; do stress and QEMU matrix evidence meet their limits; and can P4 start Stage-2 work without absorbing Host SMP redesign? A negative answer blocks P3 closure. Planned documents alone are not evidence of completion.
