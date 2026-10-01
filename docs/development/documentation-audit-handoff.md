# Repository documentation audit — continuation handoff

Chinese readers can use the [Chinese edition](documentation-audit-handoff.zh-CN.md).

**Status:** Informative historical handoff; its initial partial-audit state is superseded by the [phase-7 closure reconciliation](documentation-audit-phase7.md) and [consolidated report](documentation-audit-final.md). Owner decisions and remediation remain open; this is not an approved design or stage-completion decision.
**Scope:** Preserve the repository-wide documentation audit's findings, actual coverage, evidence checks, and remaining work.
**Version:** v0.1
**Owner/change context:** User-requested continuation branch, 2026-09-28; successor agent owns the remaining audit.
**Supersedes:** None; persists the preceding session's partial report without changing its proof boundary.

## Current disposition (2026-09-28 follow-up)

The declared current-tree audit scope is now covered: 981 Markdown paths, all 125 P0–P8 work-package maps, and 217 reconciled cross-stage package relationships. The unambiguous documentation findings AUD-007 (plans/design navigation), DOC-MECH-01 (stale anchor), and DOC-META-01 (current task-book version headers) were repaired and rechecked. The owner has selected the AUD-001 W06 shared atomic handshake direction, now recorded in the proposed detailed design; implementation, W06-DV04/P7-V14/P7-V25 evidence, and the related W08 idle-interlock gap remain open. AUD-002–006, AUTH-01 provenance, asymmetric handoff records, and stage-specific evidence/admission blockers also remain with their owning stages. No runtime closure is claimed. See phase 7 and the consolidated report for scope and validation.

## 1. Resume here

- Audit baseline: `b317b88ad6f78f82143ec6af2cf2b41a143ae741` (`main`, clean at audit start).
- Continuation branch: `docs/documentation-audit-handoff`.
- At this original baseline, the audit was **not complete**: it contained 963 tracked Markdown documents, 130,759 lines, and 125 source work-package plans. The later phase reports supersede this initial coverage status; mechanical scanning alone remains distinct from full-text semantic review.
- The original handoff branch recorded state without repairing AUD-001–007. The follow-up repaired the unambiguous documentation findings listed above; it did not approve proposed designs, change accepted ADRs, or implement later-stage mechanisms.
- The original authorization established the audit and a Chinese report with a dependency-ordered repair roadmap. Subsequent work completed the declared audit and routine documentation repairs on the current branch; no PR merge was requested.
- Read `AGENTS.md`, [documentation routing](../README.md), and the governing documents for the next area. Compare the successor checkout with the baseline before carrying findings forward.
- The original baseline ledgers below remain historical snapshots. Current coverage and owner actions are summarized above and in section 5. There is no active subagent or background test to recover.

Supporting ledgers:

- [Document inventory](documentation-audit-documents.csv): one row for every baseline Markdown document, including Git blob, stage, layer, edition, title, leading status text, and conservative review state.
- [Work-package inventory](documentation-audit-packages.csv): one row for every source plan, with design entry and separate upstream/downstream/acceptance audit states.

These are baseline snapshots, excluding this handoff and its new files. The prior session did not maintain a per-document full-read ledger; therefore no row is retroactively certified as fully reviewed. `targeted-review` means selected content was inspected, possibly across truncated outputs; full rereading and counterevidence review remain pending. Header text and path classification are discovery aids, not approval determinations. Translations, historical versions, and authority relationships still need semantic classification.

## 2. Confirmed findings from the partial audit

Locations refer to the audit baseline; line numbers can drift. “Confirmed” means a documented contradiction or counterexample was found, not that a production vulnerability was executed. Most affected designs are Proposed. The successor review can refine or withdraw a finding with recorded counterevidence.

### AUD-001 — P7-W06 lost wakeup (implementation blocker)

Evidence: [block/wakeup protocol](../stages/p7/implementation/p7-w06-block-wakeup/01-block-wakeup-architecture.md), lines 90–122; [block API](../stages/p7/implementation/p7-w06-block-wakeup/02-code-contracts-block-path.md), B-1; [lifecycle engine](../stages/p7/implementation/p7-w02-scheduler-admission-lifecycle/03-code-contracts-lifecycle.md), `try_transition`.

The blocker reads no pending event before committing `Blocked`; the waker records the event but returns if it still sees `Running`. `block_intent` is not consumed by the waker. Counterexample: intent → empty poll → event publication → waker observes Running → blocker commits Blocked. A sequentially consistent model enumerated 10 legal interleavings of these five steps and found this one stranded-event trace; weak memory is unnecessary.

Impact: a vCPU can remain blocked with an eligible event pending, affecting P7-V13/V14/V25 and P8 consumers. Original repair direction: specify the shared synchronization protocol and linearization point for event check/block commit/wake; extra acquire/release prose alone does not remove the interleaving. **Current disposition (2026-09-28):** the owner selected the shared atomic phase/event-word direction, recorded in [P7-W06 detailed design](../stages/p7/implementation/p7-w06-block-wakeup/README.md). Implementation and W06-DV04/P7-V14/P7-V25 evidence remain outstanding, and the related W08 idle-interlock gap is separate. AUD-001 remains open pending those deliverables. Synchronize W02/W06/W08/W11. Normally no ADR; design correction before implementation.

### AUD-002 — P3-W08 timeout reuse contradicts receiver ownership (implementation blocker)

Evidence: [slot state machine](../stages/p3/implementation/p3-w08-tlb-shootdown-transport/02-architecture-and-state.md), lines 52–64; [timeout recovery](../stages/p3/implementation/p3-w08-tlb-shootdown-transport/04-code-contracts-transport-initiator.md), lines 127–135; [receiver](../stages/p3/implementation/p3-w08-tlb-shootdown-transport/03-code-contracts-transport-request.md), lines 142–152.

The state machine allows Empty/Completed → Pending, but recovery promises replacement of stale Pending. A late receiver can have read the old control word when a new descriptor/request is published; it can consume the new descriptor with the old sequence, or fail completion CAS after performing the old operation. CAS failure is classified fatal. The initiation lock does not exclude receivers still executing after timeout. The initiator pseudocode also needs reconciliation with the documented Completed → Pending edge.

Impact: transport recovery/acceptance is not implementable as consistently specified; a later real invalidation binding inherits the hazard. P3 currently specifies TransportNoop, so this is not observed runtime TLB leakage. Repair direction: define claim/execution/cancellation/drain/reuse and sequence-wrap semantics, preserving descriptor identity. Synchronize W08/W12 and downstream invalidation consumers. Acceptance: delayed receiver, timeout during execution, late completion, repeated reuse, wrap, and normal second request. Normally no ADR.

### AUD-003 — P1 mapping guarantee is overstated by P3/P4 (handoff blocker)

Evidence: [P1 Host address space](../stages/p1/contracts/host-address-space.md), line 34 onward, excludes unused RAM/general mapping service; [P3-W04 input](../stages/p3/implementation/p3-w04-per-cpu-runtime/01-scope-and-foundations.md), line 41, assumes allocated areas/stacks are P1-mapped; [P4 write precondition](../stages/p4/implementation/p4-w02-stage2-address-space/03-code-contracts-stage2-core.md), line 135, assumes P1 maps all EL2-managed RAM.

Impact: successful physical allocation does not establish writable Host access; stack initialization or table writes can fault. Repair direction: assign the actual mapping producer and freeze coverage, attributes, lifetime, unmap and rollback rules; synchronize P2-W10, P3-W04, P4-W01/W02/W03. Acceptance: real allocated pages are safely addressable, with negative and rollback cases. Do not expand P1's historical completion claim. Stage-boundary reassignment needs Architecture Change Request review; an ADR is needed only if the resolution changes an accepted decision. Existing W04-MAP/W05-MAP gates do not by themselves establish these broader consumer guarantees.

### AUD-004 — P4 single-space contract cannot directly supply P7 multi-VM switching (handoff blocker)

Evidence: [P4 create/activate](../stages/p4/implementation/p4-w02-stage2-address-space/03-code-contracts-stage2-core.md), lines 209 and 400–405; [P7 switch ownership](../stages/p7/implementation/p7-w04-preemption-context-switch/02-architecture-and-state.md), line 56; [P7-W05 prerequisite](../stages/p7/implementation/p7-w05-shared-mn-multivm/01-scope-and-foundations.md), line 40.

P4 restricts creation to one live space and returns early for Active on the same pCPU. P7 assumes multiple VM/vCPU objects and address-space activation on each switch. P4 is coherent within its limited scope; the defect is the unassigned extension between producer and consumer. Merely relaxing creation would leave A→B→A unable to infer the installed hardware context from A's per-object Active state.

Repair direction: assign multi-space creation, per-pCPU current context, reactivation and invalidation contracts, and record the concrete P7-IN-05 blocker. Synchronize P4-W02/W04/W09 and P7-W01/W04/W05. Acceptance: two VMs with equal IPAs and distinct HPAs, repeated rotation and failure containment. Multi-VM is already an architecture goal; stage ownership still needs explicit review rather than coding-time invention.

### AUD-005 — P6-W04 SGI accounting compares different units (acceptance blocker)

Evidence: [SGI send/accounting](../stages/p6/implementation/p6-w04-smp-interrupt-routing-sgi/03-code-contracts-sgi-send.md), lines 34, 85 and 126–129.

The sender counts encoding writes; receipts count receiving pCPUs. One encoding covering two targets yields one send and two receipts, falsely reported as drift even with correct delivery. Repair direction: distinguish writes, target notifications and actual receipts, and specify coalescing/observation semantics. Synchronize W04/W11/W13. Acceptance: single-target, multi-target and broadcast accounting, with declared failure/coalescing cases. No ADR expected; affects P6-V04/V05 predicates.

### AUD-006 — P8 closure assigns competing authoritative homes (contract ambiguity)

Evidence: [W03 boot contract](../stages/p8/implementation/p8-w03-linux-boot-contract/01-boot-contract-facts.md), line 8, specifies `docs/machine-types/`; [W14 compatibility](../stages/p8/implementation/p8-w14-machine-abi-compatibility/01-compatibility-matrix-and-policy.md), lines 9–16, specifies `docs/abi/`; [W20 closure](../stages/p8/implementation/p8-w20-documentation-closure-handoff/01-closure-artifact-contract.md), lines 11–21, assigns stage implementation locations and calls them sole authoritative homes.

Impact: competing normative specifications at publication/closure and ambiguity for P9 consumers. Repair direction: retain one authoritative public contract; closure links it with implementation deviations/evidence. Synchronize W02/W03/W14/W20 paths and publication rules. Acceptance: one normative source per contract with consistent incoming references. No ADR expected; resolve before publication/closure.

### AUD-007 — Root documentation layout conflates plans with designs (navigation defect)

Evidence: [documentation index](../README.md), line 91, calls `plans/` approved implementation-level design; [stage workflow](stage-workflow.md), lines 32–33, separates L3 plans from L4 detailed designs under implementation directories.

Repair direction: correct the directory description and Chinese edition, then check templates/skills for the same conflict. Acceptance: entry-point routing preserves plan/design/code admission distinctions. No ADR expected. This handoff deliberately leaves the defect unchanged.

## 3. Existing blockers and exclusions

- P2 W04-LAYOUT, W04-MAP, W05-MAP, W05-GLOBAL, W07-ADAPTER, W09-DTB and P2-ACR-01 already have owners in the [reconciliation record](../stages/p2/implementation/p2-contract-reconciliation-record.md). Its September 27 baseline prose is historical: current [W03 runtime verification](../stages/p2/verification/p2-w03-runtime-verification.md) supplies bounded runtime closure; W04 and whole-P2 completion remain unclaimed.
- [P6-W12](../stages/p6/implementation/p6-w12-fault-isolation-robustness/README.md) explicitly records the ADR-061 reconciliation prerequisite. NC6 is P6-V29, not a passed P1 test. This is an existing blocker, not a new finding.
- P8 concrete IPA/windows/slot counts and freeze authority remain routed decisions. Missing future artifacts are not defects by themselves.
- P3–P8 proposed designs are not implementation/evidence. P9–P21 compatibility review is in audit scope, but their not-yet-required stage documents are not missing deliverables.
- Historical P1/P6 v0.1 task books carry supersession headers. Comparing them to current gates without following successors produces false positives.
- P6-W06 future-deadline restoration is not automatically a blocked-vCPU active wake guarantee. Reconcile P7's deadline fold and failure behavior with that producer before treating the handoff as proven; this remains a follow-up, not an additional confirmed finding.

## 4. What actually ran in the preceding audit

All checks below were read-only relative to tracked repository files. Results are historical audit observations at the baseline, not new QEMU runs or current online-check claims.

| Check | Recorded result / proof boundary |
|---|---|
| CI inline documentation Python block from `.github/workflows/ci.yml` | links, ≤4-hop reachability, required headers passed; its exemptions remain applicable; not full anchor or semantic validation |
| `python3 -B scripts/check-doc-translations.py --coverage` | 33 valid pairs / 926 eligible source documents; not full translation coverage |
| `PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s tests -p test_doc_translations.py` | 7 passed |
| Baseline inventory and validation-ID scan | 963 Markdown files, 130,759 lines; no explicit same-stage `Pn-Vnn` reference outside the latest task book's ID set; no claim of complete requirement/acceptance mapping |
| P7 abstract protocol enumeration | one lost-wakeup trace among ten order-preserving interleavings; not a test of implemented scheduler code |
| P1 archive custody | 1,597,624,320 bytes; SHA-256 `160cab674613225bdfa6921e1222c927bdc237d074fbf3e37b2e05e5aa4829bf`, matching the [custody record](../stages/p1/verification/p1-local-evidence-archive.md) |
| P1 archived normal evidence | 100/100 outcome status 0, exactly one Stable marker and no PANIC/FATAL/BOOT REJECT per serial; summary 100 PASS |
| P1 NC1–NC5 archived summaries | each passed, two runs, all stored check flags true; not independent re-execution of every negative-test predicate |
| P2-W03 local raw-artifact manifest | all 111 files matched size and SHA-256 from [manifest](../stages/p2/verification/p2-w03-runtime-evidence/manifest.json); no missing/mismatched files |
| Source spot checks | actual W03 boot adapter retains backing/map and reports allocator absent; P1 assembly has 16 vector slots; not a whole-code audit |

P1 archive: `/home/angus/dev/Zelyr-Hypervisor-evidence/p1-worktree-targets-2026-09-26.tar`. Members retain `.worktrees/p1completion/target/p1-completion-evidence/…`. P2 manifest paths are checkout-relative `target/…`. Both are local custody, not off-host backups; another agent/machine may not have them. Missing local artifacts on a successor machine are an evidence-access gap, not proof that the historical run failed. Preserve them before cleaning target/worktrees.

Not run: QEMU, full Rust build/tests, hardware, online branch-protection audit, external-link checks, exhaustive anchor checks, full requirement-ID semantics, or all package handoffs. No raw log of the prior inline audit commands was saved; the session observations above are not a substitute for retained stage runtime evidence.

## 5. Remaining audit queue and completion criteria

1. **Re-establish baseline and ledgers — complete.** The pinned input and current-tree delta are reconciled; all 34 added/changed rows carry current blobs and complete line ranges. Historical source rows remain intact.
2. **Complete full-text coverage — complete for the declared current tree.** All 967 pinned inputs were reviewed; the 14 added reports and 20 changed current versions are recorded in the delta ledger. The current tree contains 981 Markdown documents. Future edits require a new delta review.
3. **Complete 125 package mappings — complete for the declared P0–P8 scope.** The package ledgers cover inputs, outputs, edges, and acceptance/evidence; phase 7 reconciles 217 directed cross-stage relations and explicitly retains asymmetric or missing status records.
4. **Deepen priority boundaries — complete for this pass.** See [phase 4 boundary review](documentation-audit-phase4.md) and its [evidence ledger](documentation-audit-phase4-boundaries.csv). P2→P3/P4, P3→P6, P4–P7 lifecycle, P8→P9–P21 constraints, P1 NC6→P6-V29, AUD-001–007, W07/W08 coalescing, and P7 deadline-fold failure have been re-reviewed. This closes only the queued review pass; identified contract and evidence blockers remain unresolved.
5. **Verify implemented claims selectively — complete for this pass.** See [phase 5 review](documentation-audit-phase5.md) and its [evidence ledger](documentation-audit-phase5-evidence.csv). Current P0 CI configuration and host tests, P1 exception source/tests and local archive custody, and P2-W03 adapter/tests/111-file manifest were cross-checked. Source review, host tests, finite runtime evidence and hardware proof remain distinct; no QEMU or hardware rerun was performed. This pass does not independently revalidate every P0–P2 package claim or online branch protection.
6. **Complete mechanical checks — complete for this pass.** See [phase 6 report](documentation-audit-phase6.md) and [mechanical ledger](documentation-audit-phase6-mechanical.csv). Existing QG-DOCS and translation checks passed; supplemental scans found two stale fragments; both are repaired. Explicit Version metadata is now present in the nine current task books and matching current Chinese editions. The five exercised forward-reference exemptions are documented future P8 artifacts. Mechanical checks do not establish semantic correctness; semantic contract and owner-level findings remain open.
7. **Deliver the consolidated report and close the documentary audit — complete for the declared current-tree P0–P8 scope.** See the [consolidated report](documentation-audit-final.md), [phase-7 closure reconciliation](documentation-audit-phase7.md), [current-tree delta ledger](documentation-audit-current-tree-delta.csv), and [cross-stage bidirectional ledger](documentation-audit-crossstage-bidirectional.csv). All 981 current Markdown paths and 125 package plans are covered; 217 identified directed cross-stage package relations have both endpoint views or an explicit missing-side record. This closes audit coverage, not remediation: findings, asymmetric mappings, owner decisions, and runtime blockers remain open and must not be reported as fixed.

Suggested repair ordering (proposal, not authorization): resolve owner decisions first (AUD-006, AUTH-01 and producers for AUD-003/004), then upstream lifecycle/mapping/transport contracts (AUD-002–004), then consumer synchronization/accounting (AUD-001/005), then gather the stage-owned validation evidence. Navigation, anchors, and current task-book metadata are repaired. Architecture questions go to their owning decision process; this handoff selects no new architecture.

## 6. Coverage snapshot

| Area | Markdown documents | Prior semantic coverage |
|---|---:|---|
| P0 | 151 | partial completion-report review |
| P1 | 131 | selected contracts/completion evidence/source |
| P2 | 99 | current W03 state/handoff/custody |
| P3 | 109 | selected notification/transport/memory inputs |
| P4 | 64 | selected Stage-2/P7 inputs |
| P5 | 63 | selected handles/rights contracts |
| P6 | 89 | selected SGI/timer/NC6 transfer |
| P7 | 88 | selected lifecycle/switch/block-wakeup |
| P8 | 108 | selected governance/compatibility/closure |
| Other | 61 | selected governance entry points |

All rows received mechanical inventory/scanning; none of these area totals denotes full semantic coverage. Source plan counts: P0 22, P1 12, P2 10, P3 15, P4 9, P5 10, P6 13, P7 14, P8 20. P2 additionally has ten translated plans, not ten extra work packages.

## 7. Change boundary and cumulative delivery

The initial handoff added this note, its Chinese edition and the two baseline ledgers. Subsequent phases add audit reports and ledgers, all linked from the development index. The [phase-7 closure record](documentation-audit-phase7.md) closes the declared documentary coverage for the current checkout and P0–P8 package-handoff census. No code, accepted ADR, stage contract, unsafe, public API/ABI, dependency, toolchain or runtime behavior changed. This remains an audit record, not a repair series.

The original handoff checks on 2026-09-28 passed: the CI documentation block, translation checker (34 valid pairs / 927 eligible sources after adding that edition), seven translation-checker tests, and whitespace review. Baseline ledger consistency was checked against `git ls-tree`: 963 unique document paths with matching blobs, 125 unique source package IDs, and all listed design entries present. Later phase ledgers extend those records; see phase 7 for current coverage and checks. No new QEMU, target, hardware, or runtime verification was run for the documentation closure.
