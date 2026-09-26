# P0 — Repository, Specification & Toolchain Baseline

Chinese readers can use the [Chinese edition](README.zh-CN.md).

Chinese editions: [task book](task-book-v0.1.zh-CN.md) and
[completion report](verification/p0-completion-report.zh-CN.md).

The normative P0 scope is [task-book-v0.1.md](task-book-v0.1.md). Start a
specific package from the [work-package plan index](plans/README.md), which
maps P0-W01 through P0-W22 to prerequisites and downstream consumers.

Stage outputs for downstream stages: the [P0 handoff
map](p0-handoff-map.md) (deliverable register and consumption mapping) and
the [P0 completion report](verification/p0-completion-report.md) (the
stage's completion claim, per-validation-ID evidence, open issues, and the
assembled P1 handoff package).

This folder holds P0 planning, implementation traceability, and verification
evidence in separate locations. P0 work-package plans describe bounded tasks,
large work steps, acceptance, and handoff; they do not substitute for an
approved detailed design when a code change needs one. P0 must not prematurely
decide crate APIs, module trees, or later-stage runtime designs.
