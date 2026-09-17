# P0-W20 — CI Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

让 GitHub PR 与 `main` 持续执行 P0 required checks，只有通过在线验证的 PR
才能合并到主线，并清楚标注尚未自动化的工作。

## Scope

GitHub PR/`main` CI 触发与报告责任、required vs informational vs future
manual/hardware 分类、产物与失败可见性，以及 `main` 的 PR required-check
protection。

## Out of scope

不把 P1 EL2 smoke、Linux guest 或 Orange Pi hardware test 伪装成 P0 已完成检查。

## Work sequence

1. 将 W07 的质量门禁和 W19 的入口映射为 CI 可报告检查。

2. 为 format、lint、host tests、AArch64 build 和基础文档检查设定 required 状态。

3. 为 QEMU runner、未来 EL2 smoke、Linux 与真机验证定义合适的非 P0 分类和启用条件。

4. 配置 GitHub，使 post-policy 开发不能直接进入 `main`：PR 以 `main` 为
   base，所有 configured required checks 通过后才允许合并；不得把管理员绕过
   视为常规流程。

5. 复核 PR 与 `main` 的 CI 输出能帮助定位失败，保存必要 artifact/metadata
   关联，并通过真实 GitHub PR 记录保护和检查状态证据。

## Acceptance and closure

P0-V08：PR 和主线 CI 的 required 检查均执行并可见；GitHub 在合并前要求 PR
的 required checks 成功；未来检查不会被误报为已验证。配置审阅或本地 workflow
语法检查不单独证明 GitHub 已执行或保护已生效。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P0 completion review、P1+ 主分支保护和所有 PR 作者提供持续的在线验证
证据与失败定位信息。
