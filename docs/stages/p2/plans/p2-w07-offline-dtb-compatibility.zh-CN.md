# P2-W07 — 离线 DTB 兼容性检查

**Translation status:** Current
**Translation source:** [English source](p2-w07-offline-dtb-compatibility.md)
**Source blob:** `bc1edd1b13625cac52cf28f1745e4669eb3824c5`
**Authority:** 本文是供中文读者使用的译文；[英文原文](p2-w07-offline-dtb-compatibility.md)具有权威性。

**状态：** 已规划的工作包；不声称已实施。
**上级文档：** [P2 任务书](../task-book-v0.1.zh-CN.md)
**前置条件与消费者：** [P2 计划索引](README.zh-CN.md)

## 目标

定义离线 DTB 就绪检查，在 EL2 启动前暴露 P2 平台描述缺口，
并在不同样本中测试共同的发现语义。

## 范围

P2-I01–I05：离线输入、P2 必需信息检查、客观的就绪报告、QEMU `virt` 样本，
以及 Orange Pi 3B／RK3566 样本语义。

## 范围之外

完整 SoC 驱动分析、运行时 Hypervisor 启动、Orange Pi EL2 支持、
特定板卡的 Core 逻辑，或校验器 CLI／解析器实现。

## 工作顺序

1. 仅以 W01 的输入安全边界和 W02 的规范化发现语义为离线检查依据。
2. 确立 CPU、RAM、保留区、GIC、定时器、PSCI、chosen／控制台及安全忽略无关设备的就绪报告结果。
3. 定义检验共同语义的 QEMU 和 RK3566 样本预期，不依赖 QEMU 特定布局假设。
4. 评审报告分类，确保 PASS／WARN／不受支持的结果不暗示运行时板卡支持声明。
5. 指定适用于 Host 侧回归的样本与报告证据。
6. 向 W08、W10 和平台规划者移交样本／就绪预期。

## 验收与关闭

P2-V09 要求证据表明两个样本都得到客观的 P2 就绪报告，
且无关的不受支持设备得到安全诊断。P2-V10 在负向回归中使用此成果。
检查通过不证明 Orange Pi EL2 运行时支持。

## 移交

W08 可使用已记录的样本和预期语义。后续平台工作获得兼容性信号，
而不是实施或 BSP 契约。
