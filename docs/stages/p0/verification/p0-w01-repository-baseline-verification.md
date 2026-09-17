# P0-W01 Repository Baseline — Verification Evidence

**Status:** Partial evidence recorded; W01 closure blocked by the missing
owner-approved license decision.
**Date:** 2026-09-17 (Asia/Shanghai)
**Environment:** `/home/angus/dev/Zelyr`, Linux shell, unversioned scaffold at
start; no remote URL supplied.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W01-DV01 / P0-V01 | Git repository bootstrap | **passed** | `git init -b main`; initial W01 commit exists on `main`; staged scope was reviewed before commit; `codex.sh` is ignored. |
| W01-DV02 / P0-V01 | Fresh-clone layout | **passed** | A disposable clone of the initial commit was checked; all README-promised top-level areas and intentional empty subdirectories are represented by files or tracked markers. |
| W01-DV03 / P0-V01 | Machine-local prerequisite scan | **passed** | Tracked project files were searched for `/home/`, `/tmp/`, user-specific paths, private files, and unconditional sibling-repository requirements; none remain as project requirements. The ignored helper is explicitly non-project. |
| W01-DV04 / P0-V09 | Root navigation links | **passed** | README links to `AGENTS.md` and `docs/README.md`; both targets exist. The README does not claim that reading it authorizes changes. |
| W01-DV05 / P0-V09 | Text and ignore policy | **passed** | `.editorconfig` states UTF-8/LF/final newline/four-space indentation/trailing-whitespace policy; `.gitignore` covers generated/local noise and `codex.sh`; all markers are tracked. |
| W01-DV06 / P0-V01/P0-V09 | License review | **blocked** | No approved license text, copyright holder, or year was supplied. No `LICENSE` was invented, so the required root artifact and README link remain pending. |
| W01-DV07 / P0-V01/P0-V09 | Commit and scope review | **passed with blocker** | Initial commit contains the existing P0 baseline plus W01 root navigation, markers, implementation record, and verification record; no Rust, Cargo, ABI, dependency, runtime, or build contract was added. License remains unresolved. |

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
  <recorded after the initial commit below>
git diff-tree --no-commit-id --name-status -r HEAD
  <recorded after the initial commit below>
```

The clone review used a disposable directory under `/tmp`; it verified the
initial commit's tracked tree and removed the disposable clone after review.
The tracked tree was also searched for machine-local path indicators. The only
`/home/...` path is in ignored `codex.sh`, which is intentionally absent from
the commit.

## Not run / not proved

- No Cargo, Rust, toolchain, target, build, QEMU, guest, hardware, or CI test
  was run; W01 does not authorize those interfaces.
- No remote publication was attempted because no destination was supplied.
- The package is not fully closed until the owner-approved `LICENSE` is
  installed and the README link is added, followed by a rerun of W01-DV06 and
  the final scope review.
