# P2-W11 普通分配帧 Host 映射——详细设计

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `7203979b4d7488e97274f778957fe125d59e3cb1`
**Authority:** 中文译文；英文原文具有权威性。

**状态：** 责任方于 2026-10-02 批准本详细设计（v0.2）；实现和运行证据未完成。
**上级：** [W11 计划](../../plans/p2-w11-host-allocated-frame-mapping.zh-CN.md)，M01–M05／V14。
**责任：** P2-ACR-02／AUD-003；替代 v0.1 的直接分配所有权草图。
编码前读取 P2 任务书、Coding Guidelines 和 [W12 设计](../p2-w12-minimal-memory-objects/README.zh-CN.md)。
这里的内部接口名称不是现有 Rust API。

**批准记录：** 项目责任方于 2026-10-02 明确表示“我确认批准”；范围见[批准记录](../../../../testing/documentation-audit/design-approval.zh-CN.md#owner-approval)。批准具体设计作为后续实现依据，不替代其中的前置条件、架构／容量核验和运行证据。

## 1. 基线与基础交付

当前 workspace 为 hypervisor 与 host-test-baseline。P1 Stage-1 和 W01 RO/XN DTB
窗口都不授予任意分配帧的可写访问。W04 视为未实现并将在后续重做，启动集成仍报告
allocator absent，W04-MAP 独立待完成。W12 拥有后备与视图保留；W12/W11 尚无生产实现。

| 要求 | 运行基础／责任方 | 设计处理 |
|---|---|---|
| M01 所有权覆盖 | W04 启动分配、W12 | 消费 W12 Host 区域，不消费裸 HPA 或 AllocatedFrames |
| M02 属性／别名 | W11 架构适配器 | RW/XN、Normal WB，W12 全局别名检查及保护范围排除 |
| M03 生命周期 | W12 记录、W11 作用域 | 稳定身份、未初始化字节访问、外部使用固定凭据 |
| M04 回滚 | W11 日志／完成后端 | 发布前预留；无法证明撤销时保留 |
| M05 有界启动访问 | 镜像内页表／窗口、独占 CPU 权限 | 几何已指定；最终链接容量与目标执行仍需证据 |

必需：启动 CPU 上普通 RAM 的视图与显式撤销。预留：独立准入的 P3 串行化和 shootdown。
范围外：Guest Stage-2、MMIO、Host 可执行映射、堆、全部 RAM 直映及替代 W04-MAP。

## 2. 单元、状态与容量

平台无关 HostFrameMapper 拥有映射槽、VA 预留与事务 ID；记录只保留 W12 MemoryRegion，
不持分配句柄。架构适配器拥有镜像内页表和描述符日志；启动组合在整个转换期间保留存储。
消费者持不可复制 MappedFrames。初始对象均 !Send/!Sync；丢失 token 保留资源，Drop 不撤销或释放。
不使用堆、递归、等待或 IRQ 路径分配。

启动配置：4 KiB 页，128 MiB VA 窗口 `[0x1_0000_0000, 0x1_0800_0000)`，
Stage-1 L1 槽 4，一张 L2 加 64 张 L3，共 65 页镜像内存储。与 P1、W01 槽 2、
W04 元数据槽 3 分离。这是 VA 预留，不是假设物理 RAM。Ready 前检查实际 P1 几何、
根槽空闲、地址宽度和所有既有映射；不匹配在修改根之前拒绝。

调用方提供 64 条映射记录和最多 32768 个叶条目的日志。每条记录是一段连续 VA 和一个连续 W12 区域。
有界 first-fit 查找 VA；碎片导致 Capacity，即使空闲页总数足够。初始化时清零并一次性链接静态页表，
保留至重启；后续只改叶条目。页表、日志和记录须满足链接器／启动可读范围及内存清单，不能把大日志放栈上。
根可能发布后的初始化失败保留所有表并终止启动，不返回 Ready；最终镜像容量必须实际验证。

## 3. 接口与生命周期

| 操作 | 契约／失败 |
|---|---|
| init | 验证几何、不相交、容量和执行权；发布清零表树。发布前失败不改根，发布不确定保留表并停止 |
| map_owned(region) | 仅 W12 Host RW/XN Normal WB；先预留 VA／槽／日志，再绑定事务并写原无效叶。拒绝返回原预留区域；发布后仅完成回滚才退役，否则返回 RetainedId |
| with_uninit_bytes | 验证身份、Live、无外部固定；回调只访问可用字节，生命周期不能逃逸；不提供裸 PA／指针访问或假设已初始化 |
| coverage | 返回对象／区域、VA／物理范围／属性快照；不是权力 |
| pin_use | 消费访问凭据，建立一个 W12 外部使用固定；禁止普通字节回调，消费者静止完成凭据才能解除 |
| unmap_owned | 无借用／固定；清叶、完成 Stage-1 失效、提交 W12 退役。Busy 返回 Live 凭据；完成不明则隔离 VA／租约；成功不返回分配句柄 |

对象控制方在所有视图／固定退役后单独调用 W12 take_back，随后才可 W04 free。
生命周期：Vacant→Reserved→Publishing→Live→Revoking→Vacant；发布或撤销不确定进入 Quarantined。
槽／事务序号检查且不复用；隔离保留 VA、W12 租约和日志。Reserved 才可无后端证明取消。
回滚只清本事务叶，覆盖整个范围完成失效后才签发退役凭据；不确定不返回“安全可用”的原区域。
初始化拥有的静态表不会被单次映射回滚释放。

## 4. 架构与访问边界

非 VHE EL2 适配器检查现有能力；RW 数据使用现有 MAIR Normal WB 属性，W11 叶明确设置
Inner Shareable，不能直接照搬 P1 非共享叶属性。P1 的 MAIR/TCR 与既有映射保持其原契约。
Core 不接触位域或寄存器。初次映射写叶后执行 DSB ISHST、本地 EL2 Stage-1 失效、DSB ISH、ISB，
才发布完成；撤销先清叶，再执行相同失效／屏障，才退役。允许保守的完整本地 EL2 失效。
启动配置下 AP 从未使用窗口；允许 AP 后本地完成不足，必须拒绝沿用这个后端。

每处描述符写和 VA→slice 都需证明：表存储有效、边界／对齐、唯一写者、属性、完成发布、
无 W12 冲突别名及受限借用。长度为检查后的可用页数×4096，不暴露 buddy 余量。
回调拿到 `&mut [MaybeUninit<u8>]`；消费者初始化后才可类型化读取，并负责类型／对齐。
Guest 代码还需 P4 指令可见性操作，之后才能撤销 Host 写视图并发布 Guest RX。
栈／页表是 UsePin，不是持久 Rust 可变 slice；页表 volatile 操作由架构责任方提供。

## 5. 错误、实现与验收

Rejected：未发布、归还输入；Busy：借用／固定阻止撤销；Retained：完成不确定、返回保留身份；
FatalInvariant：状态／凭据／硬件契约损坏，终止并保留。无隐藏重试。
顺序为有界模型与 W12→静态窗口初始化→架构发布／撤销→作用域字节→启动组合。
先评审 API 和容量，再 unsafe；随后验证最终链接容量与目标操作，不能用正在构建的映射引导其自身页表。

V14 覆盖外来／过期 ID、可用前缀溢出、VA 碎片／耗尽、别名、借用逃逸编译失败、固定撤销拒绝、
逐叶故障注入、撤销后无旧 TLB 访问、同 VA 不同对象复用及最终 W12/W04 精确计数。
Host 仅证明逻辑／算术，QEMU／架构证据证明实际映射／撤销／复用；AP 用例是准入拒绝而非 SMP 完成。
剩余执行门禁：ADR 正式集成、W12 实现、W04-MAP、最终架构与链接评审、QEMU V14。
几何和所有权已写明，不再是缺失设计；P2-HOST-MAP 仍需生产路径证据。
未来记录位于对应 implementation record 和 verification 文件，不因本设计命名而视为已存在。
