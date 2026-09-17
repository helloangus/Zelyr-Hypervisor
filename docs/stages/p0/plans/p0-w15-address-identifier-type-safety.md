# P0-W15 — Address & Identifier Type-Safety Requirement

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

固化未来设计不得把不同地址空间和资源身份混为裸整数的约束。

## Scope

必须区分的地址/ID 语义、设计审查要求及转换必须显式化的原则。

## Out of scope

不定义 Rust newtype、字段、trait、constructor 或 conversion API。

## Work sequence

1. 列出至少应分离的 host physical/virtual、guest physical/IPA/virtual 地址语义。

2. 列出 VM、vCPU、physical CPU、VMID、IRQ 等身份语义并说明不可隐式互换。

3. 制定未来 Plan/Coding review 的检查项，要求转换与边界有明确语义。

4. 审阅本要求不反向指定后续模块或对象实现。

## Acceptance and closure

P0-V09：类型安全约束在设计入口可发现，且没有落入具体 API 设计。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P1+ 架构、平台、内存、VM 和 IRQ 详细设计提供语义红线。

