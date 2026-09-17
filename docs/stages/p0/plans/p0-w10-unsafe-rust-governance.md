# P0-W10 — Unsafe Rust Governance

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

在第一段低层 unsafe 之前建立理由、库存、审查与边界制度。

## Scope

unsafe justification、inventory 字段、允许边界类别、review 规则和安全 Rust 优先原则。

## Out of scope

不决定具体封装、寄存器接口、指针 API 或 ASM 实现。

## Work sequence

1. 定义每段非平凡 unsafe 必须记录的安全前提、责任人与复核信息。

2. 确定 inventory 的位置、更新时机和可审计字段。

3. 划定典型允许边界与禁止以便利性扩大 unsafe 的规则。

4. 通过首个未来低层变更的审查视角复核制度可用性。

## Acceptance and closure

P0-V11：policy 与 inventory 位置明确，P1 可在不重新讨论基本制度下开始审查 unsafe。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P1+ 的 arch、MMIO、页表、ASM 和 FFI 设计提供审查基线。

