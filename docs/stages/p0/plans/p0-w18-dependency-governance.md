# P0-W18 — Dependency Governance

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

允许使用生态依赖，同时防止 Hypervisor TCB 的依赖无审查扩张。

## Scope

依赖评估标准、记录要求、更新/review 责任与 TCB 进入规则。

## Out of scope

不批准或拒绝某个未来 crate，也不设计依赖包装 API。

## Work sequence

1. 定义每个候选依赖必须评估的 no_std、license、maintenance、unsafe、传递依赖、架构、分配、稳定性和安全风险。

2. 区分一般开发依赖、host-only 依赖与可能进入 hypervisor TCB 的更高审查边界。

3. 规定引入、升级、弃用和异常处理应留下的记录。

4. 用未来候选依赖的评审路径检查规则完整，而不选择具体依赖。

## Acceptance and closure

P0-V09：依赖治理路径和 TCB 审查维度可定位。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P1+ 所有依赖决策提供统一审查基线。

