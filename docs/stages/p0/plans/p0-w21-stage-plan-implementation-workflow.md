# P0-W21 — Stage / Plan / Implementation Workflow

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

固定大型项目从架构到验收的文档与责任流，避免 Coding Agent 无设计重做一级架构。

## Scope

ADR、task book、work-package plan/detailed design、implementation、verification 和 completion report 的职责、输入输出与升级路径。

## Out of scope

不以流程文档取代实际 detailed design，也不允许 task book 承载函数或模块实现。

## Work sequence

1. 明确每层回答的问题、产生的工件、必要输入及不可越权的决策范围。

2. 定义从 plan 到 code 的准入条件：何时需要 approved detailed design、何时需要 ADR/architecture change。

3. 规定 implementation traceability 与 verification evidence 的落点和相互引用。

4. 通过一个 P0 包和未来 P1 包的交接演练检查文档可发现性。

## Acceptance and closure

P0-V09、P0-V15：阶段层次无混写，后续 agent 能确定自己应读取和产出的文档。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向所有后续阶段提供统一工程工作流。

