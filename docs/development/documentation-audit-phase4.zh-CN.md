# 全仓文档审计——阶段 4 跨阶段边界复核

English edition: [phase 4 report](documentation-audit-phase4.md).

**Translation status:** Current
**Translation source:** [English source](documentation-audit-phase4.md)
**Source blob:** `f7293d482e2b2408a2e491377b7dc01490b67333`
**Authority:** 本文供中文读者使用；英文原文具有权威性。

**状态：** 阶段性审计报告；不是批准的设计、架构决策或阶段完成决定。
**快照：** `085dae0cf49fca69ce269afeecd6078582a6189b`（2026-09-28）。
**范围：** 接手队列第 4 项：跨阶段边界、P1 NC6→P6-V29、AUD-001–007 复核、P7-W07 合并与 W08 消费、P7 deadline-fold 失败语义。
**方法：** 只读检查现行工作包计划、详细设计和可用证据记录；源快照哈希见 `documentation-audit-phase4-boundaries.csv`。

## 发现

| 边界 | 当前评估 | 证据影响 |
|---|---|---|
| P2→P3/P4 映射与所有权 | **职责大体清晰，交付仍受阻。** P2-W02 已有实现证据，P2-W03 只有有界证据。P2-W10 区分平台事实、内存图、分配和下游机制；P2 输入不授权 AP 启动、SMP 锁或 Stage-2/Guest 执行。P2-W04 仍受 W04-LAYOUT/W04-MAP 阻塞，P2-W10 无实现/验证记录，P3-W01/P4-W01 reconciliation 尚未执行。 | 不得把计划交接合同或物理分配当作可写映射或下游实现证据。AUD-003 仍开放。见 B01–B03。 |
| P3 通知/transport→P6 SGI carrier | **概念兼容，但尚无已交付 carrier 合同。** P3-W07 是单槽 mailbox 加 WFE/SEV，明确不是中断；SGI 承载仍属 Reserved。P3-W08 在消费后才完成软件 Pending→Completed。P6-W04 只负责物理 SGI 发送/接收/EOI。尚无 kind→SGI ID 映射或经过验证的 P3 poll/consumer↔P6 receive adapter。物理 SGI 接收不能代替 P3-W08 软件完成。 | 在宣称 P6 承载 P3 transport 前，必须明确归属映射/adapter，并区分计数及确认语义。见 B04–B06。 |
| P4–P7 object/VMID/timer/IRQ 生命周期 | **所有权分属各包，跨包合同仍有未解冲突。** P4 address-space 拥有页表、VMID 和 mapping ledger；Guest frame 属于 W03，销毁要求 quiescence/invalidate/free/release 顺序。P5 handle 并非通用对象注册表；vIRQ/memory/shared-region handle 仍 Reserved。P6 分别负责物理 IRQ 完成、pCPU host deadline、每 vCPU Guest timer、vIRQ、LR 呈现和 maintenance。明确冲突是 P6-W03 combined EOIR 与 P6-W06 在 Guest 完成前保持 timer PPI active 的要求。 | 不要从 P4/P5 API 推断通用对象/VMID 生命周期。先解决物理中断 deactivate 合同，才能认为 Guest timer delivery 已闭合。见 B07–B09。 |
| P7-W07 合并→W08 消费 | **同层状态合并兼容；transport fan-out 尚未规定。** W07 先发布持久 pause/control 状态，再请求重新决策。W08 消费每 pCPU 的 level intent、重跑调度决策并复查。但 P3 单槽可能覆盖不同 consumer kind；W08 未说明每个 doorbell 是否扫描所有持久 consumer 状态，也未分配其 notification kind。W08 架构伪码看似每次调用都发送，而 R1 要求仅在 false→true 时发送。 | 明确共享 doorbell fan-out 或隔离通道、kind/SGI 分配、重触发规则和 false→true 发送语义。P3/P6 carrier 及远程控制时限仍无证据。见 B10。 |
| P7 deadline-fold 失败 | **失败保证不足，且引用的上游保证并不负责唤醒。** 即使 fold 失败，W06 仍提交 Blocked，并声称 P6-W06 最终会通过某在线 pCPU 送达。P6-W06 明确不会唤醒 absent vCPU；它只保证 Guest 下次进入时最终送达，而此保证不会触发下一次进入。W08 拒绝 idle 并返回决策循环。因此无 runnable work 且持续失败时，可能立即重试，没有保证唤醒、退避或终止恢复状态。 | 这是活性/避免忙转合同阻塞，不是已观察到的运行时故障。需定义由生产者负责的恢复/唤醒和有界非忙转策略（或在进入 Blocked 前 fail closed），并验证 timer delivery 与进展。见 B11。 |
| P8 machine compatibility→P9–P21 | **策略约束明确；值及若干后续阶段合同仍未批准/保留。** W14 要求 machine facts 走 W02 governance；未批准或发生漂移时阻断；区分 machine version 与管理/schema ABI，并规定 P9+ 约束。P9+ 必须消费有证据的事实，但自行设计设备协议；P17 migration/snapshot policy 为 Reserved；P14 DMA/IOMMU、P15 板级行为、P20 x86、P21 时限/威胁模型均须各自定义合同。具体 v1 值仍未批准，P8-V19 受阻。 | 这些属于准入约束，不代表未来阶段计划或制品缺失本身就是缺陷。不得从 QEMU 推导 machine 值，也不得将 machine version 用于不相关 ABI。见 B12–B13。 |
| P1 NC6→P6-V29 | **责任转移正确；证明缺失且详细设计覆盖不足。** ADR-061、P1 completion/known limitations、P6 task book 和 W12 plan 一致：NC6 未在 P1 通过，归 P6-V29。W12 详细设计/矩阵仍只有 FI-A–D 和 DV01–08，没有 V29；W13 plan 要求纳入 V29 或标记 blocked，但其 coverage 与 DOC-04 schema 仍停在 V28。未发现执行/验证记录。 | 继续标记 NC6 未通过；先把 V29 纳入 P6 owner design 和证据矩阵。无需重开 P1 完成声明。见 B14–B15。 |

## AUD-001–007 复核

现行源文件复核和阶段 3 映射均未发现能解决这些问题的修订或反证。范围如下：

| ID | 状态 | 复核结果 |
|---|---|---|
| AUD-001 | 开放 | P7-W06 仍存在合法交错：事件发布后 waker 看到 Running 返回，随后 blocker 提交 Blocked；内存序说明没有提供共享线性化点。属于 proposed-contract blocker，未在 scheduler 运行时复现。 |
| AUD-002 | 开放 | P3-W08 timeout 复用仍与 receiver ownership 和迟到 consumer 行为冲突。TransportNoop 不证明运行时 stale-TLB 暴露。 |
| AUD-003 | 开放 | 分配仍不代表普通分配页具有可写 Host 映射。P2 W04-MAP 只覆盖 allocator metadata；P1 有界完成声明保持不变。 |
| AUD-004 | 开放 | P4 单 address-space 激活假设仍未定义 P7 多 VM 的 current-context/reactivation 合同。 |
| AUD-005 | 开放 | P6 SGI send writes/encodings 与按 pCPU 统计的 receipts 在 fan-out 时仍不是同一单位。 |
| AUD-006 | 开放 | P8 W03/W14/W20 仍指定了相互竞争的 machine compatibility 权威位置。 |
| AUD-007 | 开放 | `docs/README.md` 仍称 `plans/` 为 approved implementation-level design，而 stage workflow 将 L3 plan 与 L4 detailed design 分开。 |

当前证据的路径、章节/行号和 blob ID 见边界 ledger。这些发现表示文档矛盾或交接缺失，不代表已证明运行时故障。

## 修复顺序与边界

先解决权威来源和 machine value gate，再处理映射及生命周期所有权；随后明确物理 IRQ/timer 完成与通知 fan-out；再处理 scheduler 活性、避免忙转和计数；最后将 NC6 纳入 P6 验证合同及证据记录。本报告不选择架构方案，也不授权实现。

没有修改代码、accepted ADR、阶段设计或运行时证据。未运行 Rust tests、QEMU、硬件或在线检查。文档/翻译机械检查随阶段交付记录。全仓审计尚未完成；接手队列第 5–6 项仍开放。
