# 全仓文档审计——后续接手记录

**Translation status:** Current
**Translation source:** [English source](documentation-audit-handoff.md)
**Source blob:** `46438e71238c65391e1f940b414d7cb7d9640ae5`
**Authority:** 本文是供中文读者使用的译文；[英文原文](documentation-audit-handoff.md)具有权威性。

**Status:** 资料性历史交接；其初始的部分审计状态已由[阶段 7 闭环核对](documentation-audit-phase7.zh-CN.md)和[综合报告](documentation-audit-final.zh-CN.md)更新。责任方决策与整改仍待处理；本文不是已批准设计或阶段完成决策。
**Scope:** 保存全仓文档审计的发现、实际覆盖、证据检查及剩余工作。
**Version:** v0.1
**Owner/change context:** 用户于 2026-09-28 要求建立接手分支；后续 agent 负责剩余审计。
**Supersedes:** 无；将前一会话的阶段性报告持久化，不扩大证明边界。

## 当前状态（2026-09-28 后续更新）

声明范围内的当前树审计已完成覆盖：981 份 Markdown、全部 125 个 P0–P8 工作包映射，以及核对后的 217 条跨阶段工作包关系。明确且无歧义的文档发现 AUD-007（计划／设计导航）、DOC-MECH-01（过期锚点）和 DOC-META-01（当前任务书版本头）已修复并复核。责任方已选择 AUD-001 的 W06 共享原子握手方向，并记入拟议详细设计；实现、W06-DV04/P7-V14/P7-V25 证据和相关 W08 idle-interlock 缺口仍待完成。AUD-002–006、AUTH-01 批准来源、不对称交接记录及阶段证据／准入阻塞也仍待所属阶段处理。未声称运行时闭环。范围和验证见阶段 7 与综合报告。

## 1. 从这里接手

- 审计基线：`b317b88ad6f78f82143ec6af2cf2b41a143ae741`，开始时为干净的 `main`。
- 接手分支：`docs/documentation-audit-handoff`。
- 在该初始基线上，审计**尚未完成**：当时有 963 份受 Git 跟踪的 Markdown、130,759 行、125 个源语言工作包计划。后续阶段报告已更新覆盖状态；机械扫描仍不能代替全文语义审阅。
- 原始交接分支只记录状态，未修复 AUD-001–007。后续工作已修复上文列出的明确文档问题；没有批准拟议设计、修改已接受 ADR 或实现后续阶段机制。
- 原始授权启动审计并交付中文报告和按依赖排序的修复路线。后续工作已在当前分支完成声明范围内的审计及常规文档修复；用户未要求合并 PR。
- 接手时读取 `AGENTS.md`、[文档路由](../README.zh-CN.md)和待审领域的治理文件，先比较当前检出与基线的差异，再沿用发现。
- 下列清单仍是历史快照。当前覆盖和责任方待办见上文及第 5 节。没有待恢复的子 agent 或后台测试。

配套清单：

- [文档清单](documentation-audit-documents.csv)：基线每份 Markdown 一行，含 Git blob、阶段、层级、语言版本、标题、开头状态文本及保守审阅状态。
- [工作包清单](documentation-audit-packages.csv)：每个源语言计划一行，含详细设计入口，以及独立的上游、下游、验收审计状态。

清单是基线快照，不含本交接及新增文件。前一会话没有维护逐篇全文阅读台账，因此不会追溯认证任何行已完整审阅。`targeted-review` 表示看过选定内容，部分工具输出可能被截断；全文重读及反证检查仍待完成。标题状态和路径分类只是定位辅助，不代表批准认定。译本、历史版本和权威关系仍需语义分类。

## 2. 阶段性审计确认的问题

位置对应审计基线，后续行号可能变化。“确认”表示发现了文档矛盾或反例，不表示执行过生产漏洞。多数受影响设计是 Proposed。接手审阅可以依据记录的反证修订或撤回发现。

### AUD-001 — P7-W06 丢失唤醒（实现阻塞）

证据：[阻塞／唤醒协议](../stages/p7/implementation/p7-w06-block-wakeup/01-block-wakeup-architecture.md)第 90–122 行；[阻塞 API](../stages/p7/implementation/p7-w06-block-wakeup/02-code-contracts-block-path.md)的 B-1；[生命周期引擎](../stages/p7/implementation/p7-w02-scheduler-admission-lifecycle/03-code-contracts-lifecycle.md)的 `try_transition`。

阻塞方先读取无 pending 事件，再提交 `Blocked`；唤醒方记录事件，但若仍看到 `Running` 就返回。唤醒方不消费 `block_intent`。反例：intent → 空事件检查 → 发布事件 → 唤醒方看到 Running → 阻塞方提交 Blocked。顺序一致模型枚举这五步的 10 种合法交错，发现这一条事件滞留路径，不需要弱内存行为。

影响：vCPU 可能在存在合格 pending 事件时持续阻塞，影响 P7-V13/V14/V25 及 P8。原修复方向：定义事件检查、阻塞提交和唤醒的共享同步协议与线性化点；单纯补充 acquire/release 描述不能消除该交错。**当前处置（2026-09-28）：**责任方已选择共享原子阶段／事件字方向，并记录于 [P7-W06 详细设计](../stages/p7/implementation/p7-w06-block-wakeup/README.md)。实现及 W06-DV04/P7-V14/P7-V25 证据仍待完成，相关 W08 idle-interlock 缺口为独立问题。上述交付完成前 AUD-001 仍保持开放。同步 W02/W06/W08/W11。通常不需要 ADR；实现前修正设计。

### AUD-002 — P3-W08 超时重用与接收方所有权冲突（实现阻塞）

证据：[槽位状态机](../stages/p3/implementation/p3-w08-tlb-shootdown-transport/02-architecture-and-state.md)第 52–64 行；[超时恢复](../stages/p3/implementation/p3-w08-tlb-shootdown-transport/04-code-contracts-transport-initiator.md)第 127–135 行；[接收方](../stages/p3/implementation/p3-w08-tlb-shootdown-transport/03-code-contracts-transport-request.md)第 142–152 行。

状态机允许 Empty/Completed → Pending，但恢复契约承诺覆盖旧 Pending。迟到接收方可能已经读取旧控制字，此时新 descriptor／请求发布；接收方可能用旧序列消费新 descriptor，或执行旧操作后完成 CAS 失败。CAS 失败被归类为 fatal。发起锁不排除超时后仍在执行的接收方。发起伪代码也需与文档中的 Completed → Pending 边统一。

影响：传输恢复及验收无法按这些条款一致实现；后续真实失效操作绑定会继承风险。P3 当前规定 TransportNoop，因此不是已观测到的运行时 TLB 泄漏。修复方向：定义领取、执行、取消、排空、重用及序列回绕语义，保证 descriptor 身份。同步 W08/W12 和下游失效消费者。验收覆盖接收方延迟、执行中超时、迟到完成、连续重用、回绕及正常第二次请求。通常不需要 ADR。

### AUD-003 — P3/P4 扩大了 P1 映射保证（交接阻塞）

证据：[P1 Host 地址空间](../stages/p1/contracts/host-address-space.md)第 34 行起排除未使用 RAM 和通用映射服务；[P3-W04 输入](../stages/p3/implementation/p3-w04-per-cpu-runtime/01-scope-and-foundations.md)第 41 行假设分配的区域／栈已经由 P1 映射；[P4 写入前提](../stages/p4/implementation/p4-w02-stage2-address-space/03-code-contracts-stage2-core.md)第 135 行假设 P1 映射全部 EL2-managed RAM。

影响：物理分配成功不能证明 Host 可写访问；栈初始化或页表写入可能 fault。修复方向：指定实际映射交付者，冻结覆盖、属性、生命周期、解除映射和回滚；同步 P2-W10、P3-W04、P4-W01/W02/W03。验收真实分配页安全可访问，包含负例和回滚。不要扩大 P1 历史完成声明。阶段职责重新分配需要 Architecture Change Request 审议；仅在改变已接受决策时需要 ADR。已有 W04-MAP/W05-MAP 门槛本身不能证明这些更广的消费者保证。

### AUD-004 — P4 单地址空间契约不能直接支持 P7 多 VM 切换（交接阻塞）

证据：[P4 create/activate](../stages/p4/implementation/p4-w02-stage2-address-space/03-code-contracts-stage2-core.md)第 209、400–405 行；[P7 切换所有权](../stages/p7/implementation/p7-w04-preemption-context-switch/02-architecture-and-state.md)第 56 行；[P7-W05 前置条件](../stages/p7/implementation/p7-w05-shared-mn-multivm/01-scope-and-foundations.md)第 40 行。

P4 限制只有一个存活地址空间，并在同一 pCPU 上为 Active 时提前返回。P7 假设存在多个 VM/vCPU 对象，且每次切换激活地址空间。P4 在限定范围内可以成立，缺陷是生产者与消费者之间的扩展无人交付。若只放宽创建限制，A→B→A 时仍不能从 A 对象自己的 Active 状态推断硬件当前上下文。

修复方向：指定多地址空间创建、每 pCPU 当前上下文、重新激活和失效契约，记录具体 P7-IN-05 阻塞。同步 P4-W02/W04/W09 和 P7-W01/W04/W05。验收两个 VM 使用相同 IPA、不同 HPA，反复轮转及失败隔离。多 VM 已属于架构目标，但阶段归属仍需明确审议，不能编码时临时发明。

### AUD-005 — P6-W04 SGI 统计比较了不同单位（验收阻塞）

证据：[SGI 发送／统计](../stages/p6/implementation/p6-w04-smp-interrupt-routing-sgi/03-code-contracts-sgi-send.md)第 34、85、126–129 行。

发送方统计编码写入数，接收方统计接收 pCPU 数。一次编码覆盖两个目标时，得到一次发送和两次接收，即便正确交付也会误报差异。修复方向：区分写入数、目标通知数和实际接收数，规定合并／观测语义。同步 W04/W11/W13。验收单目标、多目标、广播及声明的失败／合并场景。预计无需 ADR；影响 P6-V04/V05 判据。

### AUD-006 — P8 闭环指定了竞争的权威位置（契约歧义）

证据：[W03 启动契约](../stages/p8/implementation/p8-w03-linux-boot-contract/01-boot-contract-facts.md)第 8 行规定 `docs/machine-types/`；[W14 兼容性](../stages/p8/implementation/p8-w14-machine-abi-compatibility/01-compatibility-matrix-and-policy.md)第 9–16 行规定 `docs/abi/`；[W20 闭环](../stages/p8/implementation/p8-w20-documentation-closure-handoff/01-closure-artifact-contract.md)第 11–21 行又指定阶段 implementation 位置，并称其为唯一权威来源。

影响：发布／闭环时出现竞争规范，P9 消费者无法确定权威。修复方向：保留单一公共权威契约；闭环引用契约并记录实现偏差／证据。同步 W02/W03/W14/W20 的路径与发布规则。验收每个契约只有一个规范性来源，入站引用一致。预计无需 ADR；发布／闭环前解决。

### AUD-007 — 文档总入口混淆计划和详细设计（导航缺陷）

证据：[文档索引](../README.md)第 91 行将 `plans/` 称为已批准实现级设计；[阶段流程](stage-workflow.md)第 32–33 行将 L3 计划和 implementation 目录下的 L4 详细设计分开。

修复方向：更正目录说明及中文译本，再检查模板／技能中的同类冲突。验收入口路由保持计划、设计、代码准入区别。预计无需 ADR。本交接有意保留缺陷原文。

## 3. 已有阻塞及排除项

- P2 W04-LAYOUT、W04-MAP、W05-MAP、W05-GLOBAL、W07-ADAPTER、W09-DTB、P2-ACR-01 在[协调记录](../stages/p2/implementation/p2-contract-reconciliation-record.md)中已有责任方。该记录的 9 月 27 日基线描述属于历史状态：当前 [W03 运行时验证](../stages/p2/verification/p2-w03-runtime-verification.md)已提供有界运行时闭合；未声明 W04 或整个 P2 完成。
- [P6-W12](../stages/p6/implementation/p6-w12-fault-isolation-robustness/README.md)明确记录 ADR-061 协调前提。NC6 属于 P6-V29，不是已通过的 P1 测试。这是已有阻塞，不是新增发现。
- P8 具体 IPA／窗口／slot 数及冻结权限仍需沿决策路由解决。未来工件尚不存在，本身不构成缺陷。
- P3–P8 拟议设计不是实现／证据。P9–P21 未来兼容性属于审计范围，但尚未要求交付的阶段文档不算缺失。
- 历史 P1/P6 v0.1 任务书已有替代状态头；不沿后继文件直接与当前门槛比较会产生误报。
- P6-W06 的未来 deadline 恢复不自动等于阻塞 vCPU 的主动唤醒保证。在认定交接已证明前，核对 P7 deadline fold 及失败行为与生产者契约；这仍是待查项，不增加确认缺陷数。

## 4. 前一轮实际执行的检查

以下检查相对于 Git 跟踪文件均为只读，是基线时的历史审计观察，不是新 QEMU 执行或当前在线检查声明。

| 检查 | 记录结果／证明边界 |
|---|---|
| `.github/workflows/ci.yml` 内联文档 Python 检查 | 链接、四跳内可达性、必需头部通过；原有豁免仍适用；不是完整锚点或语义验证 |
| `python3 -B scripts/check-doc-translations.py --coverage` | 926 个适用源文档中 33 对译本有效；不代表全量翻译 |
| `PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s tests -p test_doc_translations.py` | 7 项通过 |
| 基线清点及验证 ID 扫描 | 963 份 Markdown、130,759 行；显式同阶段 `Pn-Vnn` 引用均在最新任务书 ID 集合中；不证明完整需求／验收映射 |
| P7 抽象协议枚举 | 10 种保持各方顺序的交错中有 1 条丢失唤醒路径；不是已实现调度器测试 |
| P1 归档保管 | 1,597,624,320 字节；SHA-256 `160cab674613225bdfa6921e1222c927bdc237d074fbf3e37b2e05e5aa4829bf`，匹配[保管记录](../stages/p1/verification/p1-local-evidence-archive.md) |
| P1 已归档正常证据 | 100/100 outcome 状态为 0，每份串口恰有一个 Stable，且无 PANIC/FATAL/BOOT REJECT；汇总 100 PASS |
| P1 NC1–NC5 归档汇总 | 均 passed，每项两次运行，存储的检查标志均为 true；不是独立重执行每个负例判据 |
| P2-W03 本地原始工件 manifest | [manifest](../stages/p2/verification/p2-w03-runtime-evidence/manifest.json)的 111 个文件全部匹配大小及 SHA-256，无缺失／不匹配 |
| 源码抽查 | 实际 W03 启动适配器保留 backing/map 并报告 allocator absent；P1 汇编有 16 个向量槽；不是全代码审计 |

P1 归档路径：`/home/angus/dev/Zelyr-Hypervisor-evidence/p1-worktree-targets-2026-09-26.tar`。成员保留 `.worktrees/p1completion/target/p1-completion-evidence/…`。P2 manifest 路径相对于检出目录，位于 `target/…`。二者都是本地保管，不是异机备份；其他 agent／机器可能无法访问。后续机器缺少本地工件是证据访问缺口，不证明历史运行失败。清理 target／工作树前保留证据。

未执行：QEMU、完整 Rust 构建／测试、硬件、在线分支保护审计、外部链接、穷尽锚点、完整需求 ID 语义或全部工作包交接。前一轮内联审计命令没有保存原始日志；上述会话观察不能代替保留的阶段运行时证据。

## 5. 剩余审计队列与完成标准

1. **重建基线与台账——已完成。** 固定输入与当前树差异已核对；34 条新增／改动记录均有当前 blob 和完整行范围。历史源记录保持不变。
2. **完成全文覆盖——声明的当前树范围已完成。** 固定输入 967 份均已审阅；新增的 14 份报告和 20 份有改动的当前版本均记入差异台账。当前树共 981 份 Markdown。后续修改需要新一轮差异审阅。
3. **完成 125 个工作包映射——声明的 P0–P8 范围已完成。** 工作包台账覆盖输入、输出、边和验收／证据；阶段 7 核对了 217 条有向跨阶段关系，并明确保留不对称或缺少状态的记录。
4. **深入重点边界——本轮已完成。** 见[阶段 4 边界复核](documentation-audit-phase4.zh-CN.md)及[证据台账](documentation-audit-phase4-boundaries.csv)。已复核 P2→P3/P4、P3→P6、P4–P7 生命周期、P8→P9–P21 约束、P1 NC6→P6-V29、AUD-001–007、W07/W08 合并语义和 P7 deadline-fold 失败。本项仅表示完成本轮审阅；发现的合同与证据阻塞仍未解决。
5. **选择性核验已实现声明——本轮已完成。** 见[阶段 5 核验报告](documentation-audit-phase5.zh-CN.md)及[证据台账](documentation-audit-phase5-evidence.csv)。已交叉核验当前 P0 CI 配置与 host tests、P1 异常源文件/测试和本地归档 custody，以及 P2-W03 adapter/tests/111 项 manifest。源码审阅、host tests、有限运行时证据和硬件证明仍须区分；本轮未重跑 QEMU 或硬件。本轮不代表独立复核了 P0–P2 每个工作包声明或在线分支保护。
6. **补齐机械检查——本轮已完成。** 见[阶段 6 报告](documentation-audit-phase6.zh-CN.md)和[机械检查台账](documentation-audit-phase6-mechanical.csv)。现有 QG-DOCS 和翻译检查通过；补充扫描发现两处过期锚点，现均已修复。九份当前任务书和对应的当前中文译本现均含显式版本元数据。五项实际触发的前向引用豁免均有 P8 未来产物文档依据。机械检查不代表语义正确；架构、批准来源、追溯和运行时证据阻塞仍开放。
7. **交付综合报告并闭合文档审计——在声明的当前树 P0–P8 范围内完成。** 见[综合报告](documentation-audit-final.zh-CN.md)、[阶段 7 闭环核对](documentation-audit-phase7.zh-CN.md)、[当前树差异台账](documentation-audit-current-tree-delta.csv)和[跨阶段双向台账](documentation-audit-crossstage-bidirectional.csv)。当前 981 份 Markdown 和 125 个工作包计划均已覆盖；217 条已识别的有向跨阶段工作包关系均有两端视图或明确的缺失侧记录。这只闭合审计覆盖，不代表修复完成：发现、不对称映射、责任方决策和运行时阻塞仍开放，不得报告为已修复。

建议修复顺序（建议，不是授权）：先解决责任方决策（AUD-006、AUTH-01 及 AUD-003/004 的交付者），再处理上游生命周期／映射／传输（AUD-002–004），随后修复消费者同步／统计（AUD-001/005），最后收集阶段责任方验证证据。导航、锚点和当前任务书元数据已修复。架构问题沿责任方决策流程处理；本交接不选择新架构。

## 6. 覆盖快照

| 范围 | Markdown 数 | 前一轮语义覆盖 |
|---|---:|---|
| P0 | 151 | 完成报告部分审阅 |
| P1 | 131 | 选定契约／完成证据／源码 |
| P2 | 99 | 当前 W03 状态／交接／保管 |
| P3 | 109 | 选定通知／传输／内存输入 |
| P4 | 64 | 选定 Stage-2／P7 输入 |
| P5 | 63 | 部分句柄／权限契约 |
| P6 | 89 | 选定 SGI／timer／NC6 移交 |
| P7 | 88 | 选定生命周期／切换／阻塞唤醒 |
| P8 | 108 | 选定治理／兼容性／闭环 |
| 其他 | 61 | 部分治理入口 |

全部行均经过机械清点／扫描；任何范围的数量都不代表完整语义覆盖。源语言计划数量：P0 22、P1 12、P2 10、P3 15、P4 9、P5 10、P6 13、P7 14、P8 20。P2 另有十份译本计划，不是额外十个工作包。

## 7. 变更边界与累计交付

初始交接新增本文、英文源文档和两份基线清单；后续阶段新增审计报告和台账，并均由开发索引链接。[阶段 7 闭环记录](documentation-audit-phase7.zh-CN.md)已闭合当前检出及 P0–P8 工作包交接清点的声明范围。没有修改代码、已接受 ADR、阶段契约、unsafe、公共 API/ABI、依赖、工具链或运行时行为。这些均为审计记录，不是修复系列。

初始交接于 2026-09-28 通过本地检查：CI 文档检查块、翻译检查器（加入当时译文后，927 份适用源文档中有 34 对有效译本）、七项翻译检查器测试及空白检查。基线清单也与 `git ls-tree` 核对：963 个唯一文档路径均匹配 blob、125 个唯一源语言工作包 ID，所列设计入口均存在。后续阶段扩展了这些记录；当前覆盖与检查见阶段 7。本次文档闭环没有新跑 QEMU、target、硬件或运行时验证。
