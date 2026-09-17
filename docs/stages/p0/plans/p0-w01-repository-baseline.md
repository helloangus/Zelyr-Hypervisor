# P0-W01 — Repository Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

建立可独立 clone、可被后续阶段长期使用的仓库根工程约定和入口。

## Scope

统一仓库目录职责、根级开发约定、文本/编辑器/忽略规则、许可证位置，以及 README 与文档入口。

## Out of scope

不决定最终 crate 数量、crate 职责、Rust module tree 或任何运行时对象。

## Work sequence

1. 盘点仓库根部现有工件与隐含前提，确定必须显式记录的目录、生成物和本机依赖。

2. 整理根级工程约定及入口说明，使源码、文档、工具、测试、CI 与未来实验空间的职责可定位。

3. 补齐 clone 后所需的可追踪配置和说明，确保没有未说明的必建目录、环境变量或本机文件。

4. 复核根入口与文档入口的链接、范围和后续工作包依赖。

## Acceptance and closure

P0-V01、P0-V09：从空白 clone 视角审阅；确认仓库根约定和文档入口完整、无机器私有前提。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W02、W05、W19、W20 提供稳定的仓库入口、目录约定和初始开发说明。

