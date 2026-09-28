# Documentation audit — phase 1 baseline and classification

Chinese readers can use the [Chinese report](documentation-audit-phase1.zh-CN.md).

**Status:** Informative audit record; handoff queue item 1 complete for the pinned input snapshot, with conservative authority classifications. Full semantic audit and approval-provenance review remain open.
**Scope:** Baseline reconciliation, document identity, authority routing, lifecycle/version/owner classification, and continuation ledger integrity.
**Version:** v0.1
**Owner/change context:** Codex, user-requested first audit phase, 2026-09-28 (Asia/Shanghai).
**Supersedes:** None. The original handoff and both historical ledgers remain unchanged.

## 1. Scope and identities

“Phase 1” means item 1 of the [handoff queue](documentation-audit-handoff.md#5-remaining-audit-queue-and-completion-criteria), not hypervisor P1, the entire audit, or the proposed repair batches.

| Identity | Value |
|---|---|
| Original audit baseline | `b317b88ad6f78f82143ec6af2cf2b41a143ae741` |
| Pinned continuation input | `f5b23932e7d2036811b2f9e89a9b71c174e4f5b4` |
| Local work branch | `docs/audit-phase1-baseline`, based on the continuation input |
| Integration | User requested no PR merge; PR #72 remains open and draft; no merge performed |

The current branch was initially created at `main`, then fast-forwarded locally to the handoff commit before any phase-1 edits. PR #72 was briefly marked ready during the initial integration attempt and restored to draft after the user's correction. These operations did not merge a PR or change `main`.

The snapshot has **965 Markdown documents**: 963 original rows plus two handoff editions. Of the original 963 paths, 961 blobs are unchanged and two development-index editions changed. The complete repository delta is six paths: those two modifications and four additions (two handoff editions and two CSV inventories). There are no deletions or production-code changes between these commits.

## 2. Delivered ledgers and interpretation

- [Historical document inventory](documentation-audit-documents.csv): all 963 rows retained byte-for-byte.
- [Historical package inventory](documentation-audit-packages.csv): all 125 source-package rows retained byte-for-byte; all plan and design-entry blobs still match the original baseline.
- [Phase-1 classified document ledger](documentation-audit-phase1-documents.csv): exactly one row for each of the 965 input Markdown paths. `cohort=original-963` preserves baseline membership; `cohort=handoff-addition` separately identifies the two new editions.
- [Full baseline delta](documentation-audit-phase1-delta.csv): six paths, old/new blobs and change kinds, including the non-Markdown inventories.

Ledger fields distinguish facts from interpretation:

| Fields | Meaning |
|---|---|
| `snapshot`, `path`, `git_blob`, `baseline_blob`, `change`, `cohort` | Reproducible file identity and baseline membership; empty baseline blob means newly added |
| `stage`, `layer`, `edition`, `source_path` | Routing classification; translations point to their actual authoritative source |
| `declared_status`, `version`, `declared_version`, `version_basis` | Source declaration, normalized version and its provenance; translations use source declarations. `not-declared` is explicit, not an invented version |
| `authority`, `lifecycle`, `authority_refs`, `amendment_refs` | Conservative authority and lifecycle classification, governing references and required amendments; no approval is inferred from a path or heading |
| `owner`, `owner_basis` | Declared owner/context, parent entry's owner, or explicitly labeled routing responsibility; no inferred individual approver |
| `translation_check`, `pinned_source_blob`, `current_source_blob` | Exact translation/source blob comparison; not a semantic fidelity verdict |
| `classification`, `full_text_review`, `reviewer`, `review_date`, `notes` | Phase-1 classification, separate pending full-text audit, responsible reviewer/date, and exceptions |

The classification procedure joins Git trees and historical rows by path, extracts complete leading metadata (including plain `Status:` in P1 plans), resolves translations, applies the [documentation taxonomy](documentation-baseline.md) and [L1–L7 responsibilities](stage-workflow.md), then records the exceptions in section 3. Generic classification is a routing decision, not 965 full-document semantic reviews. All 965 `full_text_review` fields remain `pending`. A proposed design is conditional, and an implemented package does not automatically approve each old design page.

There are 931 source documents and 34 translations; all 34 source-blob pins match. The 135 package-plan editions represent **125 source packages and ten translations**, not 135 packages. Four public-contract-home indexes are classified as placeholders for future contracts. There are 18 editions without an explicit leading status and 846 without an explicit version in the source header/title; their roles remain classifiable through governance, source/parent relationships and Git identity. Missing metadata alone is not declared a new defect: applicability/exemptions need phase-2 review.

## 3. Authority exceptions reviewed

| Case | Classification and evidence | Owner / follow-up |
|---|---|---|
| ADR-000 | The [ADR index](../adr/README.md#6-index) records Accepted. Its authoritative [Chinese source](../adr/adr-000-architecture-baseline-v0.1.md) retains “Draft for Implementation” and a register with settled, reserved, pending and rejected items. Neither that heading nor the English translation replaces the index or changes individual register states. | Architecture decision owner; no ADR edited |
| ADR-061 | [Accepted decision and change history](../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md) govern the NC6 transfer; a translation has no independent authority. | P6-W12 / P6-V29 owns executed NC6 evidence |
| P1/P6 task-book v0.1 | Explicitly superseded by [P1 v0.2](../stages/p1/task-book-v0.2.md) and [P6 v0.2](../stages/p6/task-book-v0.2.md); historical files remain in the ledger. | Stage task-book owners; use successor gates |
| P1-W11 verification | The [record](../stages/p1/verification/p1-w11-negative-fault-validation-verification.md), lines 228–247, preserves historical blocking results and adds ADR-061/custody context. The header alone is not the current P1 acceptance decision. | P1-W11/L7 and P6-W12; no new execution claimed |
| P2-W01/W02 | The [current-baseline amendment](../stages/p2/implementation/p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md) takes precedence over original assumed inputs and pseudocode within its declared scope. | P2-W01/W02; amendments recorded separately |
| P2-W03 and W04 | The old [host verification](../stages/p2/verification/p2-w03-boot-memory-map-ownership-verification.md) withdraws its package-completion claim; [runtime verification](../stages/p2/verification/p2-w03-runtime-verification.md) is the later bounded closure. Reconciliation/audit snapshots cannot re-open W03 merely by stale wording or close W04. | P2-W03/W04; existing admission gates remain |
| AUTH-01: P0-W01 approval wording | [Design entry](../stages/p0/implementation/p0-w01-repository-baseline/README.md) lines 3–4 say Proposed; lines 23–24 call it approved. The [implementation index](../stages/p0/implementation/README.md) records completion, which is counterevidence to “nothing was implemented,” but not independent approval provenance for every design page. | P0-W01 / design-governance owner; classify `approval-wording-conflict`, review historical approval in phase 2; no newly confirmed AUD defect or architecture choice |
| Empty public contract homes | [ABI](../abi/README.md), [machine types](../machine-types/README.md), [architecture](../architecture/README.md), and [platform](../platform/README.md) establish homes/routing, not frozen future contracts. | Owning future stage; absence alone is not a missing deliverable |

AUD-001–007 remain the handoff's prior findings, without fresh confirmation or closure here. The architecture index's `plans/` wording is also a phase-2 follow-up under the existing AUD-007 routing topic; no repair is made during baseline classification.

## 4. Checks and limits

Checks performed on 2026-09-28:

- Compared the original inventory against `git ls-tree -r` at the audit baseline: 963 unique Markdown paths, all exact blob matches.
- Compared phase-1 ledger against the pinned continuation tree: 965 unique paths, exact blobs, complete cohort/delta accounting, and all authority/amendment target files present.
- Checked 125 unique source packages, unchanged plan/design-entry blobs, retained historical CSV bytes, and 34 translation/source blob pins.
- Ran the existing CI inline documentation block (relative links, four-hop reachability, required headers), translation coverage checker, seven translation-checker tests, and whitespace checks successfully.

The final local documentation tree includes this new report pair: 967 Markdown files and 35 valid translation pairs / 928 eligible sources. Those two new reports and the two new CSVs are **phase-1 outputs**, outside the pinned 965-document input. The only modified existing files are the two development indexes. This explicit output set avoids a self-referential snapshot/hash claim. On continuation, compare the local delivery commit with the pinned input and append these report editions to the next review ledger.

Not run: Rust build/tests, QEMU, hardware, retained-runtime-artifact revalidation, external-link checking, complete anchors/requirement semantics, or phase-1 online checks. No push or new PR was performed. The earlier PR #72 check results are not validation of these changes. No production code, `unsafe`, ABI/public API, dependencies, accepted ADRs or stage contracts changed. AUTH-01 approval provenance remains unresolved; other unresolved architecture issues retain their existing owners.

## 5. Next phase

Resume handoff item 2 with the pinned ledger and output delta. Read every document without truncation; record covered sections, source/approval authority, findings and counterevidence. Resolve AUTH-01 and metadata applicability through actual approval/history evidence. Then complete the 125 package producer/consumer mappings and later queue items. This phase establishes a usable, conservative inventory; it does not certify the full audit or authorize design repairs.
