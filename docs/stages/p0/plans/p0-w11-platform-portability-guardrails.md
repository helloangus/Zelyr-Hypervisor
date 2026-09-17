# P0-W11 — Platform Portability Guardrails

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

在平台代码出现前固化 Core、Arch、SoC、Board/BSP/Quirk 的隔离规则。

## Scope

平台依赖禁止项、capability-driven 选择、review checklist 和文档约束。

## Out of scope

不设计 Platform trait、DTB parser、BSP API 或具体 QEMU/RK3566 驱动。

## Work sequence

1. 将 ADR 的分层要求转为可审阅的工程规则和反例。

2. 明确 Core 不得依赖 board/QEMU，Arch 不得依赖 SoC/Board，quirk 的允许边界。

3. 规定功能按 PlatformCapabilities 而非平台名选择的审查要求。

4. 检查文档、review checklist 与 agent 入口都可发现这些规则。

## Acceptance and closure

P0-V12：规则不允许任何平台名称分支污染 Core，也不允许 Board 依赖进入 Arch。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P1+ 的平台、内存、IRQ 和 BSP 工作提供分层红线。

