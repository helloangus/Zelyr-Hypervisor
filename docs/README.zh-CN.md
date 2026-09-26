# Zelyr 文档索引与治理

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `ef0eca4e52dd511cb9738a8073bbbe773f192f22`
**Authority:** 本文是供中文读者使用的译文；[英文原文](README.md)具有权威性。

**状态：** 规范性文档治理，规定阅读路径、优先级和文档布局。
**范围：** 说明如何查找、分类仓库文档；分类与元数据细节见[文档基线](development/documentation-baseline.md)。
**版本：** v0.2
**所有者／变更背景：** P0 工程基线；双语文档更新；文档布局变化时更新。
**替代：** 无。

## 阅读规则

每位 agent 和贡献者在进行非简单工作前，都必须阅读[英文原文](README.md)及仓库根目录的
[AGENTS.md](../AGENTS.md)，再根据下表读取本次任务对应的文档。中文读者可从本文开始，
但发生措辞分歧时以英文原文为准；ADR-000 的中文原文除外。

| 工作类型 | 必读文档 |
|---|---|
| 涉及架构的工作 | [ADR 基线](adr/adr-000-architecture-baseline-v0.1.md)、[ADR 生命周期与流程](adr/README.md)、对应阶段任务书 |
| 阶段规划／详细设计 | ADR 基线、对应任务书、必须阅读的[简明 Plan Agent 指南](development/plan-agent-guidelines.md)、其指定的详细参考章节，以及[阶段工作流程](development/stage-workflow.md)中的分层职责与准入规则 |
| 编码 | ADR 基线、对应任务书、已批准的详细设计、必须阅读的[简明编码指南](development/coding-guidelines.md)，以及其指定的详细参考章节 |
| 质量门禁问题 | [质量门禁](development/quality-gates.md) |
| 仓库贡献／集成 | [分支与 PR 集成流程](development/integration-workflow.md)、对应工作包计划及相关实施／验证记录 |
| 新贡献者入门 | [贡献者工作流程](development/contributor-workflow.md) |
| CI、必需检查或分支保护 | [CI 基线](development/ci-baseline.md) |
| 工具链安装、恢复或更新 | [工具链基线](development/toolchain-baseline.md)和根目录的 `rust-toolchain.toml` |
| 目标平台／构建类别／AArch64 构建路径 | [构建目标基线](development/build-target-baseline.md)和根目录的 `Cargo.toml` |
| 新功能、profile、构建开关或开关分类 | [构建 profile 与功能治理](development/build-profile-governance.md) |
| 创建、版本化或分类文档 | [文档基线](development/documentation-baseline.md) |
| ABI 或虚拟机机型变更 | 上述文档，以及 `abi/`、`machine-types/` 中适用的契约 |
| 平台／BSP 工作 | 上述文档，以及 `platform/` 契约；按[平台可移植性规则](development/platform-portability-rules.md)保持 Core／Arch／SoC／Board 分层 |
| 测试或完成声明 | 对应任务书、`testing/` 契约及阶段验证记录 |
| 编写或评审 Host 测试 | [Host 测试基线](testing/host-test-baseline.md) |
| QEMU 自动化或 runner 入口 | [QEMU runner 入口契约](testing/qemu-runner-entry.md)；P0 阶段仅为接口占位 |
| 引入或评审 `unsafe` Rust | [Unsafe Rust 政策](security/unsafe-rust-policy.md)和[unsafe 清单](security/unsafe-inventory.md) |
| 失败路径设计或致命路径评审 | [失败分类](security/failure-classification.md) |
| 新接口中的地址／标识符语义 | [地址与标识符类型安全要求](development/address-identifier-type-safety.md) |
| 诊断通道、等级或可见性 | [诊断基线](development/diagnostics-baseline.md) |
| Trace 事件命名或遥测命名空间 | [Trace 事件命名空间](development/trace-event-namespace.md) |
| 制品标识／版本元数据 | [版本与构建元数据基线](development/version-build-metadata.md) |
| 命名离开构建树的制品 | [制品命名基线](development/artifact-naming.md) |
| 添加或更改依赖 | [依赖治理](development/dependency-governance.md)和[依赖登记表](development/dependency-register.md) |

## 规范性文档与优先级

当前规范性来源包括[架构基线 ADR](adr/adr-000-architecture-baseline-v0.1.md)、对应阶段任务书、
已批准的详细设计和冻结的契约，以及相关简明 agent 指南和其指定的详细参考章节。
采用主管指南规定的顺序：

- 规划：ADR → 当前阶段任务书 → 冻结的接口／ABI／机型 → 已确立的模块契约 → 阶段内部的设计自由度。
- 编码：明确任务 → 详细模块设计 → 接口／ABI／状态机契约 → ADR → 编码指南 → 个人偏好。

基线 ADR、简明指南和详细参考文件都是受版本管理的源文档。简明指南是必读入口；
当其路由条件满足时，相关详细参考章节也必须阅读。不得暗中改变这些文档的决策；
架构变更须提出新 ADR。

## 文档布局

```text
adr/            架构决策及其生命周期
architecture/   跨领域架构说明
abi/            版本化 ABI 与线格式契约
machine-types/  Guest 虚拟机机型契约
platform/       PlatformInfo、支持等级、BSP 与 quirk 契约
testing/        测试策略、环境与证据
security/       安全、威胁模型与 unsafe 审计记录
development/    贡献者、Plan Agent 与 Coding Agent 指南
stages/<id>/    任务书、详细计划、实施说明、验证
templates/      已批准的项目文档模板
```

每个阶段必须严格分开：

```text
task-book-v*.md             要完成什么
plans/                      已批准的实施级计划
implementation/             实施说明和可追溯记录
verification/               证据和完成报告
```

新的规范性文档必须声明状态、范围、版本、所有者／变更背景，以及适用时的替代关系。
资料性说明必须明确标注为资料性。代码变更若影响契约，须在同一次变更中更新相关文档。
