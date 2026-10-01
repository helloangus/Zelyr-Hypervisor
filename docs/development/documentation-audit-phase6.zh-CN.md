# 全仓文档审计——阶段 6：机械检查

English edition: [phase 6 report](documentation-audit-phase6.md).

**Translation status:** Current
**Translation source:** [English source](documentation-audit-phase6.md)
**Source blob:** `deb9c8e79cb0dd39a6c101f9b98170dce2c6326c`
**Authority:** 本文供中文读者使用；英文原文具有权威性。

**状态：** 资料性机械审计报告；不是语义审计完成决定。
**版本：** v0.1
**快照：** 085dae0cf49fca69ce269afeecd6078582a6189b（2026-09-28）。
**负责人／变更背景：** Codex，应用户要求继续文档审计，2026-09-28（Asia/Shanghai）。
**取代：** 无；记录当前机械检查和发现的文档一致性缺口。
**范围：** 接手队列第 6 项：现有文档门、锚点／版本头／validation ID 扫描，以及对当前实际触发的 CI 前向引用豁免进行审查。

## 结果

现有 CI 的 QG-DOCS 链接、可达性和文档头检查通过。翻译检查通过；加入本报告后，英文源文档有 42/935 份配对；7 项翻译检查单测通过。git diff --check 通过。已从 .github/workflows/ci.yml 提取并执行原 CI Python 检查块。

| 检查 | 结果 | 边界 |
|---|---|---|
| 相对链接／QG 豁免规则 | 9,031 个现存相对目标均解析；5 个缺失目标被 CI 前向引用豁免接受；未发现其他缺失目标。 | CI 只检查目标文件是否存在，不检查片段锚点。路径豁免规则本身不证明 owner 文档记录；本轮逐项检查了这 5 个实际豁免。 |
| Markdown 片段锚点 | 固定快照扫描的 39 个相对片段链接中发现 1 个过期锚点。当前树复扫 68 个链接时又发现 P8-W08 的一处过期自引用；两处均已修复为当前标题 slug，复扫通过。 | 原 CI 链接检查不验证片段；此修复属于文档维护，不是运行时结论。 |
| 版本元数据 | CI 的 QG-DOCS 规范文档集合共 35 份（含 docs/README.md），都有 Version 头，且包含 vN.N 格式。 | P0–P8 当前 9 份英文 task book 现也有显式 `Version:` 字段；当前中文译本已添加对应字段并刷新源 blob。其他阶段文档的规范性尚未通过本次机械扫描全面分类。 |
| 阶段 validation ID | 9 份最新版英文 task book 共声明 182 个唯一 validation ID，没有重复定义行。跨文档扫描找到 4,929 个显式同阶段 validation-ID 引用，没有超出现行 task book 声明集的 ID。 | 这只核对 ID token 与集合；不验证需求语义、区间缩写或需求→验收→证据映射语义。 |
| 前向引用豁免 | 当前 5 项实际豁免均是 P8 未来产物：W02→W01 implementation record，以及 W20→W16/W17/W18/W19 verification record。 | W02 明确把 W01 路径标为 future，并说明记录创建前以 W01 design 为依据。W01 说明工作开始时才创建 record。W20 的 gate matrix 将 W16–W19 verification record 列为 closure 输入；各 owner design 均指定了未来证据位置。这证明当前触发的豁免有文档依据，但不代表对宽泛 CI 路径条件未来可能匹配的所有链接都已核验。 |

支撑来源路径、行号和 blob ID 见[阶段 6 台账](documentation-audit-phase6-mechanical.csv)。DOC-MECH-01 与 DOC-META-01 已作为文档维护问题修复；这些编辑没有改变运行时或架构发现。

## 限制与后续

只读审计之后已修复过期链接并补齐当前任务书版本元数据；也没有把自定义扫描器加入 CI 或仓库。锚点和 ID 扫描只是只读审计探针。未修改代码、accepted ADR、API、依赖或 unsafe。

接手队列第 6 项及其两项机械文档问题现已闭合。审计快照发现和责任方契约阻塞仍分别跟踪。第 7 项的覆盖核对记录于阶段 7；这不代表架构、批准来源、不对称交接或阶段证据阻塞已解决。
