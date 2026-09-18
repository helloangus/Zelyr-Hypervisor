# Zelyr Hypervisor Plan Agent Design Guidelines v0.1 — Detailed Reference

**Status:** Normative detailed reference (routed from the mandatory concise guide).  
**Version:** v0.1  
**Owner/change context:** P0 engineering baseline; updated when planning policy changes.  
**Supersedes:** None.

> **Use:** This is the complete, topic-indexed reference. Every Plan Agent
> must first read [the mandatory concise guide](plan-agent-guidelines.md), then
> read the relevant sections of this document when its task triggers them.
> The concise guide does not weaken this document's requirements.

## 1. 文档目的

本准则用于约束 Plan Agent 在执行某一阶段任务书时，如何把已经确定的架构决策、阶段目标和实现方向进一步细化为：

- 子系统；
- 模块；
- 核心对象；
- 数据结构职责；
- 模块间接口；
- 函数及方法的功能职责；
- 状态机；
- 数据流；
- 控制流；
- 生命周期；
- 并发与同步语义；
- 错误语义；
- 权限与资源边界；
- 初始化与销毁顺序；
- 验收标准。

本准则只规范“设计什么”和“各部分应承担什么职责”。

本准则**不规定具体 coding 实践**，包括但不限于：

- Rust 语法风格；
- 命名风格；
- 格式化规则；
- 注释格式；
- `unsafe` 的具体编码方式；
- 使用何种迭代器、宏或语言技巧；
- 具体锁类型的代码写法；
- 文件中的代码排版；
- 单元测试代码风格。

Plan Agent 的产出是后续 Coding Agent 可以直接实现的**实现级设计文档**，而不是源代码。

---

# 2. Plan Agent 的职责边界

Plan Agent 的职责是：

> 在既定 Architecture ADR 和当前阶段任务书约束下，把阶段目标转换为完整、闭合、可实现、可验证的模块和函数设计。

Plan Agent 不负责重新定义整个 Hypervisor 的总体架构。

Plan Agent 必须遵循以下优先级：

```text
Architecture ADR
    >
当前阶段任务书
    >
已经冻结的接口 / ABI / Machine Model
    >
此前阶段已经实现并稳定的模块契约
    >
当前阶段内部的设计自由度
```

若低优先级设计与高优先级约束冲突：

```text
不得自行修改高优先级设计。
```

必须明确记录冲突，并将其作为：

```text
Architecture Change Request
```

或：

```text
ADR Required
```

提交，而不是在阶段设计中静默改变架构。

---

# 3. Plan Agent 不得做的事情

Plan Agent 在普通阶段设计任务中不得：

1. 私自修改已确定的 Architecture ADR。
2. 因为当前阶段实现方便而破坏长期预留接口。
3. 把 Stage 任务书中没有要求的高级功能大规模提前实现。
4. 把暂时实现方案写成永久架构约束。
5. 为某个平台方便而让 Hypervisor Core 直接依赖具体 Board。
6. 为某个 Guest 特例破坏通用 VM 模型。
7. 以固定 VM ID 代替 capability 权限模型。
8. 把 Control Domain 角色等同于无限权限。
9. 为了减少模块数量而建立跨层 God Object。
10. 为了追求所谓“统一接口”而把完全不同的硬件机制错误抽象成同一寄存器级接口。
11. 把 Host platform model 与 Guest virtual machine model 混合。
12. 把配置格式直接变成 Hypervisor 内部机制接口。
13. 把当前测试环境 QEMU 的行为当作硬件架构保证。
14. 把临时 stub/mock 的限制传播成正式 API 语义。
15. 因为当前阶段只运行单 VM、单 vCPU，而设计只能支持单 VM、单 vCPU 的数据模型。

---

# 4. 设计工作的基本原则

## 4.1 先定义责任，再定义接口

任何模块设计必须首先回答：

```text
这个模块负责什么？
不负责什么？
```

然后才能设计接口。

禁止先罗列函数，再反推模块职责。

每个模块都应有：

```text
Responsibilities
Non-responsibilities
Inputs
Outputs
Owned state
External dependencies
Lifecycle
Failure boundary
```

---

## 4.2 机制与策略分离

Hypervisor Core 优先提供：

```text
Mechanism
```

例如：

- 创建地址空间；
- 映射内存；
- 注入虚拟中断；
- 创建 vCPU；
- 修改 capability；
- 调度实体入队；
- 建立共享内存；
- 创建通知对象。

而以下内容通常属于：

```text
Policy
```

例如：

- 哪个 VM 获得多少内存；
- 哪个 VM 优先运行；
- 是否允许设备直通；
- 用户是否有 VM 管理权限；
- 何时 balloon；
- 选择哪个存储镜像；
- 哪个 VM 使用哪个网络。

Plan Agent 设计接口时，应尽量避免 Core API 内嵌高级策略。

---

# 5. 阶段设计必须严格受 Stage Scope 控制

每次 Plan Agent 收到阶段任务书后，首先必须将任务拆成三类。

## 5.1 本阶段必须完成

即：

```text
Required
```

必须存在完整设计并有验收条件。

---

## 5.2 本阶段必须预留，但不实现

即：

```text
Reserved
```

例如 P4 Stage-2 阶段可能需要考虑：

```text
dirty tracking
huge page
snapshot
migration
```

但本阶段不得因此实现完整 snapshot subsystem。

正确做法是确保：

```text
当前接口不会阻止未来实现。
```

---

## 5.3 本阶段明确不涉及

即：

```text
Out of Scope
```

必须在设计文档中明确写出。

例如：

```text
P4 不设计 Linux boot。
P6 不设计 ITS/LPI 完整实现。
P9 不设计 live migration。
```

这样可以阻止 scope creep。

---

# 6. 每个阶段必须建立需求追踪关系

阶段任务书中的每一条任务最终都必须能够追踪到：

```text
Stage Requirement
      ↓
Subsystem
      ↓
Module
      ↓
Object / State
      ↓
Function / Operation
      ↓
Validation
```

Plan Agent 应建立类似：

| Requirement | Module | Operation | Validation |
|---|---|---|---|
| Stage-2 map | guest_memory | map_region | mapping test |
| Stage-2 fault | exception | handle_stage2_fault | invalid IPA test |
| EL1 entry | vcpu | run_vcpu | validation guest |

任何 Stage Requirement 如果无法追踪到具体模块和验收方法，则设计尚未闭合。

---

# 7. 模块设计规则

## 7.1 模块必须有单一清晰职责

模块不要求极小，但必须具有明确领域边界。

例如：

```text
guest_memory
```

可以负责：

- GuestAddressSpace；
- Stage-2 mapping orchestration；
- MemoryRegion；
- 地址空间生命周期。

但不应该同时负责：

- vCPU scheduler；
- GIC；
- VM 配置解析；
- UART driver。

---

## 7.2 不以代码文件数量作为模块划分依据

Plan Agent 设计的是：

```text
logical module
```

而不是：

```text
.rs 文件数量
```

模块未来可能对应：

- 一个 crate；
- 一个 Rust module；
- 多个文件；
- 一个完整 subsystem。

此时只需要定义逻辑边界。

---

## 7.3 模块必须声明 owner state

必须明确：

> 哪个模块拥有哪一份状态？

例如：

```text
VM Manager
owns VM lifecycle state

Scheduler
owns runnable queue state

GuestAddressSpace
owns mapping metadata

IRQ subsystem
owns physical→virtual routing state

Capability subsystem
owns authority relation
```

禁止两套模块分别维护同一状态的 authoritative copy。

---

# 8. 核心对象设计规则

Plan Agent 设计对象时必须区分：

```text
Identity
State
Ownership
Reference
Capability
Role
```

例如：

```text
VmId
```

只表示身份。

```text
VmHandle
```

表示可引用对象。

```text
Capability<Vm>
```

表示有权访问某 VM。

```text
DomainRole::Control
```

表示角色。

四者不得混为一谈。

---

# 9. 对象之间必须明确 ownership

所有重要对象必须说明：

```text
谁创建？
谁拥有？
谁引用？
谁可以销毁？
销毁前需要解除什么关系？
```

例如 Vcpu：

```text
VM owns Vcpu
Scheduler temporarily references runnable Vcpu
PhysicalCpu references current Vcpu
Capability may reference Vcpu
```

VM 销毁前必须：

```text
stop vCPUs
remove from scheduler
remove interrupt routes
release address space
revoke capabilities
destroy Vcpu objects
```

这种生命周期顺序必须由 Plan Agent 明确。

---

# 10. 生命周期必须设计成显式状态机

所有具有复杂生命周期的对象不得只设计成若干独立操作。

至少包括：

```text
VM
vCPU
VirtualDevice
AssignedDevice
Backend
Migration
Snapshot
Domain
```

需要定义：

```text
State
Allowed transition
Trigger
Precondition
Side effects
Failure behavior
Rollback
```

例如：

```text
Paused → Running
```

必须说明：

```text
Preconditions:
- VM address space valid
- required vCPU online
- devices ready

Side effects:
- enqueue runnable vCPUs

Failure:
- remain Paused
```

---

# 11. 函数设计原则

Plan Agent 设计到函数级别时，不需要写函数实现，但必须定义函数契约。

每个重要函数至少说明：

```text
Purpose
Inputs
Outputs
Preconditions
Postconditions
State changes
Ownership changes
Concurrency assumptions
Possible errors
Side effects
Security checks
```

例如：

```text
map_memory(vm, memory_object, ipa, size, permissions)
```

不能只写：

> 映射内存。

而应该说明：

```text
Purpose:
将 MemoryObject 指定范围映射到 GuestAddressSpace。

Preconditions:
- caller owns mapping capability
- IPA aligned
- range not overlap incompatible region
- MemoryObject pages valid
- pages not assigned to conflicting owner

Postconditions:
- mapping metadata exists
- Stage-2 entry visible
- required TLB invalidation completed

Failure:
- address conflict
- permission denied
- out of memory
- invalid alignment
```

---

# 12. 函数应以语义操作为中心

优先设计：

```text
map_region()
inject_interrupt()
create_vcpu()
assign_device()
pause_vm()
grant_capability()
```

而不是把硬件操作直接泄漏到上层：

```text
write_vttbr()
write_ich_lr()
modify_vmcs_field()
```

后一类只应存在于 architecture backend 内部。

---

# 13. 不允许 Boolean Hell

Plan Agent 不应设计类似：

```text
create_vm(
    secure,
    passthrough,
    dynamic,
    pinned,
    debug,
    ...
)
```

如果这些参数代表独立语义，应设计成：

```text
VmSpec
SecurityPolicy
CpuPolicy
DeviceAssignment
```

或独立 operation。

此规则属于 API 设计，而非 coding 风格。

---

# 14. 架构无关层只表达架构无关语义

Core 可以设计：

```text
GuestAddressSpace
VirtualInterrupt
VcpuContext
ExitReason
VirtualTimer
CpuAffinity
```

但不得要求：

```text
VTTBR_EL2
ICH_LR
VMCS
EPTP
APIC register
```

上层 API 不得泄漏硬件实现。

Architecture backend 负责把统一语义翻译为具体机制。

---

# 15. 不为跨架构统一而制造虚假抽象

Plan Agent 必须接受：

```text
共享语义
≠
所有底层机制必须共用相同函数
```

如果 AArch64 与 x86_64 的机制明显不同，可以在 architecture backend 内拥有完全不同的内部结构。

只有真正跨架构存在的语义才进入公共接口。

---

# 16. 平台适配设计规则

平台相关设计必须遵循：

```text
Architecture
↓
Platform Discovery
↓
SoC
↓
Reusable Driver
↓
Board BSP
↓
Quirk
```

Core 不得依赖具体 Board。

Plan Agent 在设计平台功能时应优先判断：

```text
这是不是 DTB/ACPI 可以描述的信息？
```

如果可以，就优先进入：

```text
PlatformInfo
```

而不是 Board API。

---

# 17. Board BSP 必须保持薄

Board BSP 仅应承载：

- 板级启动差异；
- firmware 差异；
- 特定接线；
- 无法从标准描述获得的信息；
- 已知硬件/firmware quirk。

禁止把可复用：

- UART；
- GIC；
- PCI；
- SMMU；
- timer

驱动复制到 Board 层。

---

# 18. 平台能力必须 capability-driven

上层不得写：

```text
if platform == QEMU
```

而应依赖类似：

```text
supports_iommu
supports_pci
supports_gicv3
supports_msi
```

Plan Agent 设计 platform API 时应优先表达能力，而不是平台名称。

---

# 19. VM Model 与 Host Platform 严格分离

Plan Agent 必须始终区分：

```text
Host Platform
```

和：

```text
Guest Machine Type
```

例如：

```text
Orange Pi 3B
```

是 Host Platform。

```text
rusthv-arm-virt-v1
```

是 Guest Machine Type。

Guest 不得因为 Host 是 RK3566 就看到 RK3566 虚拟硬件。

---

# 20. CPU / vCPU 设计原则

## 20.1 PhysicalCpu 和 Vcpu 必须分离

PhysicalCpu 表示真实 CPU。

Vcpu 表示：

```text
schedulable guest execution context
```

禁止假设：

```text
Vcpu lifetime == PhysicalCpu lifetime
```

---

## 20.2 调度器只调度 Vcpu

VM 不是 scheduler entity。

正常关系：

```text
Vm
 └─ Vcpu[*]
      └─ SchedulingEntity
```

---

## 20.3 静态绑定只是一种调度策略

第一阶段：

```text
1 Vcpu ↔ 1 pCPU
```

不能导致 API 被设计成：

```text
Vcpu permanently contains PhysicalCpu
```

应该允许未来：

```text
Vcpu
  ↓ scheduled onto
PhysicalCpu
```

---

# 21. vCPU Exit 设计

架构无关层只能看到：

```text
ExitReason
```

例如：

- Hypercall；
- Stage2Fault；
- MMIO；
- SysReg；
- WFI/WFE；
- Interrupt；
- Shutdown；
- Fatal guest state。

Architecture backend 负责 ESR_EL2/VMCS exit reason 解码。

---

# 22. 内存设计原则

必须严格区分：

```text
Host physical memory management
Guest address-space management
Memory ownership
Memory mapping
```

四者不得合并成一个 MemoryManager God Object。

---

# 23. 内存所有权先于映射

某物理页能否被映射，首先取决于：

```text
ownership / sharing authority
```

然后才是：

```text
page table mapping
```

禁止把“能够生成 Stage-2 PTE”等同于“有权映射该页”。

---

# 24. GuestAddressSpace 是核心长期对象

任何 Stage-2 设计必须通过：

```text
GuestAddressSpace
```

表达。

不能让 VM Core 到处直接操作：

```text
page-table pages
VTTBR_EL2
```

这样以后才能映射到：

```text
EPT
NPT
```

---

# 25. MemoryObject 与 MemoryRegion 分离

推荐语义：

```text
MemoryObject
= 内存资源 / backing / ownership

MemoryRegion
= 某地址空间中的映射视图
```

这样才能自然支持：

```text
SharedRegion
COW
snapshot
balloon
hotplug
```

---

# 26. Snapshot 和 Migration 不得反向污染早期路径

Plan Agent 在 P4/P8 等早期阶段只需保证：

- vCPU state 可序列化；
- Device state 有明确 owner；
- mapping metadata 可枚举；
- machine model 有版本；
- dirty tracking 可以未来加入。

不得为了未来 migration 提前实现完整 migration engine。

---

# 27. 中断设计原则

必须区分：

```text
Physical Interrupt
Interrupt Route
Virtual Interrupt
Guest-visible Interrupt Controller
```

禁止把：

```text
physical IRQ number == guest IRQ number
```

作为长期架构假设。

---

# 28. 中断路由是显式对象或状态

需要能够表达：

```text
physical IRQ
    ↓
route
    ↓
EL2 handler

或

physical IRQ
    ↓
Service Domain notification

或

physical IRQ
    ↓
Virtual IRQ
    ↓
target vCPU
```

这对于：

- passthrough；
- service domain；
- migration；
- IRQ affinity

都是必要的。

---

# 29. Timer 属于 vCPU 虚拟状态

Virtual timer 不应只是 GIC 子模块的附属字段。

需要设计：

```text
VirtualTimerState
```

随 Vcpu：

```text
save
restore
block
wake
migrate
```

---

# 30. 设备设计必须拆分 Frontend 与 Backend

Guest-visible device state：

```text
VirtualDevice
```

真实 I/O 实现：

```text
DeviceBackend
```

二者禁止强耦合。

例如：

```text
VirtioBlockDevice
```

不应该直接知道 ext4/NVMe/qcow2。

它只处理：

```text
block requests
```

---

# 31. Backend 必须可替换

同一种 VirtualDevice 应能够连接：

```text
EL2 backend
Service Domain backend
memory backend
physical device backend
```

这既是架构需求，也是研究性能的重要前提。

---

# 32. Virtio Descriptor 是安全边界

Plan Agent 在设计 virtio 时必须明确：

```text
descriptor parsing
地址验证
长度验证
链长度限制
循环检测
读写权限
indirect descriptor
```

这些不是实现细节，而是协议安全设计的一部分。

---

# 33. DMA 必须独立于 CPU 地址转换考虑

Plan Agent 不得认为：

```text
Stage-2 MMU
```

可以保护 device DMA。

任何 direct passthrough 设计必须回答：

```text
DMA translation 由谁执行？
```

如果没有 IOMMU：

```text
不得将其描述为 isolated direct passthrough。
```

---

# 34. Device Assignment 必须是事务

设备从 A 转移到 B 不能只是：

```text
owner = B
```

必须定义：

```text
stop/quiesce
disable IRQ
stop DMA
detach IOMMU
reset
change ownership
attach new IOMMU domain
configure IRQ
start
```

并设计失败回滚。

---

# 35. Capability 设计原则

Capability 表示：

```text
对对象执行某些操作的权力
```

而不是：

```text
对象本身的身份
```

必须至少允许表达：

```text
object reference
rights
generation
```

---

# 36. Role 不等于 Capability

例如：

```text
DomainRole::Control
```

只表示逻辑角色。

不能自动意味着：

```text
all permissions
```

权限必须来自显式 capability。

---

# 37. Authentication 不进入 EL2

Plan Agent 设计管理架构时：

```text
password
token
certificate
RBAC
user database
```

都属于 Control Domain。

EL2 仅处理：

```text
authenticated management entity
↓
capability
↓
operation authorization
```

---

# 38. Capability 委派必须遵循权限缩减

未来如果：

```text
Control Domain
```

向：

```text
Monitoring Domain
```

委派权限：

```text
READ_STATS
```

新 capability 不得自动获得：

```text
DESTROY
DEVICE_ASSIGN
```

即 delegation 必须支持：

```text
rights attenuation
```

---

# 39. IPC 必须建立在少量原语上

Hypervisor Core 的原生 IPC 原语保持少量：

```text
Endpoint
Notification
SharedRegion
Capability
```

不要为：

```text
virtio
management
driver
storage
network
```

分别设计完全独立的 IPC substrate。

---

# 40. 控制面与数据面允许使用不同模式

推荐：

```text
Control Plane
Endpoint / small message

Data Plane
SharedRegion + Notification
```

这不是两个独立 IPC 子系统。

它们必须共享：

```text
ownership
capability
lifecycle
isolation
```

模型。

---

# 41. 跨 Domain 接口必须可失败

任何 Service Domain RPC 都必须假设：

```text
peer may crash
peer may timeout
peer may send invalid response
peer may restart
```

所以接口设计不能依赖：

```text
call always succeeds
```

---

# 42. 配置体系设计原则

必须区分：

```text
Build Configuration
Boot Configuration
Bootstrap Domain Configuration
Control Domain Configuration
VM Configuration
Runtime Policy
```

禁止把这些合并为一个“大配置文件模型”。

---

# 43. EL2 不接受高级配置格式作为核心 ABI

EL2 不直接消费：

```text
YAML
XML
libvirt XML
JSON
```

这些由 Control Domain 解析为：

```text
VmSpec
```

再转换成原生 Hypervisor operation。

---

# 44. VmSpec 是声明，不是内部对象引用

VmSpec 描述：

```text
用户希望创建什么 VM
```

而不是：

```text
当前 VM 内部对象状态
```

例如：

```text
memory.size = 4GiB
```

不能直接等价于：

```text
4GiB HPA 已分配
```

---

# 45. Desired State 与 Actual State 分离

Control Plane 长期必须允许：

```text
Desired:
VM memory = 8 GiB

Actual:
VM memory = 6 GiB
hotplug pending
```

所以 Plan Agent 不应将用户配置文件直接当作实际运行状态。

---

# 46. Bootstrap Domain 单独设计

普通 VM 由 Control Domain 创建。

Control Domain 本身则由：

```text
Hypervisor Boot / Bootstrap Manager
```

创建。

这属于不同生命周期。

不要试图用“Control Domain 自己创建自己”的统一模型消除 bootstrap。

---

# 47. Control Domain 不是普通管理进程的简单放大

Plan Agent 需要区分：

```text
Control Domain VM
Management Service
Policy Engine
Storage Service
Network Service
Capability Broker
libvirt adapter
```

它们可以最初部署在同一个 Linux VM 内，但逻辑职责要分开。

否则以后迁移到多个 Service Domain 时会很困难。

---

# 48. Native Management ABI 设计原则

Native ABI 必须表达 Hypervisor 原生能力，而不是围绕 libvirt 设计。

例如可以有：

```text
CreateVm
CreateVcpu
CreateMemoryObject
MapMemory
AssignDevice
ReadTrace
DelegateCapability
```

libvirt 只是：

```text
adapter
```

---

# 49. ABI 必须版本化

任何跨隔离边界接口都必须考虑：

```text
version
structure size
feature negotiation
unknown field handling
```

至少包括：

```text
HVC ABI
Management ABI
Backend Protocol
Machine Type
Snapshot Format
Migration Protocol
```

---

# 50. Telemetry 是正式接口，不是 debug print

Plan Agent 设计 subsystem 时应同时定义关键 telemetry。

例如 Stage-2：

```text
fault count
map/unmap count
TLB shootdown
huge-page split
```

Scheduler：

```text
run time
switch count
blocked time
queue latency
```

Device：

```text
requests
bytes
latency
errors
```

---

# 51. Debug 能力不得侵入核心语义

例如：

```text
page table dump
IRQ dump
VM register dump
```

可以作为 diagnostic interface。

不得让正常运行依赖 Debug Monitor 存在。

---

# 52. 并发设计是模块设计的一部分

Plan Agent 不能把同步完全留给 Coding Agent 决定。

设计至少需要明确：

```text
哪些状态可以并发访问？
谁拥有写权限？
是否允许跨 CPU 修改？
是否需要 rendezvous？
是否需要 TLB shootdown？
是否需要 stop-the-world？
```

但不要求指定：

```text
Rust 中具体使用哪种锁类型
```

除非阶段任务本身要求。

---

# 53. Locking 应描述语义而不是代码类型

例如应该写：

```text
GuestAddressSpace mapping update requires exclusive modification serialization.
```

而不是必须写：

```text
使用某某具体 Rust Mutex。
```

后者属于 coding/implementation choice。

---

# 54. 跨 CPU 操作必须设计同步协议

尤其：

- Stage-2 修改；
- IRQ route 修改；
- VM stop；
- snapshot；
- vCPU migration；
- scheduler reconfiguration。

必须明确：

```text
request
acknowledgement
completion
visibility
```

语义。

---

# 55. 错误必须按来源分类

设计时至少区分：

```text
Guest error
Caller error
Permission error
Resource exhaustion
Device/backend failure
Hardware failure
Hypervisor invariant violation
```

不能所有错误都设计成：

```text
Error
```

---

# 56. Guest 错误不得默认升级为 Hypervisor Fatal

例如：

```text
invalid virtio descriptor
invalid HVC
invalid IPA
illegal guest register state
```

默认应该：

```text
reject operation
inject guest fault
fail device
stop affected VM
```

而不是让整个 Hypervisor panic。

只有真正 Hypervisor invariant 被破坏才属于 global fatal 候选。

---

# 57. Resource Exhaustion 必须有确定语义

例如：

```text
no physical memory
no VMID
no IRQ vector
no capability slot
runqueue full
```

必须定义：

```text
operation fails cleanly
```

不能设计成“理论上不会发生”。

---

# 58. 初始化过程必须显式分阶段

不能只设计：

```text
hypervisor_init()
```

然后内部什么都做。

至少应有逻辑阶段：

```text
early boot
architecture init
platform discovery
memory init
interrupt init
SMP init
core subsystem init
bootstrap resource init
guest/domain creation
runtime
```

---

# 59. 初始化依赖必须是有向图

Plan Agent 应明确：

```text
A depends on B
```

例如：

```text
PageAllocator
  before
VM creation

GIC
  before
Guest IRQ

Scheduler
  before
multi-vCPU runtime
```

禁止形成：

```text
A init needs B
B init needs A
```

循环初始化。

---

# 60. Shutdown / Destruction 与 Initialization 同样重要

每个动态对象必须设计：

```text
create
activate
quiesce
destroy
```

至少 VM、vCPU、MemoryObject、VirtualDevice、AssignedDevice、SharedRegion、Endpoint 都必须考虑销毁路径。

不得只设计 happy-path creation。

---

# 61. Plan Agent 必须设计失败回滚

多步骤 operation，例如：

```text
CreateVm
AssignDevice
StartVm
HotAddMemory
```

需要说明中途失败时：

```text
已经完成的步骤如何撤销？
```

例如：

```text
CreateVm:
allocate VM
allocate memory
create vCPU
map memory
attach device
```

如果 attach device 失败：

```text
不能留下半创建的无主 VM。
```

---

# 62. 原子 operation 与长事务必须区分

某些操作可以是：

```text
atomic synchronous operation
```

例如：

```text
map one region
```

某些管理操作应该是：

```text
transaction / async operation
```

例如：

```text
migration
large snapshot
device detach
```

Plan Agent 需明确。

---

# 63. 测试需求属于设计闭环，但测试代码不属于本准则

每个模块/operation 至少应设计对应验证方式：

```text
normal path
boundary
invalid input
resource failure
concurrency
repeated lifecycle
fault recovery
```

但 Plan Agent 不需要规定具体测试代码怎么写。

---

# 64. Validation Guest 是核心验证工具

涉及：

- vCPU；
- exception；
- Stage-2；
- HVC；
- IRQ；
- timer；
- SMP；
- MMIO；

的设计，优先考虑是否能用：

```text
Rust Validation Guest
```

构造确定性测试。

不要一开始依赖 Linux 才能验证底层机制。

---

# 65. Linux Guest 用于系统级验证

Linux 主要用于验证：

```text
标准 guest ABI
SMP
PSCI
GIC
virtio
PCI
real workload
```

不是底层机制的唯一调试手段。

---

# 66. QEMU 与真实硬件承担不同验证职责

QEMU reference platform 用于：

```text
deterministic test
CI
fault injection
large CPU/RAM configurations
SMMUv3
repeatability
```

Orange Pi 3B 用于：

```text
real cache/TLB
firmware
real GIC behavior
real SMP
real MMIO
real PCIe/device behavior
```

Plan Agent 在设计阶段验收时必须标注：

```text
QEMU-only
hardware-required
both
```

---

# 67. 不能把 QEMU 行为当作架构规范

如果某机制依赖：

```text
QEMU-specific implementation detail
```

必须明确标注。

架构层行为优先来源于：

```text
Arm architecture specification
Virtio specification
PCI specification
相关正式 ABI
```

---

# 68. 阶段设计不能提前承诺未经验证的性能结论

例如：

```text
EL2 virtio backend 一定比 Service Domain 快
```

不能作为架构事实。

正确表达是：

```text
两种 backend 都被架构支持；
在后续 benchmark 阶段决定默认部署策略。
```

---

# 69. “先实现简单版本”必须写明演进点

例如 Scheduler：

```text
v0 = static pinned
```

必须同时明确：

```text
哪些对象和接口必须允许未来 M:N。
```

但不需要现在实现 M:N。

---

# 70. Temporary Implementation 不得变成 Semantic Contract

例如第一版：

```text
only 1 VM
```

只能是：

```text
implementation limitation
```

不能写成：

```text
VmManager assumes exactly one VM.
```

类似：

```text
only 1 vCPU
fixed RAM address
single virtqueue
single interrupt
```

均遵循此原则。

---

# 71. 设计文档必须区分 Public Contract 与 Internal Detail

每个模块接口需要标出：

```text
Stable/External Contract
Internal Interface
Temporary/Internal-only
```

避免后续把实验性内部接口当成 ABI。

---

# 72. 数据模型优先于操作堆叠

如果某功能需要大量：

```text
set_xxx()
get_xxx()
enable_xxx()
disable_xxx()
```

Plan Agent 应先确认是否缺少一个正式对象或状态模型。

例如：

```text
DeviceAssignment
MigrationSession
InterruptRoute
GuestAddressSpace
```

通常比几十个无关联操作更稳定。

---

# 73. 避免 Global Manager 无限制膨胀

允许：

```text
VmRegistry
ResourcePool
PlatformRegistry
```

但其职责必须受限。

禁止出现：

```text
HypervisorManager
```

同时负责：

```text
VM
memory
device
interrupt
scheduler
config
IPC
```

所有业务逻辑。

---

# 74. Manager 不应拥有被管理对象的全部实现逻辑

例如：

```text
VmManager
```

负责：

```text
VM registration
lookup
lifecycle orchestration
```

但：

```text
Stage-2 implementation
vCPU context
device backend
```

仍应属于对应 subsystem。

---

# 75. Registry 与 Ownership 必须区分

一个对象存在：

```text
registry entry
```

不代表 registry 拥有它的全部资源生命周期。

Plan Agent 必须明确：

```text
registry is index/reference
```

还是：

```text
registry is owner
```

---

# 76. 异步事件必须有来源和目标

例如：

```text
Notification
IRQ
timer
backend completion
```

必须明确：

```text
Producer
Consumer
Delivery guarantee
Coalescing semantics
Loss semantics
Wakeup semantics
```

不能简单写“发送通知”。

---

# 77. Queue 需要定义容量语义

任何：

```text
pending IRQ queue
message queue
runqueue
virtqueue proxy
event queue
```

都需要说明：

```text
bounded/unbounded
overflow behavior
backpressure
drop/coalesce/block
```

这属于设计，而非 coding。

---

# 78. 设备和 IPC 的 Shared Memory 必须定义一致性模型

Plan Agent 至少要明确：

```text
谁写哪些字段？
谁读哪些字段？
状态所有权如何转移？
何时认为更新可见？
Notification 在状态更新前还是后？
```

不必规定 Rust atomic API，但协议语义必须明确。

---

# 79. Hypervisor 配置必须遵守“能力存在”和“能力使用”分离

例如：

```text
Virtio support compiled in
```

表示：

```text
Binary Capability
```

某 VM 使用 virtio-net：

```text
Runtime Configuration
```

两者不得混淆。

---

# 80. Profile 不是 Architecture Fork

例如：

```text
minimal
research
secure
general
```

只能选择：

```text
features/default policy
```

不能形成四套不同 Core 架构。

---

# 81. Machine Type 必须稳定表达 Guest-visible ABI

`rusthv-arm-virt-v1` 需要冻结：

- Guest memory layout；
- interrupt controller ABI；
- timer；
- boot protocol；
- virtual device layout；
- PCI window；
- firmware expectations；
- supported migration state。

内部实现可改变。

Guest-visible ABI 不能随意改变。

---

# 82. Machine Type Version 与 Hypervisor Version 分离

例如：

```text
Hypervisor 0.8
```

仍应可以运行：

```text
rusthv-arm-virt-v1
```

不能默认：

```text
Hypervisor version == machine ABI version
```

---

# 83. 架构预留必须尽量体现在数据模型，而不是空函数

不推荐为了“以后支持 migration”提前添加大量：

```text
todo_migration()
```

更合理的是：

```text
设备状态有明确 owner
机器类型版本化
内存映射可枚举
vCPU 状态独立对象
```

未来 migration 才有实现基础。

---

# 84. Plan Agent 输出的最低完整度

对于一个阶段中的每个核心模块，设计文档至少需要包含：

```text
1. Purpose
2. Scope
3. Non-scope
4. Dependencies
5. Owned objects/state
6. Public operations
7. Internal operations
8. Lifecycle
9. Data flow
10. Error model
11. Concurrency model
12. Security checks
13. Telemetry
14. Validation
15. Future extension points
```

缺少上述关键部分时，不应直接交给 Coding Agent。

---

# 85. 函数设计最低完整度

对于核心函数或 operation，至少定义：

```text
Name / logical operation

Purpose

Caller

Inputs

Outputs

Preconditions

Postconditions

State mutation

Ownership mutation

Authorization requirement

Synchronization requirement

Possible errors

Failure-state guarantee
```

不是所有小 helper 都需要达到这个粒度，但所有：

- lifecycle；
- memory；
- interrupt；
- scheduling；
- device；
- capability；
- IPC；
- management

关键 operation 必须达到。

---

# 86. Plan Agent 设计文档推荐结构

每个阶段设计建议采用：

```text
1. Stage Goal
2. ADR Constraints
3. In Scope
4. Reserved
5. Out of Scope

6. Existing Dependencies

7. Subsystems
   7.1 Module A
   7.2 Module B

8. Core Objects

9. State Machines

10. Interface Design

11. Function Contracts

12. Initialization Sequence

13. Runtime Data Flow

14. Error and Recovery

15. Concurrency and Synchronization

16. Security Boundaries

17. Telemetry

18. Platform Differences

19. Validation Plan

20. Exit Criteria Mapping

21. Future Extension Points

22. Open Questions / ADR Required
```

---

# 87. 必须提供初始化时序

对于启动相关阶段，至少提供：

```text
Firmware
↓
entry
↓
arch early init
↓
console
↓
platform discovery
↓
memory
↓
interrupt
↓
SMP
↓
scheduler
↓
VM
↓
guest
```

如果阶段只覆盖其中一部分，也要明确：

```text
当前阶段从哪里接入
结束后系统处于什么状态
```

---

# 88. 必须提供关键 runtime 时序

例如 vCPU：

```text
scheduler
↓
select Vcpu
↓
prepare
↓
enter guest
↓
VM exit
↓
decode
↓
dispatch
↓
state transition
↓
scheduler
```

Virtio：

```text
guest write queue
↓
kick
↓
trap / notification
↓
backend
↓
completion
↓
virtual interrupt
↓
guest
```

Device passthrough：

```text
management request
↓
quiesce
↓
detach old domain
↓
reset
↓
attach IOMMU
↓
route IRQ
↓
activate
```

---

# 89. 状态改变必须有唯一 authority

例如：

```text
VM state
```

必须明确只能由：

```text
VM lifecycle subsystem
```

执行合法迁移。

Scheduler 不应直接写：

```text
VM = Stopped
```

类似地：

```text
Device ownership
Memory ownership
Capability rights
```

均应有唯一权威模块。

---

# 90. Plan Agent 应优先减少隐式关系

不推荐：

```text
“因为这个 VM 是 Control Domain，所以自然拥有所有设备”
```

应显式设计：

```text
Control Domain
  has DeviceCapability(NVMe)
```

不推荐：

```text
“当前 CPU 上运行的就是这个 VM，所以 IRQ 属于它”
```

应显式：

```text
InterruptRoute
```

---

# 91. 核心机制必须可被观测

任何长期难以调试的核心路径，设计时必须至少预留：

```text
state dump
trace point
counter
```

尤其：

- scheduler；
- Stage-2；
- IRQ；
- capability；
- virtio；
- IPC；
- device assignment。

---

# 92. 设计不得依赖日志文本作为机器接口

日志用于人读。

机器管理必须使用：

```text
structured status
telemetry
native ABI
```

不能让 Control Domain 通过解析 EL2 日志判断状态。

---

# 93. 硬件错误和 Guest 错误必须分层传播

例如 Stage-2 fault：

```text
Guest invalid access
```

与：

```text
page-table corruption
```

语义完全不同。

Plan Agent 必须定义不同处理路径。

---

# 94. 资源回收必须设计为确定性

VM Destroy 完成后，至少应保证：

```text
no runnable vCPU
no active IRQ route
no DMA mapping
no guest memory mapping
no service backend binding
no authority capability allowing stale access
```

不能只从 VmRegistry 删除对象。

---

# 95. Capability revoke 与对象销毁要区分

销毁对象：

```text
object ceases to exist
```

revoke capability：

```text
caller loses authority
```

二者不是同一操作。

---

# 96. Service Domain 失败不能产生悬挂硬件资源

例如 Storage Domain crash 后：

```text
virtio queue
SharedRegion
Notification
physical device binding
```

必须有 ownership/recovery 设计。

---

# 97. 性能优化不得破坏语义边界

以后可以：

```text
batch IRQ
zero-copy
lazy context save
huge page
direct interrupt
EL2 fast backend
```

但这些优化必须保持：

```text
与基础语义等价
```

Plan Agent 不应为了 fast path 设计第二套不兼容对象模型。

---

# 98. Fast Path 与 Slow Path 可以分离

例如：

```text
Fast path:
virtqueue data

Slow path:
device config / reset
```

这是允许的。

但必须共享：

```text
device lifecycle
authorization
ownership
```

模型。

---

# 99. 每次阶段设计结束必须进行架构一致性检查

Plan Agent 最后必须逐项回答：

```text
是否违反 ADR？

是否引入 Board 特判到 Core？

是否把策略写入机制？

是否把角色当成权限？

是否引入单 VM / 单 CPU 永久假设？

是否让 Host platform 泄漏到 Guest machine？

是否把复杂文件系统引入 EL2？

是否让 Guest 输入未经验证进入核心状态？

是否使未来 x86_64 无法实现公共语义？

是否使未来动态 VM/CPU/Memory 无法实现？

是否阻塞 snapshot/migration/capability delegation 等已预留能力？
```

只要任意答案存在风险，都必须在设计文档中说明。

---

# 100. 每次阶段设计结束必须进行 Scope 检查

必须明确：

```text
本阶段到底增加了哪些能力？
```

以及：

```text
哪些能力仍然不存在？
```

防止 Coding Agent 根据设计误以为未实现能力已经存在。

---

# 101. 每个阶段必须映射退出条件

阶段任务书中的退出条件必须转换为：

```text
Observable Result
```

例如：

```text
“Guest 打印 Hello from EL1”
```

必须能追踪到：

```text
VM creation
Stage-2 mapping
vCPU init
EL1 entry
UART output
```

所有必要组件。

---

# 102. Open Question 必须明确分类

Plan Agent 遇到无法依据 ADR 决定的问题时，应分类为：

```text
Implementation Choice
```

当前阶段可以自行决定。

```text
Architecture Choice
```

需要 ADR。

```text
Platform-specific Unknown
```

需要硬件调查。

```text
Specification Unknown
```

需要查 Arm/Virtio/PCI 等规范。

不得把架构问题伪装成普通实现细节。

---

# 103. Plan Agent 的最终输出目标

最终设计应达到：

> Coding Agent 不需要重新推导模块职责、调用顺序、对象关系、状态机、边界条件或错误语义。

Coding Agent 仍需要决定：

```text
具体 Rust 实现方式
代码组织细节
局部算法实现
具体语言特性
```

但是不应该还需要决定：

```text
这个模块到底负责什么？
谁拥有这个对象？
这个函数应该改变哪些状态？
失败后留下什么状态？
这个 API 是否允许跨 VM？
谁有权限调用？
设备什么时候 reset？
Stage-2 修改后什么时候 TLBI？
```

这些属于 Plan Agent 必须提前完成的设计。

---

# 104. 最核心的一条总原则

Plan Agent 的任务不是：

> “把阶段任务拆成若干待写代码的 TODO。”

而是：

> **把阶段任务转化为一套边界清晰、状态明确、资源归属明确、调用关系闭合、失败行为可定义、未来演进不被封死的实现级系统设计。**

设计应做到：

```text
当前阶段足够具体
+
长期架构不过度提前实现
+
Coding Agent 无需重新做架构设计
```

三者同时成立。

---

# 105. Plan Agent 最终自检清单

在交付设计文档前，必须确认：

- [ ] 所有 Stage Requirements 均有对应模块；
- [ ] 所有模块职责和非职责明确；
- [ ] 所有核心对象 ownership 明确；
- [ ] 所有生命周期有状态机或明确顺序；
- [ ] 所有关键函数有前置/后置条件；
- [ ] 所有资源分配都定义失败行为；
- [ ] 所有跨 VM 操作都有授权要求；
- [ ] 所有 Guest 输入都被视为不可信；
- [ ] 所有跨 CPU 修改都有同步语义；
- [ ] 所有 Stage-2 修改考虑 TLB 可见性；
- [ ] 所有设备 assignment 考虑 IRQ/DMA/reset；
- [ ] 所有 Service Domain 交互考虑 peer failure；
- [ ] Host Platform 与 Guest Machine Type 未混淆；
- [ ] Core 未新增 Board-specific 依赖；
- [ ] Role 未替代 Capability；
- [ ] Build capability 与 runtime policy 未混淆；
- [ ] 当前临时限制未变成永久 API 假设；
- [ ] 长期预留功能没有被当前接口封死；
- [ ] 没有无必要提前实现未来大型功能；
- [ ] 所有关键路径都有 telemetry/diagnostic 方案；
- [ ] 所有退出条件都可以客观验证；
- [ ] 所有无法确定的问题均被明确标记为 Implementation Choice / ADR Required / Platform Investigation / Specification Investigation。

---

## 106. 与其他项目文档的关系

```text
Architecture ADR
    ↓
决定“系统原则上是什么”

Stage Task Book
    ↓
决定“当前阶段要完成什么”

Plan Agent Guidelines
    ↓
决定“如何把阶段任务设计到模块/对象/函数级”

Stage Implementation Design
    ↓
决定“本阶段各模块和函数具体承担什么职责”

Coding Guidelines
    ↓
决定“这些设计在 Rust 中如何规范实现”

Source Code
```

其中：

```text
Plan Agent 不替代 ADR
Plan Agent 不替代 Coding Guidelines
Plan Agent 位于两者之间
```

它负责把架构决策可靠地转换成可编码的实现设计。
