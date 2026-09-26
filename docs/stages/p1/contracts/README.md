# P1 contract set

**Status:** Proposed stage-local contract index; P1 completion is not claimed.\
**Scope:** W12 assembly of P1 boundaries and P2 handoff, not implementation evidence.\
**Version:** v0.1.\
**Owner/change context:** P1-W12, 2026-09-25.\
**Supersedes:** None.

These documents assemble the current W01–W11 boundaries. Each links to its
owning design, implementation record or verification record. A verification
link denotes an evidence location, not an automatic pass. The [evidence
map](stage-gate-evidence-map.md) lists open gates and prevents this set from
being read as a P1 completion report.

| Contract | Subject |
|---|---|
| [AArch64 boot](aarch64-boot-contract.md) | canonical entry and retained DTB |
| [EL2 initialization](el2-initialization-contract.md) | ordered boot lifecycle |
| [Host address space](host-address-space.md) | fixed Stage-1 classes and transition |
| [Exception diagnostics](exception-diagnostics-contract.md) | vectors and terminal reports |
| [Reference QEMU environment](reference-qemu-environment.md) | recipe, runner and verdict |
| [Known limitations](known-limitations.md) | assumptions and unresolved coverage |
| [P2 handoff](p2-handoff.md) | consumable inputs and non-goals |
| [Stage-gate evidence map](stage-gate-evidence-map.md) | V01–V21 and exit review |

The `contracts/` location is the W12 detailed-design placement; repository-wide
taxonomy ratification remains an open coordination item, not a new taxonomy
decision. See the [W12 implementation record](../implementation/p1-w12-p1-documentation-handoff-record.md).
