# P0-W14 — Panic / Failure Classification Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

防止 guest 错误、资源不足和 hypervisor 不变量破坏被混作一种失败。

## Scope

不变量失败、guest-caused、resource exhaustion、unsupported feature/hardware、platform failure 的架构分类及传播原则。

## Out of scope

不定义 Rust error enum、panic handler 或 guest fault implementation。

## Work sequence

1. 定义各失败来源及其必须保留的语义差异。

2. 规定 guest-caused fault 原则上只影响对应 guest，而非直接升级为 global panic。

3. 将资源、能力和平台失败与内部不变量失败的处理边界写入诊断/审查要求。

4. 以代表性后续场景检查分类没有提前决定具体实现。

## Acceptance and closure

P0-V09：分类规则与 ADR 一致，后续设计可据此选择恢复/停止/升级路径。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W10、W12 和 P1+ 低层错误模型提供基础语义。

