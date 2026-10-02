# P2 — Platform Discovery & Host Memory Foundation

Chinese readers can use the [Chinese edition](README.zh-CN.md).

The normative P2 scope is [task-book-v0.1.md](task-book-v0.1.md). Start a
specific package from the [work-package plan index](plans/README.md), which
maps P2-W01 through P2-W12 to prerequisites and downstream consumers.

This folder separates P2 planning, later implementation traceability, and
verification evidence. P2 plans establish bounded outcomes and handoffs; they
do not authorize implementation detail or assert that platform or memory
functionality exists.

Current implementation status is tracked in the [implementation index](implementation/README.md).
W01/W02 have bounded reference evidence; W03 has host and bounded runtime
evidence, including its own storage/lifetime and stack-fit verification
([runtime closure](verification/p2-w03-runtime-verification.md)).
W04 is unimplemented and will be restarted later; no W04 completion evidence is claimed.
The [contract reconciliation record](implementation/p2-contract-reconciliation-record.md)
separates corrected documentation from remaining design and runtime foundations.
