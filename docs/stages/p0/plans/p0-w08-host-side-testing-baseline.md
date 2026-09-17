# P0-W08 — Host-Side Testing Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

让未来可脱离裸机验证的逻辑从一开始具备 host-side test 入口。

## Scope

host test 组织原则、可测试逻辑边界、CI 执行入口和适用的负向/边界测试类别。

## Out of scope

不实现 parser、capability、地址、页表或 ABI 机制本身。

## Work sequence

1. 界定适合 host 验证的未来逻辑类别与不应依赖 QEMU 的验证目标。

2. 建立最小 host test 基线和可被质量门禁调用的执行路径。

3. 规定后续测试应覆盖的正常、边界、无效输入、资源失败与重复生命周期类别。

4. 执行基线测试并记录其不证明裸机/硬件语义的边界。

## Acceptance and closure

P0-V03、P0-V04：host 构建和测试入口可执行并被 CI 使用。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W07、W19、W20 和后续模块设计提供 host-testable 基础。

