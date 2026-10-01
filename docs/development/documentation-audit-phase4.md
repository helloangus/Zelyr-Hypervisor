# Repository documentation audit — phase 4 boundary review

Chinese readers can use the [Chinese edition](documentation-audit-phase4.zh-CN.md).

**Status:** Informative, partial audit report; not an approved design, architecture decision, or stage-completion decision.
**Version:** v0.1
**Snapshot:** `085dae0cf49fca69ce269afeecd6078582a6189b` (2026-09-28).
**Owner/change context:** Codex, user-requested continuation of the documentation audit, 2026-09-28 (Asia/Shanghai).
**Supersedes:** None; records the phase 4 review without changing earlier phase ledgers or proof boundaries.
**Scope:** Handoff queue item 4: priority cross-stage boundaries; P1 NC6→P6-V29; AUD-001–007 re-review; P7-W07 coalescing/W08 consumption; P7 deadline-fold failure.
**Method:** Read-only source inspection against current package plans, detailed designs, and available evidence records. Source snapshot hashes are recorded in `documentation-audit-phase4-boundaries.csv`.

## Findings

| Boundary | Current assessment | Evidence consequence |
|---|---|---|
| P2→P3/P4 mapping and ownership | **Responsibilities are mostly clear; delivery remains blocked.** P2-W02 and bounded P2-W03 evidence exist. P2-W10 explicitly separates platform facts, memory map, allocation and downstream mechanisms; P2 inputs do not authorize AP startup, SMP locks or Stage-2/Guest execution. P2-W04 remains blocked by W04-LAYOUT/W04-MAP, P2-W10 has no implementation/verification record, and P3-W01/P4-W01 reconciliation is not executed. | Do not treat planned handoff contracts or physical allocation as delivered writable mappings or downstream implementation evidence. AUD-003 remains open. See rows B01–B03. |
| P3 notification/transport→P6 SGI carrier | **Conceptually compatible, but no delivered carrier contract.** P3-W07 is a one-slot mailbox with WFE/SEV, explicitly not an interrupt; SGI carriage is Reserved. P3-W08 completion is software Pending→Completed after consumption. P6-W04 owns physical SGI send/receipt/EOI only. No kind→SGI-ID mapping or verified P3 poll/consumer↔P6 receive adapter exists. Physical SGI receipt cannot stand in for P3-W08 software completion. | Require an owned mapping/adapter and distinct accounting/acknowledgement semantics before claiming P6 carries P3 transport. See B04–B06. |
| P4–P7 object/VMID/timer/IRQ lifecycle | **Ownership is divided, with unresolved cross-owner contracts.** P4 address-space owns page tables, VMID and mapping ledger; Guest frames belong to W03, and destruction requires quiescence/invalidate/free/release ordering. P5 handles are not a universal object registry; vIRQ/memory/shared-region handles remain Reserved. P6 divides physical IRQ completion, pCPU host deadlines, per-vCPU Guest timers, vIRQ, LR presentation and maintenance. A concrete incompatibility remains: P6-W03 combined EOIR completion versus P6-W06 holding the timer PPI active until Guest completion. | Do not infer a universal object/VMID lifecycle from P4/P5 APIs. Resolve the physical interrupt deactivation contract before treating Guest timer delivery as closed. See B07–B09. |
| P7-W07 coalescing→W08 consumption | **Same-layer state coalescing is compatible; transport fan-out is not specified.** W07 publishes durable pause/control state before asking for reconsideration. W08 consumes a per-pCPU level intent, reruns the scheduler decision and rechecks it. But P3's one-slot mailbox may overwrite different consumer kinds; W08 does not say whether each doorbell scans every durable consumer state, nor assign its notification kind. W08's architecture pseudocode also appears to send on each call while R1 specifies a send on false→true transition. | Freeze shared-doorbell fan-out or isolated channels, kind/SGI assignment, retrigger rules, and false→true send semantics. The P3/P6 carrier and bounded remote-control latency remain unevidenced. See B10. |
| P7 deadline-fold failure | **Failure guarantee is insufficient and cites a non-wakeup guarantee.** W06 commits Blocked even if folding fails, then claims P6-W06 eventual delivery through an online pCPU. P6-W06 explicitly does not wake an absent vCPU; its eventual delivery is at the next Guest entry, which does not trigger that entry. W08 refuses idle and returns to the decision loop. Repeated failure with no runnable work can therefore retry immediately without a guaranteed wake, backoff, or terminal recovery state. | This is a liveness/non-busy-loop contract blocker, not an observed runtime failure. Define a producer-owned recovery/wakeup and bounded non-spinning policy (or fail closed before Blocked), then verify both timer delivery and progress. See B11. |
| P8 machine compatibility→P9–P21 | **Policy constraints are explicit; values and several later-stage contracts remain unapproved/reserved.** W14 routes machine facts through W02 governance, blocks unapproved/drifting facts, distinguishes machine version from management/schema ABI, and states P9+ constraints. P9+ must consume evidenced facts but design its own device protocols; P17 migration/snapshot policy is Reserved; P14 DMA/IOMMU, P15 board behavior, P20 x86 and P21 timing/threat models require their own contracts. Concrete v1 values remain unapproved and P8-V19 is blocked. | These are admission constraints, not evidence that later-stage plans or artifacts are missing defects. Do not derive machine values from QEMU or reuse machine version for unrelated ABI. See B12–B13. |
| P1 NC6→P6-V29 | **Ownership transfer is correct; proof is absent and detailed-design coverage incomplete.** ADR-061, P1 completion/known limitations, P6 task book and W12 plan agree NC6 is not passed in P1 and belongs to P6-V29. W12 detailed design/matrix remains FI-A–D and DV01–08 without V29; W13 plan asks to carry V29 or mark blocked, but its coverage and DOC-04 schema stop at V28. No execution/verification record was found. | Keep NC6 unpassed; update the P6 owner design and evidence matrix before execution. No P1 completion claim is reopened. See B14–B15. |

## AUD-001–007 re-review

The current source review and phase 3 mapping show no amendment or counterevidence that resolves these findings. Their scopes remain bounded as follows:

| ID | Status | Re-review result |
|---|---|---|
| AUD-001 | Open | P7-W06 still has a legal publish-before-Blocked/waker-sees-Running trace; memory-order prose does not supply a shared linearization point. Proposed-contract blocker; no scheduler runtime reproduction. |
| AUD-002 | Open | P3-W08 timeout reuse still conflicts with receiver ownership and late-consumer behavior. TransportNoop means no runtime stale-TLB exposure is established. |
| AUD-003 | Open | Allocation still does not establish writable Host mappings for ordinary allocated pages. P2 W04-MAP covers allocator metadata; P1's bounded completion claim remains intact. |
| AUD-004 | Open | P4's single-space activation assumptions still do not define P7's multi-VM current-context/reactivation contract. |
| AUD-005 | Open | P6 SGI send writes/encodings and per-pCPU receipts remain unlike units under fan-out. |
| AUD-006 | Open | P8 W03/W14/W20 still name competing authoritative homes for machine compatibility facts. |
| AUD-007 | Open | `docs/README.md` still calls `plans/` an approved implementation-level design while stage workflow distinguishes L3 plans from L4 detailed designs. |

Exact current evidence, section/line, and blob IDs are in the boundary ledger. These findings describe document-level contradictions or missing handoffs, not demonstrated runtime defects.

## Repair order and limits

Resolve authority and value gates first; then mapping and lifecycle ownership; then physical IRQ/timer completion and notification fan-out; then scheduler liveness and coalescing/accounting; finally carry NC6 into the P6 validation contract and evidence records. This report proposes no architecture choice and authorizes no implementation.

No code, accepted ADR, stage design, or runtime evidence was changed. No Rust tests, QEMU, hardware, or online checks were run. Mechanical documentation/translation checks are recorded with the phase delivery. The repository-wide audit remains incomplete; handoff queue items 5–6 remain open.
