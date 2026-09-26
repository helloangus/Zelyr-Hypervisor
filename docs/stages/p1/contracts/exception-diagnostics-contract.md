# P1 exception and diagnostic contract

**Status:** Proposed assembly of W05/W07/W09 boundaries; NC1–NC5 paired evidence recorded, NC6 execution transferred to P6-V29.\
**Scope:** EL2 vector classification and bounded terminal reports; no IRQ/GIC service or recovery policy.\
**Version:** v0.1.\
**Owner/change context:** P1-W12 assembly of [W05](../implementation/p1-w05-el2-exception-entry-baseline-record.md), [W07](../implementation/p1-w07-fatal-crash-diagnostics-record.md) and [W09](../implementation/p1-w09-initialization-sequencing-record.md), 2026-09-25.\
**Supersedes:** None.

W05 installs 16 EL2 vector slots spanning the exception origins and four
categories: synchronous, IRQ, FIQ and SError. It captures origin/category,
ELR/SPSR, ESR, validity-qualified FAR, GPRs and SP in a bounded frame.
Classification distinguishes expected synchronous context from unexpected
categories; the P1 path does not deliver interrupts or implement a GIC.
See the [W05 classification contract](../implementation/p1-w05-el2-exception-entry-baseline/04-code-contracts-classification-routing.md).

W07 owns the non-recursive terminal report. Its internal marker classes are
`ZELYR P1 PANIC` (kind P), `ZELYR P1 FATAL` (exception E or phase failure F),
and `ZELYR P1 REPORT END`. The [field matrix](../implementation/p1-w07-fatal-crash-diagnostics/02-code-contracts-report-model.md)
requires build identity, current EL/CPU context and W09 phase for all kinds.
Exception reports additionally carry origin/category, disposition, syndrome,
PC/return state, valid fault address and register frame. Panic has message
and handler-entry SP/LR, not a fabricated interrupted-PC frame; phase failure
has a static reason, not invented ESR/FAR/GPR values. Unavailable fields are
labeled N/A, output is bounded, and a second entry takes a minimal stop rather
than recursively formatting. Failed output never permits normal continuation.

[W07 verification](../verification/p1-w07-fatal-crash-diagnostics-verification.md)
includes an earlier QEMU panic-path probe. [W11's merged scenario
record](../verification/p1-w11-negative-fault-validation-verification.md)
adds two concordant runs each for NC1–NC5. NC1 is W01's pre-transfer EL2
rejection: it has the exact single rejection line, no W07 report fields or
`REPORT END`, and no Runtime/Stable continuation. NC2–NC5 instead use W07's
terminal report; NC3 observes a synchronous undefined-instruction report,
NC4 a panic report, and NC5 a post-MMU translation-fault report with
syndrome, FAR and phase. For NC2–NC5 the verdict checks applicable report
identity/context fields and exactly one end marker. NC6 has
no accepted genuine unexpected IRQ/FIQ/SError injection. The historical W11
record correctly marks the original P1-V18 blocked; [ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md)
revises P1-V18 to NC3–NC5 and transfers NC6 execution to P6-V29. Pre-vector exceptions remain outside
the owned diagnostic window, as recorded in [limitations](known-limitations.md).
