# P0-W12 — Logging & Diagnostic Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

定义 P1 bring-up 及后续运行时共用的诊断语义。

## Scope

日志等级、人类日志、结构化 trace、metrics、crash dump、release 裁剪、panic 信息和版本身份的职责边界。

## Out of scope

不实现 telemetry transport、ring buffer 或最终 logging subsystem。

## Work sequence

1. 建立稳定日志等级和各诊断通道的目的，避免把它们互相替代。

2. 定义 release/debug 的可见性、裁剪和最低 panic/crash 信息原则。

3. 规定每次诊断关联的 build/version identity 需求，并与 W16 对齐。

4. 审阅该基线能约束 P1 但不提前决定具体 telemetry 实现。

## Acceptance and closure

P0-V09、P0-V14：诊断类别、最低 crash 信息和 metadata 关联规则完整。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W13、W16 和后续 bring-up/telemetry 工作提供统一语义。

