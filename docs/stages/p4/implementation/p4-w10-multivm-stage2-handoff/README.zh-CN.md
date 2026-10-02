# P4-W10 多 VM Stage-2 生产者扩展——详细设计

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `b9fcc0838d0a858443d0bf90e568e1d850dde67c`
**Authority:** 中文译文；英文原文具有权威性。

**状态：** 责任方于 2026-10-02 批准本详细设计（v0.1）；实现和运行证据未完成。
**上级：** [W10 计划](../../plans/p4-w10-multivm-stage2-handoff.zh-CN.md)，X01–X06／V17–V22。
适用 P4 任务书、[生产者要求](../p4-w02-stage2-address-space/06-multivm-handoff-requirements.zh-CN.md)
和 Coding Guidelines。不替代已接受 ADR，不意味着 P7 完成。

**批准记录：** 项目责任方于 2026-10-02 明确表示“我确认批准”；范围见[批准记录](../../../../testing/documentation-audit/design-approval.zh-CN.md#owner-approval)。批准具体设计作为后续实现依据，不替代其中的前置条件、架构／容量核验和运行证据。

## 1. 基线与基础交付

当前 P4 仍处于设计阶段，没有生产 Stage-2、多 VM 执行或 P7 调度器。W02/W03 已按
[W12](../../../p2/implementation/p2-w12-minimal-memory-objects/README.zh-CN.md)／
[W11](../../../p2/implementation/p2-w11-host-allocated-frame-mapping/README.zh-CN.md)修订。
P3-W06/W08 提供短锁／获准远程执行协议，不提供 Stage-2 退役证明；Host 假寄存器不是运行证据。

| 要求 | 责任方／设计结果 | 所需证据 |
|---|---|---|
| X01 多空间 | W02/W12；独立根、对象、非复用 VMID | V17 |
| X02 每次调度安装 | W10 每 CPU selector、W04 | 精确安装身份和执行租约；V18 A→B→A |
| X03 非当前空间修改／销毁 | W10 生命周期协调 | 残留历史、冻结事务、保持 B；V19 |
| X04 跨 CPU 退役 | P3 短锁／W08、W10 TLBI 适配器 | 每个残留 CPU 的绑定完成；V20 |
| X05 安全失败 | W10 分类，P7 撤销 | 不隐式改调度状态；V21 |
| X06 消费者证据 | P4 生产、P7 准入 | 容量／配置／CPU 集合及交接；V22 |

生产前置是有证据的 W01–W09 基础、W12/W11 路径和 P3 锁／传输。须审阅 P7 消费者契约，
但 P7 运行实现不是 W10 测试前置，不产生反向循环。
扩展配置：普通 RAM、4 KiB、非 VHE、管理器统一的已验证 VTCR 几何、多根、非复用 VMID、显式跨 CPU 完成。
预留 VMID 复用、迁移策略、未知硬件状态恢复；不含 DMA/IOMMU、脏页、大页、设备虚拟化／重分配或共享直映控制台。
W10 在登记前拒绝带控制台的空间。扩展 Guest 通过受控同步异常后的保存寄存器报告结果，
不共享 PL011，也不引入 hypercall ABI；基础 P4 控制台测试保持独立。

## 2. 模块、身份与有界存储

space_registry 拥有空间、根／表／Guest 租约；context_selector 拥有每 CPU 安装记录和执行租约；
retirement_coordinator 拥有修改事务；arch_stage2 拥有寄存器、TLBI、屏障和私有完成凭据。
只有 P7 改 current_vcpu、运行状态和队列；W10 不写这些字段，也不靠它们推断硬件执行。

调用方提供稳定 S 个空间、C 个 CPU、T 个事务记录，均非零，容量／对齐计算受检查。
每空间最多一项修改，每 CPU 一项选择；全局传输还可能 Busy。SpaceId 包含管理器身份和不回绕序号；
TransactionId 包含空间、映射 epoch 和序号。耗尽拒绝，不回绕。残留集合恰有 C 位，CPU 下标受检查。
VMID 使用验证过的 8 位配置，保留 0，单调消耗 1–255；适用时 VTCR.VS 选择该配置。
签发后的创建失败也消耗值；销毁只留墓碑，不复用。更宽硬件不自动启用。Ready 前验证物理宽度、
根几何、临时 IPA 范围和表预算，不支持则拒绝。

短 P3 数据锁只做状态预留／提交；锁外做硬件、W08、等待、回调、free 和 Guest 进入。
W12 的 P3 串行化 facade 单独加锁，不嵌套：先准备 W12 租约，再预留 W10，失败在两个锁外按 token 协调。
凭据绑定每个阶段的完整身份。

## 3. 空间、CPU 与执行租约

空间：Open→Frozen(tx)→Open；销毁 Open→Retiring→Destroyed；完成不明为 Quarantined。
冻结禁止新的选择、进入和修改。已有安装引用可在没有 Guest 执行时保留；不能把生命周期写成 Active(cpu)。

CPU：Idle／Stable(space,root,vmid,epoch)／Switching(tx,old,new)／Maintenance(tx,saved)／Unknown(tx)。
只有 selector 修改安装权威。残留历史包含所有可能产生转换的 CPU，包括不确定安装；切到其他空间不删除历史。
CPU offline 也需已证明的退役握手；Unknown 保留旧／新根。

acquire_entry 验证本 CPU 安装凭据／epoch、空间 Open、无已有执行租约，返回不可复制 EntryLease。
W04 真实进入时消费，真实退出后退役；未进入的取消走独立操作，Drop 不代表退出。
每个可执行范围还须当前 PreparedCode 凭据覆盖该 CPU；W03 拥有内容／缓存准备，可写映射发布使凭据失效。
新增执行 CPU 须先静止并准备，不能把 Stage-2 TLBI 当指令同步。
修改要求无该空间执行租约和在途选择，否则 Busy 不写 PTE；停止 vCPU 的策略归 P7。

## 4. 安装与进入前失败

select_context(cpu,expected_current,desired) 在该 CPU、Guest 停止且满足本地异常边界时调用。
检查精确旧身份、目标 Open、配置／容量；短锁内预留旧新根引用，先记新残留历史，再释放锁。
架构操作安装 VTTBR 根／VMID 和公共 VTCR，完成 DSB/ISB 等顺序并检查安装结果，返回私有凭据。
A→B→A 必须实际选择；同空间只有权威 CPU 元组／epoch 匹配且寄存器独占未破坏时才可省写。
Guest 返回路径不能另写这些寄存器。

成功提交 Stable(new)，释放旧安装引用但保留其残留历史。select_idle 显式有序关闭 Stage-2，
销毁不能擅自关闭另一个当前空间。

| 结果 | 硬件／资源事实 | 消费者处理 |
|---|---|---|
| Rejected/Busy | 未改寄存器、旧上下文保持、无新执行租约 | P7 可执行其自己的准入撤销 |
| Restored | 新 Guest 从未进入，精确旧／Idle 已有序恢复，临时引用退役 | 其他子系统也安全后才撤销 |
| Indeterminate | 可能部分安装或恢复失败；CPU Unknown，保留两边资源 | 终止该调度路径，不普通入队或释放 |

成功安装后、Guest 进入前取消，也要退役未使用 EntryLease 并显式恢复旧／Idle。
一个子系统安全不代表其他子系统已清理；selector 不修改 P7 状态或队列。

## 5. 修改、BBM 与残留退役

map/unmap/protect/destroy 在执行／选择引用消失后预留 Frozen/Retiring，竞争准入看到冻结即 Busy。
首写前验证所有范围、保留 W12 租约、预留全部表／日志。禁止原地覆盖有效描述符；初始配置不支持类型改变。

unmap/protect：先清叶，DSB ISHST，对所有残留 CPU 退役旧转换并收齐精确完成；protect 之后才发布新权限，
完成发布顺序及保守失效后重新 Open。初次 map 仅无效→有效，发布并完成所需顺序／失效再开放。
从未暴露且从未驻留的空间可无远程退役，但仍需描述符顺序；“非当前”本身不足以免失效。
摘除的中间表和 Guest 租约在完整遍历／转换完成前持续保留。发布后回滚也必须退役；Err 不是归还权。

### 5.1 本地与远程架构操作

所有空间采用统一验证过的 VTCR。目标 CPU 暂停 Guest，保存当前 B 或 Idle 安装元组，进入 Maintenance，
选择目标 A 的根／VMID，然后执行本地完整 VMID 的 Stage-1＋Stage-2 失效：
DSB ISHST；TLBI VMALLS12E1；DSB ISH；ISB。再恢复原根／VMID／使能状态并完成所需 ISB，才离开 Maintenance。
这里只用本地 TLBI，因此显式覆盖每个残留 CPU。寄存器 accessor 仅在 Arch；保留位／不支持配置在准入前拒绝。
不能在原来是 B 的 CPU 上留下 A。

开始修改前 A 无执行租约。远端正在运行 B 时可由 P3 handler 服务：B 租约仍活动，handler 保存其架构状态，
完成恢复后才回 B；维护不产生 A 执行租约。无法恢复 B 属于 fatal／indeterminate，不发布远程完成。
编码前架构评审须固定 Arm 版本并核对寄存器／TLBI／屏障；Host 状态测试不验证硬件语义。

### 5.2 W08 绑定

稳定事务记录保存空间／VMID／根、epoch、操作、精确目标集合及本地／各目标恢复完成。
opaque payload 是不复用查询 ID，不能是栈指针；记录覆盖所有 Pending 和迟到接收者寿命。
W08 排除发起者，本地完成须单独处理。
接收者检查精确事务，执行失效及恢复后才发布完成；只有绑定该操作／目标集合且本地完成的
W08 Completed 才可成为退役依据，普通 ACK 不够。

遵循 W08-SYNC A：一次强 CAS 准入、竞争 TransportBusy、collect 不持数据锁、接收者独立、受检查释放。
超时不能覆盖 Pending。若尚未修改 PTE 就 Busy，可撤销预留并 Open；PTE 已变后拒绝／超时／部分发布则隔离，
保留事务／页表／区域。记录迟到 ACK 但不自动 free；无隐藏重试／公平性假设。

## 6. 销毁

destroy 要求无安装／执行／选择引用，否则无效果 Busy。调用方先让各安装 CPU 显式选择 Idle／其他空间，
残留历史仍参与最终退役。预留 Retiring、清所有映射、完成每个残留 CPU 的退役，之后依次退役 Guest 租约、
页表使用固定、W11 页表视图；W12 take_back 后才 W04 free 页表对象。Guest 对象由 W03 控制。

B 当前时销毁非当前 A，必须保持 B 身份、权限和可运行性；不全局关闭、不重置其他根、不复用 VMID。
计数分开记录真正 free、保留失败、永久消耗 VMID。W04 free 失败继续保留返回句柄，不算容量恢复。

## 7. P7 准入撤销

[P7-W02 的补充设计](../../../p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md)
用 DispatchAttempt 绑定 CPU／vCPU／epoch。Guest 未进入、P6 timer/LR 和 W10 上下文都完成安全清理后，
P7 在单个责任方临界区通过既有 Deschedule 事件把 Running→Runnable 并清 current_vcpu。
返回候选给 W05，不直接入队；W05 处理 pending stop/pause 并复核资格后最多插入一次。
硬件不确定保留准入／资源并终止。出站 Guest 已完成的退出及生命周期效果永不回滚。

## 8. 实现、验证和交接

顺序：W12 多空间／非复用 VMID→假寄存器 selector／执行租约→本地架构和 QEMU A/B→
P3 传输／W12 串行化适配→P7 责任方撤销测试与交接。局部 CPU 通过不关闭 X04 或整个 W10。

| 验收 | 必须观察 |
|---|---|
| V17 | A/B 同时存活、独立根／VMID、相同 IPA 不同 canary |
| V18 | A→B→A 的真实 Guest 访问和权限正确，每 CPU 安装轨迹 |
| V19 | 非当前 A 修改／销毁后无旧转换，B 寄存器／映射／运行正常 |
| V20 | 至少两个准入 CPU，完成后无 A 旧转换；offline／迟到目标安全保留 |
| V21 | 每个发布／安装／恢复／collect 故障，区分安全撤销与终止，无提前 free／重复队列 |
| V22 | 配置、几何、容量、CPU 拓扑、事务身份、证据链接覆盖七条 P7 关系 |

模型只证明状态／计数；寄存器／Guest／QEMU 只证明声明配置，不推断硬件或 DMA。
未来 implementation record 和 verification record 独立记录实际工作。
未来 unsafe 限于 Arch 寄存器／页表和经评审跨 CPU facade；逐处 SAFETY 审阅。本次无代码／公共 ABI 变更。

## 9. 架构来源边界

英文稿链接 Arm 102142 issue 01 AArch64 virtualization guide，支持区分安装上下文与 VMID 标记的残留转换。
该指南本身不是本设计维护序列的完整规范证明；生产准入仍须适用 Arm ARM 指令／寄存器评审及目标证据。
文档补齐不等于架构批准。
