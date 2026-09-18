# P4-W08 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the tree: `git ls-files`,
the P0-W09 runner plan/status, and the W05/W06/W07 implementation records
(grammar versions as delivered). Entry-order rule: manifest schema, verdict
rules, and evidence layout can be built and reviewed with host-side unit
tests against synthetic streams; every execution row (DV03–DV08,
P4-V13–V15) is blocked until M1–M6 deliver. A missing upstream is a recorded
blocked prerequisite per W01 §4 — never masked by relaxed verdicts.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the P0-W09 entry or its parameter surface is absent or unsuitable —
  record the blocked prerequisite (open item O1); do not create a
  P4-local runner;
- a grammar version differs from what this design consumed — sync by
  agreement with the owning design and bump `SM-T<n>`; do not fuzzy-match;
- making a scenario pass appears to require changing Guest/Hypervisor
  behavior — that is a defect report against the owning package (D8), not
  an automation change;
- an environment fact (QEMU version, accelerator) is undeclared — verdict
  `UNSUPPORTED` per [03 §4](03-automation-contract.md) until declared
  (open item O2);
- CI wiring pressure appears — out of scope (Reserved); route to the P0 CI
  package.

## 2. Ordered implementation steps

### Step 1 — manifest schema and scenario data

Target: the scenario manifest (artifact group, [03 §1–§2](03-automation-contract.md))
with `SM-T1` data from [02](02-scenario-matrix.md).

Work: fix the manifest schema (scenario rows keyed to VG/IS ids, repeat
sets, timeout budgets, grammar versions, environment label); encode the
SM-series rows and RS-A–RS-D sets as data.

**Acceptance:** schema validates; every matrix row is expressible; versions
declared; no expectation is embedded in code rather than data.  
**Failure/blocker:** a row that cannot be expressed data-driven is a design
gap — fix the schema here, not the tooling ad hoc.  
**Evidence:** implementation record; review row in
`../../verification/p4-w08-qemu-integration-regression-verification.md`
(DV01).

### Step 2 — collection and parsing rules

Target: the tooling's collection module ([03 §3](03-automation-contract.md)).

Work: implement marker-line extraction (VG grammar, version-checked), the
P4-RR record extractor (first/last line discipline, unknown-version
rejection), and the `HV-DIAG` presence check; all against synthetic streams
first.

**Acceptance:** golden-stream tests pass for well-formed, truncated,
version-mismatched, and noisy captures; extraction is line-exact (no
reordering tolerance, D4).  
**Failure/blocker:** a grammar mismatch stops the step for joint sync; the
parser never guesses.  
**Evidence:** verification record DV02.

### Step 3 — verdict engine

Target: the verdict module ([03 §4](03-automation-contract.md)).

Work: implement the five-verdict taxonomy with rule ids, set aggregation
per D7, and panic/fatal detection mapping; unit-test against synthetic
evidence per verdict.

**Acceptance:** every synthetic case maps to exactly one determinate
verdict with its rule named; set aggregation is all-or-nothing.  
**Failure/blocker:** a case that cannot be classified exposes a taxonomy
gap — fix the taxonomy in this design first.  
**Evidence:** verification record DV06/DV07.

### Step 4 — entry point integration

Target: the single P4 regression command over the P0-W09 entry
([03 §2–§3](03-automation-contract.md)).

Work: integrate build invocation (documented build commands, pinned
toolchain), parameter passing (open item O1), boot, capture, extraction,
verdict, and evidence writing per §6.

**Acceptance:** one command runs the declared set end to end on a stubbed
QEMU (host-side integration test with a scripted fake stream); evidence
layout produced correctly; no second QEMU path exists (review).  
**Failure/blocker:** runner surface gaps are blocked prerequisites recorded
against P0-W09; execution rows stay blocked, tooling review proceeds.  
**Evidence:** verification record DV01/DV08.

### Step 5 — execution and evidence (gated on M1–M6 delivered)

Target: real runs; evidence under `docs/stages/p4/verification/`.

Work: execute RS-D (P4-V13/V14 rows), then RS-A/RS-B, then RS-C with ≥ 5
iterations (P4-V15); record every iteration's verdict and evidence per
[03 §6](03-automation-contract.md); summarize in the verification record
with run/not-run status per row.

**Acceptance:** each executed row returns a determinate verdict; RS-C is
stable per [02 §3](02-scenario-matrix.md); every non-`PASS` has evidence
and a defect-report destination.  
**Failure/blocker:** failures are recorded as failures with diagnosis;
they never alter expectations (D2/D8). Timeout budget changes require a
recorded manifest change, never an in-place stretch.  
**Evidence:** verification record DV03–DV07 rows and the raw-run
directories.

### Step 6 — closure review and handoff

Work: run the review matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md), confirm the
handoff checklist, and record implementation facts (entry-point usage,
manifest version, declared environment, limitations) in
`../p4-w08-qemu-integration-regression-record.md` when work starts. Results
are claimed only in the verification record, with evidence, only for what
actually ran; this package designs and builds the regression — it never
reports a pass in a design document.

## 3. Validation matrix

See [05-validation-and-handoff.md](05-validation-and-handoff.md) §1
(W08-DV01 through W08-DV09). Each row is recorded as **passed / failed /
blocked / not run** with command or review input, environment, date, and
reason. No row here proves P4-V01–V12 or P4-V16, and none may be reported
as doing so; QEMU rows prove the stated reference environment only (W01
A7).
