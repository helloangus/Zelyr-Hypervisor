# P0-W13 — Trace Event Namespace Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

为 telemetry-first 架构建立一致、可演进的 trace 事件命名和版本规则。

## Scope

事件领域分类、命名、版本/兼容和新增事件 review 规则。

## Out of scope

不定义事件字段、ring buffer、transport 或具体 instrumentation。

## Work sequence

1. 建立覆盖 boot、cpu、vm、vcpu、scheduler、memory、stage2、irq、device、virtio、ipc、capability、platform、management 的分类空间。

2. 规定新事件的命名、归属、兼容性和弃用处理，避免临时字符串成为接口。

3. 协调日志/metrics 与 trace 的边界，避免同一语义多套未声明名称。

4. 以未来 P1 事件类别演练 namespace 归属，不添加实际事件。

## Acceptance and closure

P0-V09：namespace 与版本规则可定位，新增事件可被一致分类。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 P1+ telemetry 设计提供稳定命名治理。

