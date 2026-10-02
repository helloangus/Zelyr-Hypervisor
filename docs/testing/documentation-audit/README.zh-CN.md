# 文档审计——当前结论与后续工作

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `9161bdc2b83198c74ca02ee740a6db2b834cd5e3`
**Authority:** 中文译文；英文原文具有权威性。

**状态：** 资料性综合审计记录；覆盖审计已完成并保留发现，阶段实现／证据门禁仍开放。
**范围：** P0–P8 文档审计及已记录整改，不是阶段完成报告。
**版本：** v0.2。
**责任方／变更背景：** 2026-10-02，责任方批准详细设计并要求清理中间文档。
**替代：** development 中的审计接手、阶段 1–7 及不断追加的综合报告；按责任方明确要求删除中间记录，不保留归档。

<a id="current-findings"></a>
## 当前发现

这里是唯一的当前审计摘要；阶段设计／记录仍是契约和运行证据来源。
[内存／Stage-2 设计批准](design-approval.zh-CN.md#owner-approval)记录责任方
2026-10-02 的明确批准，不代替实现、硬件或阶段完成证据。

| 发现／责任方 | 当前处置 | 剩余工作 |
|---|---|---|
| AUD-001：P7 唤醒 | 已选择 W06 共享原子 phase/event 握手 | 实现并验证 W06-DV04、P7-V14/V25；独立关闭 W08 idle check/wait 和 deadline-fold 进展缺口 |
| AUD-002：P3 传输寿命 | 已选 W08-SYNC A；不替换 Pending，强 CAS 准入，collect 不持数据锁，检查释放、终止保留 | 本次内存／Stage-2 批准范围外的 W06/W08 设计准入、实现及 DV03/DV05/DV06 证据 |
| AUD-003：P2 Host 映射 | W12 v0.1、W11 v0.2 已批准；唯一后备所有者与保留视图已指定 | ADR-062 流程、W04-MAP、最终架构／容量核验、W12/W11 实现、消费者 Host/SMP 证据 |
| AUD-004：P4/P7 Stage-2 | W02/W03 v0.2、W10 v0.1、P7-W02 撤销配套 v0.1 已批准；安装／残留、退役、失败撤销已指定 | S2-INSTALL/RETIRE 生产路径、P7 清理／撤销实现及 V17–V22；尚无多 VM 运行准入 |
| AUD-005：P6 SGI 单位 | 已删除混合单位比较，指定调用／写入／目标尝试／确认／完成和受控／合并场景 | 实现及相关 P6-V04/V05 证据；不能从差值推算合并次数 |
| AUD-006：P8 权威位置 | 拟议设计冲突已解决：W02/W03 归 machine-types，W14 归 abi，W20 引用 | 实际获批公共契约、机器值及所属阶段证据 |
| AUD-007／DOC-MECH-01／DOC-META-01 | 导航、过期锚点、当前任务书版本头已修复 | 随变更维护检查，不代表运行结论 |
| AUTH-01：P0-W01 批准来源 | 已确认没有独立历史批准，删除错误批准声明 | 保持 P0-W01 Proposed 与单独完成证据；本次不追溯批准它 |
| P6 物理 IRQ／Guest timer 完成 | combined EOIR 与延后物理完成仍有生产者／消费者问题 | 明确含 orphan EOI 的完成接口／顺序并验证真实中断路径 |
| P5 先授权后使用的寿命 | 检查后、使用前可能撤销／销毁，仍为已记录契约问题 | 稳定对象／grant 保留或原子使用／撤销协议及验证 |
| P1 NC6／P6-V29 | 真正异步 unexpected-vector 执行未证明 | P6 执行并保存证据；P1 参考 QEMU 完成范围不变 |
| 其他阶段问题 | P6 W1C/timer/LR、P7 生命周期／串行化、P4 权限／重入、P8 timer/fault/fixture 已在历史审阅中记录 | 由各阶段源文档继续承担；本次清理不代表问题解决 |

P2 W05/W07/W09、P8-W02 机器值／P8-V19、ADR-057 RoundRobin v0 及其他前置契约继续由各自记录跟踪。
P8 后续 P9–P21、DMA/IOMMU、迁移、硬件及广泛安全／性能保证不因此成为当前阶段义务。

## 决策与实现边界

2026-10-02 责任方明确要求本次不提交 P2-W04，视为未完成，后续重新开展。
已排除其本地代码、新增包内设计补充、交付／验证记录；W04 是尚未提供的前置，旧本地测试不授予准入。
ADR-062 A 指定 P2-W12 通用基础、P4 Guest 适配；正式 ADR 状态仍见
[ADR-062](../../adr/adr-062-p2-minimal-memory-object-foundation.zh-CN.md)。

本次批准的设计入口：
[W11](../../stages/p2/implementation/p2-w11-host-allocated-frame-mapping/README.zh-CN.md)、
[W12](../../stages/p2/implementation/p2-w12-minimal-memory-objects/README.zh-CN.md)、
[P4-W02](../../stages/p4/implementation/p4-w02-stage2-address-space/README.md)、
[P4-W03](../../stages/p4/implementation/p4-w03-guest-memory-image/README.md)、
[P4-W10](../../stages/p4/implementation/p4-w10-multivm-stage2-handoff/README.zh-CN.md)、
[P7 撤销](../../stages/p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md)。
它们在声明门禁内作为实现设计依据；按用户要求在当前工作区记录批准，不声称也不要求为此合并 PR。

实现仍按生产者前置→消费者集成→实际验证推进；批准不默许跳过未完成门禁。
线上分支保护、运行工件异地保管、未来机器值仍未重新核验；清理没有新做外部或运行调查。

<a id="historical-records"></a>
## 历史记录

原审计快照为 `085dae0cf49fca69ce269afeecd6078582a6189b`，有 967 份全文输入、125 个历史计划，
后续补充至 128 个计划。原交接清点记录 217 条跨阶段关系：17 条双向、60 条仅生产者、
106 条仅消费者、34 条仅断言，117 条缺少带状态断言。这些不对称仍需责任方复核，不是已交付契约。
W10 七条 P7 关系及 W12 三条 P4 关系属于单独规划补充。

2026-10-02 责任方明确要求：“压缩包不要提交，这些记录不需要存档”。
原 development 下的 36 个审计文件已移除；临时压缩包和清单也已删除，不保留审计归档或当前差异台账。
历史数量只作背景，不作为当前文件统计，
当前契约、上表未完成项和专项批准记录继续保留，本摘要替代旧阶段／接手导航。

## 维护与验证

development 回归指南／治理。本 testing 子目录仅保留当前摘要和专项批准记录，不保留历史工作文件归档；
后续直接更新现有记录，不再按轮次堆叠报告。这属于现有 Testing 类别中的证据整理，不创建新的架构／政策类别。

清理检查包括中间文件／压缩归档已移除、当前相对链接／锚点、CI 文档检查、翻译元数据和空白。
不重跑 Guest/QEMU、代码测试或架构核验。无生产代码、unsafe、ABI、依赖变更；保留已有工作区修改，未合并 PR。
