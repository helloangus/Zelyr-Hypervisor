# P0-W01 Repository Baseline — Implementation Record

**Status:** Completed; verification evidence recorded.
**Work package:** [P0-W01](../plans/p0-w01-repository-baseline.md)
**Detailed design:** [W01 implementation design](p0-w01-repository-baseline/README.md)

## Current-state finding

Implementation began from an unversioned P0 scaffold. No Git metadata or
tracked-file view existed. The repository therefore required the foundation
deliverable of a local Git repository on `main` with an initial committed
baseline before clone and scope reviews could be meaningful.

## Changed artifacts

- Updated the root `README.md` with mandatory navigation, P0 scope wording,
  the complete root layout, and the tracked-marker clone contract.
- Added `.gitkeep` markers to the intentionally reserved empty directories
  listed below.
- Retained the existing `.editorconfig` and `.gitignore` after review; the
  local `codex.sh` helper remains ignored and is not a project entry point.
- Added this implementation record and the W01 verification record.
- Added root `LICENSE` with the owner-approved complete Apache License 2.0
  text and `Copyright 2026 Angus Lee`; README links to it.

## Final marker inventory

The following empty directories receive tracked markers so the layout survives
a fresh clone:

```text
.github/workflows
boards/orangepi-3b
boards/qemu-virt
control/linux-agent
control/rust-domain
crates
docs/stages/p0/verification
docs/stages/p1/implementation
docs/stages/p1/plans
docs/stages/p1/verification
guests/validation-aarch64
hypervisor/src
scripts
soc/rk356x
tests
```

Each marker is used only where no substantive tracked file or scoped README
exists. Existing documentation directories with README files were not given
additional markers.

## Machine-specific assumptions

- The current working directory was `/home/angus/dev/Zelyr`; this is an
  execution environment, not a repository prerequisite.
- `codex.sh` contains a machine-specific `/home/angus/dev/prebuilt_manager/prebuilt`
  path and is therefore ignored as a local helper. It is not documented or
  required by the repository baseline.
- No tracked project file was found to require an absolute path, private file,
  sibling repository, environment variable, toolchain, target, or generated
  directory.

## Deliberately unchanged / later owners

- No Cargo manifest, Rust source, target, build profile, CI job, QEMU command,
  or runtime interface was added. These remain with their applicable P0 work
  packages.
- GitHub is the selected hosting platform. No remote was configured or
  published because this environment has no `gh` CLI, no detected GitHub
  authentication, and no owner/repository name or canonical URL. Remote
  creation remains pending until those inputs are available; it must not be
  guessed from the local email address.

## Resolved license decision

The owner selected Apache License 2.0 with `Copyright 2026 Angus Lee` on
2026-09-17. The root `LICENSE` contains the full approved text. W18 remains
the owner of any future dependency or third-party notice analysis.
