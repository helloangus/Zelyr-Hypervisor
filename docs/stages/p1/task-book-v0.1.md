# Zelyr Hypervisor — P1 Stage Task Book v0.1

**Stage ID：** P1  
**Stage Name：** AArch64 EL2 最小启动  
**Parent Architecture：** Rust Type-1 Hypervisor Architecture Decision Record v0.1  
**目标平台：** QEMU `virt` / AArch64  
**阶段性质：** Bring-up / Architecture Foundation  
**前置阶段：** P0 — 仓库、规范与工具链基线  
**后继阶段：** P2 — 平台发现与内存基础设施  
**文档职责：** 规定 P1 必须完成的任务、范围、交付物和退出条件，不规定具体实现设计。

---

# 1. 阶段目标

P1 的唯一核心目标是建立一个**可重复、可诊断、可验证的 AArch64 EL2 Rust Hypervisor 最小运行环境**。

完成 P1 后，系统必须能够在 QEMU `virt` reference platform 上：

1. 从既定启动入口进入 Hypervisor。
2. 确认当前执行级别为 Non-secure EL2。
3. 建立最小 Rust `no_std` 运行环境。
4. 建立可靠的 early console 与启动日志。
5. 获取并报告关键 AArch64 CPU/虚拟化能力。
6. 建立已知、受控的 EL2 architectural state。
7. 安装并验证 EL2 异常入口。
8. 对同步异常和 fatal error 提供足够的诊断信息。
9. 建立 Hypervisor 自身使用的 EL2 Stage-1 地址空间。
10. 在启用 MMU 后继续稳定执行。
11. 支持自动化、重复性的 QEMU 启动回归。
12. 形成后续 Stage-2、SMP、GIC、VM/vCPU 工作可以依赖的稳定 EL2 基础环境。

P1 **不运行 Guest**。

P1 的成功标志不是“已经能虚拟化 CPU”，而是：

> Hypervisor 已经成为一个可稳定运行、可观察、可诊断的 EL2 execution environment。

---

# 2. P1 在整体研发路线中的位置

```text
P0
Repository / Toolchain / Rules
        │
        ▼
P1
AArch64 EL2 Runtime
        │
        ├── Rust runtime alive
        ├── EL2 architectural baseline
        ├── exception handling
        ├── early console
        ├── crash diagnostics
        └── EL2 Stage-1 MMU
        │
        ▼
P2
Platform discovery + physical memory infrastructure
        │
        ▼
P3
SMP / per-CPU
        │
        ▼
P4
Stage-2 + first EL1 Validation Guest
```

P1 不得为了“顺便完成更多功能”提前引入 P2/P3/P4 的主体能力。

---

# 3. P1 前置条件

开始 P1 前，应认为 P0 已经提供以下工程基础：

- Cargo workspace 可以稳定构建。
- AArch64 bare-metal target/toolchain 已确定。
- Hypervisor binary 可以生成。
- 基础 linker/build infrastructure 已存在。
- 项目拥有统一代码格式与 lint 规则。
- `unsafe` 使用原则已经确定。
- 基础 CI 可以执行 build/check。
- 项目版本与构建信息有统一来源。
- ADR v0.1 已进入项目文档体系。
- QEMU 已被确定为 AArch64 reference platform。

如果其中任一条件不存在，应作为 P0 遗留问题解决，而不是在 P1 中重新设计整个工程体系。

---

# 4. Stage Scope

P1 的工作范围划分为十一个 Task Group。

---

# 5. P1-T01 — Reference Boot Contract

## 任务目标

建立并验证第一版 **AArch64 Hypervisor Boot Contract**。

P1 必须明确 Hypervisor 在进入第一条自身指令时可以依赖什么，以及不可以依赖什么。

## 必须完成

明确并记录：

- reference platform 为 QEMU `virt`。
- canonical boot path。
- Hypervisor image 的加载条件。
- Hypervisor entry point。
- entry 时要求的 Exception Level。
- Secure / Non-secure world 假设。
- 初始 CPU 为 boot CPU。
- 初始 MMU 状态假设。
- 初始 cache 状态假设。
- 启动参数传递约定。
- DTB 是否存在以及其传递方式。
- Hypervisor image、stack、boot data 所需最小内存条件。
- 不满足启动要求时的失败行为。

必须建立至少一条**官方支持的、可复制执行的 QEMU 启动命令或启动脚本**。

## 必须验证

正常 reference boot path：

```text
QEMU
  ↓
Hypervisor entry
  ↓
EL2 confirmed
  ↓
Rust runtime
```

可以稳定完成。

错误执行级别或不满足必需 architecture capability 时，不允许继续“带病启动”。

## 本任务不包含

- 通用 bootloader framework。
- UEFI 完整支持。
- Orange Pi 3B boot chain。
- TF-A 自研。
- EL3 实现。
- Guest boot protocol。

---

# 6. P1-T02 — Minimal Rust EL2 Runtime

## 任务目标

建立可以长期作为 Hypervisor 基础的最小 Rust EL2 runtime。

## 必须完成

启动后必须正确建立：

- Hypervisor execution stack。
- Rust entry environment。
- 静态数据可用状态。
- 零初始化数据正确状态。
- 只读数据可访问状态。
- 必需的启动上下文保存。
- 基础 panic 路径。
- Hypervisor build/version identity。

Rust runtime 必须能够持续执行，而不仅仅是完成一次 `println` 后退出。

## 必须验证

启动后能够执行：

```text
entry
→ runtime initialization
→ EL2 initialization
→ diagnostics
→ stable idle / halt state
```

启动过程不存在依赖“偶然寄存器初值”才能工作的隐式假设。

---

# 7. P1-T03 — AArch64 Architectural Capability Inventory

## 任务目标

Hypervisor 必须知道自己运行在什么 CPU virtualization environment 上。

## 必须完成

至少采集、解析并报告与当前项目直接相关的 CPU architecture information，包括：

- 当前 Exception Level。
- CPU identification。
- CPU affinity / processor identity。
- AArch64 architecture version relevant information。
- EL2 virtualization support。
- physical address size。
- virtual address translation capability。
- Stage-2 translation capability。
- page granule capability。
- timer-related architecture capability。
- virtualization-related architectural extensions。
- 后续 Stage-2/GIC/VM bring-up 所依赖的重要能力。

需要区分：

```text
Required capability
Optional capability
Future capability
```

必需能力缺失时应明确拒绝继续启动。

可选能力缺失不得导致无意义 panic。

## 阶段产出

形成一份可用于开发和 bug report 的 CPU Capability Report。

例如：

```text
Architecture:
EL:
CPU:
PA range:
Supported granules:
Virtualization:
Timer:
Optional extensions:
```

具体展示格式由后续设计决定。

---

# 8. P1-T04 — EL2 Architectural State Baseline

## 任务目标

消除 firmware/QEMU 留下的未知 EL2 状态，使 Hypervisor 在进入主体运行阶段前处于已知 architecture state。

## 必须完成

P1 需要建立并验证与以下类别有关的 EL2 基础状态：

- execution state。
- exception routing。
- trap policy baseline。
- floating-point/SIMD access baseline。
- debug/performance trap baseline。
- timer access baseline。
- EL1/EL0 execution preparation baseline。
- Stage-1 translation control baseline。
- system register state required by later Guest execution。

目标不是在 P1 完成 Guest virtualization policy，而是保证：

> 后续 Stage 不需要依赖 firmware 恰巧留下的 EL2 配置。

所有关键状态必须来自 Hypervisor 自身建立的明确 baseline。

## 必须验证

Hypervisor 在不同 QEMU cold boot 中建立相同的逻辑 EL2 baseline。

---

# 9. P1-T05 — EL2 Exception Entry Baseline

## 任务目标

建立 Hypervisor 最基础的异常安全网。

P1 必须拥有完整的 EL2 exception vector coverage，即使部分异常类型直到以后阶段才真正处理。

## 必须覆盖的异常类别

至少包括：

- synchronous exception。
- IRQ。
- FIQ。
- SError。

并正确区分适用的 exception origin / execution context。

## P1 要求

对于当前阶段能够产生的异常：

- 正确进入 EL2 exception path。
- 不发生错误递归。
- 保存足够的 diagnostic context。
- 能识别异常类别。
- 能读取并解释核心 syndrome 信息。
- 能决定该异常当前是否可恢复。
- 对不可恢复异常执行确定的 fatal path。

P1 不要求建立完整 IRQ subsystem。

因此：

- IRQ vector 必须存在。
- IRQ 可以进入可诊断的 unexpected/unhandled path。
- GIC initialization 和 IRQ dispatch 不属于 P1。

## 必须完成的验证场景

至少建立可重复测试覆盖：

- 主动触发的同步异常。
- 非法/未支持操作产生的异常。
- 地址访问类异常。
- Hypervisor panic。
- unexpected vector path。

必须证明：

> 异常发生后能够看到足以确定异常位置和原因的诊断信息，而不是只看到“Hypervisor crashed”。

---

# 10. P1-T06 — Early Console and Bring-up Logging

## 任务目标

建立不依赖完整平台框架的最小诊断输出能力。

P1 early console 的职责是：

> 在平台发现、动态内存、driver framework 尚不存在时，仍然能够调试 Hypervisor。

## 必须完成

在 QEMU `virt` reference environment 下支持：

- Hypervisor 启动日志。
- architecture capability 输出。
- exception diagnostics。
- panic diagnostics。
- MMU transition diagnostics。
-关键阶段 marker。

需要明确区分：

```text
Early console
≠
未来完整 console subsystem
```

P1 的 reference-platform 固定信息属于 bring-up 条件，不得被定义为未来永久平台架构。

## 必须验证

从 Hypervisor 第一阶段初始化开始，直到进入稳定 EL2 idle state，diagnostic channel 始终可用。

---

# 11. P1-T07 — Fatal Error and Crash Diagnostics

## 任务目标

确保 P1 之后出现的大部分 early bring-up failure 都有诊断价值。

## Fatal diagnostic 至少应能够报告

- Hypervisor build/version。
- CPU identity。
- Exception Level。
- 当前执行位置。
- exception return information。
- exception syndrome。
- fault address information。
- 与 translation fault 相关的必要信息。
- panic message。
- 当前启动阶段或 execution context。
- 能够定位问题的核心通用寄存器上下文。

具体寄存器布局和 dump 格式不属于本任务书决定范围。

## 必须处理

- panic during normal EL2 execution。
- synchronous EL2 fault。
- MMU enable 前后的 fault。
- early initialization failure。
- unsupported CPU capability。
- unexpected exception。

## 必须满足

fatal path 本身应尽量保持可诊断。

不能出现：

```text
发生原始异常
→ crash reporter 再异常
→ 无限递归
→ 无任何有效输出
```

作为可接受的常态行为。

---

# 12. P1-T08 — EL2 Stage-1 Host Address Space

## 任务目标

建立 Hypervisor 自身长期使用的 EL2 Stage-1 virtual memory environment。

这里的 Stage-1 是：

```text
Hypervisor VA
      ↓
EL2 Stage-1
      ↓
Host Physical Address
```

不是 Guest Stage-2。

## 必须完成

P1 必须让 Hypervisor 从初始启动环境迁移到一个明确受控的 Host Stage-1 地址空间。

至少需要覆盖 Hypervisor 当前运行所需的：

- executable code。
- read-only data。
- writable data。
- zero-initialized data。
- boot stack。
- exception vectors。
- boot-time data。
- early console MMIO。
- 当前阶段必需的其他 MMIO。

必须为这些区域设置合理的：

- access permission。
- execution permission。
- memory type。
- cacheability / device semantics。

## 需要证明

启用 EL2 MMU 后：

- Hypervisor 继续稳定执行。
- early console 继续可用。
- exception vectors 继续工作。
- panic/crash diagnostic 继续工作。
- 不出现隐式 identity-mapping 假设导致的故障。

## P1 不要求

- 通用动态 virtual memory manager。
- 动态 `map()/unmap()` 服务。
- physical page allocator。
- buddy allocator。
- slab allocator。
- Guest Stage-2。
- huge-page optimization。
- NUMA。

这些属于后续阶段。

---

# 13. P1-T09 — Controlled Transition and Initialization Sequencing

## 任务目标

建立清晰的 Hypervisor early initialization 生命周期。

P1 必须使启动过程能够区分至少以下逻辑阶段：

```text
Boot Entry
   ↓
Minimal Runtime
   ↓
Early Diagnostics
   ↓
Architecture Validation
   ↓
Exception Environment
   ↓
EL2 Architectural Baseline
   ↓
Host Stage-1 MMU
   ↓
Stable EL2 Runtime
```

具体函数、模块和状态对象由 plan agent 决定。

Stage Task Book 只要求这些逻辑状态之间：

- 顺序明确。
- 前置条件明确。
- 失败结果明确。
- 不允许无序初始化产生隐式依赖。

必须避免：

> 某项设施实际上依赖另一项设施已经初始化，但依赖关系从系统行为上不可见。

---

# 14. P1-T10 — QEMU Automated Boot Regression

## 任务目标

将“Hypervisor 可以启动”从人工观察变成可重复验证的工程事实。

## 必须完成

建立自动化 QEMU P1 smoke/regression test。

测试至少需要判断：

- QEMU 成功启动。
- Hypervisor 成功进入 EL2。
- Rust runtime 成功启动。
- capability validation 通过。
- exception environment 安装完成。
- Host Stage-1 MMU 已启用。
- Hypervisor 到达 P1 stable state。
- 未发生 panic。
- 测试能自行判断 pass/fail，而不是完全依赖人工读日志。

## 重复启动要求

P1 退出前应完成至少：

**100 次连续 clean QEMU boot regression**

并满足：

- 无随机启动失败。
- 无偶现 panic。
- 无输出顺序相关的未知竞态。
- 每次均达到同一 stable marker。

P1 尚未引入 SMP，因此这里主要用于发现：

- 未初始化数据。
- linker/layout 问题。
- MMU transition 问题。
- boot state 假设问题。
-异常入口问题。

---

# 15. P1-T11 — Negative and Fault Injection Validation

## 任务目标

P1 不只验证“正常路径能跑”，还必须验证基础失败路径。

## 必须包含的负向测试

### 15.1 Unsupported execution environment

在不满足要求的环境中启动时：

- 检测不符合要求。
- 输出明确原因。
- 不继续进入正常 Hypervisor runtime。

### 15.2 Intentional synchronous fault

人为产生同步异常：

- exception vector 被执行。
- syndrome 可读。
- fault location 可定位。
- fatal path 正常。

### 15.3 Intentional panic

主动触发 panic：

- panic information 可见。
- 系统进入明确终止状态。
- 不出现静默死机。

### 15.4 Translation / MMU fault

在已启用 Host Stage-1 后制造受控 translation/access failure：

- fault 被 EL2 正确捕获。
- fault address 与 syndrome 可诊断。

### 15.5 Unexpected vector

对于当前 Stage 不支持处理的异常：

- 不允许无信息 hang。
- 必须进入统一可诊断失败路径。

---

# 16. P1-T12 — Architecture Documentation

P1 完成时必须同步形成文档，而不是只留下代码。

至少需要形成：

## 16.1 AArch64 Boot Contract

描述：

- 如何启动。
- 进入条件。
- EL 要求。
-启动参数。
- firmware 假设。
- 当前限制。

## 16.2 P1 EL2 Initialization Contract

描述逻辑初始化阶段，以及每个阶段建立的 system condition。

不需要描述函数级实现。

## 16.3 Host Address Space Description

描述 P1 Host virtual address space 的逻辑布局与映射类别。

不要求冻结未来完整 memory architecture。

## 16.4 Exception Diagnostic Contract

说明 fatal crash report 至少保证提供哪些类别的信息。

## 16.5 Reference QEMU Environment

记录：

- reference machine。
- CPU model / virtualization requirement。
- RAM requirement。
- console。
- canonical boot command。
- pass marker。

## 16.6 Known Limitations

必须明确记录 P1 尚未支持的能力，防止后续把“暂未实现”误认为 bug 或架构承诺。

---

# 17. P1 交付物清单

P1 完成时至少应产生以下可审查交付物：

| ID | 交付物 | 必需 |
|---|---|---:|
| P1-D01 | 可启动 AArch64 EL2 Hypervisor image | 是 |
| P1-D02 | Canonical QEMU `virt` 启动入口 | 是 |
| P1-D03 | AArch64 Boot Contract | 是 |
| P1-D04 | Architecture Capability Report | 是 |
| P1-D05 | Early console output | 是 |
| P1-D06 | EL2 exception baseline | 是 |
| P1-D07 | Fatal crash diagnostic output | 是 |
| P1-D08 | EL2 architectural-state baseline | 是 |
| P1-D09 | 已启用的 EL2 Stage-1 environment | 是 |
| P1-D10 | QEMU automated smoke test | 是 |
| P1-D11 | 100-cycle boot regression evidence | 是 |
| P1-D12 | Fault-injection evidence | 是 |
| P1-D13 | Host address-space description | 是 |
| P1-D14 | P1 known-limitations 文档 | 是 |
| P1-D15 | P1 Stage Completion Report | 是 |

---

# 18. P1 验收矩阵

| 验收 ID | 验收内容 | 通过标准 |
|---|---|---|
| P1-A01 | EL2 entry | 启动后确认运行于要求的 Non-secure EL2 |
| P1-A02 | Rust runtime | Rust `no_std` runtime 持续稳定执行 |
| P1-A03 | Console | 初始化全过程拥有可靠 diagnostic output |
| P1-A04 | CPU capability | 关键 virtualization capability 可检测和报告 |
| P1-A05 | Unsupported environment | 缺少必需能力时 fail-fast |
| P1-A06 | EL2 baseline | firmware 遗留状态不作为长期隐式依赖 |
| P1-A07 | Exception vector | 同步异常进入正确 EL2 handler |
| P1-A08 | Syndrome dump | fault 类型、位置和地址可诊断 |
| P1-A09 | Panic | panic 有稳定 crash output |
| P1-A10 | Host MMU | EL2 Stage-1 启用后继续正常运行 |
| P1-A11 | MMIO after MMU | early console 在 MMU enable 后正常 |
| P1-A12 | Exception after MMU | exception path 在 MMU enable 后正常 |
| P1-A13 | Repeatability | 100 次 clean boot regression 全部通过 |
| P1-A14 | Automated verdict | CI/test 能自动判断启动成功或失败 |
| P1-A15 | Scope control | 未引入不属于 P1 的 Guest/SMP/GIC/allocator 主体功能 |

---

# 19. P1 明确非目标

以下内容**不属于 P1**。

即使实现起来“顺手”，也不应该成为 P1 完成条件。

## 19.1 Guest virtualization

不包括：

- `Vm` runtime。
- `Vcpu` runtime。
- Guest context。
- Guest EL1 entry。
- Stage-2 translation。
- HVC ABI。
- Guest loader。
- Validation Guest。

它们从 P4 开始成为主体工作。

## 19.2 SMP

不包括：

- secondary CPU bring-up。
- PSCI CPU_ON。
- per-CPU scheduler。
- cross-CPU IPI。
- TLB shootdown。
- spinlock体系验证。

这些属于 P3。

P1 只需要 boot CPU。

## 19.3 GIC

不包括：

- GICv3 initialization。
- IRQ routing。
- physical interrupt dispatch。
- virtual interrupt。
- vGIC。
- List Registers。

这些属于 P6。

P1 只需拥有 IRQ/FIQ exception vector 基线。

## 19.4 通用 Platform Discovery

不包括：

- 完整 DTB parser。
- `PlatformInfo`。
- driver probing。
- compatible registry。
- Board BSP framework。
- PlatformCapabilities framework。

这些属于 P2 及后续平台阶段。

P1 可以使用 reference platform 的固定 bring-up 条件。

## 19.5 动态内存系统

不包括：

- physical frame allocator。
- buddy allocator。
- slab allocator。
- Hypervisor general-purpose heap architecture。
- MemoryObject。
-动态 map/unmap service。

这些属于 P2。

## 19.6 Orange Pi 3B

P1 不要求 RK3566 / Orange Pi 3B 真机启动。

Orange Pi 真机属于后续独立阶段。

但是 P1 不得故意建立阻止真实 AArch64 平台适配的 architecture dependency。

## 19.7 设备虚拟化

不包括：

- Virtio。
- PCI。
- SMMU。
- passthrough。
- device backend。
- Service Domain。

## 19.8 Control Domain

不包括：

- Linux Control Domain。
- Bootstrap Domain。
- management ABI。
- VM configuration。

---

# 20. P1 临时实现允许项

为了避免 P1 被“未来完整架构”拖慢，本阶段允许使用适当的 bring-up 临时条件。

例如 P1 可以允许：

- reference QEMU machine 固定启动参数。
- reference UART 固定信息。
- 静态 boot stack。
- 静态 early page-table storage。
- 单 boot CPU。
- 静态 early memory regions。

但这些内容必须满足：

> “是 P1 bring-up scaffold”，而不是被宣称为未来通用平台架构。

后续 P2/P3 可以替换这些临时机制。

---

# 21. P1 不得形成的架构债务

P1 虽然是 bring-up 阶段，但不得为了快速启动形成以下永久假设：

### 禁止假设 1

```text
Hypervisor 永远只运行在 QEMU
```

### 禁止假设 2

```text
Hypervisor 永远只有一个 CPU
```

P1 可以只运行 boot CPU，但不能把“单 CPU”写成项目架构要求。

### 禁止假设 3

```text
Hypervisor 永远使用固定物理地址
```

P1 可以暂时使用 reference layout，但不能把它定义成跨平台 ABI。

### 禁止假设 4

```text
Host VA 永远等于 PA
```

即使 P1 选择某种简单映射方式，也不得把 identity mapping 变成上层永久语义。

### 禁止假设 5

```text
所有设备都在固定 MMIO 地址
```

P1 reference console 可以例外，但后续必须由平台发现体系接管。

### 禁止假设 6

```text
Firmware 会帮 Hypervisor 初始化所有 EL2 状态
```

P1 正是要消除这一隐式依赖。

---

# 22. P1 安全要求

虽然 P1 尚未运行不可信 Guest，但必须从此阶段建立 Hypervisor 基础安全纪律。

至少要求：

- 启动入口参数经过合理性验证。
- architecture capability 不满足要求时不得继续。
- 地址与范围计算避免明显 overflow。
- MMU mapping 不产生无意 RWX 区域。
- code/data/device memory 属性明确区分。
- fatal path 不执行不受控恢复。
- `unsafe` 新增项进入 P0 已建立的 unsafe inventory。
- MMU/exception 相关 `unsafe` 必须能够被后续审计。
- 不能通过“关闭保护机制”作为 P1 正常运行的默认解决方式。

---

# 23. P1 可观测性要求

P1 必须至少能够观察以下事件：

```text
Hypervisor entry
Rust runtime ready
Current EL validated
CPU capability validated
Exception vectors ready
EL2 baseline established
Host Stage-1 preparation
Host Stage-1 enabled
Stable EL2 runtime entered
Fatal exception
Panic
```

具体 trace framework、event representation 和编译选项不在本任务书决定范围。

但 P1 完成后必须能够回答：

> Hypervisor 是在哪一个启动阶段失败的？

---

# 24. P1 测试类别

P1 的测试证据至少覆盖五类。

## 24.1 Build test

证明 AArch64 Hypervisor artifact 可重复构建。

## 24.2 Boot test

证明 QEMU clean boot 可到 stable EL2 state。

## 24.3 Architecture validation test

证明 CPU/EL/virtualization capability detection 工作。

## 24.4 Fault test

证明 exception 和 panic diagnostic 工作。

## 24.5 MMU transition test

证明 Host Stage-1 开启前后：

- console 正常。
-代码执行正常。
- exception 正常。
- fatal diagnostic 正常。

---

# 25. P1 Stage Completion Report 必须回答的问题

P1 结束时，completion report 必须明确回答：

1. Hypervisor 从什么环境进入 EL2？
2. 哪些 entry-state assumptions 已经消除？
3. 哪些 entry-state assumptions 仍然存在？
4. Hypervisor 如何确认 CPU 满足最低 virtualization requirements？
5. EL2 exception environment 是否已经稳定？
6. Host Stage-1 是否已经正式成为默认运行环境？
7. 开启 Host Stage-1 后哪些资源已经映射？
8. panic/fault 时能够获得哪些 diagnostic 信息？
9. 100 次连续 QEMU clean boot 是否全部通过？
10. 是否存在偶现启动失败？
11. 当前 P1 中有哪些 temporary reference-platform assumptions？
12. 哪些 temporary assumptions 将在 P2 被移除？
13. 是否引入了超出 P1 scope 的功能？
14. 是否产生新的架构决策需要进入 ADR？
15. 是否存在阻塞 P2 的已知问题？

---

# 26. P1 Definition of Done

只有同时满足以下条件，P1 才能标记为 `DONE`。

### 功能

- AArch64 Hypervisor 在 QEMU `virt` 稳定启动。
- 进入并确认 EL2。
- Rust runtime 稳定。
- early console 稳定。
- CPU virtualization capability 可检查。
- EL2 architectural baseline 建立。
- exception vector baseline 建立。
- fatal crash diagnostics 可用。
- Host Stage-1 MMU 正式启用。
- MMU 启用后 Hypervisor 继续稳定运行。

### 验证

- 正常 boot test 通过。
- intentional synchronous exception test 通过。
- intentional panic test 通过。
- Host Stage-1 fault test 通过。
- unsupported environment failure path 得到验证。
- 至少 100 次 clean QEMU boot regression 连续通过。
- 自动测试能够自行给出 pass/fail。

### 文档

- Boot Contract 完成。
- Reference QEMU boot documentation 完成。
- Host address-space description 完成。
- Exception diagnostic contract 完成。
- Known limitations 完成。
- Stage Completion Report 完成。

### 架构边界

- 未开始 Guest Stage-2 主体实现。
- 未开始 SMP 主体实现。
- 未开始 GIC 主体实现。
- 未开始通用 DTB/platform discovery 主体实现。
- 未开始动态物理内存 allocator 主体实现。
- 未把 QEMU 特性定义成 Core architecture requirement。

---

# 27. P1 → P2 交接条件

P2 可以假设 P1 已经提供一个稳定环境：

```text
AArch64 boot CPU
        │
        ▼
Non-secure EL2
        │
        ▼
Rust runtime
        │
        ├── reliable console
        ├── exception diagnostics
        ├── architectural capability knowledge
        ├── known EL2 state
        └── Host Stage-1 MMU
        │
        ▼
P2 may start:
DTB → PlatformInfo
Physical memory discovery
Reserved memory
Page allocator
Hypervisor dynamic memory infrastructure
```

P2 不应该再重新解决：

- “Rust 能不能在 EL2 跑起来？”
- “异常发生以后为什么完全没有输出？”
- “MMU 一开就死机但无法诊断。”
- “不知道当前是不是 EL2。”
- “不知道 CPU 支不支持基本虚拟化。”
- “每次 QEMU 启动结果随机。”

这些问题必须在 P1 结束前关闭。

---

# 28. P1 最终阶段定义

P1 最终交付的不是一个“虚拟机监控器功能”。

它交付的是 Hypervisor 后续所有能力赖以建立的：

**AArch64 EL2 Execution Foundation**

其稳定边界为：

```text
Firmware / QEMU
      ↓
Boot Contract
      ↓
AArch64 EL2 Entry
      ↓
Rust Runtime
      ↓
Architecture Validation
      ↓
EL2 Exception Environment
      ↓
EL2 Architectural Baseline
      ↓
Host Stage-1 Address Space
      ↓
Observable / Diagnosable Stable Runtime
```

只有这个基础稳定之后，P2 的平台发现、P3 的 SMP，以及 P4 的 Stage-2/EL1 Guest 才具有可靠的调试基础。

**P1 不追求“功能多”，而追求“EL2 基础环境完全可信”。**