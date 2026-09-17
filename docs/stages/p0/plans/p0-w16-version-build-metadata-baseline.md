# P0-W16 — Version & Build Metadata Baseline

**Status:** Planned work package; implementation not claimed  
**Parent:** [P0 task book](../task-book-v0.1.md)  
**Prerequisites and consumers:** [P0 plan index](README.md)

## Goal

让第一份 target artifact 起就可关联其来源、能力和兼容信息。

## Scope

project version、revision、profile、target、timestamp policy、dirty indicator、capability summary，以及 ABI/machine version 的预留。

## Out of scope

不冻结最终 wire format，也不定义未来 ABI 或 machine model。

## Work sequence

1. 定义必须能够回答的 artifact 身份问题及每项信息的来源。

2. 建立时间戳与 dirty-tree 的可重复性/可追踪性政策。

3. 区分 project/build identity 与将来独立版本化的 ABI、machine model，并保留位置。

4. 与诊断和 artifact naming 规则交叉审阅，确认 metadata 可被一致引用。

## Acceptance and closure

P0-V14：目标产物能关联源码版本及声明的构建/兼容信息。

Record implementation decisions and changed artifacts in ../implementation/;
record command output and review evidence in ../verification/. Before marking the
package complete, check its task-book requirement, prerequisite compatibility,
document links, and downstream handoff.

## Handoff

向 W17、W19 和 P1+ 提供 artifact identity 契约。
