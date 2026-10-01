# Repository documentation audit — phase 6 mechanical checks

Chinese readers can use the [Chinese edition](documentation-audit-phase6.zh-CN.md).

**Status:** Informative mechanical audit report; not a semantic audit completion decision.
**Version:** v0.1
**Snapshot:** 085dae0cf49fca69ce269afeecd6078582a6189b (2026-09-28).
**Owner/change context:** Codex, user-requested continuation of the documentation audit, 2026-09-28 (Asia/Shanghai).
**Supersedes:** None; records current mechanical checks and uncovered consistency gaps.
**Scope:** Handoff queue item 6: existing document gates; anchor, version-header, and validation-ID scans; inspection of currently exercised CI forward-reference exemptions.

## Results

The existing CI QG-DOCS link, reachability, and header block passed. The translation checker passed, with 42/935 English sources paired after this report was added; seven translation checker unit tests passed. git diff --check passed. The exact CI Python block was executed from .github/workflows/ci.yml.

| Check | Result | Boundary |
|---|---|---|
| Relative links / QG exemption rule | 9,031 existing relative targets resolved; five missing targets were accepted by the CI forward-reference exemption; no other missing relative target was found. | The CI check tests target existence, not fragments. Its path-based exemption does not itself prove owner documentation; the five active exemptions were reviewed individually below. |
| Markdown fragment targets | The pinned scan checked 39 relative fragment links and found one stale anchor. A current-tree rescan checked 68 links and found that anchor plus one stale self-reference in P8-W08; both are repaired to their current heading slugs and the rescan is clean. | The original CI link check does not validate fragments; this correction is documentation maintenance, not a runtime claim. |
| Version metadata | The 35-document normative set selected by the CI QG-DOCS job, including docs/README.md, has a Version header with a vN.N token. | The nine current English P0–P8 task books now also have explicit Version: metadata; current Chinese editions received matching translated fields and refreshed source blobs. Other stage documents were not exhaustively classified for normative status by this mechanical pass. |
| Stage validation IDs | Nine latest English task books declare 182 unique validation IDs with no duplicate validation rows. The cross-document scan found 4,929 explicit same-stage validation-ID references and zero IDs outside the current task book set. | This is token/set consistency only. It does not validate requirement meaning, range shorthand, or requirement→acceptance→evidence semantics. |
| Forward-reference exemptions | The five active exemptions are all P8 future artifacts: W02→W01 implementation record, and W20→W16/W17/W18/W19 verification records. | W02 explicitly labels the W01 path future and says to use the W01 design until its record exists. W01 says the record is created when work starts. W20's gate matrix makes W16–W19 verification records closure inputs; each owner design names the future evidence destination. This validates the currently exercised exemptions, not every future path the broad CI predicate could match. |

Supporting source paths, line numbers, and blob IDs are in the [phase 6 ledger](documentation-audit-phase6-mechanical.csv). DOC-MECH-01 and DOC-META-01 are resolved as documentation-maintenance findings; no runtime or architecture finding changed.

## Limits and next work

The stale link and current task-book metadata were repaired after the read-only audit pass. No custom scanner was added to CI or the repository; the anchor and ID scans were read-only audit probes. No code, accepted ADR, API, dependency, or unsafe change was made.

Handoff queue item 6 and its two mechanical documentation findings are now closed. Audit-snapshot findings and owner-level contract blockers remain tracked separately. Queue item 7 coverage reconciliation is recorded in phase 7; this does not close architecture, approval-provenance, asymmetric handoff, or stage evidence blockers.
