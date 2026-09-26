# Zelyr Hypervisor — P1 阶段任务书 v0.2

**Translation status:** Current
**Translation source:** [English source](task-book-v0.2.md)
**Source blob:** `88ef67539c743ce72edab41c83d19ea6646c9091`
**Authority:** 本文是供中文读者使用的译文；[英文原文](task-book-v0.2.md)具有权威性。

**阶段 ID：** P1
**阶段名称：** AArch64 EL2 最小启动
**状态：** 当前修订后的规划基线；实施与验证证据属于工作包记录，不属于本任务书。
**所有者／变更背景：** P1 NC6 验收移交 P6，2026-09-26。
**替代：** [P1 任务书 v0.1](task-book-v0.1.md)。
**主管文档：** [架构基线 ADR](../../adr/adr-000-architecture-baseline-v0.1.md)、[ADR-061](../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md)、[文档索引](../../README.zh-CN.md)和 [Plan Agent 指南](../../development/plan-agent-guidelines.md)。

## 1. 目的与边界

P1 在 QEMU `virt` 上建立可重复、可观测、可诊断的 AArch64 Non-secure EL2 Rust 运行环境。
成果是具有已知架构状态、异常诊断和受控 EL2 Stage-1 地址空间的稳定 EL2 启动 CPU 运行时。
P1 不运行 Guest，也不提供通用平台、内存、SMP、中断控制器或 VM 服务。

[ADR-061](../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md) 将 P1 的 IRQ/FIQ/SError
向量结构准备，与 P6 中真正执行异步投递验证分开。P1 不宣称已投递或诊断真实的意外异步事件。
P6-W12 在 P6 Host GIC/IRQ 就绪后负责延期的 NC6 执行门禁；这是依赖披露，而非通过的 P1 测试。

本阶段遵循 ADR → 任务书 → 冻结契约 → 已确立契约 → 工作包内部选择的权威顺序。
ADR 约束包括优先启动 AArch64、QEMU `virt` 作为参考平台、Core/Arch/SoC/Board 分离、
受控 `unsafe`、按能力选择行为、结构化诊断及分层验证。

P0 须提供已记录的 workspace／工具链、AArch64 目标、构建和 QEMU 入口、日志／panic 基线、
版本／构建元数据、unsafe 治理、地址／错误约定及 CI 门禁。缺少 P0 输入是上游缺陷，
不构成在 P1 内重新设计 P0 的许可。

## 2. 范围分类

### 必需

- 规范的 QEMU 启动契约和入口验证；
- 最小 Rust `no_std` EL2 运行时及稳定空闲状态；
- 必需／可选／未来 AArch64 能力清单及报告；
- 明确的 EL2 架构状态基线；
- 完整的 EL2 异常向量基线及同步／致命故障诊断；
- 早期控制台与结构化启动标记；
- 受控的 Host Stage-1 地址空间转换及 MMU 启用后稳定性；
- 有序的早期初始化生命周期和失败边界；
- 自动化 QEMU 冒烟／回归及负面／故障注入验证；
- P1 启动、初始化、Host 地址空间、异常诊断及限制契约；
- P2 移交契约及阶段门禁证据图。

### 预留

以 P2 平台发现和动态内存基础设施替换参考控制台和静态启动存储；可复用的平台能力与物理内存抽象；
secondary CPU、GIC、timer 虚拟化、Stage-2、VM/vCPU 和 Guest 入口；未来 Host 虚拟内存服务及
分配器策略；真机启动。P1 契约可约束这些预留项，但本阶段不实施，也不声称完成。

### 范围外

Guest EL1 入口、Stage-2、VM/vCPU、HVC ABI 或加载器；secondary-CPU/SMP/PSCI；
GIC 初始化、IRQ 路由或 vGIC；通用 DTB 解析、PlatformInfo、驱动探测、BSP 框架或 Board 运行时；
页分配器、动态堆、map/unmap 服务或内存所有权对象；virtio、PCI、SMMU/IOMMU、直通、
Control Domain 和管理 ABI；Orange Pi 3B/RK3566 运行时；通用 Core 中按 QEMU 或板卡名称分支。

## 3. 工作包图

| 工作包 | 必需成果 | 主要验证 |
|---|---|---|
| [P1-W01](plans/p1-w01-reference-boot-contract.md) | 规范参考启动契约和入口拒绝边界 | P1-V01、P1-V02 |
| [P1-W02](plans/p1-w02-minimal-rust-el2-runtime.md) | 稳定 Rust EL2 运行时和受控空闲状态 | P1-V03、P1-V04 |
| [P1-W03](plans/p1-w03-aarch64-capability-inventory.md) | 能力清单及报告 | P1-V05、P1-V06 |
| [P1-W04](plans/p1-w04-el2-architectural-state-baseline.md) | 明确的 EL2 架构状态基线 | P1-V07 |
| [P1-W05](plans/p1-w05-el2-exception-entry-baseline.md) | EL2 向量覆盖与致命故障边界 | P1-V08、P1-V09 |
| [P1-W06](plans/p1-w06-early-console-logging.md) | 可靠的早期控制台及标记 | P1-V10 |
| [P1-W07](plans/p1-w07-fatal-crash-diagnostics.md) | 不递归的崩溃和 panic 诊断 | P1-V11、P1-V12 |
| [P1-W08](plans/p1-w08-host-stage1-address-space.md) | Host Stage-1 映射及 MMU 转换契约 | P1-V13、P1-V14 |
| [P1-W09](plans/p1-w09-initialization-sequencing.md) | 有序的初始化生命周期及失败契约 | P1-V15 |
| [P1-W10](plans/p1-w10-qemu-boot-regression.md) | 自动判定及 100 次循环回归计划 | P1-V16、P1-V17 |
| [P1-W11](plans/p1-w11-negative-fault-validation.md) | NC1–NC5 负面与有意故障验证；记录 NC6 移交 | P1-V18、P1-V19 |
| [P1-W12](plans/p1-w12-p1-documentation-handoff.md) | P1 契约、限制及 P2 移交 | P1-V20、P1-V21 |

每个工作包在 [plans/](plans/README.md) 中恰好有一个计划。实施可追溯性属于
[implementation/](implementation/)；命令、环境、结果和完成证据属于
[verification/](verification/)。本任务书及其计划不声明实施或验证完成。

## 4. 依赖与执行图

```text
W01 -> W02 -> W03 -> W04 -> W05 -> W06 -> W07 -> W08 -> W09 -> W10 -> W11 -> W12
  |      |      |      |      |      |      |      |      |      |      |
  +------+------+- - -+------+------+- - -+------+------+- - -+------+------+
```

主链采用保守顺序：后续工作包消费较早的契约。W05 后可并行设计 W06 与 W07；
W10 与 W11 消费已初始化的运行时、异常及 MMU 契约。该图无环。

## 5. 需求到验证的可追溯性

| 需求组 | 验证 ID | 通过条件 |
|---|---|---|
| T01 启动契约 | P1-V01、P1-V02 | 规范启动到达经验证的 Non-secure EL2；不支持的入口条件被明确拒绝且不正常继续。 |
| T02 运行时 | P1-V03、P1-V04 | 在稳定空闲前建立运行状态、启动上下文、panic 路径与身份，且不依赖偶然寄存器值。 |
| T03 能力 | P1-V05、P1-V06 | 缺少必需能力时快速失败；支持、可选与未来事实保持区分。 |
| T04 EL2 基线 | P1-V07 | 必需的 EL2 状态明确由 Hypervisor 拥有，且在干净启动间一致。 |
| T05 异常 | P1-V08、P1-V09 | 16 个同步／IRQ／FIQ／SError 向量槽的入口、捕获与分类契约经过评审；异步投递执行证据延至 P6-V29。 |
| T06–T07 诊断 | P1-V10–P1-V12 | MMU 转换前后均可观察标记、panic、syndrome、故障位置和核心上下文。 |
| T08–T09 MMU／生命周期 | P1-V13–P1-V15 | 必需映射类别具有明确属性，MMU 后启动遵循声明的前置条件与失败路径。 |
| T10 回归 | P1-V16、P1-V17 | 存在自动判定，100 次干净启动到达同一稳定标记且无 panic。 |
| T11 负面验证 | P1-V02、P1-V06、P1-V18、P1-V19 | NC1–NC5 对不支持入口、必需能力拒绝、同步故障、panic 和 MMU 后故障提供有界且可重复的证据；NC6 的意外向量执行证据属于 P6-V29。 |
| T12 文档 | P1-V20、P1-V21 | 契约、限制、证据位置及 P2 假设可审查且不越界。 |

## 6. 阶段验证矩阵

| ID | 所需证据 | 成功条件 |
|---|---|---|
| P1-V01 | 规范启动评审／测试 | 记录入口／镜像假设、EL／安全状态、启动参数、DTB 处理及失败政策，参考启动到达 EL2。 |
| P1-V02 | 不支持入口的证据 | 不允许的环境带原因被拒绝，运行时不正常继续。 |
| P1-V03 | 运行时评审／测试 | 栈、Rust 数据状态、启动上下文、panic 路径及构建身份按顺序建立。 |
| P1-V04 | 稳定状态启动证据 | 重复正常启动到达稳定 EL2，不依赖隐藏的初始寄存器假设。 |
| P1-V05 | 能力报告证据 | EL、CPU、affinity、PA/VA/translation/granule/timer/virtualization 事实被分类并报告。 |
| P1-V06 | 能力负面评审 | 缺少必需能力时明确失败；缺少可选能力不导致无关 panic。 |
| P1-V07 | EL2 基线评审 | 路由、陷入、FP/SIMD、调试／性能、timer、EL1/EL0 准备及转换控制被明确建立。 |
| P1-V08 | 结构性向量覆盖证据 | 源码和 Host 评审表明全部 16 个同步、IRQ、FIQ、SError EL2 槽有有效入口、有界现场与来源／上下文分类契约；不证明异步事件已投递。 |
| P1-V09 | 同步异常证据 | 有意触发和其他未处理同步路径暴露 syndrome 与位置，然后遵循规定结果。 |
| P1-V10 | 控制台／标记证据 | 诊断从入口到稳定状态有效，并标识失败阶段。 |
| P1-V11 | 崩溃报告评审／测试 | 致命输出包括构建、CPU/EL、PC/返回地址、syndrome、故障地址、阶段及有用的寄存器上下文。 |
| P1-V12 | 不递归证据 | panic、早期失败及故障不会暗中递归为不可观察的崩溃。 |
| P1-V13 | Host 映射评审 | 代码、数据、栈、向量、启动数据和 MMIO 具有明确的权限、执行及内存属性。 |
| P1-V14 | MMU 后证据 | 启用 MMU 的执行、控制台、向量和致命故障诊断正常，且没有恒等映射契约。 |
| P1-V15 | 生命周期评审 | 阶段有可见的前置条件、顺序和失败结果；不接受隐藏依赖。 |
| P1-V16 | 自动判定证据 | QEMU 测试独立判定通过／失败，具有有界标记、超时／退出行为并保留失败证据。 |
| P1-V17 | 重复运行证据 | 连续 100 次干净启动到达同一稳定标记，无随机启动失败。 |
| P1-V18 | P1 所属故障注入证据 | NC3 同步故障、NC4 panic、NC5 MMU 后转换／访问故障均在可重复的成对执行中产生所需诊断。NC1／NC2 分别追溯到 P1-V02／P1-V06。NC6 经 ADR-061 移交 P6-V29，在此既未通过也未豁免。 |
| P1-V19 | 范围／安全评审 | 输入／范围检查、无意外 RWX、unsafe 清单及 P2–P4 边界可审查。 |
| P1-V20 | 文档评审 | 启动、初始化、地址空间、诊断、参考环境及限制契约一致。 |
| P1-V21 | 治理评审 | 每包一个计划、客观条件、可解析链接、无环依赖且无完成声明。 |

## 7. 退出条件与移交

只有 P1-V01 至 P1-V21 均有证据，且满足以下条件，才能将 P1 标为完成：

1. QEMU `virt` 经规范路径到达稳定的 Non-secure EL2 Rust 运行时。
2. 必需能力与明确的 EL2 基线得到验证；缺少必需条件时快速失败。
3. Host Stage-1 MMU 启用前后，向量、控制台、panic 和致命故障诊断仍有用。
4. 必需的代码／数据／栈／向量／MMIO 映射类别有明确属性，且不承诺永久恒等映射。
5. 有自动判定及 100 次循环的干净启动证据。
6. 负面／故障证据覆盖不支持入口、必需能力拒绝、同步故障、panic 和 MMU 后故障。P1 只有异步向量的结构性覆盖；真正执行意外向量验证是 P6-V29 的必需门禁。
7. P1 契约、限制、unsafe／API／依赖报告和 P2 移交可审查，而 Guest／SMP／GIC／平台发现／分配器机制仍在本阶段范围外。

P2 可依赖稳定的启动 CPU、Non-secure EL2、Rust 运行时、早期控制台、可诊断异常路径、
能力知识、已知 EL2 状态及 Host Stage-1 运行时。P2 仍负责 DTB 到 PlatformInfo 的发现、
物理内存发现与动态分配。P2 和 P6 都不得根据 P1 完成推断 IRQ/FIQ/SError 已执行投递；
P6 必须先关闭 P6-V29 才能声称该行为。

## 8. 完成评审

提出完成声明前，检查消除了哪些入口假设、仍有哪些临时参考假设、致命路径是否保留阶段／位置／syndrome、
MMU 后行为是否避免 ABI 承诺，以及 Guest、SMP、GIC、平台发现、分配器或板卡运行时机制是否进入 P1。
任一答案不符合要求，都阻止完成。
