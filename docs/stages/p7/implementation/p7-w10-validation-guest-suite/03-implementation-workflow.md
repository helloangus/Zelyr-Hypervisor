# P7-W10 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W10 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
preflight set, and the supporting files for its step; and it re-inspects the
current tracked tree (`git ls-files`). The Guest asset, HVC surface, timer/SGI
scaffolding, and W04–W09 contracts are assumed inputs
([02-guest-test-asset-contract.md](02-guest-test-asset-contract.md) §2):
confirm the W01 reconciliation record
(`../p7-w01-entry-contract-reconciliation-record.md`, when it exists) has not
flagged them as blocked.

Stop and obtain direction instead of guessing when:

- the maintained Guest asset (G-1) does not exist or its scenario scaffolding
  differs structurally from the P4-W05 contract — W10 cannot bootstrap it;
  record the blocked prerequisite;
- a scenario appears to need a new Guest-visible interface, DTB convention,
  or hypervisor API — the scenario is wrong per §5 rule 1; redesign the
  scenario, and if truly impossible, record the gap as a finding;
- implementing workloads appears to require timing-based pass conditions —
  redesign per decision 5 of the parent README; or
- work reaches for Linux artifacts, machine-ABI documentation, permanent
  Guest SMP policy, or automation scripts — scope violation (W12 and P8
  own those).

## 2. Ordered implementation steps

### Step 1 — reconcile upstream assets and behavior contracts

Target: implementation record
(`../p7-w10-validation-guest-suite-record.md`, created in this step).

Work: read the P4/P5/P6 Guest records actually present and the W04–W09
sibling designs; record per-contract status for G-1..G-6 as satisfied,
changed, or absent/blocked; map each catalog scenario of
[01-scenario-suite.md](01-scenario-suite.md) to its available prerequisites.

Acceptance: per-scenario prerequisite map exists with blocked rows named.

Failure/blocker: a blocked prerequisite blocks its scenarios, not the
package's other rows.

Evidence: implementation record.

### Step 2 — declare and verify the regression suite

Target: regression declaration (suite file §2) plus the expectation table
entries for the inherited scenario sets.

Work: enumerate the inherited scenarios (P4 VG set, P5 HVC/security set,
P6 VG-TIMER/VG-IRQ set) with their original expected markers from the
upstream records; verify none were altered; record the P7-V22 evidence
procedure (run under the scheduled entry path; compare against upstream
expectations).

Acceptance: the regression declaration is complete and unchanged-inherited;
no scenario edit was needed (if one was, that is a P7 finding — record it,
do not edit the scenario).

Failure/blocker: an inherited set absent or changed — blocked prerequisite.

Evidence: implementation record.

### Step 3 — implement the VG-SCHED workload modules

Target: the Guest asset's scheduler workload modules, per scenario of
[01-scenario-suite.md](01-scenario-suite.md), honoring
[02-guest-test-asset-contract.md](02-guest-test-asset-contract.md) §3/§5.

Work: implement workload modules in upstream-defined dependency order
(VG-SCHED-01/02 first — they need only W04/W05/W06 behavior; then -03/-04
with P6 scaffolding; then -05/-06/-07/-08). Keep the Guest `no_std`,
bounded, marker-deterministic; build under the pinned toolchain and target
per the P0-W02/W03 baselines.

Suggested observation: the Guest build path declared by the workspace
baseline; scenario dry runs require the P1/P4 QEMU baselines.

Acceptance: each implemented module compiles in the Guest target; markers
match the protocol; no timing-dependent pass logic.

Failure/blocker: a missing upstream Guest surface (G-3/G-4) blocks that
scenario; record and continue with others.

Evidence: implementation record.

### Step 4 — build the host-side expectation tables and correlation

Target: expectation tables pairing markers with W09 telemetry per §4 of the
asset contract.

Work: for each scenario, encode the expected marker multiset/sequence and
the telemetry predicates (counts, reasons, sources); implement the
authority split; ensure discrepancies are representable as failed findings.

Acceptance: every catalog scenario has a machine-checkable expectation
entry; no expectation depends on timing or on trusting Guest-reported
scheduler facts.

Failure/blocker: a predicate no W09 event can express — reconcile with the
W09 design or drop the predicate to the record's qualitative check.

Evidence: implementation record.

### Step 5 — platform-contract review

Target: review per §5 of the asset contract.

Work: run the five review rules against the implemented deltas; confirm the
inherited scenarios are unchanged; confirm no ABI/machine-types documentation
was touched; confirm placement declarations use the W03/W05 surfaces.

Acceptance: all five rules hold; violations are corrected in the suite, not
waived.

Evidence: implementation record (review outcome).

### Step 6 — evidence pass

Target: verification record
(`../../verification/p7-w10-validation-guest-suite-verification.md`).

Work: execute the matrix in
[04-validation-and-handoff.md](04-validation-and-handoff.md) as far as
prerequisites exist; record passed/failed/blocked/not-run per scenario with
commands, environment (QEMU version, CPU counts), timestamps, and marker/
telemetry evidence paths. Without the P1/P4 QEMU baselines, all execution
rows are recorded not run with the owning stage named.

Acceptance: every scenario row has a status; failures include both
observation sides' evidence.

Failure/blocker: a failed scenario is recorded with diagnosis; scenarios are
never weakened to pass.

## 3. Evidence destinations

- Implementation decisions, scenario status, deltas:
  `../p7-w10-validation-guest-suite-record.md`.
- Command output, marker/telemetry evidence, run/not-run status:
  `../../verification/p7-w10-validation-guest-suite-verification.md`.
- Neither file may claim W10 complete; completion evidence lives only in the
  verification record, only for what actually ran.

## 4. Ordering and review constraints

- The Guest asset remains Guest-untrusted and `no_std`; no scenario requires
  hypervisor trust in Guest reports (README decision 6).
- No hypervisor source changes are authorized by W10 except where a scenario
  exposes a genuine W04–W09 defect — in that case the defect is fixed under
  the owning design, never patched inside the suite.
- No new `unsafe` is authorized by W10; the Guest asset's existing audited
  boundaries keep their `SAFETY` commentary.
- New ABI/public API, dependency, or machine-contract changes: none
  authorized; their appearance is a review failure.
- W12 owns automation: W10 delivers scenario definitions and expectation
  tables, not workflow scripts.
