# Zelyr 编码指南 — 必读简明版

**Translation status:** Current
**Translation source:** [English source](coding-guidelines.md)
**Source blob:** `38eb03c33410c7dba290bc2e70bc1acb427479a3`
**Authority:** 本文是供中文读者使用的译文；[英文原文](coding-guidelines.md)具有权威性。

**状态：** 每项代码变更任务的必读指南。
**范围：** 实施已批准设计中的 Rust 内容；不授权更改架构、ABI、状态机、所有权、锁或分层。
**版本：** v0.1。
**所有者／变更背景：** P0 工程基线；编码政策变化时更新。
**替代：** 无。

编码前阅读本文、仓库的 [agent 指令](../../AGENTS.md)、基线 ADR、适用的阶段任务书及
已批准的详细设计。随后查阅[详细参考](coding-guidelines-v0.1.md)中的相关章节。

## 权威顺序与事前检查

编码工作的权威顺序是：

```text
明确任务 > 已批准的详细设计 > 冻结的接口／ABI／状态机
> 架构 ADR > 本指南 > 个人偏好
```

编辑前应确定目标模块／函数、输入／输出、所有权、生命周期状态机、并发上下文、错误含义、
平台边界、ABI 稳定性，以及是否允许分配或阻塞。若必需决策缺失或矛盾，停止并提出设计冲突。

## 不可协商的实施规则

- Hypervisor EL2 代码以 Rust 和 `no_std` 为先；仅在必需的架构边界使用汇编。
  没有明确批准，不得更改工具链、目标、edition 或增加不稳定特性／依赖。
- 保持 crate 分层。通用 Core 不得依赖具体板卡、SoC、QEMU 常量或架构寄存器。
  平台行为放在 Arch／SoC／Board／Driver／Quirk 层，并按能力而非板卡名称选择。
- 为 HPA／HVA／GPA／GVA、ID、长度、页及 handle 使用带语义的新类型。
  不得以裸 `usize`／`u64` 代替；外部输入影响的地址／长度运算与转换必须检查溢出。
- 将 Guest、设备、固件、DMA descriptor、MMIO 和管理输入视为不可信。
  使用前验证范围、对齐、溢出、权限、状态和格式。Guest 错误不得导致 Hypervisor panic。
- 用类型和明确状态机维持不变量。不得直接修改生命周期字段、用角色／VM ID 检查代替
  capability 检查，或增加便捷的全局单例。
- 尽量减少并隔离 `unsafe`。每个 `unsafe` 块／函数附近都需要简明的 `SAFETY` 说明和
  小范围审计边界。不得用 `unsafe` 绕过借用检查器；避免 `static mut`、随意 `transmute`
  及未经审慎设计的原始指针生命周期假设。
- 对 MMIO、寄存器、页表、TLB、DMA、IOMMU、IRQ 和汇编，遵守 volatile 访问、保留位处理、
  barrier、顺序、cache 维护、TLB invalidation 及 SMP shootdown 语义。
  QEMU 测试通过并不证明可以省略这些硬件规则。
- 有意识地限制锁、中断上下文工作、分配、原子操作、递归、队列和热路径分派。
  记录同步含义；在 IRQ／VM-exit 路径中不得阻塞或执行繁重工作。
- 外部 ABI 与持久／快照格式须明确表示方式、宽度、字节序、填充、版本及兼容行为。
  不得把原始 Rust struct 内存直接序列化为契约。
- 保持改动最小。不得用宽泛 `allow` 隐藏警告，不得在非测试路径留下可触达的
  `unwrap`／`expect`／`todo!`／`unimplemented!`，也不得借修复问题进行无关重构。

## 质量、文档与完成

使用仓库固定的格式和 lint 命令。按变更契约添加／更新聚焦的单元、集成、QEMU、
Validation Guest、安全及硬件测试。准确说明已运行和未运行的验证；构建或 QEMU 启动
不是所有测试的通用证据。

记录公开行为、重要的硬件假设、`unsafe` 不变量、TODO／FIXME 所有者、ABI 变更和新依赖。
完成报告应列出：变更文件、实施的设计要求、新增 unsafe、ABI／公开 API 与依赖变更、
已运行／未运行的验证、TODO／FIXME，以及设计冲突。

## 何时查阅详细参考

| 触发条件 | 详细参考章节 |
|---|---|
| Rust 结构、类型、运算、错误、状态 | 2–27 |
| Unsafe、原始指针、初始化 | 10–18 |
| MMIO、寄存器、barrier、TLB、汇编、异常 | 28–34 |
| ABI、feature／cfg、平台／驱动／quirk | 35–44、89、135–139 |
| 锁、原子操作、IRQ、分配、所有权 | 45–60、107–115 |
| Guest 输入、virtio、DMA、IOMMU、模拟 | 61–67、125–127 |
| Trait、宏、crate、注释、遥测、性能 | 68–88 |
| 测试、评审、API、重构 | 91–106、128–134 |
| Capability、机型、配置、Control Domain、迁移 | 144–154 |
| Agent 事前检查、完成定义、禁止行为 | 155–163 |

若任务跨越安全、ABI、硬件、并发或安全防护边界，即使改动很小，也必须查阅详细参考。
