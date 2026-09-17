# Zelyr Hypervisor Architecture Decision Record v0.1

**副标题：** 架构决策基线、完整分层架构与分阶段研发路线图  
**日期：** 2026-09-16  
**状态：** Architecture Baseline / Draft for Implementation  
**范围：** AArch64-first Type-1 Hypervisor；QEMU virt + Orange Pi 3B (RK3566)；x86_64 后续扩展

> 本文档是当前项目的架构基线。除标记为“待定”的条目外，已确定决策应被视为后续实现和评审的默认约束；修改已确定决策应新增 ADR 并记录替代关系。

## 1. 架构原则

1. **研究优先，通用虚拟化长期目标**：允许保留实验性机制与可观测能力，但所有核心抽象必须能够向长期通用 Hypervisor 演进。
2. **机制与策略分离**：EL2 提供安全、资源、虚拟化机制；复杂策略尽量由 Control Domain / Service Domain 承担，但性能关键服务允许驻留 EL2。
3. **模块化微内核式代码组织**：“微内核”首先指严格模块化、低耦合与清晰边界；并不强制把所有可外置功能都搬出 EL2。
4. **共享语义，不强行共享硬件表示**：AArch64 与 x86_64 尽可能共享 VM/vCPU/内存/中断/设备语义；VMCS、HCR_EL2、VTTBR_EL2 等机制保持在架构后端。
5. **动态能力从数据模型开始预留**：第一阶段可静态分配，但 VM、vCPU、内存、设备、IRQ 等对象从一开始按可创建、销毁、迁移、重新绑定设计。
6. **Guest 默认不可信**：普通 Guest 被视为潜在恶意实体；所有 hypercall、MMIO、virtio descriptor、Stage-2 fault、IRQ 注入路径均需边界检查。
7. **可移植性优先于 Board 特判**：Core 不依赖 Board；Arch 不依赖 SoC；Board quirks 必须集中在 BSP/Quirk 层；平台信息由 DTB/ACPI 转为统一 PlatformInfo。
8. **验证 Guest 先于 Linux**：先用极小 Rust EL1 裸机 Guest 验证 EL2 机制，再引入 Linux，降低调试变量。
9. **配置是一等架构对象**：Build、Boot、Bootstrap Domain、Control Domain、VM、兼容机型与安全策略分别建模；EL2 不解析复杂 YAML/XML/JSON。
10. **可观测性不是事后补丁**：trace、exit 统计、IRQ latency、Stage-2 fault、调度器、内存 ownership 等从早期阶段即预留结构化 telemetry。

## 2. Architecture Decision Register

| ID | 状态 | 主题 | 决策 | 主要后果 |
|---|---|---|---|---|
| ADR-001 | 已确定 | 项目定位 | 研究型 Type-1 Hypervisor 为首要目标，长期向通用虚拟化演进。 | 保留实验机制、追踪与可替换策略；不以最小功能产品作为最终边界。 |
| ADR-002 | 已确定 | 首要 ISA | 第一阶段仅实现 AArch64；x86_64 在架构稳定后加入。 | 避免双 ISA 同时 bring-up；核心抽象仍需避免 AArch64 细节泄漏。 |
| ADR-003 | 已确定 | 首要平台 | QEMU virt 为 reference/CI 平台；Orange Pi 3B (RK3566) 为首个真实硬件平台。 | QEMU 用于确定性测试，Orange Pi 用于验证真实 cache/TLB/IRQ/firmware/PCIe 行为。 |
| ADR-004 | 已确定 | 微内核含义 | 首先采用模块化微内核式代码架构，而非强制最小 EL2 TCB。 | 性能关键功能可以驻留 EL2；第三方成熟驱动与复杂服务优先放 Service Domain。 |
| ADR-005 | 已确定 | 服务部署模型 | 采用“模块化 EL2 核心 + 可选 Service Domain”混合模型。 | 同一服务接口允许 EL2 实现或 Service-VM 实现，用于性能/隔离研究。 |
| ADR-006 | 已确定 | 语言边界 | Rust-first；允许必要汇编和受控 unsafe；允许依赖成熟 crate。 | unsafe 需集中在 arch/HAL/低层数据结构边界，并形成审计清单。 |
| ADR-007 | 已确定 | Guest 威胁模型 | 普通 Guest 默认可能恶意。 | 所有来自 Guest 的地址、长度、索引、状态迁移、descriptor 和 hypercall 必须验证。 |
| ADR-008 | 已确定 | 启动固件 | 不实现 EL3 固件；依赖 TF-A/U-Boot 等现有 firmware，以 Non-secure EL2 启动。 | Hypervisor 处理/代理必要 PSCI/SMC，但不接管 Secure World。 |
| ADR-009 | 已确定 | 第一个 Guest | 第一个进入 EL1 的 VM 为 Rust 裸机 Validation Guest。 | 用于独立验证 HVC、Stage-2、timer、IRQ、MMIO、SMP 等机制。 |
| ADR-010 | 已确定 | Linux Guest | Validation Guest 稳定后引入定制 Linux，之后扩展到通用 Linux。 | 早期允许定制 DTB、内核参数和设备集合。 |
| ADR-011 | 已确定 | Control Domain 演进 | 早期允许 Linux Control Domain；后期开发 Rust 原生 Control Domain/Service Runtime。 | 尽早复用 Linux 文件系统、网络、存储与设备驱动，降低初期工程量。 |
| ADR-012 | 已确定 | 管理能力委派 | Control Domain 不是唯一可管理实体；其他 VM 可获得受限管理能力。 | 授权基于 Hypervisor capability；身份认证留在管理域，不在 EL2 存密码。 |
| ADR-013 | 已确定 | Capability 模型 | 资源与管理操作采用对象 capability/handle + rights + generation。 | 支持最小权限、委派、权限削减与未来撤销；避免 vm_id==0 特权逻辑。 |
| ADR-014 | 已确定 | EL2 动态内存 | EL2 允许动态页分配、heap/slab/object allocator。 | 动态 VM/vCPU/内存不可建立在完全静态内核分配模型上。 |
| ADR-015 | 已确定 | SMP 时机 | SMP 属于早期基础能力。 | 避免先形成单核假设；QEMU 多核与 RK3566 四核都用于早期验证。 |
| ADR-016 | 已确定 | CPU 调度 | 早期静态 vCPU↔pCPU；逐步演进到抢占式 M:N overcommit。 | 最终同时支持 dedicated CPU VM、shared CPU VM、affinity/pinning 和多调度类。 |
| ADR-017 | 长期预留 | 实时调度 | 长期加入 partition/RT scheduler 与 latency instrumentation。 | 不阻塞通用调度器 bring-up，但 scheduler API 不得锁死单一策略。 |
| ADR-018 | 已确定 | 内存模型 | Stage-2 address space 是核心对象；支持动态 map/unmap/protect、ownership 与 TLB shootdown。 | 为 balloon、COW、snapshot、migration、共享内存预留页级元数据。 |
| ADR-019 | 长期预留 | Memory overcommit | 允许 overcommit；早期优先 balloon/reclamation + hard limit，不在 EL2 实现 swap daemon。 | 若未来做透明 paging，策略与 backing store 放管理/内存服务，EL2 提供 revoke/fault/remap 原语。 |
| ADR-020 | 长期预留 | Snapshot/Live Migration | 必须支持 snapshot、dirty tracking、pre-copy/stop-and-copy 与设备状态迁移。 | 机器模型、设备状态、vCPU 状态和配置 schema 从早期即版本化。 |
| ADR-021 | 已确定 | Guest firmware/boot | 演进顺序：ELF 裸机 → Linux Image+DTB+initramfs → UEFI/EDK2。 | 避免早期 firmware 复杂度，同时保留 Windows/BSD 等长期兼容路径。 |
| ADR-022 | 已确定 | Guest EL | 第一阶段 Guest kernel 运行 EL1；不向 Guest 暴露虚拟 EL2。 | 简化 vCPU/Stage-2/GIC；nested virtualization 后期单独实现。 |
| ADR-023 | 长期预留 | Nested virtualization | 虚拟 EL2、nested Stage-2、nested interrupt virtualization 后期实现。 | 不影响第一代 VM ABI，但 arch API 避免把“Guest 永远是 EL1”写死到上层。 |
| ADR-024 | 已确定 | 虚拟 ARM 平台 | 定义与 Host SoC 无关的 versioned Generic ARM64 VM machine。 | Guest 不应看到“RK3566 虚拟版”；迁移与镜像兼容依赖稳定虚拟硬件 ABI。 |
| ADR-025 | 已确定 | Guest 描述 | DTB-first；ACPI later。 | Linux/裸机先用 DTB；UEFI/Windows 阶段加入 ACPI。 |
| ADR-026 | 已确定 | Virtio 定位 | Virtio 是首要 paravirtual I/O ABI。 | 优先 virtio-mmio，随后 virtio-pci；遵循当前稳定 Virtio 规范并做 feature negotiation。 |
| ADR-027 | 已确定 | 设备模型 | 混合模式：virtio + 设备模拟 + mediated device + direct passthrough。 | 按安全级别和性能目标选择，不将所有设备强制统一为一种实现。 |
| ADR-028 | 已确定 | 复杂驱动位置 | NVMe、Ethernet、USB、文件系统等复杂/第三方成熟驱动原则上不进入 EL2。 | 优先由 Linux/Rust Service Domain 承担；EL2 可保留少量性能关键或早期验证后端。 |
| ADR-029 | 已确定 | 文件系统边界 | EL2 不实现通用文件系统；启动期由 bootloader 加载 Boot Package 到内存。 | Control/Storage Domain 负责真实文件系统、镜像、qcow2/raw、网络存储等。 |
| ADR-030 | 已确定 | DMA 隔离分级 | 无 IOMMU 时不能把软件路径称为安全 direct passthrough。 | 区分 paravirtualized、mediated、trusted direct、IOMMU-isolated direct 四级。 |
| ADR-031 | 已确定 | IOMMU/SMMU | 抽象 IOMMU capability；QEMU virt 使用 SMMUv3 路径验证安全 passthrough。 | 真实板按硬件能力启用；上层查询 capability，不按平台名字分支。 |
| ADR-032 | 已确定 | 中断基线 | AArch64 主线直接以 GICv3 为基准。 | 早期 SGI/PPI/SPI + vIRQ；随后 LR/maintenance、vGIC distributor/redistributor、ITS/MSI/LPI。 |
| ADR-033 | 长期预留 | GICv2 | 作为旧 ARM 平台兼容后端，而非主线。 | 保持 InterruptController trait 可容纳不同 backend。 |
| ADR-034 | 已确定 | IPC 原语 | Hypervisor 原生原语收敛为 Capability、Endpoint、Notification、SharedRegion。 | 控制面用小消息 Endpoint；数据面用 SharedRegion + Notification，不维护两套无关 IPC。 |
| ADR-035 | 已确定 | 故障隔离 | Service/Driver Domain 故障不应破坏 Hypervisor 或其他 Domain。 | IPC、shared-memory mapping、resource revoke、backend reset 需支持故障恢复。 |
| ADR-036 | 已确定 | 管理接口 | 内部使用 versioned native management ABI；外部后期提供 libvirt driver/adapter。 | 不让 libvirt API 限制研究型功能；libvirt 只映射其可表达的生命周期/设备能力。 |
| ADR-037 | 已确定 | 配置分层 | Build / Hypervisor Boot / Bootstrap Domain / Control Domain / VM / Policy 分层配置。 | EL2 不解析复杂通用配置格式；Control Domain 将 YAML/XML/API 归一化为内部 Spec。 |
| ADR-038 | 已确定 | Bootstrap Architecture | Control Domain 由 Hypervisor Boot Manifest 创建和授予初始 capability。 | Control Domain 不能自我创建；未来可有多个 bootstrap/service domain。 |
| ADR-039 | 已确定 | Desired/Actual State | Control Domain 长期采用期望状态与实际 Hypervisor 状态分离的 reconciler。 | 便于 hotplug、失败恢复、持久化和编排。 |
| ADR-040 | 已确定 | 版本化 | schema_version、machine_version、management_abi_version 独立版本化。 | 支撑 snapshot、migration、长期 Guest ABI 和工具兼容。 |
| ADR-041 | 已确定 | 平台适配层次 | Arch / SoC / Board / Firmware / Discovery / Driver / Quirk 分层。 | 目标是新增平台尽量组合已有驱动与描述，不修改 Core。 |
| ADR-042 | 已确定 | 平台发现 | AArch64 以 DTB 为主要发现源，x86_64 后续以 ACPI+PCI 等为主要来源，统一转 PlatformInfo。 | 核心只依赖统一平台中间表示。 |
| ADR-043 | 已确定 | Board 特判规则 | Core 禁止出现具体 Board 条件分支；特判集中到 BSP/Quirk。 | 保证长期多平台可维护性。 |
| ADR-044 | 已确定 | 平台能力查询 | 上层按 PlatformCapabilities 查询功能，不按 QEMU/RK3566/PC 名字判断。 | 支持 GIC、IOMMU、PCI、MSI、hotplug 等能力的运行时差异。 |
| ADR-045 | 已确定 | 平台支持等级 | 定义 Reference/Tier-1/Experimental 等支持等级。 | 控制多板卡维护成本，并明确 CI 与回归责任。 |
| ADR-046 | 已确定 | Workspace | 从第一天使用 Cargo workspace + 多 crate，但初期不做过度细粒度拆分。 | 边界稳定后再分拆驱动、SoC、平台和服务 crate。 |
| ADR-047 | 已确定 | Feature/Profile | 支持 minimal/research/embedded/general/secure/full 等 profile；feature 表示“二进制具备能力”。 | VM 数量、RAM 等运行时策略不得滥用 Cargo feature。 |
| ADR-048 | 已确定 | Telemetry | 结构化 tracing/metrics 为一等能力。 | 至少覆盖 VM-exit、Stage-2 fault、TLB、IRQ、scheduler、virtio、lock contention、memory ownership。 |
| ADR-049 | 已确定 | 验证策略 | unit + host-side + QEMU integration + guest self-test + Linux boot regression + fuzz/property + unsafe audit。 | 关键状态机与页所有权可进一步 model checking/formal verification。 |
| ADR-050 | 否决 | EL2 通用文件系统 | 不把 ext4/FAT/qcow2/VFS 等通用文件系统栈作为 Hypervisor Core 基础设施。 | 避免 EL2 TCB/驱动栈膨胀。 |
| ADR-051 | 否决 | 单一“特权 VM ID=0”模型 | 不允许通过固定 VM ID 获得所有管理权限。 | 改用 capability + policy；避免未来多管理域与最小权限无法实现。 |
| ADR-052 | 否决 | 按 Board 名称驱动核心逻辑 | 禁止核心业务代码通过 platform==QEMU / OrangePi 选择路径。 | 统一使用接口、PlatformInfo、capability 与 quirk。 |
| ADR-053 | 否决 | 从第一天兼容传统 PC 全套设备模拟 | AArch64 早期不投入 i440fx/Q35 类 legacy PC 设备模型。 | 优先标准 ARM 虚拟平台 + virtio，x86_64 以后按兼容需求增加。 |
| ADR-054 | 待定 | 项目正式名称 | 暂使用工作名 RustHV / rust-type1-hypervisor。 | 命名不影响 ABI 设计；在公开仓库/协议前冻结。 |
| ADR-055 | 待定 | 首个 Linux Control Domain 发行版/根文件系统 | 可采用极简 Linux + initramfs/BusyBox 或定制发行版。 | 在 Storage/Network service bring-up 前决定。 |
| ADR-056 | 待定 | Native Management ABI 编码 | 候选：固定二进制结构 + TLV、Cap’n Proto-like schema、自定义 little-endian wire format。 | 必须 no_std 可解析、版本化、边界检查清楚；不把 JSON 作为 EL2 ABI。 |
| ADR-057 | 待定 | 默认通用调度器算法 | 候选 round-robin/weighted fair/credit-like；第一阶段仅需正确的 preemptive runnable scheduler。 | 先冻结 scheduler trait 与 accounting，再通过 benchmark 选择默认策略。 |
| ADR-058 | 待定 | EL2 内 virtio backend 默认范围 | 早期可以保留 console/block/net 的实验性 backend；长期默认启用范围待 benchmark。 | 服务位置作为可替换实现，不在 v0.1 强制性能结论。 |
| ADR-059 | 长期预留 | 安全启动/签名 | Boot Manifest、Hypervisor、Control Domain、VM image 可加入签名/测量链。 | 需要与实际 firmware/TPM/TEE 环境联合设计。 |
| ADR-060 | 长期预留 | 机密计算/TEE 集成 | 保留 TrustZone/Realm/其他 confidential VM 研究方向。 | 不进入当前非安全 EL2 主线。 |

## 3. 总体系统架构

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

### 3.1 分层

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

## 4. 核心对象模型

| 对象 | 职责 |
|---|---|
| Hypervisor | 全局根对象；维护只读系统能力、全局 registries 与启动阶段；避免成为所有子系统的 God Object。 |
| PhysicalCpu | 一个 pCPU 的运行时状态：MPIDR/APIC ID、local scheduler、current_vcpu、per-CPU stacks、TLB/IRQ mailbox。 |
| Vm | 虚拟机生命周期根对象；持有 machine profile、GuestAddressSpace、vCPU 集合、虚拟设备与 capability namespace。 |
| Vcpu | 可调度执行实体；含架构寄存器上下文、run state、virtual timer、pending event、exit accounting。 |
| GuestAddressSpace | Guest IPA/GPA → Host PA 的映射对象；AArch64 对应 Stage-2，x86 对应 EPT/NPT。 |
| MemoryObject | 可被映射/共享/捐赠的内存所有权对象；底层由物理页集合、连续区或外部 backing 构成。 |
| MemoryRegion | 某 AddressSpace 中的一段映射视图，带权限、memory type、dirty/access tracking 属性。 |
| Capability | 不可伪造对象引用；包含 slot/generation/object/right，并支持 delegation/revocation 的演进。 |
| Endpoint | 小型控制消息/请求-响应端点；与 caller capability 绑定。 |
| Notification | 轻量异步事件/doorbell；用于调度唤醒、virtqueue、event channel 等。 |
| SharedRegion | 显式共享内存对象；用于高吞吐 IPC、virtqueue、grant-like buffer。 |
| InterruptSource | 物理 IRQ/MSI/LPI 或软件事件源的统一对象。 |
| VirtualInterrupt | 面向 vCPU/vGIC 的虚拟 IRQ 对象，含 route/priority/pending/active 状态。 |
| VirtualDevice | Guest 可见设备状态机；实现 MMIO/PCI config trap 与设备快照状态。 |
| DeviceBackend | 虚拟设备的数据/控制后端；可在 EL2、本地 driver、Service Domain 中实现。 |
| AssignedDevice | 直通或 mediated 物理设备对象；绑定 DMA domain、IRQ route、reset/isolation policy。 |
| PlatformInfo | DTB/ACPI/firmware 解析后的统一平台描述。 |
| ResourcePool | CPU/Memory/IRQ/Device 等可分配资源集合及 ownership ledger。 |
| MachineType | 版本化 Guest 虚拟硬件 ABI，例如 rusthv-arm-virt-v1。 |
| Domain | VM 的角色/安全上下文：normal/control/driver/service/bootstrap；角色不自动等于权限。 |

### 4.1 生命周期

- VM：`Created -> Loading -> Ready -> Runnable -> Running -> Paused -> Suspended -> Stopping -> Stopped -> Destroyed -> Faulted`
- vCPU：`Offline -> Runnable -> Running -> Blocked -> Paused -> Stopped -> Faulted`
- Device：`Created -> Configured -> Active -> Quiescing -> Suspended -> Resetting -> Failed -> Detached`

## 5. CPU / vCPU / Scheduler 体系

- Host CPU 初始化分为 BSP 与 AP；所有 pCPU 拥有 per-CPU 状态和本地 runqueue/调度上下文。
- Vcpu 是唯一被调度的 Guest CPU 实体；Vm 本身不进入调度器。
- 架构后端负责寄存器保存/恢复、虚拟化控制寄存器、进入/退出 Guest；Core 只看到标准 ExitReason。
- ExitReason 建议至少包含 Hypercall、Stage2Fault、Mmio、WfiWfe、SysReg、InterruptWindow、Shutdown、InternalError。
- 早期 static pinned；随后抢占式 round-robin；最终 scheduler trait 支持 weighted/fair/RT/partition。
- CPU affinity 与 dedicated CPU 是策略，不应形成单独 VM 类型。
- 每次 vCPU switch 必须明确处理：arch context、virtual timer、vGIC state、lazy FP/SIMD（后续）、TLB/VMID 生命周期。

## 6. 内存体系

- Host memory manager 管理 HPA；GuestAddressSpace 管理 IPA/GPA→HPA；两者职责严格分离。
- 所有可映射内存以 MemoryObject 表示，MemoryRegion 是 address-space view；共享/捐赠通过对象所有权转换实现。
- Stage-2 API：map/unmap/protect/translate/query_dirty/clear_dirty；实现必须处理 break-before-make、TLB invalidation 和并发。
- 页元数据至少记录 owner、ref/share count、mapping flags、pin/DMA state、dirty/access tracking mode。
- 大页是优化，不改变 MemoryObject 语义；allocator 与 Stage-2 mapper 都需支持可降级粒度。
- balloon/hotplug/snapshot/migration 均建立在统一 ownership + mapping primitives 之上。
- overcommit 早期采用承诺量与实际驻留量分离；物理压力优先 balloon/reclaim，默认不实现 EL2 swap。

## 7. 中断与时间体系

- AArch64 主线为 GICv3；物理中断、虚拟中断、路由策略分别建模。
- Physical IRQ handler 尽量短：acknowledge→查路由→记录/注入或通知服务→EOI。
- vIRQ 注入从 software pending queue 起步，逐渐利用 GIC virtualization List Registers 和 maintenance interrupt。
- ITS/MSI/LPI 属于 PCI/高阶阶段；不能污染基础 SGI/PPI/SPI API。
- 虚拟 timer 状态归属 Vcpu；scheduler block/wakeup 与 WFI、timer deadline 联动。
- 跨 CPU 操作统一通过 IPI mailbox：reschedule、TLB shootdown、vIRQ kick、stop-the-world。

## 8. 设备与 I/O 体系

- Guest-facing VirtualDevice 与 physical/platform Driver 分离。
- VirtualDevice 处理 guest ABI 和状态机；DeviceBackend 处理真实 I/O；两者通过明确 backend contract 解耦。
- Virtio 是主 ABI：先 MMIO、后 PCI；split ring first、packed ring later；descriptor walker 视为安全边界。
- 设备后端可为 EL2 local、Control/Service Domain proxy、hardware-mediated、direct assigned。
- 安全等级固定为：paravirtualized/emulated；mediated；trusted direct；IOMMU-isolated direct。
- 设备 assignment 必须绑定 ownership、IRQ route、DMA domain、reset/quiesce policy。
- 复杂文件系统/存储格式不进入 EL2；block backend 只消费 block requests。

## 9. Capability / IPC / 故障隔离

- 认证与授权分离：用户名、密码、证书、RBAC 位于 Control Domain；EL2 只验证 capability。
- Capability v0 采用 slot+generation+rights；未来支持 delegation、attenuation、revoke tree。
- Endpoint 面向小控制消息；SharedRegion+Notification 面向数据面；二者共享同一 capability/ownership 基础。
- Service crash 时可撤销 mapping、标记 device failed、重新绑定 backend；不允许后端持有无法回收的隐式全局指针。
- 所有跨 domain shared memory 必须显式创建、映射、授权、销毁。

## 10. 管理与配置体系

- Build Configuration 只控制 binary 能力；Boot Manifest 控制 Hypervisor/Bootstrap Domain 初始资源；Runtime VmSpec 由 Control Domain 管理。
- Control Domain 负责镜像路径、文件系统、网络、用户认证、策略、libvirt、配置持久化。
- Hypervisor 负责不可伪造 capability、资源实际归属与机制执行，不保存密码/RBAC 数据库。
- Boot Manifest 创建 Control Domain，并授予其初始管理 capability；角色不是权限的替代品。
- VmSpec 分解为 Machine/Cpu/Memory/Boot/Device/Security；所有外部格式归一化到 Spec 后再调用 native ABI。
- 长期管理器使用 Desired State→Reconciler→Actual State 模型。

## 11. 平台适配体系

- Architecture：ISA/虚拟化机制；SoC：芯片级控制器与 glue；Board：接线/启动差异；Driver：可复用控制器；Quirk：受控特殊处理。
- AArch64 DTB、x86 ACPI/PCI 等都转换为 PlatformInfo；Core 禁止解析原始平台描述。
- PlatformCapabilities 表示 SMP/GIC/PCI/IOMMU/MSI/CPU hotplug 等能力，上层按能力选择路径。
- Board BSP 应尽量薄；同一 RK356x SoC 上的多块板复用 SoC/driver 实现。
- 提供 hv-platform-inspect 与 host-side DTB compatibility checker，加速新平台 bring-up。
- 支持等级至少划分 Reference（QEMU）、Tier-1（正式真机）、Experimental。

## 12. 安全、可靠性与可观测性

- 不可信边界：Guest register/input、hypercall buffer、MMIO、virtio descriptors、DMA config、management ABI。
- 必须存在资源 ownership invariant：一个可写物理页不能在无显式 SharedRegion 的情况下同时属于互不信任 VM。
- 所有 object handle 使用 generation 防 UAF/stale reference；所有长度/偏移计算使用 checked arithmetic。
- panic policy 区分 fatal hypervisor invariant 与 guest-caused fault；Guest 错误尽量仅 fault/stop 对应 VM。
- trace 事件必须可编译裁剪并支持运行时 filter；生产/benchmark 可关闭高开销事件。
- 核心指标包括 VM-exit 分布、Stage-2 fault/TLB、IRQ latency、scheduler switch、virtqueue latency、lock contention、内存碎片。

## 13. 工程与 crate 边界建议

- 初期 workspace：hv-core、hv-arch-aarch64、hv-platform、hv-drivers、hv-abi、hypervisor-bin、validation-guest。
- 边界稳定后拆：hv-vm、hv-memory、hv-scheduler、hv-resource、hv-cap、hv-ipc、hv-device、各 driver/SoC/BSP crate。
- 禁止循环依赖；Core 依赖 trait/数据模型，具体 arch/platform/driver 由 binary composition 注入。
- 优先使用 newtype 封装 HPA/IPA/GVA/VMID/VCPU ID，避免裸 usize 混用。
- 错误类型区分 GuestFault、InvalidInput、PermissionDenied、ResourceExhausted、HardwareFailure、InvariantViolation。

## 14. 建议的目标代码结构

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

## 15. 细粒度研发阶段树


### P0 — 仓库、规范与工具链基线
**目标：** 建立可持续工程骨架，任何后续功能都有可回归的构建/测试路径。

**任务：**
- 建立 Cargo workspace、rust-toolchain、target JSON、链接脚本、构建脚本和统一 feature/profile 约定。
- 建立 docs/adr、docs/architecture、docs/abi、docs/platform、docs/testing 目录及 ADR 模板。
- 建立 no_std 基础 crate、错误类型、bitflags、新类型 ID、地址类型（PhysAddr/GuestPhysAddr/VirtAddr）。
- 建立 CI：fmt、clippy、cargo test（host crates）、cargo build（aarch64 target）、QEMU smoke placeholder。
- 建立禁止核心出现 board-name cfg 的 lint/code-review 规则。
- 定义 unsafe 使用准则与 SAFETY 注释模板；建立 unsafe inventory。
- 定义日志等级、trace event ID、panic policy、构建版本字符串。
- 冻结 v0.1 Architecture Decision Register 与变更流程。
**退出条件：**
- 主分支可重复构建
- host-side tests 自动执行
- 文档与 ADR 可版本管理

### P1 — AArch64 EL2 最小启动
**目标：** 从 firmware 可靠进入 Rust EL2，并建立基本异常/串口诊断。

**任务：**
- 确定 QEMU virt 启动方式（直接 kernel 或 U-Boot/TF-A 路径）并记录两条启动配方。
- 实现最小汇编入口、栈、BSS 清零、Rust entry。
- 读取 CurrentEL、MPIDR_EL1、ID_AA64* 能力寄存器并输出。
- 建立 EL2 vector table 与同步异常/IRQ/FIQ/SError 最小保存现场。
- 建立 early console（QEMU PL011）；抽象 EarlyConsole。
- 实现 panic/crash dump：ESR_EL2、ELR_EL2、FAR_EL2、HPFAR_EL2、SPSR_EL2。
- 设置 HCR_EL2/CPTR_EL2/CNTHCTL_EL2 等最小安全基线。
- 验证 EL2 MMU 开/关策略并最终启用 Host stage-1 映射。
**退出条件：**
- QEMU 输出稳定启动日志
- 异常能打印完整 syndrome
- 连续重启无随机故障

### P2 — 平台发现与内存基础设施
**目标：** 从 DTB 构建 PlatformInfo，并拥有可靠物理页与内核对象分配器。

**任务：**
- 引入/实现 FDT parser，只解析所需节点与 cells/address/size/interrupt specifier。
- 解析 /cpus、/memory、/reserved-memory、/chosen、PSCI、timer、GIC、UART。
- 生成 PlatformInfo 与 PlatformCapabilities。
- 实现 boot memory map：可用 RAM、Hypervisor image、DTB、boot package、reserved ranges。
- 实现 page-frame allocator（先 bitmap/buddy 二选一）与 allocation ownership debug metadata。
- 实现 small object/slab allocator 或经过封装的 heap。
- 定义 MemoryObject/MemoryRegion 的最小数据结构。
- 增加 hv-platform-inspect 模式，输出拓扑与资源。
- 实现 DTB 静态 compatibility checker 的 host-side 原型。
**退出条件：**
- QEMU PlatformInfo 与实际启动参数一致
- 内存分配压力测试通过
- reserved 区域绝不被分配

### P3 — SMP 与 per-CPU 基础
**目标：** 早期消除单核假设。

**任务：**
- 通过 PSCI CPU_ON 启动 secondary CPUs。
- 建立每 CPU 独立栈、per-CPU data、CPU local ID 映射。
- 实现全局 barrier 与 boot rendezvous。
- 实现基础 spinlock、irq-save lock、atomic helpers；明确锁顺序。
- 建立 cross-CPU IPI/SGI primitive。
- 实现 TLB shootdown mailbox 框架。
- 验证 2/4/8 vCPU QEMU host SMP；Orange Pi 阶段验证 4×A55。
- 加入 lock contention 与 IPI latency trace event。
**退出条件：**
- 所有 pCPU 进入 online 状态
- 并发 allocator test 通过
- IPI/TLB mailbox 可回归

### P4 — Stage-2 与 Rust Validation Guest v0
**目标：** 首次安全进入 EL1 Guest。

**任务：**
- 实现 AArch64 Stage-2 page table builder、walker、map/unmap/protect。
- 封装 VTCR_EL2/VTTBR_EL2/TLBI 操作与 VMID allocator。
- 实现 GuestAddressSpace 创建与销毁。
- 实现 VM/VCPU 最小对象、寄存器初始化、EL1 entry。
- 建立 Rust no_std validation guest crate，提供 UART/semihost-like debug 通道。
- 加载 ELF 或 flat binary 到 Guest RAM。
- ERET 首次进入 Guest，并捕获返回 EL2 的同步异常。
- 实现 WFI/WFE、未知异常的基本处理。
- 加入 Stage-2 fault 详细 trace 与 page-table dump。
**退出条件：**
- Guest 打印 Hello from EL1
- 越界 IPA 可靠触发 Stage-2 fault
- 错误 Guest 不能破坏 EL2

### P5 — Hypercall、对象句柄与 Capability v0
**目标：** 建立后续管理与 IPC 的安全调用骨架。

**任务：**
- 定义 HVC ABI：调用号、寄存器约定、错误码、版本查询。
- 实现 copy_from_guest/copy_to_guest 安全访问 helper。
- 实现 object table + Handle(slot,generation)；防 stale handle。
- 实现 Rights bitset 与 capability lookup。
- 实现基础 capability grant（仅 bootstrap→domain）与 revoke 占位接口。
- Validation Guest 测试合法/非法 HVC、随机句柄、越界 buffer。
- 对 hypercall dispatcher 做 fuzzable host-side parser 分层。
**退出条件：**
- 非法 handle 不越权
- ABI version 可查询
- HVC fuzz 无 panic/越界

### P6 — Timer、GICv3 与虚拟中断 v0
**目标：** 建立可运行定时中断和外部中断的 Guest。

**任务：**
- 实现物理 GICv3 distributor/redistributor/CPU interface 初始化。
- 实现 SGI/PPI/SPI descriptor 与 routing。
- 实现 EL2 physical timer 与 per-CPU tick/event timer。
- 实现 vCPU virtual timer 保存/恢复。
- 实现 software vIRQ pending queue 和注入。
- 逐步利用 GIC virtualization interface/List Registers。
- 处理 maintenance interrupt。
- Validation Guest 测试周期 timer、SGI、IRQ priority/basic masking。
- 统计 physical IRQ→vIRQ latency。
**退出条件：**
- Guest timer 稳定运行
- 多 vCPU SGI 工作
- 中断风暴不丢失核心状态

### P7 — vCPU Scheduler v0 → preemptive
**目标：** 从静态绑定演进到真正 runnable vCPU。

**任务：**
- 定义 scheduler trait、RunQueue、SchedulingEntity。
- 先实现 pinned static scheduler，与现有行为等价。
- 加入 vCPU state machine：Runnable/Running/Blocked/Paused/Stopped。
- 实现 tick/preemption 与 context switch。
- 实现简单 round-robin 作为第一个 M:N scheduler。
- 实现 CPU affinity/pinning/cpuset。
- 实现 block on WFI/Notification 与 wakeup。
- 加入 runtime accounting、steal time、runqueue depth、switch reason trace。
- 多 VM CPU stress 与公平性测试。
**退出条件：**
- M>N vCPU 可稳定运行
- pause/resume 无竞态
- CPU affinity 生效

### P8 — 虚拟平台与 Linux 最小启动
**目标：** 定义 rusthv-arm-virt-v1 并启动定制 Linux。

**任务：**
- 冻结 v1 Guest IPA layout：RAM、GIC、UART/console、virtio-mmio windows、PCI 预留。
- 实现 Guest DTB builder/patcher。
- 实现 PSCI virtualization（CPU_ON/OFF/SYSTEM_OFF 等最小集）。
- 实现最小 virtual UART 或 console MMIO device。
- 加载 Linux Image、initramfs、DTB，设置 boot registers。
- 支持 Linux SMP secondary CPU bring-up。
- 实现必要系统寄存器 trap/emulation。
- 建立 Linux boot regression：到 earlycon、initramfs shell。
- 记录 rusthv-arm-virt-v1 machine ABI 文档。
**退出条件：**
- 定制 Linux 到 shell
- SMP Linux 识别期望 vCPU 数
- machine ABI 文档冻结

### P9 — Virtio Core + console/block v0
**目标：** 建立标准 para-virtual I/O 主线。

**任务：**
- 实现 Virtio feature negotiation、device status、queue abstraction。
- 实现 virtio-mmio transport。
- 实现 split virtqueue 安全 descriptor walker：环检测、长度上限、地址验证。
- 预留 packed virtqueue 接口。
- 实现 virtio-console 或 virtio-rng 作为最小设备。
- 实现内存-backed virtio-blk backend。
- 实现 Notification/doorbell 与 IRQ 注入。
- 建立 malicious descriptor 测试集与 fuzz harness。
- 加入 queue depth/bytes/latency telemetry。
**退出条件：**
- Linux 使用标准 virtio 驱动
- 坏 descriptor 不越界
- 块设备读写一致性测试通过

### P10 — Boot Package、Bootstrap Domain 与 Linux Control Domain
**目标：** 从“Hypervisor 自己加载所有 VM”转向管理域。

**任务：**
- 定义 Boot Manifest v1 与简单 in-memory Boot Package。
- 由 U-Boot/loader 负责把 Hypervisor、Control Domain image、initramfs/manifest 放入 RAM。
- 实现 Bootstrap Domain Creator。
- 给 Control Domain 分配初始 CPU/Memory/IRQ/Device/Management capabilities。
- 启动极简 Linux Control Domain。
- 实现 Control Domain 与 EL2 的 management endpoint/hypercall channel。
- 实现 create/destroy/pause/resume VM 的第一代 native management ABI。
- 实现从 Control Domain 加载第二个 Linux/validation VM image。
- 确认 EL2 不依赖通用文件系统。
**退出条件：**
- Control Domain 可动态创建/销毁 VM
- 重启普通 VM 不影响 Control Domain
- Boot Manifest 可版本检查

### P11 — 配置与管理数据模型 v1
**目标：** 把临时命令提升为可持久化/可版本化 Spec。

**任务：**
- 定义 VmSpec/CpuSpec/MemorySpec/BootSpec/DeviceSpec/SecuritySpec。
- 定义 schema_version 与 machine_version。
- Control Domain 支持 TOML/YAML 中一种人类配置格式。
- 将配置归一化后再调用 native ABI；EL2 不解析 YAML/XML。
- 实现 VM lifecycle state machine 与 operation transaction ID。
- 建立 desired vs actual state 数据模型（reconciler 可先简化）。
- 实现权限 policy 与 capability broker 雏形。
- 设计 audit log：谁在何时对何对象做了何操作。
**退出条件：**
- 配置可重复创建相同 VM
- 错误配置有结构化诊断
- 权限不足操作被拒绝并审计

### P12 — Service IPC、共享内存与外置 Virtio Backend
**目标：** 验证 EL2/Service Domain 可替换后端架构。

**任务：**
- 实现 Endpoint 小消息机制。
- 实现 Notification/event channel。
- 实现 SharedRegion 创建、映射、权限与生命周期。
- 实现 capability delegation 到 Service Domain。
- 定义 Backend Protocol v1：attach/detach/reset/queue-kick/completion。
- 将 virtio-blk backend 从 EL2 迁移为 Linux Service/Control Domain backend。
- 比较 EL2 backend 与 Service Domain backend 性能与 tail latency。
- 实现 backend crash 检测、device failed 状态与重连/reset。
**退出条件：**
- 同一前端可切换两种 backend
- Service Domain 崩溃不破坏 EL2
- 共享内存无越权映射

### P13 — 网络、PCI 与设备框架
**目标：** 从最小虚拟设备扩展到通用设备基础设施。

**任务：**
- 实现 virtio-net + Linux backend/TAP/bridge 路径。
- 实现 generic Device/Bus/MMIO region registry。
- 实现 PCI ECAM host/guest 基础模型。
- 实现 virtio-pci transport。
- 实现 MSI/MSI-X 到 virtual interrupt route 的基础框架。
- 引入 device reset/quiesce/suspend 状态机。
- 形成 Driver Registry：DT compatible/PCI ID→driver。
- 平台设备与虚拟设备分离命名空间。
**退出条件：**
- virtio-net 联网
- virtio-pci Linux driver 可用
- 设备 attach/detach 生命周期稳定

### P14 — IOMMU/SMMUv3 与设备直通
**目标：** 建立真正的 DMA 隔离路径。

**任务：**
- 定义 IommuDomain/DmaMapping/DeviceGroup 抽象。
- 在 QEMU virt iommu=smmuv3 上实现 SMMUv3 基础驱动。
- 实现 device→VM DMA domain attach/detach。
- 实现安全 DMA map/unmap 与 invalidation。
- 实现 passthrough IRQ/MSI route。
- 定义 trusted-direct 与 isolated-direct 配置检查。
- 实现 mediated/bounce-buffer 示例以支持无 IOMMU 平台实验。
- 设备 reset/ownership transfer 前必须 quiesce DMA。
- 加入恶意 DMA 配置测试与隔离回归。
**退出条件：**
- QEMU 安全 passthrough 路径可验证
- 无 IOMMU 时拒绝标记为 isolated
- 跨 VM DMA 被阻断

### P15 — Orange Pi 3B 真机移植
**目标：** 验证平台抽象，而不是为板卡重写 Hypervisor。

**任务：**
- 整理 TF-A/U-Boot→EL2 启动链与 boot artifacts。
- 解析 Orange Pi 3B 实际 DTB，验证 CPU/RAM/GIC/timer/PSCI/UART。
- 实现/复用 Rockchip early UART 与所需 SoC helper。
- 建立 rk356x SoC 层、orangepi3b BSP、quirks 文件。
- 验证 4× Cortex-A55 SMP 与 cache/TLB shootdown。
- 运行 validation guest 全套回归。
- 运行 Linux guest + virtio console/block/net（能实现的链路）。
- 评估 PCIe/NVMe/网卡可用路径，并明确无 IOMMU 条件下的安全限制。
- 建立串口抓取/远程电源控制后的硬件回归框架。
**退出条件：**
- Core 无 OrangePi 特判
- Validation Guest/ Linux 在真机稳定
- BSP 代码保持薄层

### P16 — 内存高级能力
**目标：** 为通用虚拟化引入共享、热插拔、balloon 与 snapshot 基础。

**任务：**
- 实现 memory hot-add/hot-remove 的 Hypervisor 对象模型。
- 实现 virtio-balloon 或等效 balloon 通道。
- 实现 page donation/reclaim 与 SharedRegion 高级权限变更。
- 实现 dirty/access bit 跟踪抽象。
- 实现 COW MemoryObject / snapshot layer。
- 实现 stop-the-world snapshot v1：vCPU+RAM+设备状态。
- 实现 snapshot 格式版本与完整性校验。
- 建立恢复到相同 machine type 的 regression。
- 研究 memory overcommit pressure policy；默认仍不做 EL2 swap。
**退出条件：**
- snapshot/restore 一致
- balloon 可动态回收
- 页 ownership checker 无泄漏

### P17 — Live Migration 与兼容性
**目标：** 验证 versioned machine ABI 的价值。

**任务：**
- 定义 migration stream/object schema。
- 实现 pre-copy RAM migration 与 dirty rounds。
- 实现 stop-and-copy vCPU/IRQ/device state。
- 实现 source/destination capability negotiation。
- 限制 v1 为相同 architecture + compatible machine type。
- 建立 failure rollback 与 source resume。
- 加入 migration bandwidth/downtime telemetry。
- 形成 machine compatibility policy。
**退出条件：**
- 可迁移运行中的 Linux VM
- 失败迁移可安全回滚
- downtime 可量化

### P18 — libvirt 与管理生态
**目标：** 使 Hypervisor 可被标准虚拟化工具管理。

**任务：**
- 稳定 management daemon 与 native RPC。
- 实现 libvirt hypervisor driver/adapter 的最小 domain lifecycle。
- 映射 CPU、Memory、disk、net、console、pause/resume/shutdown。
- 支持 libvirt XML → VmSpec。
- 逐步支持 storage/network/host device integration。
- 保持研究型 trace/capability API 仅在 native extension 中。
- 实现 CLI 与结构化状态查询。
**退出条件：**
- virsh 基本生命周期可用
- XML 配置可稳定映射
- native API 保持独立版本

### P19 — Rust 原生 Control Domain / Service Runtime
**目标：** 逐步减少对 Linux 管理域的强依赖。

**任务：**
- 定义 Rust service runtime 的最小 syscall/IPC/driver ABI。
- 实现 Rust management service 与配置数据库。
- 迁移简单 virtio backend。
- 按价值迁移 storage/network/driver 服务，不追求一次替换 Linux。
- 支持 Linux Control Domain 与 Rust Control Domain 并存。
- 对两种控制域进行性能、TCB、故障恢复对比。
**退出条件：**
- Rust 管理域能创建/管理 VM
- 可选择 Linux 或 Rust 控制域
- 服务接口保持兼容

### P20 — x86_64 架构后端
**目标：** 验证“共享语义、独立机制”的架构抽象。

**任务：**
- 建立 x86_64 boot/UEFI 路径与 VMX first（SVM 后续）。
- 实现 VMCS、EPT、VM-exit、APIC/x2APIC backend。
- 将 core Vm/Vcpu/GuestAddressSpace/Scheduler/Capability 无修改复用。
- 建立 ACPI/PCI platform discovery → PlatformInfo。
- QEMU x86_64 启动 validation guest 与 Linux。
- 定义 rusthv-x86-virt-v1 machine type。
- PC 兼容机真机 bring-up。
- 逐步增加 IOMMU（VT-d/AMD-Vi）与 passthrough。
**退出条件：**
- Core 不出现 VMX 特殊分支
- QEMU x86 Linux 启动
- 至少一台 PC 真机验证

### P21 — 高级安全、实时与研究特性
**目标：** 进入长期研究扩展期。

**任务：**
- 实现多 scheduler class 与 RT/partition scheduler。
- 建立 interrupt/timer worst-case latency benchmark。
- capability delegation tree 与高效 revoke。
- 安全启动、manifest/image 签名与测量。
- nested virtualization 原型。
- confidential/TEE integration 可行性研究。
- 形式化关键状态机/页 ownership/capability invariant。
- 持续缩减和审计 unsafe/TCB。
**退出条件：**
- 每项研究特性有独立实验报告
- 不破坏稳定 machine ABI
- 关键安全 invariant 可机械检查

## 16. 里程碑聚合

| 里程碑 | 覆盖阶段 | 可见成果 |
|---|---|---|
| M0 EL2 Alive | P0-P1 | Rust EL2 启动、异常与串口诊断 |
| M1 First Guest | P2-P4 | Rust EL1 Validation Guest + Stage-2 |
| M2 Virtual CPU Platform | P5-P7 | Capability/HVC、GICv3、timer、M:N 调度基础 |
| M3 Linux Boot | P8 | rusthv-arm-virt-v1 + SMP Linux |
| M4 Standard I/O | P9 | Virtio MMIO + block/console |
| M5 Managed Hypervisor | P10-P12 | Linux Control Domain、动态 VM、IPC、Service backend |
| M6 General I/O | P13-P14 | network/PCI/virtio-pci/SMMUv3/passthrough |
| M7 Real Hardware | P15 | Orange Pi 3B 稳定运行 |
| M8 Advanced Memory | P16-P17 | balloon/snapshot/live migration |
| M9 Ecosystem | P18-P19 | libvirt + Rust Control Domain |
| M10 Multi-Arch | P20 | x86_64 QEMU + PC |
| M11 Research Platform | P21+ | RT、安全、nested、formal 等 |

## 17. 当前明确不做（v0.x 非目标）

- 第一阶段同时支持 AArch64 与 x86_64。
- 在 EL2 中实现通用 VFS/ext4/qcow2/NVMe/完整网络协议栈。
- 第一版就实现 Windows、传统 PC 全设备模拟或 nested virtualization。
- 在无 IOMMU 的平台上宣称不可信 Guest 的 direct passthrough 具备强 DMA 隔离。
- 把 Control Domain 固定为永远唯一的特权 VM，或以 VM ID 代替权限模型。
- 为了“统一抽象”把 ARM 寄存器/VMX 控件强行映射到一个泄漏细节的 God trait。
- 第一版就追求 live migration、memory swap、NUMA、SR-IOV、GPU 虚拟化同时完成。

## 18. 尚待 ADR 冻结的近期问题

- 正式项目名与 crate 命名前缀。
- P2 物理页 allocator 首选 buddy 还是 bitmap+size-class 组合。
- Native Management ABI v1 wire format。
- 第一代 preemptive scheduler 的默认算法。
- rusthv-arm-virt-v1 具体 IPA map、virtio-mmio slot 数量、GIC/PCI 窗口。
- 首个 Linux Control Domain rootfs/发行版及管理 agent 形态。
- EL2 实验性 virtio backend 在 general/release profile 中的默认启用策略。
- Orange Pi 3B 实际 boot chain、UART/PCIe/NVMe 路径在真机勘测后的 quirk 列表。

## 19. 架构不变量（Implementation Invariants）

- **MUST:** Hypervisor Core 不得依赖具体 Board 名称。
- **MUST:** Arch 层不得依赖 SoC/Board。
- **MUST:** Guest 提供的地址、长度和索引未经验证不得用于内存访问。
- **MUST:** 除显式 SharedRegion 外，不可信 VM 之间不得共享 writable HPA。
- **MUST:** 设备转移 ownership 前必须完成 quiesce/reset/DMA detach。
- **MUST:** Capability 校验失败不能退化为角色/VM ID 的隐式放行。
- **MUST:** EL2 不保存用户密码、TLS 私钥或通用 RBAC 数据库。
- **MUST:** Machine ABI、snapshot、migration、management ABI 必须带版本。
- **MUST:** Guest-caused fault 默认只能影响对应 VM/设备上下文，不应触发全局 panic。
- **MUST:** 任何跨 CPU 修改 Stage-2/IRQ route 的操作必须定义同步与 invalidation 语义。

## 20. 参考规范与实现资料

- **QEMU Arm virt machine** — https://www.qemu.org/docs/master/system/arm/virt
- **OASIS Virtio 1.4** — https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.html
- **libvirt API / driver model** — https://www.libvirt.org/api.html
- **Arm Architecture / GIC / SMMU / PSCI** — Arm Developer architecture specifications（实施时锁定具体 revision）
- **Linux/KVM ARM virtualization** — 用于 vGIC、PSCI、timer、Stage-2 行为交叉验证，不作为代码复制来源
- **Xen / seL4 / Bao / Jailhouse / ACRN / Firecracker / Cloud Hypervisor** — 作为管理域、capability、静态分区、设备模型和工程组织的对照参考

---
**v0.1 变更规则：** 已确定 ADR 若被推翻，应新增 ADR 标注 `Supersedes ADR-xxx`，不直接静默修改历史决策。研发阶段允许拆分/合并，但必须保持退出条件可验证。
