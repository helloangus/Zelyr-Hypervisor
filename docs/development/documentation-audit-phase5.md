# Repository documentation audit — phase 5 selective evidence review

Chinese readers can use the [Chinese edition](documentation-audit-phase5.zh-CN.md).

**Status:** Informative, selective verification report; not a new stage-completion decision.
**Version:** v0.1
**Snapshot:** 085dae0cf49fca69ce269afeecd6078582a6189b (2026-09-28).
**Owner/change context:** Codex, user-requested continuation of the documentation audit, 2026-09-28 (Asia/Shanghai).
**Supersedes:** None; adds a selective evidence cross-check without replacing source verification records.
**Scope:** Handoff queue item 5. Follow representative P0/P1/P2 completion claims into current code, tests, CI configuration, and locally accessible retained artifacts. No full runtime rerun.

## Results

| Stage / claim | Current cross-check | Evidence boundary |
|---|---|---|
| P0 host/target/document gates | Current .github/workflows/ci.yml still contains host tests, host and target Clippy, AArch64 build, format, warning, and documentation jobs. The P0 completion report records historical execution and merge-protection drills. The current host suite passed in this review. | Workflow presence is verified from the checkout; remote branch-protection settings and historic GitHub check results were not queried. This does not freshly verify P0-V08 online enforcement or rerun the historical AArch64 build. |
| P1 exception vectors and classifier | Current boot assembly contains 16 vector slots; the current pure exception model is directly included by tests/p1_exceptions.rs. The full host suite passed. | This verifies current source presence and host classifier tests only. The test source explicitly says it performs no EL2 execution. Genuine asynchronous delivery remains unproven and owned by P6-V29. |
| P1 normal QEMU archive | Local P1 archive exists; its SHA-256 matches the custody record (160cab…4829bf). The archive has 15,035 entries. Its retained normal-run summary reports requested/counted/passed = 100/100/100, and all 100 entries are PASS. | This rechecks custody and the stored summary, not QEMU execution. The archive is a single local copy, not an off-host backup. The archive contains no NC6 execution artifact. |
| P2-W03 integration | Current boot/p2.rs calls W03 draft_in, seals and retains the map/backing, and prints metadata=0 allocator=absent; current W01/W02 boot path reaches p2::run. P2 host tests passed in the full host suite. | Source and host tests corroborate the bounded implementation shape, not runtime behavior. Allocator, W04 readiness, and whole-P2 completion remain explicitly absent. |
| P2-W03 QEMU evidence | Rechecked every entry in manifest.json: 111/111 files exist under the recorded local target/ paths and match both byte counts and SHA-256. | This verifies retained local artifacts and their custody, not a fresh emulator run. The evidence is not off-host archived; the report's bounded reference-QEMU scope remains unchanged. |

The detailed file/line/blob mapping is in the [phase 5 evidence ledger](documentation-audit-phase5-evidence.csv). The current checkout contains no code, test, CI, or runtime-evidence edits from this audit; only ignored build output may have been refreshed by the host test command.

## Validation and limits

Ran cargo test --workspace --exclude hypervisor: passed. Independently checked the P1 archive digest and stored summary, and recomputed size/SHA-256 for all 111 P2 manifest entries. Read current source and workflow files; no QEMU, target build, hardware, online GitHub settings, or full test matrix was run.

No new unsafe, ABI/public API, or dependency changes were made. P1 NC6/P6-V29 remains open. P2 W04 allocator gates remain open. P0 online branch protection is supported here only by its historical verification record, not a current live query. The audit remains selective and incomplete; queue item 6 remains open.
