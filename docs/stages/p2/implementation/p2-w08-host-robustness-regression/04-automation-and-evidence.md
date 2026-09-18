# P2-W08 Automation Contract and Evidence Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W08 detailed design](README.md).

## 1. Suite organization

- Five scenario groups ([01 §5](01-scope-and-foundations.md)) execute in
  fixed order S1→S5; within a group, rows execute in ID order. Group
  boundaries are hard: S2 inputs are built from S1-validated fixtures only
  where they overlap, and every group's execution is independent enough to
  be re-run alone by ID.
- Each row is one test case with a stable ID (`W08-S<group><nn>`); IDs are
  the join key between matrices ([02](02-matrices-input-and-map.md),
  [03](03-matrices-allocator-stress-determinism.md)), evidence rows, and
  the P2-V10 acceptance review.
- Harness code asserts published contracts only ([01 §2](01-scope-and-foundations.md)
  rule 1); expected observables are encoded once per row, adjacent to the
  row's driver, with the matrix row ID in a comment traceable in review.

## 2. Execution entry

The suite hangs off the P0-W08 host-test baseline's entry point (assumed
contract A8): one invocation runs all groups; per-group and per-ID
selection are supported filters, not separate entry points. Exact command
spelling follows the baseline's documented interface when it exists and is
recorded per run in evidence — the spelling is not this design's contract
(plan out-of-scope: "implementation test commands").

Blocked boundary: if the P0 baseline entry does not exist at W08
implementation time, scenario code may still be written but execution is
recorded blocked (upstream defect); no parallel runner is created.

## 3. Seeds and scale parameters

Defaults fixed by this design; changing any of them is a recorded revision
and invalidates comparability with earlier evidence:

| Parameter | Default | Applies to |
|---|---|---|
| Mutation sweep count (S112) | 256 single-byte mutants, base blob fixed | S1xx |
| Soak op count (S308) | ≥ 10 000 ops, audit per op | S3xx |
| Stress op count (S401–S403) | ≥ 100 000 ops (pages), ≥ 100 000 ops (heap) | S4xx |
| Recovery cycles (S404) | C = 16 | S4xx |
| Repeated runs (S5xx) | R = 3 in-process + 1 cross-process | S5xx |
| Seeds | `W08_SEED_DTB=0x5EED0001`, `W08_SEED_PAGE=0x5EED0002`, `W08_SEED_HEAP=0x5EED0003`, `W08_SEED_MIX=0x5EED0004` | as named |

Every evidence row for a seeded scenario records the seed and the
parameter set actually used. A run without its seed recorded is invalid
evidence, not a weaker pass.

## 4. Pass/fail aggregation

- A row passes only when its pass condition holds verbatim.
- Group status: `passed` iff every row in the group passed; otherwise the
  group carries the failed/blocked/not-run rows explicitly.
- Suite status for P2-V10: all five groups `passed`. No weighting, no
  "known-failure" waivers inside P2; a waiver would be a task-book-level
  decision, not a harness state.
- Statuses are exactly: `passed`, `failed`, `blocked` (upstream missing),
  `not run` (not executed this run). Partial suite runs record per-row
  status and leave the rest `not run`.

## 5. Evidence record

Destination: `docs/stages/p2/verification/p2-w08-host-robustness-regression-verification.md`
(created when evidence exists; never pre-filled by design or plan).

Per-row record schema (the format contract W10 references):

```text
row ::=
  id: W08-S<group><nn>
  status: passed | failed | blocked | not run
  command: <exact invocation>
  input:  <fixture/scenario inputs; generation rule for synthetic inputs>
  seed:   <seed + params, if seeded>
  expected: <matrix row's expected observable>
  actual:  <observed outcome; required when status = failed>
  environment: <host OS, toolchain (pinned per P0-W02), date/time>
  owner: <owning package of the asserted contract>
```

Plus a suite header: total per status, per-group status, suite-level
environment, and the not-run list with reasons. The verification record is
append-only per run; a re-run adds a dated section rather than overwriting
(compare [W07 fixture stability](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md)
§7 for the same principle on fixtures).

## 6. Rerun and failure-handling rules

- A failed row is re-run in isolation first (its ID filter) to confirm;
  the isolation run is recorded as its own dated section referencing the
  failing run.
- A row that cannot execute for lack of an upstream contract is `blocked`
  with the owning package named — never silently `not run`.
- A scenario failure is never fixed by adjusting the expectation; expected
  observables change only through a recorded revision of the matrix (which
  is a design change against the owning package's contract, and if the
  contract itself changed, against that package's design).
