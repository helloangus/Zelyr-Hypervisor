# 全仓文档审计——阶段 5：选择性证据核验

English edition: [phase 5 report](documentation-audit-phase5.md).

**Translation status:** Current
**Translation source:** [English source](documentation-audit-phase5.md)
**Source blob:** `a3f6c05af7341ede89a47c0a29fc3d1853691132`
**Authority:** 本文供中文读者使用；英文原文具有权威性。

**状态：** 资料性选择性核验报告；不是新的阶段完成决定。
**版本：** v0.1
**快照：** 085dae0cf49fca69ce269afeecd6078582a6189b（2026-09-28）。
**负责人／变更背景：** Codex，应用户要求继续文档审计，2026-09-28（Asia/Shanghai）。
**取代：** 无；补充选择性证据交叉核验，不替代源验证记录。
**范围：** 接手队列第 5 项。沿 P0/P1/P2 的代表性完成声明，核对当前代码、测试、CI 配置和本地可访问的保留工件；不重跑完整运行时验证。

## 结果

| 阶段／声明 | 当前交叉核验 | 证据边界 |
|---|---|---|
| P0 host/target/document gates | 当前 .github/workflows/ci.yml 仍定义 host tests、host 与 target Clippy、AArch64 build、格式、警告和文档检查任务。P0 完成报告记录了历史执行和合并保护演练。本轮运行的 host suite 通过。 | 已核验 checkout 中的 workflow；未查询远端分支保护设置或历史 GitHub checks。这不能重新证明在线 P0-V08 强制执行，也未重跑历史 AArch64 build。 |
| P1 exception vectors 与 classifier | 当前启动汇编包含 16 个 vector slots；纯异常模型由 tests/p1_exceptions.rs 直接纳入测试。本轮 host suite 通过。 | 仅核验当前源文件和 host classifier 测试。测试源码明确说明不执行 EL2。真实异步 delivery 仍未证明，归 P6-V29。 |
| P1 normal QEMU 归档 | 本地 P1 archive 存在；SHA-256 与 custody 记录一致（160cab…4829bf），归档有 15,035 个条目。保留的 normal-run summary 报告 requested/counted/passed 为 100/100/100，100 个结果均为 PASS。 | 这是重新核验 custody 和存档摘要，不是重新运行 QEMU。归档只有一份本地副本，不是异地备份；其中没有 NC6 执行工件。 |
| P2-W03 集成 | 当前 boot/p2.rs 调用 W03 draft_in、seal 并保留 map/backing，输出 metadata=0 allocator=absent；当前 W01/W02 启动路径会调用 p2::run。完整 host suite 中 P2 host tests 通过。 | 源码与 host tests 支持有界实现形态，不证明运行时行为。Allocator、W04 readiness 和 whole-P2 完成仍明确缺失。 |
| P2-W03 QEMU 证据 | 重新核对 manifest.json 全部条目：记录在本地 target/ 的 111/111 个文件均存在，大小和 SHA-256 均匹配。 | 这是核验本地保留工件及其 custody，不是新跑 emulator。证据未异地归档；验证报告的有限 reference-QEMU 范围不变。 |

逐文件路径、行号、blob 和命令见[阶段 5 证据台账](documentation-audit-phase5-evidence.csv)。本次审计未修改代码、测试、CI 或运行时证据；host test 命令可能只刷新了被忽略的构建输出。

## 验证与限制

运行 cargo test --workspace --exclude hypervisor：通过。另行核对 P1 archive digest 和存档摘要，并重新计算 P2 manifest 111 个条目的大小与 SHA-256。检查了当前源文件和 workflow；未运行 QEMU、target build、硬件、远端 GitHub 设置或完整测试矩阵。

没有新增 unsafe、ABI/公共 API 或依赖变更。P1 NC6/P6-V29 仍开放；P2 W04 allocator gates 仍开放。此处对 P0 在线分支保护的依据仍只有历史验证记录，没有当前在线查询。本次审计仍是选择性且未完成；队列第 6 项仍开放。
