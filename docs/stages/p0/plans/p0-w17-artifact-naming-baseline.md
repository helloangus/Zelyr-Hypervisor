# P0-W17 — Artifact Naming Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

避免不可区分、不可自动处理的 build、guest、firmware 与报告产物名称。

## Scope

命名维度、稳定性、机器可处理性和未来 artifact 类别覆盖范围。

## Out of scope

不冻结每种未来 artifact 的最终文件格式或生成流程。

## Work sequence

1. 盘点未来 hypervisor、validation guest、Linux guest、boot package、Control Domain、DTB、firmware、snapshot、migration、symbols 和报告类别。

2. 定义名称必须表达的 arch/platform/profile/version 等可区分维度。

3. 规定命名演进和与 build metadata 的关联原则，避免依赖人工记忆。

4. 审阅命名规则既能支持 P0 target artifact，也不限制未来格式。

## Acceptance and closure

P0-V14：产物命名规则可机器处理，并能与 build identity 关联。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W19、W20 与后续构建/验证工作提供可归档产物约定。

