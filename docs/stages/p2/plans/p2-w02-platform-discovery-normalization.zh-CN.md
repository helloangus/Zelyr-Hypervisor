# P2-W02 — 平台发现与规范化

**Translation status:** Current
**Translation source:** [English source](p2-w02-platform-discovery-normalization.md)
**Source blob:** `ce1fe9331708513c975064fb42b016ed2f92e6b9`
**Authority:** 本文是供中文读者使用的译文；[英文原文](p2-w02-platform-discovery-normalization.md)具有权威性。

**状态：** 已规划的工作包；不声称已实施。
**上级文档：** [P2 任务书](../task-book-v0.1.zh-CN.md)
**前置条件与消费者：** [P2 计划索引](README.zh-CN.md)

## 目标

从经过验证的 DTB 输入产生一份规范化、由能力驱动的 P2 所需 Host 平台信息表示。

## 范围

P2-B01–B08 和 P2-C01–C03：CPU 清单及其与启动 CPU 的关系；RAM 和保留区；
GICv3、定时器、PSCI、控制台和 chosen 节点的发现；能力摘要；
并区分缺失、不受支持和可用的信息。

## 范围之外

GIC 初始化、定时器虚拟化、PSCI 执行、AP 启动、UART 驱动、特定板卡的 Core 行为，
以及具体的 `PlatformInfo` API。

## 工作顺序

1. 消费 W01 的已验证输入契约，列出各发现结果必须确立的 P2 信息。
2. 确立 CPU、内存、保留区、GIC、定时器、PSCI、chosen／控制台信息的规范化语义结果。
3. 定义能力报告预期，保留缺漏、不受支持、未出现及受支持状态，而不合并诊断结果。
4. 针对 QEMU 和 RK3566 固件样本输入，按 ADR 平台分层及能力驱动行为评审结果。
5. 指定发现与规范化的证据，包括重复输入的确定性行为，不规定解析器内部实现。
6. 向内存图、检查、校验器、QEMU、P3 和 P4 消费者移交稳定的语义信息。

## 验收与关闭

P2-V03 和 P2-V04 要求证据表明已收集规定的必需信息、规范化保留诊断状态，
且 Core 不根据板卡名称作决定。P2-V09 和 P2-V11 后续会在样本和 QEMU 上检验这些成果。

## 移交

W03 可消费 RAM／保留区信息；W06／W07 可报告／检查规范化结果。
P3／P4 只能消费已记录的语义契约，不能依赖特定类型或实现。
