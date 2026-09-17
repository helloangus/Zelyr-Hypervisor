# P0-W19 — Reproducible Development Workflow

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

将从 clone 到 build、test、target build、QEMU entry，以及从新分支到
GitHub PR 的完整路径写成仓库知识。

## Scope

先决条件、顺序、成功证据、常见边界、产物定位、与 CI 的对应关系，以及
新分支开发、PR 提交和合并 `main` 的责任边界。

## Out of scope

不要求用户依赖未记录的 shell history；不以文档替代实际 validation；不在
W19 实现 GitHub workflow、required checks 或 branch protection。

## Work sequence

1. 汇总 W01–W03、W07–W09、W16–W17 提供的契约，形成从干净环境出发的线性工作流。

2. 记录每个阶段的输入、预期证据、失败归属和下一步，不展开具体实现命令。

3. 说明 host 与 target 的差别、QEMU entry 的 P0 placeholder 边界及 artifact 识别方式。

4. 将新分支 → GitHub PR → required online checks → `main` 合并的流程写为
   新贡献者必经路径；明确 W20 才拥有 GitHub 的技术强制与检查分类。

5. 进行 fresh-clone 与 branch-to-PR walkthrough，检查所有路径均由仓库
   文档而非个人环境补全，并且不会将尚未实施的在线检查误报为通过。

## Acceptance and closure

P0-V01–V05、P0-V13：可按文档抵达全部 P0 执行入口，并明确新开发必须经
GitHub PR 的 required online checks 后才能合并 `main`。该 walkthrough 证明
流程可发现性，不证明 W20 的 GitHub 强制已经配置。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W20、P1 Plan/Coding Agent 和新贡献者提供可重复 onboarding 与 branch/PR
流程；W20 可据此实现可验证的 GitHub 检查和 `main` 保护。
