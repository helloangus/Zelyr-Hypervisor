# P2-W12 最小 MemoryObject／MemoryRegion——详细设计

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `6ad1d5bb2d2593468243c1847f50d9c0d3ab4bbf`
**Authority:** 中文译文；英文原文具有权威性。

**状态：** 责任方于 2026-10-02 批准本详细设计（v0.1）；实现和运行证据未完成。
**上级：** [W12 计划](../../plans/p2-w12-minimal-memory-objects.zh-CN.md)，N01–N06／V15。
适用 P2 任务书、ADR-062 和 Coding Guidelines。替代 W11 直接分配所有权草图及
P4-W02/W03 的裸范围所有权草图，修订后的消费者使用本契约。

**批准记录：** 项目责任方于 2026-10-02 明确表示“我确认批准”；范围见[批准记录](../../../../testing/documentation-audit/design-approval.zh-CN.md#owner-approval)。批准具体设计作为后续实现依据，不替代其中的前置条件、架构／容量核验和运行证据。

## 1. 当前状态与基础交付

workspace 为 hypervisor 与 host-test-baseline。W04 尚未实现；W12 要求后续 W04 提供不可复制／克隆的 AllocatedFrames，
检查 owner／serial，range 是完整 buddy 块，usable_count 是可用前缀，free 失败保留句柄。
分配器实现、启动分配和 W04-MAP 均待后续重做；W12、W11、P4 适配器没有生产实现。

| 要求 | 基础／责任方 | 设计与证明 |
|---|---|---|
| N01 唯一后备所有者 | W04 真实句柄 | 仅移动一次，不导入 HPA，不另建分配台账；精确归还测试 |
| N02 身份／容量 | W12 存储和启动身份权威 | 不复用身份；检查耗尽 |
| N03 有界视图 | W12 验证器 | 可用前缀、权限／类型、别名矩阵 |
| N04 保留 | W11／P4 完成适配器 | 线性租约、绑定完成凭据；真实目标证据独立 |
| N05 回滚／计数 | W12 状态机 | 发布不确定时保留；逐边界故障注入 |
| N06 消费者 | W11、P4、P3 SMP | 按第 6 节适配；模拟仅证明逻辑 |

必需：普通 RAM、稳定身份、有界区域和显式退役。预留：P3 已准入的同一存储串行化适配器。
不包含堆、MMIO 所有权、COW、分页、Guest 策略、DMA、自动 Drop 回收、新分配器标签或生产模拟完成。

## 2. 单元、存储与身份

object_store 保存真实 W04 句柄；region_store 保存预留／活动／撤销视图；view_rules 做纯检查；
completion_adapter 绑定架构凭据。这些是平台无关内存层逻辑模块，不新增 workspace crate。
架构适配器不能生成对象身份或释放后备。

启动组合提供 O 个对象记录、R 个区域记录及独占 IdentityAuthority；存储寿命覆盖所有句柄，
可为镜像内或调用方所有，不假设堆。字节预算按检查后的 O×对象记录大小＋R×区域记录大小及对齐计算。
零容量、溢出、存储不足在 Ready 前失败。一个区域仅覆盖一个对象的连续可用子范围；有界扫描 R 做别名检查。
不决定消费者 RAM 大小，不使用 buddy 余量，不使用无界链表／递归。

启动权威签发非零单调 StoreId，各存储签发不回绕对象和区域 serial；槽号本身不是身份。
耗尽关闭该域的新建，不回绕。ObjectId=(StoreId,对象 serial)，RegionId=(ObjectId,区域 serial)。
原分配器 owner 身份仍在 W04 私有句柄内。

MemoryObject 是绑定存储寿命的不可复制控制凭据，不是第二份 allocation。
MemoryRegion 是不可复制的保留租约；范围快照不是权力。忘记凭据时记录仍拥有资源，安全泄漏有界容量。
非空存储拒绝关闭；Drop 不释放。生产初始化消费封闭启动存储租约（稳定的 `&'static mut` 记录），
不把存储交还调用方；遗忘所有 token 也不能让异步后端指针悬空。局部寿命存储仅用于 Host 测试，
不能登记生产后端；后端仍可能引用时存储保持固定。

## 3. 内部接口

名称为设计接口，不是现有公共 API；具名枚举错误，无隐式重试、panic、裸指针导入或阻塞。

| 操作 | 状态／所有权与失败 |
|---|---|
| adopt(frames,ceiling,NormalWb) | 预留对象记录并移动真实句柄，只暴露可用前缀；提交前失败归还相同句柄；提交后存储是唯一后备所有者 |
| reserve_region | 检查非空、对齐、可用范围、权限上限、存储身份／目标、别名和容量；先保留后返回；失败不改计数／后端 |
| begin_publish | 消费 Reserved 区域，绑定不复用事务与目标后才可首写描述符；身份错误归还原租约 |
| commit | 完成凭据的事务／区域／后端／目标／范围／权限必须完全匹配；不匹配隔离保留 |
| cancel_reserved | 仅从未进入发布的 Reserved；恰好移除一次保留 |
| finish_rollback | 后端证明未暴露或已完全撤销后才移除；不确定隔离 |
| begin_revoke | 无字节借用／外部固定；Busy 归还 Live，成功阻止新访问并继续保留 |
| finish_revoke | 精确凭据证明声明 CPU 范围内访问／转换已结束；只减少一次，外来／重复凭据无效 |
| take_back | 所有区域状态、借用和固定都已消失，消费对象并返回原完整 W04 句柄；Busy 保留对象 |

只在 take_back 后调用分配器；FreeFailure 返回的句柄必须继续被拥有，不能算释放成功。
W12 只管对象／区域保留计数，不另建物理 free/allocated 位图。

begin_reprotect 在同一区域上预留新权限，检查上限和所有其他别名，记录旧／新属性并进入 Revoking，
计数始终为一。旧转换退役凭据之后才可发布新属性，再凭 PublishReceipt 回到 Live。
发布前拒绝归还原 Live；硬件可能变化后的失败隔离。初始版本不支持类型改变或区域拆分。

## 4. 别名与访问

视图权限仅 R、RW、RX，写必须含读，拒绝 W+X。对象权限上限可同时允许 RW 初始化与后续 RX，
但上限不是视图，也不授权同时写／执行。初始类型为 Normal WB、Inner Shareable。
同一对象重叠区域只有在双方只读且类型相同时才允许；任一可写则与重叠的预留／活动／隔离区域冲突，
即使跨后端／VA。不同对象由唯一 W04 句柄保证不重叠。只有绑定后端能授予字节访问。

Host 使用 W11 作用域 MaybeUninit 视图。RX、硬件正在使用的栈／页表不能暴露普通可变 slice。
外部使用须不可复制 UsePin，绑定区域／消费者／epoch；固定期间无普通字节借用和撤销。
架构责任方只可按自己的安全契约进行限定 volatile／页表操作。
同一有界记录最多一个独占外部使用者；解除固定须消费者的受审计静止完成凭据，不能用公开 bool。

初始存储仅启动 CPU，!Send/!Sync。跨 CPU 须 P3 串行化 facade，以短锁执行相同状态迁移，
在后端调用／等待前返回事务 token 并释放锁。Send/Sync 实现前审计 token 移动和存储寿命，W10 不能强转绕过。

## 5. 状态与完成权威

Reserved→Publishing→Live→Revoking→Removed。Reserved 可直接取消；发布／撤销要有完成凭据。
不确定进入 Quarantined，仍占容量并保留后备、目标、事务直到重启或另行评审的恢复操作；本设计不提供乐观恢复。
对象一直 Held，直到最后区域／固定消失后显式 take_back。

完成凭据构造器只在受审计 W11/P4 架构模块中，绑定完整身份、事务序号、阶段、范围、完成 CPU 范围，
不可复制；存储检查当前阶段及字段后恰好消费一次。模拟适配器仅 host-test，不能成为生产 fallback。
若模块边界需要可扩展后端 trait，其实现是明确 unsafe 契约，普通 safe 外部代码不能伪造完成。

硬件前预留全部记录。发布前拒绝保持输入；可能发布后仅完成回滚才退役，否则隔离。
Err 本身不证明无转换。后端、跨 CPU 等待、W04 free 时不持锁；完成时重取短锁并复核事务身份。

## 6. 消费者协议

- W11 消费 Host 区域，拥有 VA／页表事务，完成 Stage-1 失效后退役，不拥有 AllocatedFrames。
- W03 持对象控制凭据；先 Host RW 清零／加载／指令可见性，结束借用并完全撤销，再预留不重叠 Guest RX/RW。
- W02 持 Guest 租约、页表对象及 W11 页表固定；无安装／遍历／旧转换后才解除页表固定，不释放 Guest 后备。
- P3 栈固定覆盖 CPU 停止与切栈；分配句柄或 CPU 状态标签不足以证明静止。
- W05 不自动改写堆；未来堆适配须遵循本契约，W12 不反向依赖堆。

## 7. 实现与验证

顺序：有界身份／实际 W04 adoption→范围／别名／状态机→仅测试完成适配器与故障注入→
能力复制、借用逃逸、凭据构造和跨线程误用的编译失败测试→批准并具备前置后的 W11/P4 集成。
测试覆盖外来身份、耗尽、可用前缀／完整句柄、溢出／容量、所有非法阶段、计数等于占用记录、
每个发布／失效／完成故障、token 丢失、提前 take_back、重复减少和 W04 free 失败。
不能只测试 enum setter，必须测试复用与外来事务。

V15 每个 N01–N06 都记录容量、身份／别名矩阵、失败、保留和归还句柄结果。
Host 假完成不证明 TLB／CPU 静止；实际 Host map/unmap/reuse 与 Guest unmap/destroy 证据独立。
未来 implementation／verification record 不是本文已存在的结果。未来架构／存储 unsafe 须逐处审计计数；
本次文档不增加依赖、公共 ABI 或生产 unsafe。
