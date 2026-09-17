# P0-W04 — Build Profile / Feature Governance

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

建立编译能力、Cargo feature、build profile 与运行时策略之间不可混淆的规则。

## Scope

开关分类、profile 用途、可预留 profile 集合、评审准则与禁止将资源策略编译化的规则。

## Out of scope

不实现所有 profile，不决定 future runtime configuration schema，也不设计 feature 的代码组织。

## Work sequence

1. 建立开关分类准则，分别识别 binary capability、构建 profile 与 runtime resource/policy。

2. 为长期 profile 记录目的、适用边界和保留状态，避免把 profile 变成架构分叉。

3. 定义新增开关的审查问题与禁止案例，尤其是 VM 数量、内存、vCPU、亲和性和设备选择。

4. 用代表性决策进行文档审阅，确认后续 agent 能判定某开关的归属。

## Acceptance and closure

P0-V09、P0-V15：治理规则可定位，且 P1 无需重新定义 feature 与 runtime policy 的边界。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W03、W07、W16 和后续阶段提供构建选择的公共语义。

