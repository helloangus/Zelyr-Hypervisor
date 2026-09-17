# P0-W09 — QEMU Automation Entry Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

建立唯一、可扩展的 QEMU virt 自动化运行入口。

## Scope

runner 责任、未来参数承载范围、串口捕获、超时、退出状态、证据保存与统一命令行治理。

## Out of scope

不要求 EL2 日志、guest boot 或任何 P0 运行成功。

## Work sequence

1. 定义 runner 的唯一责任与使用边界，避免多处维护相互漂移的 QEMU 命令。

2. 预留后续 boot smoke、SMP、内存、GIC、SMMU、guest image 和回归参数的承载方式。

3. 明确日志、超时、退出状态和失败证据的统一收集要求。

4. 验证入口可被调用或检查，并标记它在 P0 是 runner/placeholder 而非 EL2 测试。

## Acceptance and closure

P0-V13：存在唯一且已文档化的未来 QEMU runner 入口。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W19、W20 和 P1 提供统一的 QEMU 执行与证据入口。

