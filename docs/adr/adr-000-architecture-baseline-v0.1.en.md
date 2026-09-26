# Zelyr Hypervisor Architecture Decision Record v0.1

**Translation status:** Current
**Translation source:** [Authoritative Chinese source](adr-000-architecture-baseline-v0.1.md)
**Source blob:** `3dcf8766c2f2290efaea227a5b71a37a8f194e8d`
**Authority:** This is an English reading translation. The accepted Chinese source retains authority under the ADR history rules.

**Subtitle:** Architecture decision baseline, complete layered architecture, and staged development roadmap
**Date:** 2026-09-16
**Status:** Architecture Baseline / Draft for Implementation
**Scope:** AArch64-first Type-1 Hypervisor; QEMU virt + Orange Pi 3B (RK3566); later x86_64 extension

> This document is the project's current architecture baseline. Except for items marked Pending, settled decisions are default constraints for subsequent implementation and review. Changing a settled decision requires a new ADR with a supersession relationship.

## 1. Architecture principles

1. **Research first, general virtualization in the long term:** Experimental mechanisms and observability may remain, but all core abstractions must be able to evolve toward a general-purpose Hypervisor.
2. **Separate mechanism from policy:** EL2 supplies security, resource, and virtualization mechanisms. Control Domain / Service Domain should handle complex policy where possible, though performance-critical services may reside in EL2.
3. **Modular microkernel-style code organization:** “Microkernel” primarily means strict modularity, low coupling, and clear boundaries; it does not require moving every externalizable function out of EL2.
4. **Shared semantics without forced hardware representation:** AArch64 and x86_64 should share VM/vCPU/memory/interrupt/device semantics where possible. VMCS, HCR_EL2, VTTBR_EL2, and similar mechanisms remain in architecture backends.
5. **Reserve dynamic capability in the data model:** Initial allocation may be static, but VM, vCPU, memory, device, and IRQ objects should from the outset be designed for creation, destruction, migration, and rebinding.
6. **Guests are untrusted by default:** Ordinary Guests are potentially malicious. Hypercall, MMIO, virtio descriptor, Stage-2 fault, and IRQ injection paths all need boundary checks.
7. **Portability before Board special cases:** Core does not depend on Board; Arch does not depend on SoC. Board quirks belong in BSP/Quirk. DTB/ACPI platform information becomes a common PlatformInfo.
8. **Validation Guest before Linux:** First verify EL2 mechanisms with a tiny Rust EL1 bare-metal Guest, then introduce Linux to reduce debugging variables.
9. **Configuration is a first-class architecture object:** Model Build, Boot, Bootstrap Domain, Control Domain, VM, compatible machine, and security policy separately. EL2 does not parse complex YAML/XML/JSON.
10. **Observability is not an afterthought:** Reserve structured telemetry for trace, exit counts, IRQ latency, Stage-2 faults, scheduler, and memory ownership from early stages.

## 2. Architecture Decision Register

| ID | State | Topic | Decision | Main consequence |
|---|---|---|---|---|
| ADR-001 | Settled | Project position | A research Type-1 Hypervisor is the primary goal, evolving toward general virtualization. | Preserve experimental mechanisms, tracing, and replaceable policy; a minimal product is not the final boundary. |
| ADR-002 | Settled | Primary ISA | Implement only AArch64 initially; add x86_64 after architecture stabilizes. | Avoid simultaneous dual-ISA bring-up while keeping AArch64 detail out of core abstractions. |
| ADR-003 | Settled | Primary platforms | QEMU virt is the reference/CI platform; Orange Pi 3B (RK3566) is the first physical platform. | Use QEMU for deterministic tests and Orange Pi for real cache/TLB/IRQ/firmware/PCIe behavior. |
| ADR-004 | Settled | Meaning of microkernel | Start with modular microkernel-style code architecture, without requiring a minimal EL2 TCB. | Performance-critical functions may stay in EL2; mature third-party drivers and complex services favor Service Domain. |
| ADR-005 | Settled | Service deployment | Use a hybrid “modular EL2 core + optional Service Domain” model. | A service interface can have an EL2 or Service-VM implementation for performance/isolation research. |
| ADR-006 | Settled | Language boundary | Rust first; necessary assembly, controlled unsafe, and mature crate dependencies are allowed. | Concentrate unsafe in arch/HAL/low-level structures and maintain an audit inventory. |
| ADR-007 | Settled | Guest threat model | An ordinary Guest may be malicious by default. | Validate Guest-supplied addresses, lengths, indices, state transitions, descriptors, and hypercalls. |
| ADR-008 | Settled | Boot firmware | Do not implement EL3 firmware; rely on TF-A/U-Boot and other existing firmware to enter Non-secure EL2. | Handle/proxy necessary PSCI/SMC without taking over Secure World. |
| ADR-009 | Settled | First Guest | The first VM to enter EL1 is a Rust bare-metal Validation Guest. | Independently verify HVC, Stage-2, timer, IRQ, MMIO, SMP, and related mechanisms. |
| ADR-010 | Settled | Linux Guest | Introduce a customized Linux after the Validation Guest is stable, then expand to general Linux. | Custom DTB, kernel arguments, and device set are acceptable early on. |
| ADR-011 | Settled | Control Domain evolution | A Linux Control Domain is allowed early; later develop a native Rust Control Domain/Service Runtime. | Reuse Linux filesystem, network, storage, and device drivers early. |
| ADR-012 | Settled | Management delegation | Control Domain is not the only manageable entity; other VMs may receive limited management ability. | Authorization uses Hypervisor capabilities; identity authentication remains in management domains, not EL2 passwords. |
| ADR-013 | Settled | Capability model | Resource and management operations use object capability/handle + rights + generation. | Support least privilege, delegation, attenuation, and future revocation; avoid `vm_id==0` privilege. |
| ADR-014 | Settled | EL2 dynamic memory | EL2 permits dynamic page allocation and heap/slab/object allocators. | Dynamic VM/vCPU/memory cannot rest on an entirely static kernel allocation model. |
| ADR-015 | Settled | SMP timing | SMP is an early foundational capability. | Avoid single-core assumptions; test QEMU multicore and four-core RK3566 early. |
| ADR-016 | Settled | CPU scheduling | Begin with static vCPU↔pCPU binding and evolve toward preemptive M:N overcommit. | Ultimately support dedicated and shared CPU VMs, affinity/pinning, and multiple scheduling classes. |
| ADR-017 | Long-term reserved | Real-time scheduling | Add partition/RT scheduling and latency instrumentation in the long term. | Do not block general scheduler bring-up, but do not lock scheduler API to one policy. |
| ADR-018 | Settled | Memory model | Stage-2 address space is a core object, supporting dynamic map/unmap/protect, ownership, and TLB shootdown. | Reserve page metadata for balloon, COW, snapshot, migration, and shared memory. |
| ADR-019 | Long-term reserved | Memory overcommit | Allow overcommit; favor balloon/reclamation plus hard limits early, without an EL2 swap daemon. | For future transparent paging, put policy/backing store in management/memory service and supply revoke/fault/remap primitives from EL2. |
| ADR-020 | Long-term reserved | Snapshot/live migration | Support snapshot, dirty tracking, pre-copy/stop-and-copy, and device-state migration. | Version machine model, device state, vCPU state, and configuration schema early. |
| ADR-021 | Settled | Guest firmware/boot | Evolve ELF bare metal → Linux Image+DTB+initramfs → UEFI/EDK2. | Avoid early firmware complexity while retaining later Windows/BSD compatibility paths. |
| ADR-022 | Settled | Guest EL | Initially Guest kernels run at EL1; do not expose virtual EL2. | Simplify vCPU/Stage-2/GIC; implement nested virtualization separately later. |
| ADR-023 | Long-term reserved | Nested virtualization | Implement virtual EL2, nested Stage-2, and nested interrupt virtualization later. | Do not affect first-generation VM ABI; avoid hard-coding “Guest is always EL1” into upper arch API. |
| ADR-024 | Settled | Virtual ARM platform | Define a versioned Generic ARM64 VM machine independent of Host SoC. | Guest must not see a “virtual RK3566”; migration/image compatibility needs a stable virtual hardware ABI. |
| ADR-025 | Settled | Guest description | DTB first; ACPI later. | Linux/bare metal use DTB first; add ACPI with UEFI/Windows. |
| ADR-026 | Settled | Virtio position | Virtio is the primary paravirtual I/O ABI. | Prioritize virtio-mmio, then virtio-pci; follow the current stable Virtio spec with feature negotiation. |
| ADR-027 | Settled | Device model | Mix virtio, emulation, mediated devices, and direct passthrough. | Choose by security and performance; do not force all devices into one model. |
| ADR-028 | Settled | Complex drivers | Complex/mature third-party drivers such as NVMe, Ethernet, USB, and filesystems generally stay out of EL2. | Favor Linux/Rust Service Domain; retain a few performance-critical or early-test backends in EL2. |
| ADR-029 | Settled | Filesystem boundary | EL2 does not implement a general filesystem; the bootloader loads Boot Package into memory. | Control/Storage Domain owns real filesystems, images, qcow2/raw, and network storage. |
| ADR-030 | Settled | DMA isolation classes | Without IOMMU, a software path cannot be called secure direct passthrough. | Distinguish paravirtualized, mediated, trusted direct, and IOMMU-isolated direct. |
| ADR-031 | Settled | IOMMU/SMMU | Abstract IOMMU capability; verify secure passthrough with QEMU virt SMMUv3. | Enable physical boards by capability; upper layers query capabilities, not platform names. |
| ADR-032 | Settled | Interrupt baseline | The AArch64 mainline starts from GICv3. | SGI/PPI/SPI + vIRQ first, then LR/maintenance, vGIC distributor/redistributor, and ITS/MSI/LPI. |
| ADR-033 | Long-term reserved | GICv2 | Treat it as a compatibility backend for older ARM platforms, not the mainline. | Keep InterruptController trait open to different backends. |
| ADR-034 | Settled | IPC primitives | Native Hypervisor primitives converge on Capability, Endpoint, Notification, and SharedRegion. | Small Endpoint messages for control, SharedRegion + Notification for data; avoid two unrelated IPC systems. |
| ADR-035 | Settled | Fault isolation | Service/Driver Domain failure must not break Hypervisor or other domains. | IPC, shared mappings, resource revoke, and backend reset must support recovery. |
| ADR-036 | Settled | Management interface | Use a versioned native management ABI internally; add a libvirt driver/adapter externally later. | Do not let libvirt restrict research functions; map only expressible lifecycle/device capabilities. |
| ADR-037 | Settled | Configuration layers | Separate Build / Hypervisor Boot / Bootstrap Domain / Control Domain / VM / Policy configuration. | EL2 does not parse complex general formats; Control Domain normalizes YAML/XML/API to an internal Spec. |
| ADR-038 | Settled | Bootstrap architecture | Hypervisor Boot Manifest creates Control Domain and grants initial capabilities. | Control Domain cannot create itself; later multiple bootstrap/service domains are possible. |
| ADR-039 | Settled | Desired/actual state | Control Domain eventually uses a reconciler separating desired state from actual Hypervisor state. | Supports hotplug, failure recovery, persistence, and orchestration. |
| ADR-040 | Settled | Versioning | Version `schema_version`, `machine_version`, and `management_abi_version` independently. | Supports snapshot, migration, long-term Guest ABI, and tool compatibility. |
| ADR-041 | Settled | Platform adaptation layers | Separate Arch / SoC / Board / Firmware / Discovery / Driver / Quirk. | Compose existing drivers/descriptions for new platforms without changing Core where possible. |
| ADR-042 | Settled | Platform discovery | AArch64 mainly uses DTB; later x86_64 mainly uses ACPI+PCI; convert both to PlatformInfo. | Core depends only on the common platform intermediate form. |
| ADR-043 | Settled | Board special cases | Core must not branch on a specific Board; keep special cases in BSP/Quirk. | Preserve long-term multiplatform maintainability. |
| ADR-044 | Settled | Platform capability query | Upper layers query PlatformCapabilities rather than QEMU/RK3566/PC names. | Allow runtime differences in GIC, IOMMU, PCI, MSI, hotplug, etc. |
| ADR-045 | Settled | Platform support tiers | Define Reference/Tier-1/Experimental support tiers. | Bound multi-board maintenance and identify CI/regression responsibility. |
| ADR-046 | Settled | Workspace | Use Cargo workspace and multiple crates from day one, without premature fine-grained splitting. | Split driver, SoC, platform, and service crates after boundaries stabilize. |
| ADR-047 | Settled | Feature/Profile | Support minimal/research/embedded/general/secure/full profiles; feature means binary capability. | Do not misuse Cargo feature for runtime policy such as VM count or RAM. |
| ADR-048 | Settled | Telemetry | Structured tracing/metrics are first-class capabilities. | Cover at least VM-exit, Stage-2 fault, TLB, IRQ, scheduler, virtio, lock contention, and memory ownership. |
| ADR-049 | Settled | Validation strategy | Unit + host-side + QEMU integration + guest self-test + Linux boot regression + fuzz/property + unsafe audit. | Critical state machines/page ownership may also receive model checking/formal verification. |
| ADR-050 | Rejected | General filesystem in EL2 | Do not make ext4/FAT/qcow2/VFS and similar general filesystem stacks Core infrastructure. | Avoid growth of the EL2 TCB/driver stack. |
| ADR-051 | Rejected | Single “privileged VM ID=0” | A fixed VM ID must not grant all management authority. | Use capability + policy for multiple management domains and least privilege. |
| ADR-052 | Rejected | Core logic by Board name | Core business logic must not choose paths by `platform==QEMU` / OrangePi. | Use interfaces, PlatformInfo, capability, and quirk. |
| ADR-053 | Rejected | Full legacy PC emulation from day one | Do not invest in an i440fx/Q35-like legacy PC device model early on AArch64. | Prioritize standard ARM virtual platform + virtio; add x86_64 compatibility as needed later. |
| ADR-054 | Pending | Formal project name | Temporarily use RustHV / rust-type1-hypervisor. | Naming does not affect ABI design; freeze before public repository/protocol. |
| ADR-055 | Pending | First Linux Control Domain distribution/rootfs | Could use minimal Linux + initramfs/BusyBox or a custom distribution. | Decide before Storage/Network service bring-up. |
| ADR-056 | Pending | Native Management ABI encoding | Candidates: fixed binary structures + TLV, Cap’n Proto-like schema, custom little-endian wire format. | Must be parsable in no_std, versioned, and clearly boundary-checked; JSON is not an EL2 ABI. |
| ADR-057 | Pending | Default general scheduler | Candidates: round-robin/weighted fair/credit-like; initially only a correct preemptive runnable scheduler is needed. | Freeze scheduler trait/accounting first, then benchmark default policy. |
| ADR-058 | Pending | Default scope of EL2 virtio backend | Experimental console/block/net backends may remain early; long-term default enablement awaits benchmarks. | Service location is replaceable; v0.1 mandates no performance conclusion. |
| ADR-059 | Long-term reserved | Secure boot/signing | Boot Manifest, Hypervisor, Control Domain, and VM images may gain a signature/measurement chain. | Design together with actual firmware/TPM/TEE environment. |
| ADR-060 | Long-term reserved | Confidential computing/TEE | Reserve TrustZone/Realm/other confidential VM research. | Outside the current non-secure EL2 mainline. |

## 3. Overall system architecture

```text
Firmware / Bootloader (TF-A / U-Boot / UEFI)
                  |
                  v
+---------------------------------------------------------------+
|                         EL2 Hypervisor                         |
|  +----------------------+   +-------------------------------+ |
|  | Architecture Core    |   | Hypervisor Services           | |
|  | VM / vCPU / S2 MMU   |   | optional EL2 virtio backend  | |
|  | Scheduler / IRQ      |   | selected fast-path services   | |
|  | Capability / IPC     |   +-------------------------------+ |
|  | Resource / Telemetry |                                     |
|  +----------+-----------+                                     |
+-------------|-------------------------------------------------+
              | Native Management ABI / Capability / IPC
       +------+----------------------+-------------------+
       v                             v                   v
+--------------+             +--------------+      +-----------+
| Control      |             | Driver/      |      | Normal VM |
| Domain       |             | Service VM   |      | Linux/... |
| Linux first  |             | optional     |      +-----------+
| Rust later   |             +--------------+
+------+-------+
       |
       +-- config / storage / network / libvirt / policy
```

### 3.1 Layers

```text
Management / Orchestration
Control Domain / Service Runtime
Native ABI + IPC + Capability
Hypervisor Policy-Independent Core
Arch-independent VM / CPU / Memory / IRQ / Device Models
Architecture Backends (AArch64, x86_64)
Platform Discovery (DTB / ACPI -> PlatformInfo)
SoC + Generic Drivers + Board BSP + Quirks
Firmware / Hardware
```

## 4. Core object model

| Object | Responsibility |
|---|---|
| Hypervisor | Global root object; maintains read-only system capabilities, global registries, and boot phase; must not become a god object for every subsystem. |
| PhysicalCpu | Runtime state of a pCPU: MPIDR/APIC ID, local scheduler, current_vcpu, per-CPU stacks, TLB/IRQ mailbox. |
| Vm | VM lifecycle root; owns machine profile, GuestAddressSpace, vCPU set, virtual devices, and capability namespace. |
| Vcpu | Schedulable execution entity with architecture register context, run state, virtual timer, pending event, and exit accounting. |
| GuestAddressSpace | Mapping object for Guest IPA/GPA → Host PA: AArch64 Stage-2 or x86 EPT/NPT. |
| MemoryObject | Owns memory that can be mapped, shared, or donated, backed by page sets, contiguous regions, or external backing. |
| MemoryRegion | Mapping view in an AddressSpace with permissions, memory type, and dirty/access tracking properties. |
| Capability | Unforgeable object reference with slot/generation/object/right and evolution toward delegation/revocation. |
| Endpoint | Small control-message or request-response endpoint bound to caller capability. |
| Notification | Lightweight asynchronous event/doorbell for scheduler wakeup, virtqueue, event channel, etc. |
| SharedRegion | Explicit shared-memory object for high-throughput IPC, virtqueue, and grant-like buffers. |
| InterruptSource | Common object for physical IRQ/MSI/LPI or software event source. |
| VirtualInterrupt | vCPU/vGIC-facing virtual IRQ with route/priority/pending/active state. |
| VirtualDevice | Guest-visible device state machine implementing MMIO/PCI configuration traps and device snapshot state. |
| DeviceBackend | Data/control backend of a virtual device, implemented in EL2, a local driver, or Service Domain. |
| AssignedDevice | Passed-through or mediated physical device with DMA domain, IRQ route, and reset/isolation policy. |
| PlatformInfo | Unified platform description parsed from DTB/ACPI/firmware. |
| ResourcePool | Allocatable CPU/Memory/IRQ/Device resources and ownership ledger. |
| MachineType | Versioned Guest virtual-hardware ABI, e.g. rusthv-arm-virt-v1. |
| Domain | VM role/security context: normal/control/driver/service/bootstrap; role does not automatically confer rights. |

### 4.1 Lifecycles

- VM: `Created -> Loading -> Ready -> Runnable -> Running -> Paused -> Suspended -> Stopping -> Stopped -> Destroyed -> Faulted`
- vCPU: `Offline -> Runnable -> Running -> Blocked -> Paused -> Stopped -> Faulted`
- Device: `Created -> Configured -> Active -> Quiescing -> Suspended -> Resetting -> Failed -> Detached`

## 5. CPU / vCPU / scheduler system

- Host CPU initialization distinguishes BSP and AP; each pCPU owns per-CPU state and a local runqueue/scheduling context.
- Vcpu is the only scheduled Guest CPU entity; Vm itself does not enter the scheduler.
- Architecture backend saves/restores registers, handles virtualization controls, and enters/exits Guest; Core sees only standard ExitReason.
- Suggested ExitReason members include at least Hypercall, Stage2Fault, Mmio, WfiWfe, SysReg, InterruptWindow, Shutdown, and InternalError.
- Start with static pinned scheduling, then preemptive round-robin; eventually scheduler trait supports weighted/fair/RT/partition.
- CPU affinity and dedicated CPU are policy, not separate VM types.
- Every vCPU switch explicitly handles arch context, virtual timer, vGIC state, later lazy FP/SIMD, and TLB/VMID lifecycle.

## 6. Memory system

- Host memory manager owns HPA; GuestAddressSpace owns IPA/GPA→HPA. Their responsibilities are strictly separate.
- Represent all mappable memory as MemoryObject; MemoryRegion is an address-space view. Sharing/donation uses object ownership transitions.
- Stage-2 API: map/unmap/protect/translate/query_dirty/clear_dirty; implementation must handle break-before-make, TLB invalidation, and concurrency.
- Page metadata records at least owner, ref/share count, mapping flags, pin/DMA state, and dirty/access tracking mode.
- Huge pages are an optimization, not a change to MemoryObject semantics; allocator and Stage-2 mapper both support fallback granularity.
- Balloon/hotplug/snapshot/migration build on common ownership and mapping primitives.
- Early overcommit separates committed and resident amounts. Under physical pressure, prefer balloon/reclaim and do not implement EL2 swap by default.

## 7. Interrupt and time system

- The AArch64 mainline uses GICv3; model physical interrupts, virtual interrupts, and routing policy separately.
- Keep physical IRQ handlers short: acknowledge → find route → record/inject or notify service → EOI.
- Start vIRQ injection with a software pending queue, then use GIC virtualization List Registers and maintenance interrupts incrementally.
- ITS/MSI/LPI belong to PCI/advanced stages and must not pollute basic SGI/PPI/SPI API.
- Virtual timer state belongs to Vcpu; scheduler block/wakeup interacts with WFI and timer deadlines.
- Use a common IPI mailbox for cross-CPU reschedule, TLB shootdown, vIRQ kick, and stop-the-world.

## 8. Device and I/O system

- Separate Guest-facing VirtualDevice from physical/platform Driver.
- VirtualDevice handles Guest ABI and state machine; DeviceBackend handles actual I/O; an explicit backend contract decouples them.
- Virtio is the primary ABI: MMIO before PCI, split ring before packed ring; treat descriptor walking as a security boundary.
- A backend can be EL2 local, Control/Service Domain proxy, hardware mediated, or direct assigned.
- Fixed security classes: paravirtualized/emulated; mediated; trusted direct; IOMMU-isolated direct.
- Device assignment binds ownership, IRQ route, DMA domain, and reset/quiesce policy.
- General filesystems and storage formats stay out of EL2; block backend consumes only block requests.

## 9. Capability / IPC / fault isolation

- Separate authentication and authorization: usernames, passwords, certificates, and RBAC live in Control Domain; EL2 verifies only capabilities.
- Capability v0 uses slot+generation+rights, with future delegation, attenuation, and revoke tree.
- Endpoint handles small control messages; SharedRegion+Notification handles the data plane. Both share one capability/ownership foundation.
- On service crash, revoke mappings, mark device failed, and rebind backend; backend must not hold unreclaimable implicit global pointers.
- Every cross-domain shared-memory region is explicitly created, mapped, authorized, and destroyed.

## 10. Management and configuration system

- Build Configuration controls binary capabilities only. Boot Manifest controls initial Hypervisor/Bootstrap Domain resources. Control Domain manages runtime VmSpec.
- Control Domain owns image paths, filesystems, network, user authentication, policy, libvirt, and configuration persistence.
- Hypervisor owns unforgeable capabilities, actual resource ownership, and mechanism execution; it stores no passwords or RBAC database.
- Boot Manifest creates Control Domain and grants initial management capabilities; role does not replace rights.
- Decompose VmSpec into Machine/Cpu/Memory/Boot/Device/Security. Normalize all external formats to Spec before calling native ABI.
- The long-term manager uses Desired State → Reconciler → Actual State.

## 11. Platform adaptation system

- Architecture: ISA/virtualization mechanism; SoC: chip controllers/glue; Board: wiring/boot differences; Driver: reusable controller; Quirk: controlled special case.
- Convert AArch64 DTB and x86 ACPI/PCI into PlatformInfo; Core must not parse raw platform descriptions.
- PlatformCapabilities expresses SMP/GIC/PCI/IOMMU/MSI/CPU-hotplug capabilities; upper layers select by capability.
- Keep Board BSP thin; boards sharing RK356x reuse SoC/driver implementation.
- Provide hv-platform-inspect and host-side DTB compatibility checker to accelerate new platform bring-up.
- Support tiers include at least Reference (QEMU), Tier-1 (official hardware), and Experimental.

## 12. Security, reliability, and observability

- Untrusted boundaries: Guest registers/input, hypercall buffers, MMIO, virtio descriptors, DMA config, management ABI.
- Memory ownership invariant: a writable physical page cannot belong to mutually untrusted VMs simultaneously without explicit SharedRegion.
- Object handles use generation against UAF/stale references; lengths/offsets use checked arithmetic.
- Panic policy distinguishes fatal Hypervisor invariant from Guest-caused fault. Guest errors should fault/stop only the relevant VM where possible.
- Trace events can be compiled out and filtered at runtime; production/benchmark mode can disable expensive events.
- Core metrics: VM-exit distribution, Stage-2 fault/TLB, IRQ latency, scheduler switch, virtqueue latency, lock contention, memory fragmentation.

## 13. Suggested engineering and crate boundaries

- Initial workspace: hv-core, hv-arch-aarch64, hv-platform, hv-drivers, hv-abi, hypervisor-bin, validation-guest.
- After boundaries stabilize, split hv-vm, hv-memory, hv-scheduler, hv-resource, hv-cap, hv-ipc, hv-device, and driver/SoC/BSP crates.
- No cyclic dependencies. Core depends on traits/data models; binary composition injects concrete arch/platform/driver implementations.
- Prefer newtypes for HPA/IPA/GVA/VMID/VCPU ID rather than mixing raw usize values.
- Distinguish GuestFault, InvalidInput, PermissionDenied, ResourceExhausted, HardwareFailure, and InvariantViolation errors.

## 14. Suggested target code layout

```text
workspace/
├── hypervisor/
│   └── src/main.rs
├── crates/
│   ├── core/          # VM/vCPU/resource/policy-independent orchestration
│   ├── arch-aarch64/  # EL2, Stage-2, sysreg, exception, vCPU entry/exit
│   ├── arch-x86_64/   # later: VMX/SVM/EPT/APIC
│   ├── platform/      # PlatformInfo, discovery, capability
│   ├── drivers/       # GIC, UART, PCI, SMMU, ...
│   ├── abi/           # HVC/native management/wire ABI
│   ├── device/        # virtual device + virtio core
│   ├── telemetry/
│   └── test-support/
├── soc/
│   └── rk356x/
├── boards/
│   ├── qemu-virt/
│   └── orangepi-3b/
├── guests/
│   └── validation-aarch64/
├── control/
│   ├── linux-agent/   # early
│   └── rust-domain/   # later
└── docs/
    ├── adr/
    ├── architecture/
    ├── abi/
    ├── machine-types/
    └── platform/
```

## 15. Detailed development stage tree

### P0 — Repository, specification, and toolchain baseline
**Goal:** Build a sustainable engineering skeleton with a reproducible build/test path for every later function.

**Tasks:**
- Establish Cargo workspace, rust-toolchain, target JSON, linker script, build scripts, and common feature/profile conventions.
- Establish docs/adr, docs/architecture, docs/abi, docs/platform, docs/testing, and an ADR template.
- Establish no_std foundation crates, error types, bitflags, newtyped IDs and address types (PhysAddr/GuestPhysAddr/VirtAddr).
- Establish CI: fmt, clippy, cargo test (host crates), cargo build (aarch64 target), QEMU smoke placeholder.
- Establish lint/code-review rules against board-name cfg in Core.
- Define unsafe rules and SAFETY comment template; establish unsafe inventory.
- Define log levels, trace event IDs, panic policy, and build version string.
- Freeze v0.1 Architecture Decision Register and change process.
**Exit criteria:**
- Main branch builds reproducibly.
- Host-side tests run automatically.
- Documentation and ADRs are versioned.

### P1 — Minimal AArch64 EL2 boot
**Goal:** Reliably enter Rust EL2 from firmware with basic exception/serial diagnostics.

**Tasks:**
- Select QEMU virt boot mode (direct kernel or U-Boot/TF-A path) and record both boot recipes.
- Implement minimal assembly entry, stack, BSS clearing, and Rust entry.
- Read and print CurrentEL, MPIDR_EL1, and ID_AA64* capability registers.
- Establish EL2 vector table and minimal context save for synchronous exceptions/IRQ/FIQ/SError.
- Establish early console (QEMU PL011) and abstract EarlyConsole.
- Implement panic/crash dump: ESR_EL2, ELR_EL2, FAR_EL2, HPFAR_EL2, SPSR_EL2.
- Set minimal safe HCR_EL2/CPTR_EL2/CNTHCTL_EL2 baseline.
- Evaluate EL2 MMU on/off policy and finally enable Host Stage-1 mapping.
**Exit criteria:**
- QEMU prints stable boot logs.
- Exceptions print complete syndrome.
- Repeated reboots show no random faults.

### P2 — Platform discovery and memory infrastructure
**Goal:** Build PlatformInfo from DTB and reliable physical-page/kernel-object allocators.

**Tasks:**
- Introduce/implement FDT parser limited to required nodes and cells/address/size/interrupt specifiers.
- Parse /cpus, /memory, /reserved-memory, /chosen, PSCI, timer, GIC, UART.
- Generate PlatformInfo and PlatformCapabilities.
- Build boot memory map: usable RAM, Hypervisor image, DTB, boot package, reserved ranges.
- Implement page-frame allocator (choose bitmap or buddy first) and allocation-ownership debug metadata.
- Implement small-object/slab allocator or wrapped heap.
- Define minimal MemoryObject/MemoryRegion structures.
- Add hv-platform-inspect mode to print topology/resources.
- Build host-side DTB static compatibility-checker prototype.
**Exit criteria:**
- QEMU PlatformInfo matches actual boot parameters.
- Memory allocation stress test passes.
- Reserved regions are never allocated.

### P3 — SMP and per-CPU foundation
**Goal:** Remove single-core assumptions early.

**Tasks:**
- Boot secondary CPUs through PSCI CPU_ON.
- Set up separate per-CPU stack/data and CPU-local ID mapping.
- Implement global barrier and boot rendezvous.
- Implement basic spinlock, irq-save lock, and atomic helpers; define lock order.
- Establish cross-CPU IPI/SGI primitive.
- Implement TLB shootdown mailbox framework.
- Test 2/4/8-vCPU QEMU host SMP; test 4×A55 on Orange Pi in its stage.
- Add lock-contention and IPI-latency trace events.
**Exit criteria:**
- All pCPUs enter online state.
- Concurrent allocator test passes.
- IPI/TLB mailbox can be regression-tested.

### P4 — Stage-2 and Rust Validation Guest v0
**Goal:** Enter an EL1 Guest safely for the first time.

**Tasks:**
- Implement AArch64 Stage-2 page-table builder, walker, map/unmap/protect.
- Wrap VTCR_EL2/VTTBR_EL2/TLBI and VMID allocator.
- Implement GuestAddressSpace creation/destruction.
- Implement minimal VM/VCPU objects, register initialization, and EL1 entry.
- Establish Rust no_std validation-guest crate with UART/semihost-like debug channel.
- Load ELF or flat binary into Guest RAM.
- First ERET into Guest and capture synchronous return to EL2.
- Handle WFI/WFE and unknown exceptions basically.
- Add detailed Stage-2 fault trace and page-table dump.
**Exit criteria:**
- Guest prints “Hello from EL1”.
- Out-of-bounds IPA reliably triggers Stage-2 fault.
- Faulty Guest cannot corrupt EL2.

### P5 — Hypercall, object handles, and Capability v0
**Goal:** Establish a safe call skeleton for later management and IPC.

**Tasks:**
- Define HVC ABI: call number, register convention, errors, version query.
- Implement safe copy_from_guest/copy_to_guest helpers.
- Implement object table + Handle(slot,generation) to reject stale handles.
- Implement Rights bitset and capability lookup.
- Implement basic capability grant (bootstrap→domain only) and revoke placeholder interface.
- Have Validation Guest test legal/illegal HVC, random handles, and out-of-bounds buffers.
- Separate hypercall dispatcher into a fuzzable host-side parser.
**Exit criteria:**
- Invalid handles do not escalate rights.
- ABI version can be queried.
- HVC fuzzing causes no panic/out-of-bounds access.

### P6 — Timer, GICv3, and virtual interrupt v0
**Goal:** Run a Guest with timer and external interrupts.

**Tasks:**
- Initialize physical GICv3 distributor/redistributor/CPU interface.
- Implement SGI/PPI/SPI descriptors and routing.
- Implement EL2 physical timer and per-CPU tick/event timer.
- Save/restore vCPU virtual timer.
- Implement software vIRQ pending queue and injection.
- Incrementally use GIC virtualization interface/List Registers.
- Handle maintenance interrupts.
- Test periodic timer, SGI, IRQ priority/basic masking in Validation Guest.
- Measure physical IRQ→vIRQ latency.
**Exit criteria:**
- Guest timer runs stably.
- Multi-vCPU SGI works.
- Interrupt storm does not lose core state.

### P7 — vCPU Scheduler v0 → preemptive
**Goal:** Evolve from static binding to truly runnable vCPUs.

**Tasks:**
- Define scheduler trait, RunQueue, SchedulingEntity.
- Implement pinned static scheduler first, equivalent to existing behavior.
- Add vCPU state machine: Runnable/Running/Blocked/Paused/Stopped.
- Implement tick/preemption and context switch.
- Implement simple round-robin as first M:N scheduler.
- Implement CPU affinity/pinning/cpuset.
- Block on WFI/Notification and wake up.
- Add runtime accounting, steal time, runqueue depth, switch-reason trace.
- Stress multiple VMs' CPUs and test fairness.
**Exit criteria:**
- M>N vCPUs run stably.
- Pause/resume has no races.
- CPU affinity takes effect.

### P8 — Virtual platform and minimal Linux boot
**Goal:** Define rusthv-arm-virt-v1 and boot a customized Linux.

**Tasks:**
- Freeze v1 Guest IPA layout: RAM, GIC, UART/console, virtio-mmio windows, PCI reservation.
- Implement Guest DTB builder/patcher.
- Implement minimal PSCI virtualization (CPU_ON/OFF/SYSTEM_OFF, etc.).
- Implement minimal virtual UART or console MMIO device.
- Load Linux Image, initramfs, DTB, and set boot registers.
- Support Linux SMP secondary CPU bring-up.
- Implement required system-register traps/emulation.
- Establish Linux boot regression through earlycon to initramfs shell.
- Record rusthv-arm-virt-v1 machine ABI document.
**Exit criteria:**
- Customized Linux reaches shell.
- SMP Linux sees expected vCPU count.
- Machine ABI document is frozen.

### P9 — Virtio Core + console/block v0
**Goal:** Establish the standard paravirtual I/O mainline.

**Tasks:**
- Implement Virtio feature negotiation, device status, queue abstraction.
- Implement virtio-mmio transport.
- Implement safe split-virtqueue descriptor walker: loop detection, length limit, address validation.
- Reserve packed-virtqueue interface.
- Implement virtio-console or virtio-rng as minimal device.
- Implement memory-backed virtio-blk backend.
- Implement Notification/doorbell and IRQ injection.
- Establish malicious-descriptor tests and fuzz harness.
- Add queue depth/bytes/latency telemetry.
**Exit criteria:**
- Linux uses standard virtio drivers.
- Bad descriptors cannot access out of bounds.
- Block-device read/write consistency test passes.

### P10 — Boot Package, Bootstrap Domain, and Linux Control Domain
**Goal:** Move from “Hypervisor loads every VM itself” to a management domain.

**Tasks:**
- Define Boot Manifest v1 and simple in-memory Boot Package.
- Have U-Boot/loader place Hypervisor, Control Domain image, initramfs/manifest into RAM.
- Implement Bootstrap Domain Creator.
- Allocate initial CPU/Memory/IRQ/Device/Management capabilities to Control Domain.
- Boot minimal Linux Control Domain.
- Implement management endpoint/hypercall channel between Control Domain and EL2.
- Implement first native management ABI for create/destroy/pause/resume VM.
- Load a second Linux/validation VM image from Control Domain.
- Confirm EL2 has no general-filesystem dependency.
**Exit criteria:**
- Control Domain dynamically creates/destroys VMs.
- Restarting an ordinary VM does not affect Control Domain.
- Boot Manifest supports version checking.

### P11 — Configuration and management data model v1
**Goal:** Promote temporary commands into persistent, versioned Specs.

**Tasks:**
- Define VmSpec/CpuSpec/MemorySpec/BootSpec/DeviceSpec/SecuritySpec.
- Define schema_version and machine_version.
- Support one human-oriented TOML/YAML configuration format in Control Domain.
- Normalize configuration before native ABI calls; EL2 does not parse YAML/XML.
- Implement VM lifecycle state machine and operation transaction ID.
- Establish desired-vs-actual state data model (initial reconciler may be simple).
- Start a permission policy and capability broker.
- Design audit log: who did what to which object, and when.
**Exit criteria:**
- Configuration reproducibly creates the same VM.
- Bad configuration has structured diagnostics.
- Insufficiently authorized operations are rejected and audited.

### P12 — Service IPC, shared memory, and external Virtio Backend
**Goal:** Verify replaceable EL2/Service Domain backends.

**Tasks:**
- Implement small-message Endpoint.
- Implement Notification/event channel.
- Implement SharedRegion creation, mapping, permissions, and lifecycle.
- Implement capability delegation to Service Domain.
- Define Backend Protocol v1: attach/detach/reset/queue-kick/completion.
- Move virtio-blk backend from EL2 to Linux Service/Control Domain backend.
- Compare EL2 and Service Domain backend performance and tail latency.
- Detect backend crash, mark device failed, and reconnect/reset.
**Exit criteria:**
- One frontend can switch between two backends.
- Service Domain crash does not corrupt EL2.
- Shared memory has no unauthorized mapping.

### P13 — Network, PCI, and device framework
**Goal:** Grow from minimal virtual devices to general device infrastructure.

**Tasks:**
- Implement virtio-net with Linux backend/TAP/bridge path.
- Implement generic Device/Bus/MMIO-region registry.
- Implement basic PCI ECAM host/guest models.
- Implement virtio-pci transport.
- Establish MSI/MSI-X to virtual-interrupt routing framework.
- Introduce device reset/quiesce/suspend state machine.
- Build Driver Registry: DT compatible/PCI ID→driver.
- Separate namespaces for platform and virtual devices.
**Exit criteria:**
- virtio-net network works.
- Linux virtio-pci driver works.
- Device attach/detach lifecycle is stable.

### P14 — IOMMU/SMMUv3 and device passthrough
**Goal:** Establish genuine DMA isolation.

**Tasks:**
- Define IommuDomain/DmaMapping/DeviceGroup abstractions.
- Implement basic SMMUv3 driver on QEMU virt iommu=smmuv3.
- Implement device→VM DMA-domain attach/detach.
- Implement safe DMA map/unmap and invalidation.
- Implement passthrough IRQ/MSI route.
- Define trusted-direct vs isolated-direct configuration checks.
- Implement mediated/bounce-buffer example for non-IOMMU platform experiments.
- Quiesce DMA before device reset/ownership transfer.
- Add malicious-DMA-configuration tests and isolation regression.
**Exit criteria:**
- QEMU secure-passthrough path can be verified.
- Without IOMMU, reject an isolated label.
- Cross-VM DMA is blocked.

### P15 — Orange Pi 3B physical port
**Goal:** Verify platform abstractions rather than rewrite Hypervisor for one board.

**Tasks:**
- Document TF-A/U-Boot→EL2 boot chain and artifacts.
- Parse actual Orange Pi 3B DTB and verify CPU/RAM/GIC/timer/PSCI/UART.
- Implement/reuse Rockchip early UART and required SoC helpers.
- Establish rk356x SoC layer, orangepi3b BSP, and quirk files.
- Verify 4× Cortex-A55 SMP and cache/TLB shootdown.
- Run full Validation Guest regression.
- Run Linux Guest + virtio console/block/net where available.
- Evaluate PCIe/NVMe/network paths and explain safety limits without IOMMU.
- Establish hardware regression after serial capture/remote power control.
**Exit criteria:**
- Core has no OrangePi special cases.
- Validation Guest/Linux run stably on hardware.
- BSP remains thin.

### P16 — Advanced memory capabilities
**Goal:** Add sharing, hotplug, balloon, and snapshot foundations for general virtualization.

**Tasks:**
- Implement Hypervisor object model for memory hot-add/hot-remove.
- Implement virtio-balloon or equivalent channel.
- Implement page donation/reclaim and advanced SharedRegion permission transitions.
- Abstract dirty/access bit tracking.
- Implement COW MemoryObject / snapshot layer.
- Implement stop-the-world snapshot v1: vCPU+RAM+device state.
- Version and integrity-check snapshot format.
- Establish restore regression to same machine type.
- Study memory-overcommit pressure policy; still no EL2 swap by default.
**Exit criteria:**
- Snapshot/restore are consistent.
- Balloon reclaims memory dynamically.
- Page-ownership checker finds no leaks.

### P17 — Live migration and compatibility
**Goal:** Demonstrate the value of versioned machine ABI.

**Tasks:**
- Define migration stream/object schema.
- Implement pre-copy RAM migration and dirty rounds.
- Implement stop-and-copy vCPU/IRQ/device state.
- Implement source/destination capability negotiation.
- Limit v1 to same architecture and compatible machine type.
- Establish failure rollback and source resume.
- Add migration bandwidth/downtime telemetry.
- Define machine compatibility policy.
**Exit criteria:**
- Running Linux VM migrates.
- Failed migration rolls back safely.
- Downtime can be quantified.

### P18 — libvirt and management ecosystem
**Goal:** Let standard virtualization tools manage Hypervisor.

**Tasks:**
- Stabilize management daemon and native RPC.
- Implement minimal domain lifecycle in libvirt hypervisor driver/adapter.
- Map CPU, Memory, disk, net, console, pause/resume/shutdown.
- Support libvirt XML → VmSpec.
- Gradually add storage/network/host-device integration.
- Keep research trace/capability APIs only in native extensions.
- Implement CLI and structured status query.
**Exit criteria:**
- Basic virsh lifecycle works.
- XML maps reliably.
- Native API remains independently versioned.

### P19 — Native Rust Control Domain / Service Runtime
**Goal:** Gradually reduce hard dependence on a Linux management domain.

**Tasks:**
- Define minimal syscall/IPC/driver ABI for Rust service runtime.
- Implement Rust management service and configuration database.
- Move simple virtio backend.
- Move storage/network/driver services according to value, without replacing Linux all at once.
- Support coexistence of Linux and Rust Control Domains.
- Compare performance, TCB, and fault recovery of both.
**Exit criteria:**
- Rust management domain creates/manages VMs.
- Linux or Rust Control Domain can be selected.
- Service interfaces stay compatible.

### P20 — x86_64 architecture backend
**Goal:** Test “shared semantics, independent mechanisms” abstractions.

**Tasks:**
- Establish x86_64 boot/UEFI and VMX-first path (SVM later).
- Implement VMCS, EPT, VM-exit, APIC/x2APIC backends.
- Reuse core Vm/Vcpu/GuestAddressSpace/Scheduler/Capability without changes.
- Convert ACPI/PCI platform discovery to PlatformInfo.
- Boot Validation Guest and Linux on QEMU x86_64.
- Define rusthv-x86-virt-v1 machine type.
- Bring up physical compatible PC.
- Incrementally add IOMMU (VT-d/AMD-Vi) and passthrough.
**Exit criteria:**
- Core has no VMX-specific branches.
- QEMU x86 Linux boots.
- At least one physical PC is verified.

### P21 — Advanced security, real-time, and research features
**Goal:** Enter long-term research expansion.

**Tasks:**
- Implement multiple scheduler classes and RT/partition scheduler.
- Establish worst-case interrupt/timer latency benchmarks.
- Implement capability delegation tree and efficient revoke.
- Add secure boot and manifest/image signing/measurement.
- Prototype nested virtualization.
- Research confidential/TEE integration feasibility.
- Formalize critical state machines/page ownership/capability invariants.
- Continuously reduce and audit unsafe/TCB.
**Exit criteria:**
- Each research feature has an independent experiment report.
- Stable machine ABI remains intact.
- Critical security invariants can be checked mechanically.

## 16. Milestone aggregation

| Milestone | Stages | Visible result |
|---|---|---|
| M0 EL2 Alive | P0-P1 | Rust EL2 boot, exception and serial diagnostics |
| M1 First Guest | P2-P4 | Rust EL1 Validation Guest + Stage-2 |
| M2 Virtual CPU Platform | P5-P7 | Capability/HVC, GICv3, timer, M:N scheduling foundation |
| M3 Linux Boot | P8 | rusthv-arm-virt-v1 + SMP Linux |
| M4 Standard I/O | P9 | Virtio MMIO + block/console |
| M5 Managed Hypervisor | P10-P12 | Linux Control Domain, dynamic VM, IPC, Service backend |
| M6 General I/O | P13-P14 | network/PCI/virtio-pci/SMMUv3/passthrough |
| M7 Real Hardware | P15 | Orange Pi 3B runs stably |
| M8 Advanced Memory | P16-P17 | balloon/snapshot/live migration |
| M9 Ecosystem | P18-P19 | libvirt + Rust Control Domain |
| M10 Multi-Arch | P20 | x86_64 QEMU + PC |
| M11 Research Platform | P21+ | RT, security, nested, formal, etc. |

## 17. Explicit current exclusions (not v0.x goals)

- Simultaneous AArch64 and x86_64 support in the first stage.
- General VFS/ext4/qcow2/NVMe/full network stack inside EL2.
- Windows, full legacy PC-device emulation, or nested virtualization in the first version.
- Claims of strong DMA isolation for untrusted-Guest direct passthrough without IOMMU.
- Permanently fixing Control Domain as the only privileged VM, or using VM ID as the authorization model.
- Forcing ARM registers/VMX controls into a detail-leaking god trait merely for “unified abstraction”.
- Simultaneously delivering live migration, memory swap, NUMA, SR-IOV, and GPU virtualization in the first version.

## 18. Near-term questions awaiting ADR freeze

- Formal project name and crate prefix.
- P2 physical-page allocator: buddy or bitmap+size-class combination.
- Native Management ABI v1 wire format.
- Default algorithm of first preemptive scheduler.
- Exact rusthv-arm-virt-v1 IPA map, virtio-mmio slot count, GIC/PCI windows.
- First Linux Control Domain rootfs/distribution and management agent shape.
- Default enablement of experimental EL2 virtio backends in general/release profiles.
- Orange Pi 3B actual boot chain and post-survey UART/PCIe/NVMe quirks.

## 19. Architecture invariants (Implementation Invariants)

- **MUST:** Hypervisor Core must not depend on a specific Board name.
- **MUST:** Arch layer must not depend on SoC/Board.
- **MUST:** Guest-provided addresses, lengths, and indices must be validated before memory access.
- **MUST:** Mutually untrusted VMs must not share writable HPA without explicit SharedRegion.
- **MUST:** Quiesce/reset/DMA detach must finish before device ownership transfers.
- **MUST:** Failed capability validation must not fall back to implicit role/VM-ID allowance.
- **MUST:** EL2 must not store user passwords, TLS private keys, or a general RBAC database.
- **MUST:** Machine ABI, snapshot, migration, and management ABI must be versioned.
- **MUST:** Guest-caused faults should by default affect only the corresponding VM/device context, not cause a global panic.
- **MUST:** Every cross-CPU Stage-2/IRQ-route mutation must define synchronization and invalidation semantics.

## 20. Reference specifications and implementations

- **QEMU Arm virt machine** — https://www.qemu.org/docs/master/system/arm/virt
- **OASIS Virtio 1.4** — https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.html
- **libvirt API / driver model** — https://www.libvirt.org/api.html
- **Arm Architecture / GIC / SMMU / PSCI** — Arm Developer architecture specifications (freeze exact revision during implementation)
- **Linux/KVM ARM virtualization** — cross-check vGIC, PSCI, timer, Stage-2 behavior; not a source for copying code
- **Xen / seL4 / Bao / Jailhouse / ACRN / Firecracker / Cloud Hypervisor** — comparison references for management domains, capability, static partition, device models, engineering organization

---
**v0.1 change rule:** Overturning a settled ADR requires a new ADR marked `Supersedes ADR-xxx`; do not silently rewrite historical decisions. Development stages may be split or merged, but their exit criteria must remain verifiable.
