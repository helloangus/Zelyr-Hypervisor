# P2 工作包计划

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `945d81a708001911212595b3421f48c089561b3e`
**Authority:** 本文是供中文读者使用的译文；[英文原文](README.md)具有权威性。

**状态：** 规划索引。各计划是有界工作包计划，而不是实施完成记录。

## 阅读顺序

处理 P2 工作包时，按顺序阅读：

1. [架构基线](../../../adr/adr-000-architecture-baseline-v0.1.md)；
2. [P2 任务书](../task-book-v0.1.zh-CN.md)；
3. 本索引与选定的 P2-Wxx 计划；
4. 能证明所列前置条件的适用 P0／P1 记录及证据；
5. 本项活动的必读指南。代码变更还需要编码指南和已批准的详细设计。

每份计划只定义其有界成果、验收与移交。实施决策记录在
[../implementation/](../implementation/)；命令、环境、结果、限制及完成证据记录在
[../verification/](../verification/)。

## 依赖与执行图

| 计划 | 前置条件 | 主要消费者 |
|---|---|---|
| [W01](p2-w01-boot-platform-description-intake.zh-CN.md) | P0 Host 测试／诊断／unsafe 基线；P1 DTB 移交及镜像范围 | W02、W07、W08、W09 |
| [W02](p2-w02-platform-discovery-normalization.zh-CN.md) | W01 | W03、W06、W07、W09、P3、P4 |
| [W03](p2-w03-boot-memory-map-ownership.zh-CN.md) | W02；P1 Hypervisor 镜像范围 | W04、W06、W08、W09、P4 |
| [W04](p2-w04-physical-page-allocation.zh-CN.md) | W03 | W05、W06、W08、W09、P3、P4 |
| [W05](p2-w05-dynamic-small-allocation.zh-CN.md) | W04 | W06、W08、W09、P3、P4 |
| [W06](p2-w06-platform-memory-inspection.zh-CN.md) | W02–W05 | W09、W10、P3／P4 评审者 |
| [W07](p2-w07-offline-dtb-compatibility.zh-CN.md) | W01、W02 | W08、W10、平台规划者 |
| [W08](p2-w08-host-robustness-regression.zh-CN.md) | W01–W05、W07 | W09、W10 |
| [W09](p2-w09-qemu-integration-regression.zh-CN.md) | W01–W06、W08；P0 QEMU runner 入口 | W10、P3／P4 规划 |
| [W10](p2-w10-p3-p4-handoff-contract.zh-CN.md) | W01–W09 | P3、P4、P2 完成评审 |

该图有意保持无环。W01–W05 建立基础链；W06、W07 提供可观测性与离线兼容性检查；
W08 测试 Host 侧稳健性；W09 提供参考平台集成证据；W10 记录消费者契约，
不声称阶段已经完成。
