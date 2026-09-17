# P0-W22 — Stage Dependency Map

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

将 P0 交付物与 P1 及后续阶段的消费关系形成可检查的 handoff map。

## Scope

P0 outputs、consumer stage、复用条件、缺失时的阻断含义和 completion handoff checklist。

## Out of scope

不规划 P1+ 的内部模块或实现顺序。

## Work sequence

1. 汇总 W01–W21 的可交付契约、验证证据及其所有者。

2. 映射 P1 EL2 boot、P2 platform/memory、P3 SMP 和更后阶段对这些契约的消费关系。

3. 定义 P0 completion report 必须提供的链接、已验证/未验证范围和遗留问题记录。

4. 用 P1 planner 的阅读顺序演练 handoff，确认不需重新设计 P0 基础设施。

## Acceptance and closure

P0-V15：P1 Plan Agent 能直接找到完整输入，并能区分已交付、未验证和后续工作。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P0 completion review 与 P1 planning 提供最终交接清单。

