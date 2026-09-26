# Zelyr Hypervisor

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `3aa3621f1f2016ebb5dc2ea03466aafdf1fa9cff`
**Authority:** 本文是供中文读者使用的译文；[英文原文](README.md)具有权威性。

Zelyr 是以 AArch64 和 Rust 为先的 Type-1 Hypervisor。QEMU `virt` 是参考平台；
Orange Pi 3B／RK3566 是后续的真实硬件目标。仓库包含已完成的 P0 工程基线和
[P1 EL2 启动完成报告](docs/stages/p1/verification/p1-completion-report.md)。
P1 在单个 Host CPU 上启动至稳定的 Non-secure EL2，具有 Host Stage-1 映射、
控制台和有界故障诊断；尚未运行 Guest，也未提供 GIC、动态平台发现或分配服务。

从 [AGENTS.md](AGENTS.md) 和[文档索引](docs/README.zh-CN.md)开始。贡献者与 agent
在开展非简单工作前必须阅读这两份文档；仅阅读本 README 并不授权修改代码或架构。

## 顶层布局

```text
docs/        架构、治理、阶段工作与验证记录
crates/      Host 测试基线及预留的可复用 crate 空间
hypervisor/  P1 AArch64 EL2 裸机镜像
soc/         预留的 SoC 支持
boards/      预留的板卡／BSP 组合及 quirk
guests/      预留的验证 Guest
control/     预留的 Control／Service Domain 组件
scripts/     可重复的开发和 CI 入口
tests/       Host、QEMU 和集成测试支持
.github/     CI 工作流
```

部分目录仍只是带有 `.gitkeep` 的 P0 占位目录；名称不代表功能已经实现。
修改代码仍需对应阶段已批准的详细设计及仓库编码指南。

## 许可证

Zelyr 使用 [Apache License 2.0](LICENSE)。

## 贡献流程

政策生效后的开发在新分支上进行，且只有配置的必需在线检查通过后才能经 GitHub PR
并入 `main`。参见[分支与 PR 集成流程](docs/development/integration-workflow.md)。
