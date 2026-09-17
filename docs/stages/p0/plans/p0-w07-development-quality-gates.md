# P0-W07 — Development Quality Gates

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

在复杂低层代码进入前明确自动化质量门禁和失败语义。

## Scope

format、lint、host test、AArch64 build、文档检查、warning policy、开发/release 最低检查和 CI failure policy。

## Out of scope

不实现完整 hypervisor integration test，也不以 P0 替代 P1 的 EL2 smoke。

## Work sequence

1. 列出必须、信息性和未来检查类别，并定义每类是否阻塞合入。

2. 确定 formatting、lint、warning、host test、target build 与文档一致性检查的最低标准。

3. 定义开发与发布场景的最低验证集合及检查失败的处理原则。

4. 使规则可被 CI W20 消费，并复核各检查有清晰证据归属。

## Acceptance and closure

P0-V06–V08：required gate 可执行、无歧义且在 CI 中可见。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W20 和所有后续实现提供合入前质量门禁。

