# P0-W02 — Rust Toolchain Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

冻结可恢复、可审查且在开发与 CI 一致的第一代 Rust 工具链基线。

## Scope

channel、版本固定方式、必要组件/targets、底层开发组件、更新流程和 unstable feature 使用规则。

## Out of scope

不选择未来 crate 依赖，不设计运行时 Rust API，也不把 nightly 偶然使用变成未审查前提。

## Work sequence

1. 确定项目需要的工具链能力及其声明边界，并定义开发者与 CI 的同一来源。

2. 记录从干净环境恢复工具链的输入、失败处理边界和必要组件类别。

3. 制定更新、审查和重新验证规则，明确何时属于常规维护、何时需要 ADR 讨论。

4. 在干净环境语义下复核声明足以恢复并支撑后续 host 与 target 工作。

## Acceptance and closure

P0-V02：依据仓库声明恢复相同工具链，并确认开发与 CI 引用同一基线。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W03、W07、W19、W20 提供已固定的工具链契约。

