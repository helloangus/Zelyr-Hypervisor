# P6-W11 Harness Contract and Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W11 detailed design](README.md).  
**Companions:** [01-scenario-matrix.md](01-scenario-matrix.md) (scenario
authority; taxonomy §3).

## 1. Preconditions and failure boundary

Before any step, load the parent README's document set and inspect the tree
(`git ls-files`; `guests/validation-aarch64/` holds only a `.gitkeep` — no
Guest asset exists yet). All prerequisites are assumed contracts from plans
and handoffs
([P4-W05](../../../p4/plans/p4-w05-validation-guest.md),
[P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md),
[P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md)).

Stop and record a blocker (never substitute or simulate a prerequisite) when:

- the P4 Validation Guest asset or its debug/result channel delivers a
  contract different from what the scenario records assume — record the
  deviate verdict; the record schema adapts to that contract, the
  observation-primacy rule does not;
- a Host-side contract (W04/W06/W07/W08/W09/W10) required by a scenario is
  absent or unevidenced — the scenario runs in BLOCKED mode or is NOT-RUN,
  per [01-scenario-matrix.md](01-scenario-matrix.md) §3;
- exercising a scenario appears to require a machine ABI, DTB artifact,
  Linux Guest, or a Host mechanism change — out of scope; stop;
- the multi-vCPU prerequisite is unevidenced — VG-SMP-01/02 must run BLOCKED,
  never skipped silently (P6-V18/P6-V19 require the block to be recorded).

## 2. Harness contract

The harness is the declared, repeatable way the suite executes; it is
suite-internal, has no public interface, and binds to the P4 Guest asset's
module contract in the implementation record.

```text
Name and stability: P6 interrupt-suite harness (logical unit; internal).
Purpose and caller: execute the declared scenario set in a QEMU
  environment, collect Guest result records and Host correlation, and
  classify outcomes; invoked by the implementing agent and later by W13
  composition.
Inputs / outputs: declared scenario selection and repetition counts ->
  per-scenario result records (01-scenario-matrix §3) and a run summary,
  plus raw artifacts stored per the P0-W09 QEMU evidence conventions
  (linked, not embedded, in the verification record).
Preconditions: environment declared (QEMU configuration class, pCPU count,
  build identifiers); W11 scenario code integrated with the P4 Guest asset;
  prerequisites of the selected scenarios confirmed.
Postconditions: no Host state change persists beyond the run; every
  declared scenario ends with exactly one classified result (PASS, FAIL,
  BLOCKED, or NOT-RUN); the summary references every raw artifact.
State and ownership change: none outside the run's own artifacts.
Concurrency/allocation context: host-side tooling; no guest impact beyond
  the declared triggers; single run at a time per environment instance.
Errors and failure guarantee: harness malfunction (Guest record absent,
  environment failure) is recorded as NOT-RUN/BLOCKED with diagnosis — a
  missing Guest record can never be promoted to PASS by Host logs
  (observation primacy).
Security/authorization checks: scenario triggers use only the
  P5-authorized control path; the harness runs no fault-injection case
  (W12's matrix) and adds no Guest-callable interface.
Logic (outline): declare environment -> confirm selected prerequisites ->
  for each selected scenario in declared order: run repetitions, collect
  Guest records, correlate Host telemetry, classify -> emit summary and
  artifacts.
Validation: W11-DV01–DV07 below.
```

Declared execution order: VG-TIMER-01 → VG-TIMER-02 → VG-TIMER-03 →
VG-TIMER-04 → VG-IRQ-01 → VG-IRQ-02 → VG-IRQ-03 → VG-IRQ-04 → VG-IRQ-05 →
VG-IRQ-06 → VG-SGI-01 → VG-SMP-01 → VG-SMP-02 (conditional rows last so
their BLOCKED mode never masks earlier evidence). Repetition counts and
timeouts are declared at run time from the §2 minimums and recorded in
evidence; they are suite policy, not contract.

## 3. Ordered implementation steps

### Step 1 — prerequisite reconciliation

Target: implementation record
(`../p6-w11-validation-guest-interrupt-suite-record.md`, created in this
step).

Work: read the P4-W05 Validation Guest contract and its evidence; verdicts
for: Guest asset existence/shape, result-channel form, exception-vector
contract, and the multi-vCPU prerequisite (P3-W14 evidence). Read the
W04/W06/W07/W08/W09/W10 designs; verdict per scenario dependency.

**Acceptance:** every scenario in
[01-scenario-matrix.md](01-scenario-matrix.md) §2 has a confirmed, deviant,
absent, or blocked prerequisite verdict with evidence locations.  
**Failure/blocker:** deviate/absent verdicts route affected scenarios to
BLOCKED/NOT-RUN mode or, where the Guest asset contract itself deviates, stop
the suite per §1.

### Step 2 — Guest scenario implementation

Target: the P4 Validation Guest asset's scenario surface (logical scenario
descriptor and result records per the matrix).

Work: implement the scenario set as Guest-side observability only: EL1
vector handlers for the exercised classes, per-scenario trigger sequences,
observation, and structured result records. No Host mechanism is implemented
here; Host cooperation uses only the declared test/control paths. The
temporary layout of [§1](01-scenario-matrix.md) is used symbolically, resolved
by the test configuration.

**Acceptance:** each scenario's Guest code maps row-by-row to its §2 entry;
no vGIC MMIO access, no machine-ABI artifact, no production driver.  
**Failure/blocker:** a needed Guest capability outside the P4 contract stops
the step — record; do not extend the Guest asset beyond its contract.

### Step 3 — Host-side integration points

Target: declared test/control hooks in W04/W07-owned paths (consumed, not
modified).

Work: confirm the injection and SGI-trigger controls exist per their owning
designs; if a control is missing, stop and record — adding one is the owning
package's change, coordinated through its design, not a W11 edit.

**Acceptance:** every Trigger column of §2 has a named, existing control.  
**Failure/blocker:** missing control → affected scenario BLOCKED; record the
gap for W13's unresolved-items list.

### Step 4 — harness and dry run

Target: the harness unit (§2).

Work: implement the harness per its contract; dry-run a minimal selection
(NOT-RUN-safe) to validate collection and classification mechanics without
claiming scenario evidence.

**Acceptance:** a dry run produces well-formed (classified) records with a
summary; no unclassified outcome is possible.  
**Failure/blocker:** classification ambiguity is a design gap — stop, amend,
re-review.

### Step 5 — exclusion and primacy review

Target: review findings in the implementation record.

Work: review the suite against plan step 5: Guest-visible behavior only; no
P8 Linux machine contract defined or implied (check §1 layout non-freeze,
§4 statement); Host logs never substitute for Guest records (spot-check the
classification code); W12 fault cases not absorbed into W11 scenarios.

**Acceptance:** each review point has a recorded verdict.  
**Failure/blocker:** a violation is a scope conflict; stop and amend the
suite, not the review.

### Step 6 — runs, evidence, and handoff

Target: `../../verification/p6-w11-validation-guest-interrupt-suite-verification.md`
(created when evidence exists).

Work: execute the declared matrix; record per-scenario status with
repetitions, environment, raw artifact links, and Host correlation; record
blocked rows with the named missing prerequisite; complete the handoff
checklist (§5). P6-V09–P6-V19 closure wording carries the plan's proof
boundary: passing proves only the declared Validation Guest behavior in the
stated environment.

## 4. Validation matrix

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W11-DV01 → P6-V09–V19 | suite coverage review | audit §2 rows against the plan's declared scenario set and the task-book validation matrix | every declared scenario present with a complete row; conditional rows carry their BLOCKED rule | the suite's designed coverage; not that any run happened |
| W11-DV02 | timer rows (VG-TIMER-01–04) | declared runs | per-row Pass conditions | Guest-observable timer behavior in QEMU; not real-hardware timer accuracy |
| W11-DV03 | vIRQ rows (VG-IRQ-01–06) | declared runs | per-row Pass conditions | Guest-observable vIRQ/masking/pressure behavior in QEMU; not Linux vGIC behavior |
| W11-DV04 | SGI row (VG-SGI-01) | declared run | row Pass condition | the SGI→injection chain's Guest-visible tail; not W04's P6-V04–V06 attribution evidence |
| W11-DV05 → P6-V18/V19 | conditional multi-vCPU rows | run when evidenced; otherwise BLOCKED record | PASS with zero cross-vCPU observation, or a named stage block | isolation in the declared configuration, or the block itself; nothing in between |
| W11-DV06 | observation primacy drill | repeat a run with Guest records suppressed (test mode) | outcome classifies FAIL/NOT-RUN, never PASS | primacy enforcement in the harness; not a scenario semantic |
| W11-DV07 | repetition stability | re-run the declared matrix; compare classifications | classifications reproduce; flaky rows are investigated, not averaged | repeatability within the environment; not determinism of hardware timing |

## 5. Handoff checklist

Before handing W11 to a reviewer, provide:

- the exact changed-file list (Guest scenario code, harness, test
  configuration) and the binding to the P4 Guest asset's module contract;
- step 1 prerequisite verdicts, especially the multi-vCPU evidence verdict;
- W11-DV01–DV07 status (passed/failed/blocked/not-run) with artifact links
  and explicit BLOCKED rows naming their missing prerequisites;
- confirmation that no Host interrupt mechanism, machine ABI, DTB artifact,
  Linux test, vGIC MMIO access, or fault-injection case was added;
- the scenario-status table for W13's P6-DOC-04 composition (P6-V09–P6-V19
  rows);
- open items for W12 (negative-machinery reuse), W13 (latency timestamps,
  asset reuse statement, blocked rows carried into unresolved items), and
  P7/P8 regression users (asset reuse under their own designs) — without
  resolving their contracts here;
- any recorded conflict or investigation with its owner.
