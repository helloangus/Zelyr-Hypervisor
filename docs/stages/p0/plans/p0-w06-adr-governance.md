# P0-W06 — ADR Governance

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

让架构 ADR 成为可执行的工程约束和变更流程。

## Scope

ADR 生命周期、适用变更、历史处理、supersession、review 与架构冲突升级路径。

## Out of scope

不修改既有已接受 ADR，不在本包解决新的架构选择。

## Work sequence

1. 定义完整 ADR 状态及其迁移含义，区分已接受、拒绝、延后和被替代的决策。

2. 界定何类变更必须提出 ADR，何类属于局部实现选择。

3. 规定冲突、替代、历史保留和评审记录流程。

4. 使用一个假想的与已接受 ADR 冲突的建议进行可追踪性演练。

## Acceptance and closure

P0-V10：审阅者能够从文档判断冲突 ADR 及正确的变更路径。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向所有后续阶段提供 architecture-change/ADR-required 的处理机制。

