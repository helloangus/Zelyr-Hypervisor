# 内存与 Stage-2 设计批准记录

**Translation status:** Current
**Translation source:** [English source](design-approval.md)
**Source blob:** `210600f12c606f1c66330d9603d4f2e990f8eb2e`
**Authority:** 中文译文；英文原文具有权威性。

**状态：** 资料性批准事实记录；下列详细设计已由责任方批准。
**范围：** 本轮 W12／W11／P4 设计修订及 P7 进入前撤销配套。
**版本：** v0.2。
**责任方／变更背景：** 项目责任方于 2026-10-02 明确确认批准。
**替代：** 原内存／Stage-2 整改复核便笺及下列范围的“待批准”状态；其他阶段批准不变。

<a id="owner-approval"></a>
## 责任方批准

**批准人：** 项目责任方（用户）。**日期：** 2026-10-02。
**决定：** 在说明详细设计批准意味着“本版作为后续实现依据”之后，责任方明确回复“我确认批准”。
**记录方式：** 遵循不合并 PR 的既有要求，在当前工作区记录；不虚构评审机构、历史 PR 或运行结果。

| 获批设计 | 版本 | 范围 |
|---|---|---|
| [P2-W12](../../stages/p2/implementation/p2-w12-minimal-memory-objects/README.zh-CN.md) | v0.1 | 通用后备／视图寿命、有界存储／身份、权限／别名、完成权威 |
| [P2-W11](../../stages/p2/implementation/p2-w11-host-allocated-frame-mapping/README.zh-CN.md) | v0.2 | W12 Host 适配、VA／表容量、作用域字节／固定、回滚／撤销 |
| [P4-W02](../../stages/p4/implementation/p4-w02-stage2-address-space/README.md) | v0.2 | README 和 01–05：Stage-2 所有权、状态、接口、流程、验证 |
| [P4-W03](../../stages/p4/implementation/p4-w03-guest-memory-image/README.md) | v0.2 | README 和 01–05：W12 Guest 适配、固定 IPA／独立 HPA、先加载后映射、指令可见性、释放 |
| [P4-W10](../../stages/p4/implementation/p4-w10-multivm-stage2-handoff/README.zh-CN.md) | v0.1 | 多空间、每 CPU 安装／执行／残留、退役、失败及 V17–V22 |
| [P7-W02 撤销配套](../../stages/p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md) | v0.1 | 身份绑定的进入前撤销及 W02 生命周期／W04 切换／W05 队列衔接修订 |

包含对应中文译本，以及这些契约需要的 P4-W04 执行租约／显式脱离衔接。
不扩大为所有其他 P4/P7 工作包、P3 W06/W08 整包设计、追溯 P0-W01 或其他未解决 ADR 的批准。
ADR-062 独立记录的正式生命周期保持不变，用户已选择的 A 仍是架构方向。

## 剩余执行门禁

批准不证明架构指令顺序、最终镜像／栈／页表容量、缺失上游能力或运行行为。
W04-MAP、W12/W11 生产实现、P3 SMP 适配、P4 Stage-2／Guest 实现、P7 清理／撤销集成，
以及各自 Host／目标／QEMU 证据仍须完成；设计在对应实现步骤保留这些检查。
AUD-003/AUD-004 因此不标为实现完成或运行闭环。

## 当前记录维护

当前问题见[审计摘要](README.zh-CN.md#current-findings)。责任方要求删除中间记录并明确不要归档。
本目录只保留此批准记录和当前摘要的中英文版本；没有压缩包、工作台账或重复阶段／接手报告。
本次只登记状态和整理文档，不新增生产代码、unsafe、ABI、依赖，也未执行 Guest／目标测试。
