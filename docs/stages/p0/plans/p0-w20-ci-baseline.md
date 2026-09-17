# P0-W20 — CI Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

让主分支持续执行 P0 required checks，并清楚标注尚未自动化的工作。

## Scope

CI 触发/报告责任、required vs informational vs future manual/hardware 分类、产物与失败可见性。

## Out of scope

不把 P1 EL2 smoke、Linux guest 或 Orange Pi hardware test 伪装成 P0 已完成检查。

## Work sequence

1. 将 W07 的质量门禁和 W19 的入口映射为 CI 可报告检查。

2. 为 format、lint、host tests、AArch64 build 和基础文档检查设定 required 状态。

3. 为 QEMU runner、未来 EL2 smoke、Linux 与真机验证定义合适的非 P0 分类和启用条件。

4. 复核 CI 输出能帮助定位失败，并保存必要 artifact/metadata 关联。

## Acceptance and closure

P0-V08：主 CI 的 required 检查均执行并可见；未来检查不会被误报为已验证。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P0 completion review 及 P1+ 主分支保护提供持续证据。

