# 文档审计第一阶段：基线与分类

**Translation status:** Current
**Translation source:** [English source](documentation-audit-phase1.md)
**Source blob:** `eacbdda4a0de6516bc611618567f7e46fd6c7d05`
**Authority:** 本文为中文译本；[英文原文](documentation-audit-phase1.md)具有权威性。

**状态：** 资料性审计记录；针对固定输入快照，接手队列第 1 项已完成，权威分类采取保守解释。全文语义审计和批准来源核验仍未完成。
**范围：** 基线核对、文档身份、权威路由、生命周期／版本／责任归属分类及接续台账完整性。
**版本：** v0.1
**所有者／变更背景：** Codex，用户要求的审计第一阶段，2026-09-28（Asia/Shanghai）。
**替代：** 无。原接手文档及两份历史台账保持原样。

## 1. 范围与身份

“第一阶段”指[接手队列](documentation-audit-handoff.zh-CN.md#5-剩余审计队列与完成标准)第 1 项，不是 Hypervisor P1、完整审计或建议的修复批次。

| 身份 | 值 |
|---|---|
| 原审计基线 | `b317b88ad6f78f82143ec6af2cf2b41a143ae741` |
| 固定接续输入 | `f5b23932e7d2036811b2f9e89a9b71c174e4f5b4` |
| 本地工作分支 | `docs/audit-phase1-baseline`，基于接续输入 |
| 集成 | 用户要求不合并 PR；PR #72 保持打开且为草稿；未执行合并 |

当前分支最初创建于 `main`，随后在第一阶段编辑前于本地快进到接手提交。初始集成尝试期间，PR #72 曾短暂改为待审；用户纠正后已恢复草稿。这些操作没有合并 PR，也没有改变 `main`。

快照包含 **965 份 Markdown 文档**：原有 963 行，加接手文档的两个语言版本。原 963 个路径中，961 个 blob 不变，两个开发索引版本改变。全仓差异共六个路径：上述两个修改及四个新增（两个接手版本、两份 CSV 清单）。两个提交间没有删除文件或生产代码变更。

## 2. 交付台账与字段解释

- [历史文档清单](documentation-audit-documents.csv)：963 行逐字节保留。
- [历史工作包清单](documentation-audit-packages.csv)：125 个源语言工作包逐字节保留；计划及设计入口 blob 均与原基线一致。
- [第一阶段分类台账](documentation-audit-phase1-documents.csv)：固定输入的 965 个 Markdown 路径各占一行。`cohort=original-963` 保留原基线成员关系；`cohort=handoff-addition` 单独标识新增的两个接手版本。
- [完整基线差异](documentation-audit-phase1-delta.csv)：六个路径的旧／新 blob 与变更类型，包括非 Markdown 清单。

字段区分可核验事实与分类解释：

| 字段 | 含义 |
|---|---|
| `snapshot`、`path`、`git_blob`、`baseline_blob`、`change`、`cohort` | 可复核的文件身份和基线成员关系；空的原基线 blob 表示新增 |
| `stage`、`layer`、`edition`、`source_path` | 路由分类；译本指向其实际权威源 |
| `declared_status`、`version`、`declared_version`、`version_basis` | 源文档声明、归一化版本及其来源；译本采用源文档声明。`not-declared` 明确表示未声明，不虚构版本 |
| `authority`、`lifecycle`、`authority_refs`、`amendment_refs` | 保守的权威／生命周期分类、治理依据及需阅读的修订；不从路径或标题推断批准 |
| `owner`、`owner_basis` | 声明的责任方／背景、父入口责任方，或明确标注的路由责任；不推定个人批准者 |
| `translation_check`、`pinned_source_blob`、`current_source_blob` | 译本和源文档 blob 的精确比较，不是翻译语义忠实性结论 |
| `classification`、`full_text_review`、`reviewer`、`review_date`、`notes` | 第一阶段分类、独立的待办全文审阅、审阅者／日期及例外 |

分类过程按路径关联 Git 树和历史行，提取完整的前导元数据（包括 P1 计划中不加粗的 `Status:`），解析译本，依据[文档分类](documentation-baseline.zh-CN.md)和 [L1–L7 职责](stage-workflow.zh-CN.md)分类，再记录第 3 节中的例外。通用分类是路由判断，不是 965 篇全文语义审阅。全部 965 行的 `full_text_review` 保持 `pending`。拟议设计仍为条件性权威；工作包已实现不自动批准每篇旧设计。

共有 931 份源文档、34 份译本，34 个源 blob 标记全部匹配。135 个工作包计划版本代表 **125 个源语言工作包和十个译本**，不是 135 个工作包。四个公共契约目录索引归为未来契约位置的占位。18 个版本没有显式前导状态，846 个版本的源头部／标题没有显式版本号；仍可依据治理、源／父文档关系和 Git 身份分类。元数据缺失本身不在此确认为新缺陷，适用条件及豁免留待第二阶段审阅。

## 3. 已核对的权威例外

| 情况 | 分类与依据 | 责任方／后续 |
|---|---|---|
| ADR-000 | [ADR 索引](../adr/README.md#6-index)记录为 Accepted。权威[中文原文](../adr/adr-000-architecture-baseline-v0.1.md)保留“Draft for Implementation”，内部登记表区分已确定、长期预留、待定、否决。该标题和英文译本均不替代索引，也不改变各登记项状态。 | 架构决策责任方；未修改 ADR |
| ADR-061 | [已接受决策及变更历史](../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.zh-CN.md)规定 NC6 移交；译本没有独立权威。 | P6-W12／P6-V29 负责 NC6 执行证据 |
| P1/P6 v0.1 任务书 | 明确被 [P1 v0.2](../stages/p1/task-book-v0.2.zh-CN.md)及 [P6 v0.2](../stages/p6/task-book-v0.2.md)替代；历史文件仍在台账中。 | 阶段任务书责任方；采用后继门禁 |
| P1-W11 验证 | [记录](../stages/p1/verification/p1-w11-negative-fault-validation-verification.md)第 228–247 行保留历史阻塞结果，并补充 ADR-061／证据保管背景。不能仅凭头部得出当前 P1 验收结论。 | P1-W11／L7 和 P6-W12；不声明新执行 |
| P2-W01/W02 | [当前基线修订](../stages/p2/implementation/p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md)在声明范围内优先于原假设输入和伪代码。 | P2-W01/W02；单独记录修订入口 |
| P2-W03 与 W04 | 旧[主机验证](../stages/p2/verification/p2-w03-boot-memory-map-ownership-verification.md)撤回了工作包完成声明；[运行时验证](../stages/p2/verification/p2-w03-runtime-verification.md)是后续有界闭合。协调／审计快照的旧措辞不能重新打开 W03，也不能关闭 W04。 | P2-W03/W04；既有准入门槛保留 |
| AUTH-01：P0-W01 批准措辞 | [设计入口](../stages/p0/implementation/p0-w01-repository-baseline/README.md)第 3–4 行为 Proposed，第 23–24 行称 approved。[实施索引](../stages/p0/implementation/README.md)记录完成，可反驳“什么都没实现”，却不是每篇设计批准来源的独立证明。 | P0-W01／设计治理责任方；归为 `approval-wording-conflict`，第二阶段核验历史批准；不是新确认的 AUD 缺陷或架构选择 |
| 空的公共契约位置 | [ABI](../abi/README.md)、[机型](../machine-types/README.md)、[架构](../architecture/README.md)、[平台](../platform/README.md)规定存放位置／路由，不代表未来契约已冻结。 | 对应未来阶段；尚不存在本身不构成交付缺失 |

AUD-001–007 保留为接手文档中的既有发现，本阶段不重新确认或关闭。架构索引中的 `plans/` 措辞也归入既有 AUD-007 路由主题，留待第二阶段核对；基线分类期间不修复。

## 4. 检查与边界

2026-09-28 执行的检查：

- 将历史清单与原审计基线的 `git ls-tree -r` 核对：963 个唯一 Markdown 路径，blob 全部精确匹配。
- 将第一阶段台账与固定接续树核对：965 个唯一路径、blob 精确匹配、成员／差异完整，全部权威及修订引用文件存在。
- 核对 125 个唯一源语言工作包、未变的计划／设计入口 blob、历史 CSV 原始字节以及 34 个翻译源 blob 标记。
- 现有 CI 内联文档检查（相对链接、四跳可达性、必需头部）、翻译覆盖检查、七项翻译检查器测试及空白检查通过。

最终本地文档树新增本报告的两个版本：共有 967 份 Markdown、928 份适用源文档中的 35 对有效译本。两个新报告和两份新 CSV 是**第一阶段产出**，不包含在固定的 965 份输入中。既有文件只修改两个开发索引。这一明确产出集合避免自引用快照／哈希声明。接续时比较本地交付提交与固定输入，将本报告两个版本追加到下一份审阅台账。

未执行：Rust 构建／测试、QEMU、硬件、保留运行时工件重新验证、外部链接、完整锚点／需求语义或第一阶段在线检查。未推送或新建 PR。此前 PR #72 的检查结果不作为本次变更的验证。未改变生产代码、`unsafe`、ABI／公共 API、依赖、已接受 ADR 或阶段契约。AUTH-01 批准来源仍未解决；其他架构未决项保留既有责任方。

## 5. 下一阶段

依据固定台账及产出差异，从接手队列第 2 项继续。无截断地全文阅读每篇文档，记录覆盖章节、来源／批准权威、发现和反证。依据真实批准／历史证据解决 AUTH-01 及元数据适用性问题，再完成 125 个工作包的生产者—消费者映射及后续队列。本阶段建立可用且保守的清单，不认证完整审计，也不授权设计修复。
