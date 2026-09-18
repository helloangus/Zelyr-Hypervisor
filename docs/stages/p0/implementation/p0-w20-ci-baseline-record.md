# P0-W20 CI Baseline — Implementation Record

**Status:** Implemented on branches `p0/w20-ci-baseline` (merged as PR #31)
and `p0/w20-ci-enforcement` (PR #32); verification evidence in [the
verification record](../verification/p0-w20-ci-baseline-verification.md).
**Date:** 2026-09-19 (Asia/Shanghai)
**Design:** [W20 detailed implementation design](p0-w20-ci-baseline/README.md)

## Delivered configuration

- **Workflow:** `.github/workflows/ci.yml` — six jobs named verbatim by the
  gate evidence labels; triggers `pull_request`(→`main`) and `push`(`main`);
  concurrency cancel for superseded same-PR runs only; no path filters or
  `if:` conditions on any required check; `timeout-minutes: 15` on every job
  (recorded value); `permissions: contents: read`; checkout via plain Git;
  rustup via the official installer invocation; toolchain, components, and
  targets provisioned exclusively from `rust-toolchain.toml`. No
  third-party marketplace action is used.
- **Protection settings (observed via API at configuration time):**
  - `required_status_checks`: `strict: false`, contexts exactly
    `QG-FMT`, `QG-LINT`, `QG-WARN`, `QG-TEST-HOST`, `QG-BUILD-TARGET`,
    `QG-DOCS`;
  - `enforce_admins: true`;
  - `required_pull_request_reviews`: `required_approving_review_count: 0`
    (a pull request is required; no approval count is imposed beyond the
    policy's maintainer-merge rule);
  - `allow_force_pushes: false`, `allow_deletions: false`;
  - up-to-date requirement **not** imposed (per contract §8.5).
- **Merge methods (observed):** `allow_merge_commit: true`,
  `allow_squash_merge: false`, `allow_rebase_merge: false` — the enabled
  method preserves the PR connection (contract §8.4).

## Same-change reconciliations

- The two routed detailed references (`coding-guidelines-v0.1.md`,
  `plan-agent-design-guidelines-v0.1.md`) lacked the metadata fields the
  documentation baseline requires; headers added (found by the QG-DOCS
  dry run).
- The contributor workflow's §3.4 enforcement status was retired in the same
  change per its own design ("retired by W20's delivery"), and its S6 step 5
  now cites the delivered check-mapping register.

## Recorded minor changes

- Job-timeout values recorded above (15 minutes per job).
- Failure-artifact mechanism recorded: the failing QG-BUILD-TARGET prints
  its complete build log into the job log (platform default retention); a
  dedicated upload action would be a third-party action and is Reserved.
- Merge-method set recorded above.

## Deviations from the design

- None of the failure boundaries (§4 of the protection design) applied: the
  remote was configured, administrator access was available, and every
  prerequisite contract had delivered with the expected gate set and labels.
