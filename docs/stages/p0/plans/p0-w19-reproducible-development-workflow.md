# P0-W19 — Reproducible Development Workflow

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

将从 clone 到 build、test、target build 与 QEMU entry 的完整路径写成仓库知识。

## Scope

先决条件、顺序、成功证据、常见边界、产物定位和与 CI 的对应关系。

## Out of scope

不要求用户依赖未记录的 shell history；不以文档替代实际 validation。

## Work sequence

1. 汇总 W01–W03、W07–W09、W16–W17 提供的契约，形成从干净环境出发的线性工作流。

2. 记录每个阶段的输入、预期证据、失败归属和下一步，不展开具体实现命令。

3. 说明 host 与 target 的差别、QEMU entry 的 P0 placeholder 边界及 artifact 识别方式。

4. 进行 fresh-clone walkthrough，检查所有路径均由仓库文档而非个人环境补全。

## Acceptance and closure

P0-V01–V05、P0-V13：可按文档抵达全部 P0 执行入口，并识别每项成功/未验证的范围。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W20、P1 Plan/Coding Agent 和新贡献者提供可重复 onboarding。

