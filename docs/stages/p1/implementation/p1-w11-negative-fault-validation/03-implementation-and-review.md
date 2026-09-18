# P1-W11 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W11 detailed design](README.md).

## 1. Preconditions and failure boundary

W11 can execute scenarios only when the mechanisms under test exist: W01
rejection boundary, W03 capability fail-fast, W05 vectors and classification,
W07 diagnostic fields, W08 Stage-1 transition, W09 phase attribution and
routes, and the W10 execution/evidence conventions — all assumed contracts
fixed by their accepted designs. Before any work, verify what exists
(`git ls-files`; accepted designs, records, buildable image, runner entry).

Stop and obtain direction instead of guessing when any of the following
occurs:

- a scenario's target mechanism does not exist or its design does not expose
  the insertion point or diagnostic class the scenario expects — record the
  blocked scenario with the gap; do not modify the supplying package's
  contract to make the scenario expressible;
- inducing a fault appears to require changing normal boot behavior, adding
  a runtime fault-mode switch, or relaxing a control — forbidden (plan work
  seq 3); stop;
- a scenario would target the unowned pre-vector window or require hardware
  injection, guest execution, or GIC/IRQ subsystems — out of scope; record
  the exclusion instead;
- W10's conventions are absent or contradictory — upstream defect to W10;
  do not invent a parallel evidence scheme;
- the W03 required list contains no reference-platform-variable capability —
  record NC2 as blocked with that finding (an upstream limitation), do not
  fabricate a capability absence in software.

## 2. Ordered implementation steps

### Step 1 — confirm contracts and fix scenario parameters

Target: implementation record
(`../p1-w11-negative-fault-validation-record.md`, created in this step).

Work: for each scenario NC1–NC6, extract from the accepted designs: the
diagnostic field list and marker tokens (W05/W07), the phase route (W09),
the insertion point for in-image triggers, and the environment-variation
technique (NC1/NC2) permitted by the W01/runner contracts. Record the exact
trigger instructions/encodings and the validation-selection mechanism
adopted in the build, per
[01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) §3.

Suggested observation: read-only inspection; one exploratory build of the
scenario selection (recorded as exploration, not evidence).

**Acceptance:** every scenario row in the record names its technique,
insertion point, expected class tokens, route, and containment; no row is
"unknown".  
**Failure/blocker:** a missing contract blocks that scenario per §1; the
others proceed.

### Step 2 — implement the selection-gated triggers

Target: the validation selection in the build (build-selection mechanics
follow the P0 build/feature governance; the semantic contract is
[01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) §3) and the
`fault_scenario` entry with its recorded insertion points.

Work: implement the diverging trigger entry and the minimal insertion calls;
ensure the default selection compiles with zero trigger references.

**Acceptance:** default build contains no reachable trigger (S6 of
[02-scope-security-review.md](02-scope-security-review.md)); each scenario
build differs only by the selection and trigger code; triggers diverge and
have no timing dependence.  
**Failure/blocker:** an insertion point that cannot be expressed without
touching a mechanism's internal state is a design conflict to record; do not
reach into the mechanism.

### Step 3 — execute the scenario matrix

Target: verification record
(`../../verification/p1-w11-negative-fault-validation-verification.md`).

Work: run NC1–NC6 through the W10 conventions (bounded run, outcome
classification, capture retention), each twice, per
[01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) §4. Record both
runs per scenario: outcome, phase attribution, diagnostic class observed,
terminal behavior, technique, image/environment identities, evidence paths.

**Acceptance:** each scenario meets its pass condition on both runs, or is
recorded failed/blocked with diagnosis and owner.  
**Failure/blocker:** a scenario that produces the wrong class, wrong
attribution, unbounded behavior, or non-reproducibility is a P1-V18 finding
against the owning package — recorded, never waived or locally patched.

### Step 4 — perform the scope/security review

Target: verification record, scope/security review section.

Work: execute [02-scope-security-review.md](02-scope-security-review.md) S1–S6
over the current baseline, recording outcomes and findings with owners.

**Acceptance:** all six items pass at the recorded baseline, or findings are
recorded with owning packages and the P1-V19 status is stated truthfully.  
**Failure/blocker:** a finding that is a stage-boundary violation (S4) stops
closure until resolved by the owning packages.

### Step 5 — reproducibility confirmation and matrix completion review

Work: confirm every executed scenario has two concordant runs; confirm the
matrix covers exactly the plan's classes (unsupported environment,
missing-required-capability, synchronous fault, panic, post-MMU fault,
unexpected vector) with exclusions recorded (pre-vector window, hardware,
guest, fuzz). Confirm R2 of
[P1-W10](../p1-w10-qemu-boot-regression/README.md) is (or becomes) anchored
on the real NC4 panic scenario so harness detection and fault evidence share
one source of truth.

**Acceptance:** the completion review table in the verification document maps
each class to its evidence and status; exclusions are explicit.  
**Failure/blocker:** a coverage gap is a P1-V18 finding, not a silent
omission.

### Step 6 — closure and handoff

Work: confirm the handoff checklist (§5); record changed files (trigger code
and build selection only), the limitations for
[P1-W12](../p1-w12-p1-documentation-handoff/README.md) (unowned pre-vector
window; environment-only coverage; no recovery semantics), and the P2-facing
boundary statement. Completion is claimed only in the verification record,
only for what ran.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W11-DV01 → P1-V18 | Matrix-coverage review | map plan classes to NC1–NC6 against [01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) §2 | every plan class has a scenario with setup, trigger, expected class, and route; exclusions recorded | coverage by design; not execution |
| W11-DV02 → P1-V18 (plan work seq 3) | Trigger-containment review/build | S6 of [02-scope-security-review.md](02-scope-security-review.md); build default vs scenario selections | default image has no reachable trigger; scenario deltas are minimal and recorded | normal scope unchanged; not scenario correctness (that is execution) |
| W11-DV03 → P1-V18 | Scenario execution | NC1–NC6 twice each via W10 conventions | expected diagnostic class, correct phase attribution, routed terminal outcome, no continuation; both runs concordant | the listed fault classes are bounded and diagnosable on the reference platform; not hardware, not unlisted classes, not recovery |
| W11-DV04 → P1-V18 | Reproducibility review | compare paired runs per scenario | identical outcome class, attribution, and terminal classification per pair | determinism of the scenarios; not statistical robustness under load |
| W11-DV05 → P1-V19 | Scope/security review | [02-scope-security-review.md](02-scope-security-review.md) S1–S6 | all items pass at the recorded baseline or findings have owners | no unintended relaxation/RWX, current unsafe inventory, intact boundary — by inspection; not absence of all vulnerabilities |
| W11-DV06 → W11 closure | Consumability review | read the outputs as W12 (evidence requirements and limitations usable?), P2-W08/W09 (boundary statement clear?) | consumers can act without inventing W11 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Only executed
scenarios with concordant pairs provide P1-V18 evidence; only the completed
S1–S6 review provides P1-V19 evidence. No validation here proves
P1-V01–P1-V17 or P1-V20–P1-V21.

## 4. Error, security, and observability model

**Errors.** Scenario failures are findings against owning packages, recorded
with diagnosis; harness and environment errors are distinguished from fault
outcomes via the W10 classification. Nothing retries or averages; a
non-reproducing scenario is failed evidence.

**Security.** The security posture of the validation itself: triggers are
build-time selected and absent from the default image (S6); fault paths must
not relax controls (S5); serial capture is untrusted input handled by W10's
bounded matching. The review items S1–S3 audit the underlying controls that
P1 owes per the Coding Guidelines and the ADR's untrusted-boundary
principles; W11 adds no new authorization or trust decision.

**Observability.** Every scenario produces a retained capture, a classified
outcome, and a verification-document section; exclusions and blocked
scenarios are recorded as such. The diagnostic classes under test are
themselves the observability contract of P1 (W05/W07); W11's contribution is
evidence that they fire on the paths that matter, not new telemetry.

## 5. Handoff checklist

Before handing W11 to a reviewer, provide:

- the exact changed-file list (trigger entry, insertion calls, build
  selection; nothing else);
- the recorded scenario parameters (techniques, insertion points, class
  tokens, routes) and any scenario blocked with its upstream finding;
- NC1–NC6 paired-run evidence paths and statuses, including explicit not-run
  entries;
- the S1–S6 review outcomes with baseline identity and findings with owners;
- confirmation that no guest, recovery, fuzzing, GIC/IRQ, or hardware
  injection mechanism was introduced, and that the default image is
  trigger-free;
- open items for W12: negative-evidence requirements for the evidence map;
  limitations (pre-vector window, environment-only coverage); R2/NC4
  anchoring with W10 — recorded, not resolved here.
