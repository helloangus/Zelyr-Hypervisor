# P0-W03 — AArch64 Build Target Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

建立 P1 可复用的 AArch64 bare-metal hypervisor 编译基线。

## Scope

目标定义、no_std 构建语义、panic/链接扩展位置、Rust 与必要 ASM 联合构建能力，以及 artifact 基础流程。

## Out of scope

不定义 EL2 入口、链接布局细节、异常向量或可运行 hypervisor 行为。

## Work sequence

1. 明确 host 程序、bare-metal hypervisor、未来 guest 与开发工具的目标边界，避免相互混淆。

2. 建立能够承载 no_std、链接与受控 ASM 的目标构建路径，并保留后续布局扩展位置。

3. 定义可识别的基线 artifact 产出边界及与 host 产物的关系。

4. 执行目标构建验证，并记录该验证只证明编译链成立而非 EL2 运行。

## Acceptance and closure

P0-V05：CI/本地可完成 AArch64 bare-metal 基础构建；不以 QEMU 运行作为成功条件。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W07、W09、W16、W19、W20 和 P1 提供目标构建入口及边界。

