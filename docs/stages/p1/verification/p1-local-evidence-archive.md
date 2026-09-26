# P1 local evidence archive after worktree cleanup

**Status:** Factual custody record; not new runtime evidence or a stage decision.
**Scope:** Local preservation of P1 build-tree/QEMU artifacts before removing
auxiliary Git worktrees; no CI artifact or remote-backup claim.
**Version:** v0.1.
**Owner/change context:** P1 worktree cleanup, 2026-09-26.
**Supersedes:** Worktree-path availability statements, not the historical
verification results or their original capture locations.

The historical W01–W12 records name paths under
`/home/angus/dev/Zelyr-Hypervisor/.worktrees/<name>/target/` because those
were the original execution worktrees. Before cleanup, the complete `target/`
directories from all 14 worktrees that had one were preserved in a single
local archive:

| Property | Recorded value |
|---|---|
| Archive | `/home/angus/dev/Zelyr-Hypervisor-evidence/p1-worktree-targets-2026-09-26.tar` |
| SHA-256 | `160cab674613225bdfa6921e1222c927bdc237d074fbf3e37b2e05e5aa4829bf` |
| Size | 1,597,624,320 bytes |
| Entries | 15,035 tar entries |
| Included target roots | `p1completion`, `w04`, `w05`, `w06`, `w07`, `w08`, `w08activation`, `w08full`, `w09`, `w09full`, `w10`, `w10full`, `w11`, `w11full` |
| Source comparison | `tar -df` against all original target directories exited 0 before worktree removal |

Archive member paths retain their original `.worktrees/<name>/target/...`
prefix. Key provenance examples are
`.worktrees/p1completion/target/p1-completion-evidence/r100/summary.txt`,
`.worktrees/w10full/target/p1-w10-evidence/r100-accepted/summary.txt`,
`.worktrees/w11full/target/p1-w11-nc5-post-w10/summary.json`, and
`.worktrees/w08full/target/p1-layout/layout-probe-evidence/`. The archive
also contains the remaining local build artifacts, scenario pairs and
earlier exploratory captures from those 14 target roots. Worktrees without
a `target/` directory held only Git-tracked content or disposable Python
bytecode; their committed source/document history remains in Git.

To inspect an individual historical file without recreating a worktree,
check the archive digest with `sha256sum`, list its members with `tar -tf`,
then use `tar -xOf` with the recorded member path. Extract into a separate
directory if repeated inspection is needed. **Do not use the old absolute
worktree paths as current filesystem locations after cleanup.**

This is one copy on the local host, outside Git and outside CI. It is not a
durable/off-host backup. The [P1 completion report](p1-completion-report.md)
and package verification records remain the committed summary and proof
boundary; the archive preserves their raw artifacts for local audit. NC6
has no executed artifact in the archive and remains P6-V29 work.
