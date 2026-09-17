# Zelyr Branch and Pull-Request Integration Workflow

**Status:** Normative project policy; GitHub enforcement is planned in P0-W20.
**Scope:** All development changes made after adoption of this policy.
**Owner/change context:** Repository integration policy, adopted 2026-09-17.
**Supersedes:** The absence of an explicit branch-and-PR integration policy.

## Policy

`main` is the sole integration branch. New development must not be performed
directly on `main`. Each coherent change starts from a new branch based on the
current `main`, is pushed to GitHub, and is merged into `main` only through a
GitHub pull request after the PR's configured required online checks pass.

The W01 bootstrap commits that predate this policy are historical baseline
records, not an exception for later development.

## Required change lifecycle

1. Create one branch for the bounded work package or coherent documentation
   change. Use a descriptive namespace such as `p0/w02-toolchain`,
   `p1/w01-el2-entry`, or `docs/topic`.
2. Implement only the approved, branch-scoped change and record validation in
   its stage-local implementation and verification locations where required.
3. Push the branch to the configured GitHub remote and open a pull request
   with `main` as its base branch. The PR description identifies the work
   package or policy change, changed contracts, validation run and not run,
   and any unresolved conflict.
4. Wait for every GitHub check configured as **required** for that PR to pass.
   A missing, cancelled, skipped, or failed required check is not passing
   evidence.
5. An authorized maintainer merges the PR into `main` only after the required
   checks pass. The selected GitHub merge method must preserve a reviewable
   connection between the merged change and its PR.

## Boundaries and enforcement

- A branch name is organizational metadata; it does not replace the approved
  detailed design, validation evidence, or work-package scope checks.
- Pull-request checks prove only the checks configured by P0-W20. They do not
  prove QEMU, guest, or hardware behavior unless such evidence is explicitly
  collected and reported.
- P0-W20 owns GitHub workflow triggers, required-check classification, branch
  protection configuration, and evidence that GitHub actually enforces this
  policy. Until that work is implemented, this document is the required human
  process, not proof of technical enforcement.
- Direct development commits and direct pushes to `main` after adoption of this
  policy are prohibited. A GitHub administrator's ability to bypass protection
  is not permission to bypass this policy.

## Related documents

- [P0 task book](../stages/p0/task-book-v0.1.md) assigns the workflow contract
  to P0-W19 and online enforcement to P0-W20.
- [P0-W19 plan](../stages/p0/plans/p0-w19-reproducible-development-workflow.md)
  owns contributor onboarding and the documented branch-to-PR path.
- [P0-W20 plan](../stages/p0/plans/p0-w20-ci-baseline.md) owns CI and
  `main`-protection implementation.
