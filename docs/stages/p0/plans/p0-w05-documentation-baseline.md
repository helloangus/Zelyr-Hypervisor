# P0-W05 — Documentation Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

建立可演进的正式文档体系与 Stage 文档分层。

## Scope

ADR、architecture、ABI、machine type、platform、testing、安全、development 和 stage 文档的职责、状态、版本与位置。

## Out of scope

不把后续详细设计写入 task book，也不冻结未来实现细节。

## Work sequence

1. 梳理文档类别及其 normative 或 informative 属性，建立清晰的权威来源。

2. 明确版本、状态、owner/change context、supersession 与代码变更同步规则。

3. 固定 stage task book、plan/detailed design、implementation、verification 的职责和目录分离。

4. 检查目录入口、交叉链接和后续工作包可引用性。

## Acceptance and closure

P0-V09：文档路径、职责和状态可被独立审阅；层次之间不混写。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向所有后续包提供文档位置、引用方式和变更记录约定。

