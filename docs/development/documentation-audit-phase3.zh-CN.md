# 仓库文档审计——第三阶段：工作包交接映射

**Translation status:** Current
**Translation source:** [English source](documentation-audit-phase3.md)
**Source blob:** `9105367a5f1f28585697bdba63725f362a1b1676`
**Authority:** 本文供中文读者使用；英文原文具有权威性。

**状态：** Informative 审计记录；针对固定快照，交接队列第 3 项已完成。仓库级审计仍未完成。
**范围：** 全部 125 个 P0–P8 工作包的生产者／消费者输入与输出、需求—验收—证据映射、所有权、生命周期、失败／回滚、可用性、验证责任，以及依赖环核对。
**版本：** v0.1
**负责人／变更背景：** Codex，应用户要求继续审计，2026-09-28（Asia/Shanghai）。
**取代：** 无；补充[第二阶段全文审阅](documentation-audit-phase2.zh-CN.md)和[续接交接文档](documentation-audit-handoff.zh-CN.md)。

## 1. 快照与覆盖范围

审阅源快照为 `085dae0cf49fca69ce269afeecd6078582a6189b`。工作包清单包含 125 份唯一计划：P0 22 项、P1 12 项、P2 10 项、P3 15 项、P4 9 项、P5 10 项、P6 13 项、P7 14 项、P8 20 项。逐包台账包含计划／详细设计路径、blob 和行数；上游输入；输出及消费者；验收／证据行；带状态的契约边；以及未决发现或证据边界。

矩阵共记录 719 条边断言和 303 条需求／验收／证据映射。边断言不等同于唯一图边：生产者和消费者文档可能从两侧描述同一关系。边状态计数为 implemented 55、verified 33、planned 434、assumed 111、blocked 86。“implemented”和“verified”只代表被引用工作包记录的状态，不会自动把证据转移给下游消费者。多数 P3–P8 机制仍为 Proposed，因此相关边表示规划契约，不代表运行时能力已交付。

[JSON Lines 台账](documentation-audit-phase3-packages.jsonl)保留每个工作包的结构化映射和审阅记录。[工作包 CSV](documentation-audit-phase3-packages.csv)、[契约边 CSV](documentation-audit-phase3-edges.csv)和[验收 CSV](documentation-audit-phase3-acceptance.csv)便于筛选。[校验元数据](documentation-audit-phase3-validation.json)记录快照、工作包和行数，以及状态计数。全文审阅仍以第二阶段台账为准；第三阶段记录为映射所检查的章节和证据。

## 2. 依赖和交接环核对

审阅依据是计划与详细设计中真实的生产者／消费者表述，再对照所属任务书的顺序，并判断反向边是否由稳定接口或事后反馈解释。工作包引用图出现环本身不能证明执行死锁：许多文档会双向描述兼容性约束。主要候选关系如下。

| 候选边 | 判断 | 准入顺序或未决阻塞 |
|---|---|---|
| P2-W03 map draft/seal ↔ P2-W04 metadata plan | 有序的两阶段握手，不是执行环。W03 先给出 draft；W04 据此规划 metadata；W03 seal；之后 W04 才能初始化。 | W03 runtime-storage 契约已明确顺序。W04 仍受 W04-LAYOUT 和 W04-MAP 阻塞；不能据此推断 allocator 已就绪。 |
| P2-W07 fixtures → W08/W09，及 W09 C7 → W07 | C7 是运行后的交叉核对，不是实现初始 checker 的前置条件。 | W07 可以先定义并运行固定 fixture 预期；W09 再回传兼容性核对。W07-ADAPTER 与 W09-DTB 仍分别阻塞相关工作。 |
| P4-W02 MappingGrant ↔ P4-W03 grant producer | 任务书允许 W01 后并行设计；表面环来自共享接口，可通过确定顺序打破。 | 先由 W02 拥有并发布 MappingGrant/map 消费契约，再由 W03 按此契约产出 grant。设备映射契约和权限切片问题仍是真实设计阻塞。 |
| P4-W04 run contract ↔ P4-W05 guest scenarios | 接口／验证反馈关系，不是不可化解的实现环。 | 先冻结 W04 entry/BootInfo/exit 和场景表标识；W05 再提供内容；最后 W04 做集成检查。当前指定的 VG-009 可延期，且其 GuestFault 后重入与停止策略冲突，因此 W04 仍缺少可靠且必需的重入见证。 |
| P5-W01 ↔ W02；P5-W03–W08 | P5 任务书的依赖顺序无环。部分反向边描述之后的 ABI 关闭或场景反馈，不是生产者启动的前置条件。 | W01 先记录 ABI 缺口和证据边界；W02 后续提供设计事实。W03/W04/W05 契约先于 W06 组合，随后才由 W07/W08 验证。另有真实生命周期竞态：W04/W05/W06 授权路径可能在检查后、使用前销毁目标或撤销 grant，仅靠调整顺序无法关闭。 |
| P6-W03/W06/W07/W08/W09 | 接口引用形成较大的 SCC，但 P6 任务书给出的实施顺序无环。多数反向边是共享契约或完成报告。 | 先冻结 W03 物理中断完成语义、W07 vIRQ inject/claim/complete 契约、W05/W06 定时器输入，再接入 W08/W09 消费者。真实阻塞仍存在：W03 的 EOImode=0 combined EOIR 与 W06 要求 Guest 完成前保持 timer PPI active 相冲突。W03 将 split completion 留作 Reserved，W06 要求生产者无法表达时走 ACR。W09 orphan EOI 顺序也需澄清。 |
| P7-W02/W03；W04/W05/W08；W06/W07/W08 | 接口引用形成局部 SCC，但 P7 任务书的顺序无环。 | 先冻结 W02 生命周期 gate，再由 W03 提供 placement predicate；先冻结 W04 deadline/TimeSlice 输入，再由 W05 提供策略值并由 W08 消费；先冻结 W06 event/wake 与 W07 pause/pending 语义，再由 W08 提供传输。真实阻塞包括：W02 缺少 Paused→Blocked，W06 有 lost-wakeup 交错，W08 在 idle 检查和 wait 之间没有原子握手。W05 RoundRobin v0 还依赖未决 ADR-057；W08 所需 P3/P6 transport 和 P6 timer 输入也未验证。 |
| P8-W09/W10 场景语义 ↔ W15 fixture 内容 | Proposed 的双向内容依赖。W09 负责抽象启动里程碑；W10 负责工作负载／场景语义；W15 负责具体程序和 marker 字节。 | 先固定 W09 里程碑条件和 W10 场景 ID／能力／参数／marker 范围，再由 W15 实现 fixture 内容。设计暗示了这个分工，但当前没有已准入的 schema 或验收顺序强制执行。 |
| P8-W16 执行外壳 ↔ W18/W19 场景内容 | Proposed 的双向依赖。W18/W19 需要 W16 harness/oracle；W16 又需要其测试用例和保留场景内容。 | 先固定通用执行范围和稳定场景 schema；W18/W19 拥有填入其中的具体内容。该所有权分工可行，但接口和准入顺序尚未验证。 |

这些 P8 候选属于设计排序缺口，不是已观察到的实现死锁：所引用材料仍为 Proposed，相关工作包没有执行／验证记录。P8-W03–W10 文档也会互相引用以保持一致；这些引用并不自动构成执行前置环。具体 machine 值仍受 W02 gate 管控，P8-W01 也尚未按当前 P0–P7 证据完成核对。P6/P7 定时器、中断和调度关系同样需要所属契约负责人复核，之后才能视为已实施交接。

## 3. 优先交接发现

工作包映射进一步确认了以下跨阶段阻塞和证据边界。这些是待所属负责人复核的审计发现，不是修改 ADR 或实施提案设计的授权。

| 边界 | 证据及影响 |
|---|---|
| P1 内存映射 → P2/P3/P4 分配消费者（AUD-003） | P2 W04 分配物理帧，其 W04-MAP gate 仅覆盖 allocator metadata 的可写访问，并未建立 P3 栈或 P4 页表所用普通分配帧的 Host 可写映射。P4 的 proposed mapping grant 也有未解决的设备页／权限切片边界。在消费者依赖“物理分配即为可写指针”之前，须指定生产者、属性、生命周期、撤销和回滚契约。 |
| P3 通知核算 → P6/P7/P8 场景 | P3 W07 允许合并／latest-wins 送达，但 W12/W13 验收文案把发送数与接收数比较，仿佛每次发送都必须到达。需要先统一单位，并把已接收发送与合并或拒绝的尝试分别计数，测试才能作为证据。 |
| P6 物理定时器中断 → Guest 完成（P6-W03/W06） | W03 描述 EOImode=0 combined completion；W06 要求物理 PPI 在 Guest 可见定时事件完成前保持 acknowledged 但 active，并依赖后续 P6-W09 hook。设计说明：若 W03 无法表达 split completion，则需要 ACR。目前没有批准的 ACR、匹配的生产者 API 或运行时证据；此边为 blocked。 |
| P7 唤醒和生命周期 → 定时器／调度消费者（AUD-001） | W06 blocker 可能在事件发布后、waker 仍看到 `Running` 并返回时提交 `Blocked`；`wake_pending` 没有在共享同步协议下被消费。W08 在 idle intent 与 WFE 之间重复了此缺口。这是 proposed 协议阻塞，不是已执行观察到的运行时失败。 |
| P4 地址空间与 P5 rights → P7/P8 生命周期 | P4 单地址空间激活假设不能提供 P7 多 VM 切换语义（AUD-004）。P5 的授权后使用流程在解锁至执行期间没有固定目标／grant 的生命周期。这需要生产者负责定义重新激活及对象使用／撤销契约。 |
| P8 machine 与 regression 契约 | P8-W01 日期较早的基线声称 P1–P7 的实现／证据不存在，与当前快照冲突；W01 自身工作流要求刷新清单。W13 fault 分类顺序可能与 W12 Stage-2 fault 预期重叠；W13 中由谁把 panic 归并到 P7 Guest-faulted 状态也未分配。P8 场景和 fixture 双向关系需要按第 2 节固定内容接口顺序。 |
| 完成证据与制品 identity | P0-W16/W17 策略和入口已存在，但 hypervisor 制品名称所需的 profile 词汇仍为空；策略交付并不证明具体制品现在可被命名。P6-W12/W13 也必须将 P1 NC6 转交为 P6-V29，而不能从旧的同步 fixture 推断异步 vector 执行已获证明。 |

逐包行中还保留了其他发现，包括 P3 CAS 顺序和 timeout slot 重用、P4 重入与 mapping grant、P5 handle/revocation 竞态、P6 GIC completion 和 SGI 单位、P7 lost-wakeup 状态转换，以及 P8 fault、timer 和 fixture 分类。若所属证据尚未解决，既有 AUD-001–007 和 AUTH-01 均保持开放。

## 4. 修复路线与边界

后续另行授权的修复工作应按此顺序进行：先确定规范来源所有权和 machine-value gate；再关闭内存映射、中断完成、生命周期、通知单位和授权对象生命周期等生产者契约；接着冻结 regression 外壳与 fixture 内容接口；最后运行所属 Host、target 和 QEMU 验证。硬件、异步异常、SMP 和阶段整体完成声明须保持在源记录明确的范围内。

本阶段没有修改代码、已接受的 ADR、工作包设计或阶段证据。未运行 Rust 测试、QEMU、硬件或工作包场景。阶段交付时另行记录文档和翻译校验。交接队列第 4–6 项仍开放，包括优先边界复审、选择性源码／运行时证据核验和其余机械检查。仓库级审计尚未完成，本阶段未合并 PR。
