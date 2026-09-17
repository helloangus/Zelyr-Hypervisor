# P0-W01 Repository Baseline — Verification Evidence

**Status:** Complete evidence recorded; W01 closure claimed.
**Date:** 2026-09-17 (Asia/Shanghai)
**Environment:** `/home/angus/dev/Zelyr`, Linux shell, unversioned scaffold at
start; GitHub selected as hosting platform, but no owner/repository URL or
GitHub CLI/authentication was available.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W01-DV01 / P0-V01 | Git repository bootstrap | **passed** | `git init -b main`; initial W01 commit exists on `main`; staged scope was reviewed before commit; `codex.sh` is ignored. |
| W01-DV02 / P0-V01 | Fresh-clone layout | **passed** | A disposable clone of the initial commit was checked; all README-promised top-level areas and intentional empty subdirectories are represented by files or tracked markers. |
| W01-DV03 / P0-V01 | Machine-local prerequisite scan | **passed** | Tracked project entry points were searched for `/home/`, `/tmp/`, user-specific paths, private files, and unconditional sibling-repository requirements; none remain as project requirements. Paths in the implementation/verification evidence are historical observations, not prerequisites. The ignored helper is explicitly non-project. |
| W01-DV04 / P0-V09 | Root navigation links | **passed** | README links to `AGENTS.md` and `docs/README.md`; both targets exist. The README does not claim that reading it authorizes changes. |
| W01-DV05 / P0-V09 | Text and ignore policy | **passed** | `.editorconfig` states UTF-8/LF/final newline/four-space indentation/trailing-whitespace policy; `.gitignore` covers generated/local noise and `codex.sh`; all markers are tracked. |
| W01-DV06 / P0-V01/P0-V09 | License review | **passed** | Root `LICENSE` contains the owner-approved Apache License 2.0 text with `Copyright 2026 Angus Lee`; README links to `LICENSE`. |
| W01-DV07 / P0-V01/P0-V09 | Commit and scope review | **passed** | Initial commit `9c3f677e08861d49a388291f92fc24e39c8d50dd` contains the existing P0 baseline plus W01 root navigation, markers, implementation record, and verification record. The W01 closure commit adds only the owner-approved license, README link, and W01 record updates; no Rust, Cargo, ABI, dependency, runtime, or build contract was added. |

## Commands and observed results

The following reviews were run from the repository root:

```text
git init -b main
  Initialized empty Git repository .../.git/
git check-ignore -v codex.sh
  .gitignore:11:codex.sh codex.sh
git branch --show-current
  main
git log -1 --format='%H %s'
  9c3f677e08861d49a388291f92fc24e39c8d50dd chore: establish P0 repository baseline
git diff-tree --root --no-commit-id --name-status -r HEAD
  70 added baseline/W01 files; no codex.sh
```

The clone review used a disposable directory under `/tmp`; it verified the
initial commit's tracked tree and removed the disposable clone after review.
The tracked tree was also searched for machine-local path indicators. The only
`/home/...` path is in ignored `codex.sh`, which is intentionally absent from
the commit.

## Not run / not proved

- No Cargo, Rust, toolchain, target, build, QEMU, guest, hardware, or CI test
  was run; W01 does not authorize those interfaces.
- No remote publication was attempted: GitHub was selected, but `gh` is not
  installed and no GitHub authentication, owner, repository name, or canonical
  URL is available.
- Remote publication remains unverified: `origin` is configured for GitHub,
  but this environment could not resolve `github.com` during the push attempt.
  This does not block the local W01 repository-baseline closure.
