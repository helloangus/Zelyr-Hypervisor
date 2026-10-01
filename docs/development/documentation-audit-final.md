# Repository documentation audit — consolidated report

Chinese readers can use the [Chinese report](documentation-audit-final.zh-CN.md).

**Status:** Informative consolidated report; the declared current-tree P0–P8 documentation audit is closed by phase 7. Design findings, evidence blockers, and owner decisions remain open for remediation.
**Version:** v0.1
**Snapshot:** `085dae0cf49fca69ce269afeecd6078582a6189b` (2026-09-28), with later audit reports and ledgers explicitly treated as supplemental records.
**Owner/change context:** Codex, user-requested continuation, 2026-09-28 (Asia/Shanghai).
**Supersedes:** None; consolidates, but does not replace, the phase reports or their evidence ledgers.

## 1. Decision and scope

This report synthesizes the audit and its closure reconciliation. The current checkout has 981 Markdown documents: 967 in the phase-2 pinned review, 14 reports added afterward, and 20 existing documents whose content changed and were re-reviewed. All 125 source work-package plans remain mapped. Phase 7 reconciles 217 distinct directed cross-stage package relationships from producer outputs, consumer inputs, and status-qualified assertions; it preserves absent mirror entries and missing status assertions. It also corrected an internal P3-W15 summary typo (P4-W15 to source-supported P4-W01). See the [phase-7 closure report](documentation-audit-phase7.md), [current-tree delta ledger](documentation-audit-current-tree-delta.csv), and [cross-stage reconciliation ledger](documentation-audit-crossstage-bidirectional.csv).

The declared documentary audit is **closed with findings**. Closure means the current-tree coverage and identified P0–P8 package-handoff census are recorded; it does not mean every one-sided relation is an accepted contract, findings are fixed, owners approved proposed designs, or later-stage mechanisms have runtime evidence. The 217 rows include 17 with both endpoint map entries, 60 producer-only, 106 consumer-only, and 34 represented only in the edge assertion ledger. A total of 117 relationships lack a status-qualified edge assertion. The specific discrepancies remain open for their owners.

The audit is read-only with respect to project behavior. It did not alter code, accepted ADRs, stage contracts, APIs/ABIs, dependencies, `unsafe` code, or runtime evidence. No PR was merged. Findings describe source contracts, proposed designs, evidence gaps, or documentation consistency; they are not automatically accepted owner decisions or observed runtime failures.

## 2. Coverage and evidence

| Pass | Completed evidence | Boundary |
|---|---|---|
| Baseline and classification | Pinned baseline and delta; classified 965 Markdown paths at the phase-1 input, preserving the 963 original documents and identifying two handoff editions. | Classification is not semantic approval or complete historical approval provenance. |
| Full text | 981/981 current Markdown paths covered; 967 pinned inputs plus 14 added reports, with 20 changed files re-reviewed. | The current-tree delta ledger records all 34 added/changed versions (14 added and 20 changed); future edits require a new delta. |
| Package mappings | All 125 P0–P8 plans mapped; 719 phase-3 edge assertions and 303 requirement/acceptance/evidence rows. Phase 7 reconciles 217 unique directed cross-stage package relationships. | Many mappings are asymmetric or lack status assertions; the ledgers preserve those gaps. Planned/assumed edges do not prove delivery. |
| Priority boundary review | Reviewed P2→P3/P4, P3→P6, P4–P7 lifecycle, P8→P9–P21 constraints, NC6→P6-V29, AUD-001–007, notification coalescing, and P7 deadline-fold failure. | Targeted priority review only; several interfaces remain unresolved or unverified. |
| Selective implementation evidence | Host suite, current CI/source/tests, P1 local archive custody and summary, and all 111 P2-W03 manifest entries were checked. | No fresh QEMU, target build, hardware, online branch-protection query, or full P0–P2 package revalidation. |
| Mechanical checks | Existing QG-DOCS, translation checker and its seven tests, and whitespace checks passed. The pinned supplemental scan found one stale fragment and missing task-book version metadata; current-tree follow-up found and repaired two stale fragments and added the missing version fields. | Mechanical consistency does not establish semantic correctness. |

The detailed records are authoritative for their respective scopes: [phase-1 inventory and delta](documentation-audit-phase1.md), [phase-2 per-document review ledger](documentation-audit-phase2.md), [phase-3 package/edge/acceptance ledgers](documentation-audit-phase3.md), [phase-4 boundary ledger](documentation-audit-phase4.md), [phase-5 evidence ledger](documentation-audit-phase5.md), [phase-6 mechanical ledger](documentation-audit-phase6.md), and [phase-7 closure reconciliation](documentation-audit-phase7.md). The continuation [handoff](documentation-audit-handoff.md) preserves the initial inventories, closure result, and open owner actions.

## 3. Severity-ranked findings

Severity here reflects the consequence if a stated contract is relied on; it does not imply that a proposed mechanism is already executing.

| Rank | Finding | Supporting evidence | Counterevidence / boundary | Required owner outcome |
|---|---|---|---|---|
| Critical — implementation/admission blocker | **AUD-001: P7 lost wakeup.** The owner selected the W06 shared atomic phase/event-word direction, now recorded in the proposed detailed design; W08 has a related idle-check/wait gap. | Full-text P7 W02/W06/W08 review and phase-4 boundary B10/B11; one bad legal abstract interleaving is recorded; W06-DV04 and P7-V14/V25 remain required. | No scheduler runtime race or implementation validation was executed; the W06 design direction is selected, but W08's gap remains. | Implement the W06 protocol under P3-W06 ordering rules, pass W06-DV04 and P7-V14/V25, and separately close the W08 idle-interlock gap before declaring the finding resolved. |
| Critical — cross-stage admission blocker | **AUD-003: physical allocation does not imply writable Host mapping.** P2 W04's metadata mapping gate does not establish mappings for ordinary allocated frames consumed by later stages. | P2/P3/P4 plan and design comparison; phase-4 B01–B03. | P2-W03 supplies bounded memory-map runtime evidence, but W04 allocator/layout/mapping work remains blocked and absent. | Assign mapping producer, attributes, lifetime, revocation, and rollback, then verify consumer-visible mappings. |
| High — implementation blocker | **AUD-002: P3-W08 timeout-slot reuse conflicts with receiver ownership.** State reuse can race with delayed completion/consumption. | Full-text transport state-machine review and package edge records. | Transport is specified as a no-op; no runtime stale-TLB exposure is established. | Reconcile slot generations, receiver ownership, timeout, and cancellation semantics; add owner evidence before claiming transport. |
| High — cross-stage contract blocker | **AUD-004: P4 single-space activation does not provide P7 multi-VM switching.** | P4/P7 lifecycle designs and phase-4 P4–P7 review. | No Stage-2 multi-VM mechanism is claimed implemented. | Define address-space selection, VMID ownership, activation, invalidation, and destruction handoff across owners. |
| High — acceptance blocker | **P6 timer/physical IRQ completion mismatch.** P6-W03 combined EOIR conflicts with P6-W06 keeping the timer PPI active until Guest completion; split completion is reserved/unprovided. | P6 W03/W06 designs, phase-3 cycle review and phase-4 B07–B09. | These remain proposed contracts; no target runtime failure was measured. | Producer and consumer owners must settle completion API/order (including orphan EOI) before implementation or acceptance. |
| High — liveness/acceptance blocker | **P7 deadline-fold failure may commit Blocked without a guaranteed wake.** The cited timer path delivers on a later Guest entry but does not wake an absent vCPU, so immediate retry/spin or indefinite lack of progress is not excluded. | Phase-4 B11 cross-check of P7-W06/W08 and P6-W06. | No runtime trace was run; this is a guarantee gap in proposed behavior. | Specify bounded recovery/backoff or fail-closed ordering and prove both delivery and progress. |
| High — authorization/lifetime blocker | **P5 authorize-then-use race.** A target or grant may be destroyed/revoked after checking and before use. | P5 W04/W05/W06 designs and phase-3 package review. | No implementation or exploit is asserted. | Establish stable object/grant lifetime or an atomic use/revocation protocol. |
| High — acceptance blocker | **AUD-005: P6 SGI accounting compares different units.** Sender writes/target-list fanout are compared with per-CPU receipts. | P6-W04 and W12/W13 acceptance comparison; phase-4 re-review. | No live multi-CPU behavior was rerun. | Separate send attempts, target deliveries, coalesced/rejected events, and receipts in contracts and tests. |
| Medium — contract/authority ambiguity | **AUD-006: P8 competing authoritative homes.** | P8 W03/W14/W20 authority language. | This is a governance conflict, not proof that machine values were implemented. | Name one normative source through the owning governance process. |
| Medium — evidence admission | **P1 NC6 / P6-V29 remains unproven.** Async unexpected-vector execution has not been demonstrated. | P1 completion limits and P6 ownership; phase-4 B14–B15 and phase-5 artifact inspection. | P1's bounded reference-QEMU result remains valid; this does not reopen its declared scope. | Carry V29 into P6 owner design/matrix, execute and retain its evidence; until then keep it blocked/unpassed. |
| Resolved — documentation maintenance | **AUD-007:** root navigation now distinguishes bounded plans from detailed designs; **DOC-MECH-01:** both stale fragments repaired; **DOC-META-01:** all nine current English task books now state `Version:` and current Chinese translations carry matching metadata. | `docs/README.md`, its Chinese edition, the P2 prerequisite-conflict record, nine task books, and three current Chinese editions. | Rechecked with current-tree link/anchor and translation checks; historical superseded books were left unchanged. | Closed in this follow-up; no architecture or runtime conclusion follows. |

Other detailed findings—including P6 W1C/timer/LR issues, P7 lifecycle and serialization mismatches, P4 permission-slice and Guest re-entry gaps, and P8 timer/fault/fixture contract issues—remain in the phase-2/3 ledgers and are not collapsed into a claim that all have been owner-confirmed. The records retain counterevidence and status distinctions.

## 4. Existing blockers, deferrals, and open questions

**Existing blockers, not newly discovered audit regressions:** P2 W04-LAYOUT/W04-MAP and related W05/W07/W09 admission gates; P2-ACR-01; P6-V29/NC6; P8-W02 machine-value gate and P8-V19; unresolved ADR-057 for P7 RoundRobin v0; and owner-design gaps recorded in the package ledgers. P2-W03's bounded runtime integration does not close W04 or whole P2.

**Reasonable deferrals:** P8 P9–P21 mechanisms remain later-stage work with their own contracts; DMA/IOMMU, board-specific behavior, migration/snapshot, x86, hardware and broad threat/performance guarantees should remain with their owning stages. Their absence from this audit is not itself a defect. Reference-QEMU P1 evidence is not hardware, SMP, Guest, IRQ-service, or asynchronous-vector proof.

**Unverified questions:** exhaustive current-tree per-document and bidirectional edge closure; owner disposition of findings; fresh QEMU/target/hardware behavior; online branch-protection state; off-host custody of local runtime archives; exact approved P8 machine values; and whether future CI forward-reference exemptions remain owner-documented as new paths appear.

## 5. Repair batches and completion route

These are proposed sequencing, not implementation authorization or architecture decisions:

1. **Authority and admission:** resolve AUD-006 and AUTH-01 provenance; identify current normative machine-contract ownership and refresh stale P8 baseline/evidence admissions.
2. **Producer contracts:** settle memory mapping/lifetime (AUD-003), transport timeout/receiver ownership (AUD-002), P4/P7 address-space lifecycle (AUD-004), P5 authorization lifetime, and P6 physical/Guest interrupt completion.
3. **Consumer progress and accounting:** resolve P7 wakeup/deadline-fold liveness (AUD-001 plus B11), notification fan-out and coalescing, and P6 SGI units (AUD-005).
4. **Evidence and documentation:** update owner matrices and requirement-to-evidence traces; the stale anchor and current task-book version headers are repaired; run each owning host/target/QEMU test at its declared scope; preserve artifacts and counterevidence.
5. **Audit closure — complete for this declared scope:** reconcile the current tree, all 125 package maps, and all identified cross-stage package relationships from both endpoint views; preserve every absent endpoint, missing status, and unresolved finding in the ledgers. Owner decisions and remediation remain follow-up work under their owning stages, not inferred from audit closure.

## 6. Validation, change boundary, and limits

Phase-6 checks recorded as passing: current QG-DOCS block, translation checker, seven checker unit tests, and `git diff --check`; 9,031 relative targets resolved with five documented P8 future-artifact exemptions. The supplemental fragment scan originally found one stale anchor; it is now repaired, and the current tree has been rescanned. Phase 5's host suite passed and its P1/P2 local evidence custody checks passed within their stated boundaries. This closure update reruns the QG-DOCS block, translation checker, seven unit tests, and whitespace check. No QEMU, fresh target build, hardware run, online GitHub settings query, or full package scenario suite was performed.

This report and its translation are documentation-only. No new `unsafe`, public API/ABI, dependency, code, accepted ADR, or runtime behavior change was made. The documentary audit is closed at the stated P0–P8 scope; AUD-001's W06 design direction is selected, but its implementation/evidence and W08 idle-interlock gap remain open, as do AUD-002–006, AUTH-01, asymmetrical handoff records, and stage evidence blockers. AUD-007, DOC-MECH-01, and DOC-META-01 were repaired in the follow-up. The internal P3-W15 summary typo was corrected in the audit ledger. No project-stage completion or runtime proof is implied.
