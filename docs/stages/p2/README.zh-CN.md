# P2 — 平台发现与 Host 内存基础

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `2b5191e1dbbd8e457049751351973a8f92fb3434`
**Authority:** 本文是供中文读者使用的译文；[英文原文](README.md)具有权威性。

P2 的规范性范围由[任务书 v0.1](task-book-v0.1.zh-CN.md)规定。开展特定工作包时，
从[工作包计划索引](plans/README.zh-CN.md)开始；其中将 P2-W01 至 P2-W10
映射到前置条件和下游消费者。

本目录分开存放 P2 规划、后续实施可追溯记录与验证证据。P2 计划确立有界成果和移交；
它们不授权实施细节，也不声称平台或内存功能已经存在。

当前实施状态见[实施索引](implementation/README.md)。W01／W02 已有有界参考环境证据；
W03 已有 host 与有界运行时证据，包含其自身的存储、生命周期和栈适配验证
（见[运行时闭环记录](verification/p2-w03-runtime-verification.md)）。
W04–W10 仍待完成；这不代表分配器或整个 P2 已完成。[契约修订记录](implementation/p2-contract-reconciliation-record.md)
区分已修正的文档与仍待完成的设计和运行时基础。
