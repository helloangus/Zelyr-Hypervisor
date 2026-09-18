# P7-W12 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W12 detailed design](README.md).

## 1. Preconditions and failure boundary

Before executing any step, the implementer verifies it has loaded the
documents named in the parent README and confirms, through the P7-W01
recorded input boundary:

- the P0-W09 runner entry exists, is documented, and provides serial capture,
  timeout, exit status, and evidence collection (or is a documented
  placeholder whose limits are recorded — in which case execution steps are
  blocked until it is real);
- P7-W02–W11 contracts are evidenced for every behavior a case asserts;
- the W10 workload suite provides every payload and marker a case references;
- the W11 handoff (required scenarios with parameters) is available.

Stop and record a blocker instead of improvising when:

- the runner entry is absent, undocumented, or diverges from the assumed
  contract — W12 is blocked as a whole; it never builds a second runner;
- a case's payload, marker, or asserted behavior lacks an evidenced contract
  — that case is blocked and named;
- execution appears to require CI changes, hypervisor source changes, or
  Guest-code changes — a scope violation; stop and record;
- a timeout floor, result class, or matrix row would need to change — follow
  the design-change path of the parent README, not a local shortcut.

## 2. Ordered implementation steps

### Step 1 — inspect runner governance and W11 scenarios

Target: implementation record
(`docs/stages/p7/implementation/p7-w12-qemu-regression-record.md`, created in
this step).

Work: record the P0-W09 runner entry's actual contract as found (invocation
surface, parameter space, capture/timeout/exit behavior, evidence handling)
and the W11 handoff contents as found. Note every divergence from what this
design assumes in [the runner contract](01-regression-matrix-and-runner-contract.md) §1.

**Acceptance:** the record states the consumed runner contract and W11
handoff with no unrecorded assumption.  
**Failure/blocker:** a blocking divergence stops the package before any case
runs.

### Step 2 — fix concrete case declarations

Target: implementation record.

Work: instantiate all fourteen cases of
[the matrix](01-regression-matrix-and-runner-contract.md) §3 into complete
declarations per the §2 schema: concrete payloads and markers, controls,
repetition counts (≥ floors), timeout values (≥ floors) for the declared
environment, and the environment declaration template. Record any reduction
of a W11-derived case and its bound.

**Acceptance:** every case has all schema fields populated; no floor is
violated; marker references resolve to declared W10/W11 markers.  
**Failure/blocker:** an unresolvable marker or payload reference blocks that
case, named.

### Step 3 — execute the matrix and classify results

Target: verification record
(`docs/stages/p7/verification/p7-w12-qemu-regression-verification.md`) and the
per-case artifact directories.

Work: run each case through the runner entry with its declaration; classify
each result per the taxonomy; assemble the artifact set of §4 for every FAIL;
record INFRA-BLOCKED causes separately from scheduler failures.

**Acceptance:** every attempted case has a result class, a complete
environment declaration, and (on FAIL) the full artifact set; every
not-attempted case is NOT-RUN with reason.  
**Failure/blocker:** repeated INFRA-BLOCKED on a case blocks the matrix
report until the cause is resolved or accepted as a recorded limitation.

### Step 4 — coverage and limit review

Target: verification record (review section).

Work: review the executed matrix against P7-V28 (all five topology points and
four control families with determinate results), confirm the QEMU-versus-
architecture statement of [the runner contract](01-regression-matrix-and-runner-contract.md)
§6 appears in the evidence, and confirm no case was widened beyond its
declared pass condition to make it pass.

**Acceptance:** the review names, per P7-V28 element, the covering cases and
their results; the limit statement is present; determinacy holds for every
reported case.  
**Failure/blocker:** a coverage hole or an indeterminate case is recorded as
a finding for P7-W14 closure; it is never silently dropped.

### Step 5 — hand off closeout inputs to W14

Target: implementation record (handoff section).

Work: emit to [P7-W14](../p7-w14-documentation-p8-handoff/README.md): the
per-case evidence index entries, the coverage statement, the QEMU-limit
statement, FAIL/INFRA-BLOCKED findings with artifacts, and the automation
input locations a later consumer (including P8-W16 via P8-W11/W14) would need
to re-run or extend the matrix.

**Acceptance:** the handoff is complete against P7-V30's "automation/evidence
locations" input and grants W14 nothing unrecorded.  
**Failure/blocker:** a missing item is recorded as an open item with owner;
it is not backfilled silently.

## 3. Validation matrix

Every row is planned evidence; none asserts a run has happened. Each is
recorded as passed / failed / blocked / not run with command/input,
environment, timestamp, and reason.

| ID | Review or test | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W12-DV01 | Prerequisite and composition review | Workflow step 1 against [runner contract](01-regression-matrix-and-runner-contract.md) §1 | Runner contract and W11 handoff recorded; every case source identified | Composition is grounded; not that cases run |
| W12-DV02 → P7-V28 | Matrix execution and classification | Workflow step 3 over all fourteen cases | Every case determinate (PASS or named FAIL with artifacts); no invalid evidence | The declared scheduler matrices behave as declared under the declared QEMU environment; not hardware semantics, not P7-V22–V27 restated |
| W12-DV03 | Coverage and limit review | Workflow step 4 | P7-V28 fully mapped; QEMU-limit statement present; no widened pass condition | Review integrity; not that the matrix is a complete scheduler test suite |
| W12-DV04 → W14 prerequisite | Closeout-handoff review | Workflow step 5; read as W14 would | Evidence index entries, findings, and automation locations complete | Handoff completeness; not that W14 closure is done |

No validation here satisfies P7-V22–V27 (owned by W10/W11) or P7-V29/V30
(W13/W14), and none may be reported as doing so.

## 4. Error, security, and observability model

W12 adds no hypervisor error path and weakens no boundary: cases run Guests
under the same untrusted-input rules as any other execution (ADR-007), and a
case must never require disabling checks, relaxing isolation, or granting the
runner privileged paths to pass. Failure reporting is the result taxonomy
plus the artifact set; an unclassifiable outcome is FAIL or INFRA-BLOCKED
with the gap named, never a pass.

Observability is the deliverable: per-case environment declarations,
declarations actually used, serial captures, and W09 diagnostic content are
the accepted proof surface. Documentation-grade failures — a missing
environment field, an unpopulated schema field, a lowered floor, or an
unrecorded INFRA-BLOCKED cause — fail the associated review and are recorded
as such.

## 5. Handoff checklist

Before handing W12 results to a reviewer, provide:

- the implementation record with the runner-contract findings and all
  fourteen case declarations;
- the verification record with per-case results, environment declarations,
  and run/not-run entries for the full matrix;
- complete artifact sets for every FAIL and named causes for every
  INFRA-BLOCKED;
- confirmation that no runner, script, CI workflow, hypervisor source, or
  Guest-code change was made by W12;
- the W14 handoff (step 5) and the standing QEMU-limit statement — without
  resolving W14's closure contract here.
