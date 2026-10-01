# Repository documentation audit — phase 2 full-text review

Chinese readers can use the [Chinese report](documentation-audit-phase2.zh-CN.md).

**Status:** Informative audit record; handoff queue item 2 is complete for the pinned snapshot. The repository-wide audit remains incomplete: package producer/consumer mappings, selected implementation checks, and the remaining handoff items are still open.
**Scope:** Full-text semantic review of the 967 Markdown documents in the pinned phase-2 input, with authority, findings, and counterevidence recorded per document.
**Version:** v0.1
**Owner/change context:** Codex, user-requested continuation, 2026-09-28 (Asia/Shanghai).
**Supersedes:** None; supplements the [phase-1 baseline](documentation-audit-phase1.md) and [continuation handoff](documentation-audit-handoff.md).

## 1. Scope and identity

This report completes item 2 of the [audit continuation queue](documentation-audit-handoff.md#5-remaining-audit-queue-and-completion-criteria). “Complete” applies to reading and documenting every input document; it does not mean the project documentation is defect-free, every finding has been accepted by its owner, or the full audit and repair roadmap are complete.

| Identity | Value |
|---|---|
| Pinned review snapshot | `085dae0cf49fca69ce269afeecd6078582a6189b` |
| Working branch | `docs/audit-phase2-fulltext` |
| Markdown documents reviewed | 967 |
| Source lines reviewed | 131,225 |
| Included source work-package plans | 125 |
| Integration | Local continuation only; no PR merge |

The snapshot inventory groups are root/governance 65, P0 151, P1 131, P2 99, P3 109, P4 64, P5 63, P6 89, P7 88, and P8 108 documents. The review ledger has exactly one normalized row per input path. For every row, the input blob, current `HEAD` blob, recorded line count, and covered range (`1-N`) match.

## 2. Method and evidence boundary

Each document was read in full without relying on a truncated batch output. Reviewers recorded the path and Git blob, complete line range, headings/sections, review date, governing authority sources, findings or references, counterevidence, and review note. English source documents were used by default; translations were checked in relation to their source and the language-edition rules. Work-package designs were treated as proposed unless their own current authority and evidence established otherwise.

The detailed record is available as [JSON Lines](documentation-audit-phase2-reviews.jsonl) and as a [CSV view](documentation-audit-phase2-reviews.csv). JSON Lines preserves each finding object and its evidence/counterevidence; the CSV is convenient for sorting and filtering. Repeated cross-document references are retained as references and are not counted as independent defects. Findings about proposed designs are implementation or acceptance blockers in those designs, not claims of an observed runtime vulnerability.

The review included targeted comparison with governing task books, plans, ADRs, current-baseline amendments, implementation records, verification records, and selected code/CI where the documentation made implemented claims. The official [Arm AArch64 Generic Timer Programmer’s Guide](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Generic%20Timer.pdf?revision=c710e7a7-9f52-4901-8c9d-91b19f44f9c7) was checked for the P8 TVAL contract finding.

The audit did not execute Rust tests, QEMU, hardware, or the full runtime suite. Full-text coverage is not package-edge completeness: item 3 still must map all 125 packages’ inputs, outputs, acceptance evidence, ownership, lifetime, failure/rollback, and verification consumer. The remaining queue items also include prioritized handoff checks and broader mechanical validation. No design, ADR, code, or finding was repaired or approved by this review.

## 3. Findings carried forward and rechecked

All seven findings already in the [handoff](documentation-audit-handoff.md#2-confirmed-findings-from-the-partial-audit) were revisited against the full source documents and their counterevidence. They remain open at the documentation-contract level:

| Finding | Phase-2 disposition |
|---|---|
| AUD-001 — P7 lost wakeup | Confirmed in the full W06/W02 synchronization contracts: event publication and `Running → Blocked` still lack a shared linearization protocol. P7-W06 repeats the same race in its blocker/waker interface. No scheduler runtime race was executed. |
| AUD-002 — P3-W08 timeout reuse | Confirmed as a state-machine/receiver-ownership mismatch. P3’s transport remains specified as a no-op, so this does not establish runtime stale-TLB exposure. |
| AUD-003 — P1 mapping guarantee | Confirmed as an unassigned producer/consumer requirement: allocation does not imply writable Host mapping for consumers. It does not reopen P1’s bounded completion claim. |
| AUD-004 — P4/P7 multi-space contract | Confirmed as a handoff gap between P4’s single-space activation assumptions and P7’s multi-VM switching requirements; no design is selected here. |
| AUD-005 — P6 SGI accounting units | Confirmed: sender write/encoding counts and per-CPU receipts are not the same unit when one target-list write fans out. |
| AUD-006 — competing normative homes | Confirmed in P8 W03/W14/W20: machine compatibility facts are assigned overlapping “sole authoritative” homes. One public contract home must be named before publication. |
| AUD-007 — plans/design navigation | Confirmed in the root documentation index and workflow: the `plans/` description conflates bounded plans with implementation-level designs. |

The previously recorded AUTH-01 approval-provenance question remains unresolved. P0-W01’s design body calls itself approved while its metadata says Proposed; implementation and verification records establish completed work and an owner-approved license, but the local history does not provide a separate approval record for the design’s status. No approval is inferred from the commit or from package completion.

## 4. Additional high-impact contract findings

These are representative findings from the full ledger, grouped by the boundary they affect. Proposed designs remain proposals; owners must review the evidence and counterevidence before deciding whether to amend them.

| Area | Finding and impact | Evidence route |
|---|---|---|
| P1/P3 | P1’s “bounded stop” description is stronger than the serial transmit polling path inspected in code; the W02 stack-limit handoff also omits a known limitation. These affect claims about bounded failure output and stack-overflow scope. | P1 W01/W02 designs and implementation/known-limit records; `P1-W01-REJECT-BOUNDED-STOP`, `P1-W02-STACK-LIMITATION-HANDOFF`. |
| P2/P3 | DTB default-cell assumptions differ; the W03 pseudocode has stale error/state behavior; W03’s page-translation contract omits a `present` field used by a consumer; and guest-memory validation does not define protection against another vCPU changing the buffer during use. | P2 W02/W03 current and proposed contracts; P3 W01/W02 producer-consumer designs. |
| P4/P5 | P4-W03’s mapping grant assumptions do not establish authority for platform MMIO or the required distinct permissions; W04/W05 authorization and object destruction/revocation can race with subsequent use; P5 error-code and query-attribute contracts do not align consistently across W02–W07. | P4 W02/W03; P5 W03–W08 detailed designs and their counterexamples in the ledger. |
| P6 | Multiple design blockers need reconciliation before implementation: physical interrupt lifecycle code uses read-modify-write against write-one-to-clear semantics; the normal timer-expiry path leaves the timer asserted; W06 split completion has no matching consumer API under W02/W03’s combined-EOI path; and W08 loses active List Register state on exit. | P6 W03/W05/W06/W08 designs; P6-W03-W1C-RMW-SEMANTICS, P6-W05-EXPIRY-DOES-NOT-DEASSERT-TIMER, P6-W06-SPLIT-COMPLETION-CONFLICT, P6-W08-ACTIVE-LR-STATE-LOST-ON-EXIT. |
| P7 | Scheduler contracts include mismatched state transitions and incomplete cross-pCPU serialization around admission, pause, block/wake, accounting, and reschedule delivery. These reinforce AUD-001 and leave multiple W02/W04/W06–W11 interfaces unsettled. | P7 W02–W13 designs; finding IDs `AUD-P7-W02-*` through `AUD-P7-W13-*` in the ledger. |
| P8 | Remaining detailed-design inconsistencies include PSCI status/affinity/error mapping, SGI priority source, TVAL read semantics, timer re-arm while masked, console backend boundedness, memory-probe reachability, panic classification/context, and repeated-boot symptom wording. | P8 W06–W18 designs; `AUD-P8-*` finding records. For TVAL, Arm’s guide says the register is a signed down-counter and continues negative after expiry; the W08 `counter − CVAL` read rule returns the opposite sign. |

The ledger also records lower-severity stale index/status text, malformed references, validation gaps, assumptions that are explicitly blocked pending upstream facts, and candidate issues where counterevidence limits the claim. A finding’s presence in the ledger is not approval to change the owning design.

## 5. Remediation route

The findings imply this review order for future, separately authorized work:

1. Resolve normative ownership and admission authority first: AUD-006/007, AUTH-01 provenance, the approved P8 machine-contract route, and each stage’s required upstream evidence.
2. Reconcile producer contracts before consumer implementation: P1 mapping coverage; P2/P3 address, buffer, lifecycle, and transport contracts; P4 multi-address-space and grant ownership; P5 handle/revocation/dispatch contracts; P6 physical/virtual interrupt completion; P7 lifecycle and synchronization.
3. Align scenario reachability and acceptance evidence after the contracts settle: guest-address probes, W13 fault classification, P8 regression oracles, and P6/P7 telemetry/accounting units.
4. Continue with handoff queue item 3’s complete package matrix, then the prioritized boundary and implementation evidence checks. Keep changes in their owning stages and do not treat this audit report as approval to edit an accepted ADR or implement a proposed design.

## 6. Completion limits and next step

Handoff item 2 is complete: 967/967 input documents received a complete-text review record. The repository audit is not complete. The next step is item 3: complete all 125 package producer/consumer and requirement-to-evidence mappings, then verify cross-stage edges and dependency cycles. Items 4–6 remain separate work, including targeted source/evidence checks and mechanical documentation checks. No PR was merged, and no runtime or hardware result was produced for this phase.
