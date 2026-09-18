# P3-W12 Workflow, Validation, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W12 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any work, the implementer verifies it has loaded the parent README
and the supporting files its step needs, and inspects the current tree
(`git ls-files`; which P3 implementation and verification records exist).
Stop and record instead of improvising when:

- a mechanism surface a scenario names does not exist or differs from its
  design — the scenario is **blocked** on that contract; never reach into
  internals;
- the P0 host-side/QEMU automation baselines are absent — QEMU-executed
  scenarios are **blocked**; W12 implements no runner or CI file;
- a scenario appears to require modifying a mechanism, adding a lock, or
  allocating to inject or observe — design conflict; raise it;
- a declared limit proves impractical on first runs — limits change
  through a recorded decision in the implementation record with a new
  rationale, never silently mid-campaign.

## 2. Ordered workflow

### Step 1 — verify scenario/mechanism surface fit

Target: the scenario matrix S1–S7.

Work: for each scenario, check the named published surfaces exist in the
current P3 implementation (or record the scenario blocked). Confirm
injection register rows against the actual seams.

**Acceptance:** per-scenario fit status recorded; no scenario left
implicitly assumed.  
**Failure/blocker:** blocked scenarios keep the campaign scoped to what
exists; the block is cited in every affected result.

### Step 2 — fix the declared constants

Target: implementation record (created in this step).

Work: fix K, N, R, rounds, seed set, and session order with rationale per
[03](03-failure-injection-and-accounting.md) §4–§5.

**Acceptance:** every scenario has complete declared limits before any
run.  
**Failure/blocker:** a constant without rationale fails review.

### Step 3 — implement host-side scenario variants

Target: host-side tests per the P0-W08 baseline.

Work: S3 (fakes), S5a (fake mailbox), S6 pure-targeting logic, and any
S1/S2 logic that separates from hardware; no mechanism modification.

**Acceptance:** host variants run and record `run-passed`/`run-failed`
honestly with the evidence template of
[03](03-failure-injection-and-accounting.md) §6.  
**Failure/blocker:** a failing host test is a real result — recorded, not
suppressed; a missing fake seam is **blocked**, not improvised.

### Step 4 — implement declared stress entry points

Target: test-support stimulus code reachable under the P0-W09 entry path
(QEMU scenarios).

Work: minimal, declared stimulus routines exercising published surfaces
only (S1/S2/S3/S6/S7 schedules; S4 is the boot itself; S5b is the
induced input). Add no mechanism, lock, allocation path, or timer.

**Acceptance:** each entry point is declared in the implementation record
with its scenario mapping; Coding-Guidelines review clean.  
**Failure/blocker:** a needed but undeclared surface is a design gap —
record and stop for that scenario.

### Step 5 — boundary review before first run

Target: the whole campaign contract.

Work: review that every scenario asserts Host SMP only (no guest/Stage-2/
GIC claims), pass conditions use no timing, the S-scenario → W13-row
mapping is consistent with
[P3-W13](../p3-w13-qemu-smp-regression/README.md)'s matrix design, and
evidence destinations match the stage layout.

**Acceptance:** W12-DV04 and W12-DV05 review evidence recorded.  
**Failure/blocker:** a boundary violation is fixed in the scenario, never
hidden in the harness.

### Step 6 — execute the campaign and record evidence

Target: verification record
`docs/stages/p3/verification/p3-w12-smp-stress-failure-tests-verification.md`
(created when evidence exists).

Work: run each unblocked scenario in the fixed session order; capture
observables; classify every run per
[03](03-failure-injection-and-accounting.md) §6; leave nothing implicit.

**Acceptance:** every scenario has an honest status; failures carry
divergence analysis; blocked items cite their blocker.  
**Failure/blocker:** a hang within declared bounds is a `run-failed`
result (bounded-abort), diagnosed via the captured state — the campaign
never "waits it out" or restarts silently.

### Step 7 — closure review

Work: run the validation matrix below and the handoff checklist; confirm
traceability (every plan scope item → scenario → status) is complete.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W12-DV01 → P3-V12 | scenario-contract review | inspect S1–S7 against the plan scope list and mechanism designs | every plan scope item has exactly one scenario; every scenario names owners, stimulus, observable, pass condition, prove/does-not-prove, repetition, environment | the contract is complete and bounded; not that mechanisms work |
| W12-DV02 → P3-V12 | limits and determinism review | inspect declared constants and seed rules against §4–§5 rules | every loop bounded; every constant has rationale; no timing-based criterion anywhere; seed rules stated | repeatability is well-defined; not that runs are stable |
| W12-DV03 → P3-V12 | accounting design review | inspect identities and sentinels against the W11/P2-W04/W04 surfaces | every scenario's pass condition is an evaluable identity over declared surfaces; quiescence obligations stated where needed | assessability by construction; not the results |
| W12-DV04 → W12 closure | Host-SMP-only boundary review | grep/inspect scenarios and harness for guest/vCPU/Stage-2/GIC assertions | none present | stage boundary kept |
| W12-DV05 → W12 closure | mapping-to-matrix review | cross-check scenario-to-row mapping with the W13 matrix design | each W13 stress/failure row cites S-scenarios; no contradiction or duplication | integration coherence |
| W12-DV06 → P3-V12 | campaign execution evidence | steps 3/6 records | per-scenario honest statuses with evidence entries per §6; P3-V12 satisfied only where `run-passed` within declared limits | the exercised limits only; never performance, hardware, or exhaustion of interleavings |

Record each validation as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason.
This design phase can honestly complete only DV01–DV05; DV06 waits for
the implementation.

## 4. Error, security, and observability model

- **Failure handling in the harness.** A scenario detects failure only
  through declared observables; the harness never mutates hypervisor
  state to "recover" a failed run. A bounded hang aborts the session with
  captured state (dump, counters, phase) for diagnosis.
- **Security boundary.** Stimulus code runs in-hypervisor test-support
  context at P3 (no guest exists yet); it exercises only published
  surfaces, adds no guest-reachable interface, and carries no platform
  names. Guest-abuse scenarios belong to P4+.
- **Observability.** The campaign is itself a W11 consumer; its evidence
  doubles as P3-V11 material (attribution under load, S7). Conversely,
  W11's seam gaps degrade specific checks — cited per evidence entry,
  never papered over.

## 5. Handoff checklist

Before handing W12 to a reviewer, provide:

- the exact changed/created file list (design + record + verification
  paths; test code if the phase reached it);
- per-scenario status table with evidence links and environment blocks;
- the declared-constants table with rationale and revisit triggers;
- the injection register with exercisable/not-exercisable rows and cited
  blockers;
- confirmation that no CI file, runner, guest asset, mechanism change,
  timing criterion, or new `unsafe` beyond pre-approved seams was added;
- open items for W13 (matrix rows, repetition depths), W15
  (traceability), and W14 (limits and proof boundaries for P4).

## 6. Future record paths

Implementation record:
`../p3-w12-smp-stress-failure-tests-record.md` (created only when
implementation begins). Verification record:
`../../verification/p3-w12-smp-stress-failure-tests-verification.md`
(created only when evidence exists). Neither exists today; this design
claims no results.
