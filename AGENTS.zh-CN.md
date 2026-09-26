# Zelyr Agent 指令

**Translation status:** Current
**Translation source:** [English source](AGENTS.md)
**Source blob:** `7400d9c364fa9c2ac99de4b46b04a96347c620fa`
**Authority:** 本文是供中文读者使用的译文；[英文原文](AGENTS.md)是 agent 指令来源。

本仓库已在完成的 P0 工程基线上，按声明的参考 QEMU 范围完成 P1。
精确证据和边界见 [P1 完成报告](docs/stages/p1/verification/p1-completion-report.md)。
P6-V29 仍负责真正的异步意外向量执行；P1 完成不代表该证明已经取得。
当前证据另见 [P0 完成报告](docs/stages/p0/verification/p0-completion-report.md)和
[P1 实施索引](docs/stages/p1/implementation/README.md)。VM、GIC 和 Guest 功能必须留在各自所属阶段。

## 必读材料

任何非简单任务开始前，须阅读 [`docs/README.md`](docs/README.md)并遵循其路由表。
尤其是：

- 涉及设计或架构决策时，提出变更前阅读 ADR 基线和适用的阶段任务书。
- 进行详细设计时，阅读简明的 [Plan Agent 指南](docs/development/plan-agent-guidelines.zh-CN.md)。
- 修改代码时，阅读简明的[编码指南](docs/development/coding-guidelines.zh-CN.md)及适用的详细设计。不得仅凭阶段任务书编写代码。

agent 默认阅读英文源文档。已接受的 ADR-000 以中文原文为权威；其
[英文版](docs/adr/adr-000-architecture-baseline-v0.1.en.md)是供 agent 直接阅读的译文。
源文与译文有分歧时遵循[语言版本规则](docs/development/documentation-baseline.md#7-language-editions-and-translation-authority)。

简明指南是必读入口。它们会说明何时还须查阅原始详细参考文档；若路由章节已经足够，
不要默认加载整份详细参考文档。

## 项目本地 Agent Skill

项目本地 skill 在 `.agents/skills/` 下进行版本管理。创建、重组或扩展阶段任务书及其
有界的 `P<stage>-Wxx` 工作包计划时，在读完必需的规划文档后，阅读
[zelyr-stage-work-package-planning](.agents/skills/zelyr-stage-work-package-planning/SKILL.md)。
该 skill 保持任务书、计划、详细设计、实施与验证的分离。

将某个已批准的 `P<stage>-Wxx` 计划转为供编码 agent 执行的实施级设计时，
在读完必需的规划文档后，阅读
[zelyr-work-package-implementation-design](.agents/skills/zelyr-work-package-implementation-design/SKILL.md)。
它要求审计当前状态，并在分解实施步骤前明确捕获计划目标隐含的基础交付物。

遵循主管文档规定的优先级：规划工作用 Plan Agent 指南，编码工作用编码指南。
来源冲突时，停止受影响的工作并记录架构变更或 `ADR Required` 问题；不要暗自选定新架构。

## 范围与交付规则

- 政策生效后的每项变更都在新分支开发。推送到 GitHub，创建目标为 `main` 的 PR，
  等待所有配置为必需的在线检查通过，并且只通过该 PR 合并。准备合并前阅读
  [集成流程](docs/development/integration-workflow.zh-CN.md)；不得直接在 `main` 开发或推送。
- 分开 P0、P1 和后续阶段的工作。不能仅因目录存在就提前实施后续阶段机制。
- 保持 crate 分层：通用 Core 不得直接依赖具体 Board、SoC 或 QEMU 实现。
- 将阶段任务书、详细设计、实施记录和验证报告分别置于 `docs/stages/<stage>/` 的相应位置。
- 变更影响已记录契约时，同步更新文档。不得修改已接受 ADR 来掩盖架构变更；应新增替代 ADR。
- 报告变更文件、已运行及未运行的验证、新增 `unsafe`、ABI／公开 API 变更、依赖以及未解决的设计冲突。

## 当前仓库状态

工作区包含已完成的 P1 Hypervisor 与 Host 测试基线。其他目录可能仍为脚手架。
不得根据名称推断 API、workspace 成员、目标布局或模块树；应检查当前工作区和适用的详细设计。
