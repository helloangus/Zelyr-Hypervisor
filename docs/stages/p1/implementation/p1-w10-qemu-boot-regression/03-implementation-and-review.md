# P1-W10 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W10 detailed design](README.md).

## 1. Preconditions and failure boundary

W10 can produce verdicts only when its prerequisites exist: the P0-W09 runner
entry (assumed contract), the W01 canonical invocation and a buildable
reference image (assumed contracts), the W09 stable state and marker
vocabulary (assumed contract), and the W07 fatal/panic marker classes (assumed
contract). Before any work, verify what actually exists (`git ls-files`;
runner entry, image build path, accepted W01/W09 designs or records).

Stop and obtain direction instead of guessing when any of the following
occurs:

- the P0 runner interface contract does not exist or lacks serial capture,
  timeout, exit status, or evidence collection — upstream defect (task book §1);
  record the blocker; do not write a second, competing QEMU invocation path;
- the W01 canonical invocation is ambiguous or the image cannot be built
  reproducibly — upstream defect to the owning packages; do not invent a
  private recipe;
- the W09/W07 marker vocabulary is not fixed enough to select unique tokens —
  raise the coordination issue; do not match on prose;
- making a scenario pass appears to require changing the boot path, the
  marker emission, or W07's reports — that is fixing the thing under test;
  stop and record the defect;
- implementing the driver seems to require CI configuration — out of scope
  (P0-W20 boundary); the contract is consumable, the wiring is not.

## 2. Ordered implementation steps

### Step 0 — supply the runner foundation

Implement the single tracked runner under `scripts/` against the
[foundation contract](01-automation-contract.md#11-runner-foundation).
Record P0's interface-only state, then implement frozen grammar, profiles,
process lifecycle, complete capture, hard timeout, six statuses and evidence
finalization. Record any W01 image conversion entry separately; the driver
does not build the hypervisor and only the runner launches QEMU. Before Step 1,
the runner must be callable for exploration and its mechanism checks must
pass. W10-DV03/DV04 record results. Missing executable code is work for this
step; a contradictory normative interface remains a contract finding.

### Step 1 — fix the marker rules and environment facts

Target: implementation record
(`../p1-w10-qemu-boot-regression-record.md`, created in this step).

Work: select the `stable` token (parent README decision 2) for uniqueness
against all marker classes and typical QEMU noise; fix the fatal/panic class
tokens from the W07 contract; fix the expected runner exit set from the P0-W09
contract; record the measured canonical boot time and the derived default
timeout with headroom rationale; record the canonical invocation and image
identity source. All of this precedes any verdict-bearing run.

Suggested observation: read-only inspection plus one exploratory boot through
the runner (recorded as exploration, not as P1-V16 evidence).

**Acceptance:** the record names the token, class tokens, exit set, timeout
value and rationale, and invocation; all are fixed before R1.  
**Failure/blocker:** a missing prerequisite contract blocks per §1; no
substitute matching rule.

### Step 2 — implement the driver

Target: the tracked tooling entry `p1-boot-regression` (form per parent
README decision 3).

Work: implement the entry contract
([01-automation-contract.md](01-automation-contract.md) §2) and the
classification of §4 as a pure function; implement the evidence layout of §6;
no QEMU details beyond the runner entry.

Suggested observation: where the P0 host-test baseline permits, host-side
unit tests of `classify` and the matcher against recorded captures; otherwise
inspection against the contract.

**Acceptance:** the driver runs one canonical boot end-to-end through the
runner; classification and evidence match the contract clause-for-clause;
output parsing has bounded caps and no shell interpretation.  
**Failure/blocker:** a runner-contract gap (missing capture, missing exit
status) is an upstream defect; record it.

### Step 3 — run the scenario matrix

Target: verification record
(`../../verification/p1-w10-qemu-boot-regression-verification.md`).

Work: execute R1–R5 of
[02-scenarios-and-verdict.md](02-scenarios-and-verdict.md) in order; perform
the R6 static review. Record every outcome with status, command, environment,
and evidence paths. W11 scenario images may be substituted by recorded
techniques until W11 lands, but R2 must eventually be re-anchored on the real
W11 panic scenario.

**Acceptance:** R1 `PASS`; R2–R5 each produce their mandated failure/error
outcome (they prove detection by failing correctly); R6 review passes.  
**Failure/blocker:** any scenario that cannot produce its mandated outcome is
recorded as failed with diagnosis — a harness defect or an upstream defect;
neither is waived.

### Step 4 — run the 100-cycle procedure

Target: verification record, repetition section.

Work: execute §3 of
[02-scenarios-and-verdict.md](02-scenarios-and-verdict.md) exactly: fresh
evidence directory, `--cycles 100`, stop-on-failure, no selective retention.

**Acceptance:** 100/100 `PASS` with the same stable token, or a stopped run
recorded as the failure evidence it is. Either way the record states the
P1-V17 status truthfully.  
**Failure/blocker:** a failed cycle is diagnosed (boot-path defect goes to the
owning package; environment flakiness is recorded as an environment
limitation); P1-V17 remains unmet until a clean run exists.

### Step 5 — closure and handoff

Work: confirm the handoff checklist (§5); record changed files, the frozen
marker rules, and the downstream inputs for W11 (execution conventions), W12
(reference environment facts, evidence locations), and P2-W09 (precedent).
Completion is claimed only in the verification record, only for what ran.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W10-DV01 → P1-V16 | Verdict-independence review | read the driver against [01-automation-contract.md](01-automation-contract.md) §§3–5 | verdict is a pure function of capture/exit/timeout; no human judgment input; no order-only predicate | the verdict mechanism is objective; not that a given boot is good |
| W10-DV02 → P1-V16 | Marker-rule review | check token uniqueness against W06/W07/W09 vocabularies and QEMU noise | tokens fixed pre-run, unique, and recorded | rules are objective and stable; not future output compatibility |
| W10-DV03 → P1-V16 | Outcome-classification review/execution | R3, R5, plus inspection of §4 coverage | every failure mode maps to exactly one outcome; timeout bounded; abnormal exits classified | bounded, classified behavior; not runner internals |
| W10-DV04 → P1-V16 | Evidence-preservation execution | R1 plus a deliberately failed run (R2 or R4) | evidence exists for every attempted cycle; failures keep full captures; verification document references raw evidence | preserved failure evidence; not long-term artifact archival policy |
| W10-DV05 → P1-V16 | Independent-verdict execution (R1) | canonical boot via the driver, fresh environment | `PASS` with complete evidence; driver exit code semantics hold | the canonical boot passes under the fixed rules; not hardware, not unexercised paths |
| W10-DV06 → P1-V16 | Order-independence execution/review (R6) | permuted and noise-varied captures through `classify` | verdict invariant under permutation and benign variation | repeatability property; not token uniqueness against unknown future output |
| W10-DV07 → P1-V17 | 100-cycle procedure review | inspect §3 steps and the recorded procedure | strict 100/100 rule, stop-on-failure, no selective retention, evidence set complete | the procedure is sound; not that it has been run |
| W10-DV08 → P1-V17 / W10 closure | 100-cycle execution and consumability review | run §3; then read the evidence as W11 (conventions usable?), W12 (environment facts and locations usable?), P2-W09 (precedent clear?) | run outcome recorded truthfully; consumers can act without inventing policy | repetition evidence (only from the run); not hardware correctness, host-load robustness, or QEMU-version portability |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. R1/R6 plus the
harness-control scenarios satisfy P1-V16; only the 100-cycle run satisfies
P1-V17. No validation here proves P1-V01–P1-V15 or P1-V18–P1-V21.

## 4. Error, security, and observability model

**Errors.** All failure is classified (§4 of the automation contract), never
silent: a missing capture, an unclassifiable exit, or a driver crash is an
ERROR record with partial evidence. The driver never edits evidence after the
fact; corrections are appended records.

**Security.** The serial capture is untrusted input: bounded line length,
bounded total size, no shell interpretation, no paths derived from output, no
regular expressions sourced from output. The driver introduces no
authorization surface and no network access beyond what the P0 runner
already performs. QEMU runs the reference image only; no guest of any kind is
in scope.

**Observability.** Observability is the evidence set itself: per-cycle
captures, outcomes, environment metadata, and the summary. The verification
document is the review surface; raw artifacts are referenced by path and
identity. No telemetry, metrics, or performance statistics are produced —
those are Reserved.

## 5. Handoff checklist

Before handing W10 to a reviewer, provide:

- the exact changed-file list (runner, driver and any tracked tooling
  wiring; nothing else);
- the frozen marker rules: stable token, class tokens, exit set, timeout
  value with rationale, canonical invocation;
- R1–R6 and 100-cycle evidence paths with run status, including explicit
  not-run entries (e.g., a repetition run not yet executed);
- confirmation that no second QEMU invocation path, no CI workflow, no guest
  artifact, no performance metric, and no boot-path change was introduced;
- confirmation that the P0-W09 runner contract was consumed, not modified,
  and that any discovered contract gap is recorded with its owning package;
- open items: W11 re-anchoring of R2 on the real panic scenario; W12
  reference-environment and evidence-map inputs; P2-W09 precedent note —
  recorded, not resolved here.
