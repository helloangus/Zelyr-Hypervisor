# P8-W16 Harness Contract, Repeated-Boot Checks, and Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W16 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any implementation, the implementer verifies it has loaded the
documents named in the parent README and inspects the tracked tree. Read-only
discovery: `git ls-files` (confirm no harness, fixture, or CI file exists) and
a search of tracked P8 documents for any prior regression statement. The
following are **assumed contracts** with explicit failure boundaries; if any
delivers differently, the affected work stops and records a blocked
prerequisite — it is never repaired locally:

- **P0-W09 single QEMU runner entry** (`p0-w09-qemu-automation-entry-baseline`)
  with its reserved parameter-carrying mechanism: W16 extends it, never forks
  a second invocation path. *Failure boundary:* no parameter mechanism ⇒ W16
  records a blocked prerequisite against P0-W09; extending the P0 contract is
  a change to that package's design, not a W16 local choice.
- **P0-W07 quality-gate and failure-semantics governance:** W16's failure
  classification must not contradict the P0 gate failure policy.
- **P0-W08 host-side testing baseline:** host-testable parts of the harness
  (marker matching, run-record assembly) follow the P0 host-test organization.
- **W15 fixture** delivering a versioned or reproducibly generated Linux asset
  (and the VG asset via the P4 chain): rows without a fixture reference are
  blocked, not approximated.
- **W09–W14 outcome vocabularies:** markers inherit owner packages' wording;
  W16 must not paraphrase Guest-visible semantics into new text.
- **W02-governed approved machine values:** RAM-class and topology rows need
  approved values (task book §8 `ADR Required` item). *Failure boundary:* the
  affected rows stay blocked until the approved contract exists; no W16 row
  may hardcode a candidate value.

## 2. Ordered implementation workflow

Execute in order; each step's evidence goes to the destinations in §4.

### Step 1 — resolve row executability

Target: implementation record (`../p8-w16-automated-linux-regression-record.md`,
created in this step).

Work: verify each §1 assumed contract; resolve fixture references, owner-package
marker vocabularies, and approved machine values per row; mark every matrix row
executable or blocked, citing the missing contract for blocked rows.

**Acceptance:** a per-row executability statement with owner attribution.  
**Failure/blocker:** a blocked row stays blocked with its blocker recorded; no
substitute fixture, vocabulary, or value.

### Step 2 — bind the harness to the P0-W09 entry

Target: the harness adapter implementing §3's behavioral contract.

Work: implement the §3 adapter that invokes the P0-W09 single entry through
its reserved parameter mechanism (row ID, fixture reference, repetition
parameters, evidence directory); implement marker matching, run-record
assembly, and evidence retention as host-testable logic under P0-W08
organization. Concrete command spelling and serialization are implementation
choices recorded in the implementation record.

**Acceptance:** one invocation executes one row repetition and emits one run
record with a complete §4 field set; no second QEMU entry exists.  
**Failure/blocker:** a missing P0-W09 parameter mechanism is a recorded
blocked prerequisite against P0-W09; forking a runner is prohibited.

### Step 3 — execute the declared matrix

Target: verification record and raw evidence (§4).

Work: run the executable LG/COMPAT/FAULT rows under the declared repetition,
seed, and timeout parameters; retain transcripts and run records.

**Acceptance:** every executed row has a run record and transcript; every
blocked row remains listed with its reason.  
**Failure/blocker:** a harness-infra failure is recorded not-run for the
affected row, never scored as pass or fail.

### Step 4 — execute the REP family

Target: verification record and raw evidence (§4).

Work: run the repeated-boot family in one harness session with the declared
repetition count; evaluate the §5 comparison checks across repetitions.

**Acceptance:** all repetitions produce run records; comparison results are
recorded per check.  
**Failure/blocker:** a single-repetition REP run is invalid for P8-V22 and is
recorded as not run for closure purposes.

### Step 5 — classify every verdict

Target: verification record.

Work: classify each FAIL/BLOCKED verdict per §6; attach transcripts and
owning-package attribution; escalate boundary violations per §6's rules.

**Acceptance:** no unclassified verdict remains.  
**Failure/blocker:** an unclassifiable failure is itself recorded as an open
block pending owner-package diagnosis.

### Step 6 — closure review

Work: run the validation matrix (§7), confirm the handoff checklist (§9),
and verify the package against its task-book requirement, prerequisite
compatibility with delivered upstream contracts, document links, and
downstream handoff wording. Completion is claimed only in the verification
record, with evidence, and only for what was actually run.

## 3. Harness behavioral contract

The harness is a shell/automation *behavioral contract*, not a script design.
One invocation executes exactly one scenario-row repetition (the REP family is
a declared sequence of such invocations inside one harness session), through
the P0-W09 single entry.

```text
Name and stability: run-one-row (temporary automation contract, not a public API)
Purpose and caller: execute one scenario row repetition and emit one run record;
    called by a human developer or a later CI consumer of the P0 CI baseline
Inputs: scenario row ID; fixture reference (W15 version or generation recipe
    digest); repetition parameters (count, per-repetition timeout class value);
    evidence directory; machine/build identity inputs
Outputs: exit status (PASS | FAIL(oracle) | BLOCKED(prereq)); one run record;
    captured serial transcript; fault-class ledger extract
Preconditions: fixture reference resolves; owning-package marker vocabulary
    is evidenced for the row; approved machine values exist for class rows
Postconditions: evidence directory contains the transcript and run record;
    no state outside the evidence directory and the P0-W09 runner's declared
    artifacts is mutated
State and ownership change: none in the repository under test; the harness owns
    its evidence directory contents
Errors and failure guarantee: every failure mode emits a run record with a
    FAIL/BLOCKED status and a reason class; the harness must not silently
    drop or overwrite a previous run record
Security/authorization checks: none (no privilege boundary is crossed); the
    harness must not weaken the P0-W09 runner's declared timeout/exit capture
Logic outline:
    resolve row -> resolve fixture -> start capture -> invoke runner entry
    -> stream transcript through the marker matcher (ordered, EXACT/REGEXP,
       ONCE/REPEAT(n)) -> observe terminal outcome -> extract fault-class
       ledger -> write run record + transcript -> exit with the row verdict
```

Non-responsibilities: the harness does not interpret durations, does not retry
failures, does not edit transcripts, does not filter diagnostics, and does not
encode machine values. Its only judgment is the declared oracle.

## 4. Run record and evidence destinations

One run record per row repetition. Concrete serialization (TOML/JSON/etc.) is
an implementation choice recorded in the implementation record; the field
contract is fixed here:

```text
row_id            declared row of 01-regression-matrix.md §4
repetition_index  1-based; total declared repetitions for the row
fixture_ref       W15 fixture version or reproducible-generation digest
machine_ref       approved machine-contract version/identity (W02-governed)
build_identity    hypervisor build/version identity per P0 version governance
guest_track       LG | VG
vcpu_count / ram_class / fault_injection  as declared by the row
seed_discipline   none | fixed:<seed>
timeout_class / timeout_value  as declared and recorded
verdict           PASS | FAIL:<oracle> | BLOCKED:<prereq>
marker_result     matched/expected counts; first mismatch position on FAIL
fault_ledger      fault classes observed, mapped to the W13 taxonomy
environment       host CPU model/cores, QEMU version, acceleration mode,
                  host OS/kernel, load policy — the W17 environment contract
                  consumes these fields verbatim
transcript_ref    relative path of the retained serial transcript
timestamps        harness start/end times (informational, never judged)
```

Evidence destinations (created when evidence exists; neither exists today):

- Verification record: `../../verification/p8-w16-automated-linux-regression-verification.md` —
  run/not-run status per validation row of §7, with command, environment,
  timestamp, and reason.
- Raw evidence: `../../verification/assets/p8-w16/<run-id>/` — transcripts and
  run records, append-only within a review cycle.
- Implementation record: `../p8-w16-automated-linux-regression-record.md` —
  factual decisions (serialization choice, timeout values used, deviations).

## 5. Repeated-boot checks (P8-V22)

The REP family executes consecutive boot-row repetitions inside one harness
session and compares them. The checks detect *symptoms* of the defect classes
the task book names; they cannot prove those classes absent everywhere.

| Check | Input | Observable | Failure condition |
|---|---|---|---|
| REP-SEQ equality | all repetition transcripts | per-repetition marker sequence, order-sensitive | any repetition's sequence differs from the first repetition's, beyond row-declared unordered classes |
| REP-FAULT delta | all fault ledgers | set of fault classes per repetition | a fault class appears in a later repetition that was absent in the first (candidate stale VMID/TLB/vCPU/IRQ or uninitialized-state symptom) |
| REP-FIRSTONLY symptom | all fault ledgers + W13 diagnostic context | diagnostic classes unique to boot #1 vs later boots | a first-boot-only symptom recurs on a later boot, or a later boot shows a class the W13 ledger marks as lifecycle-dependent |
| REP-SHUTDOWN uniformity | all terminal outcomes | terminal outcome per repetition | any repetition ends other than CLEAN_SHUTDOWN |
| REP-RACE window | SMP REP rows (REP-B4) | W10 anomaly classes during boot and workload overlap | any declared lost-event/state-race/corruption class observed in any repetition |

Cross-run comparison is evaluated by the harness after all repetitions
complete; a single failing repetition fails the whole REP row. Repetitions
never wrap a retry: a failed repetition is evidence, not an inducement to
re-run until green (see §6).

## 6. Failure classification workflow

Every FAIL or BLOCKED verdict is reviewed into exactly one class before the
next scheduled run:

```text
reproducible evidence   the failure reproduces under the declared row and
                        environment: record row, oracle, transcript reference,
                        minimal reproduction, and owning-package attribution
                        (mechanism owner, not W16, owns the fix)
blocked prerequisite    the row could not be honestly executed (fixture,
                        vocabulary, or approved values missing): record the
                        missing contract and its owner; the row is excluded
                        from P8-V21/V22 closure until unblocked
regression              the same row previously passed in a recorded run and
                        now fails with no relevant contract change: record both
                        run records; escalate to the owning package as a
                        regression with the transcript pair
```

Escalation rules:

- A failure whose diagnosis indicates a Hypervisor invariant violation or a
  Guest-untrusted boundary break is not a W16-local fix; it is escalated to
  the owning package and, if it contradicts an ADR/task-book constraint, is
  labeled `Architecture Change Request` or `ADR Required` in the verification
  record.
- Modifying a row's oracle to make a failing scenario pass is prohibited. An
  oracle change is a design change: it requires a reviewed edit of
  [01](01-regression-matrix.md), a record of the old and new oracle, and a
  fresh full run of the affected family.
- Silent deletion of blocked rows from the matrix is prohibited; blocked rows
  remain visible with their blocker.

## 7. Validation matrix

All rows are planned evidence; none asserts a run has occurred. Record each as
**passed**, **failed**, **blocked**, or **not run** with command, input,
environment, timestamp, and reason in the verification record.

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W16-DV01 → prerequisites | contract and convention review | inspect harness contract against P0-W07/W08/W09 governance and this design §1–§4 | no second QEMU entry; failure semantics compatible with P0 gates; run-record contract complete | governance coherence; not that any runner exists yet |
| W16-DV02 → P8-V21 | marker-oracle review | review [01](01-regression-matrix.md) §2 against owner-package outcome wording | oracle shape complete; no W16-invented vocabulary; fatal-class and no-host-leak exclusions present | determinate-observable design; not that markers exist or match |
| W16-DV03 → P8-V21 | matrix completeness review | check §4 rows against P8-V21 mandated elements and owner packages | every mandated element has a row; every row names content owner, pass condition, proof boundary, repetition/seed/timeout rules | declared matrix; not that any row has executed |
| W16-DV04 → P8-V22 | repeated-boot policy review | review §5 checks against task-book defect classes | all five checks present; single-repetition REP runs invalid; retry-into-pass prohibited | detection design for stale-state symptoms; not that defects are absent |
| W16-DV05 → P8-V22 | matrix execution run | execute the REP family per the declared parameters in the declared environment | all repetitions pass; comparison checks empty; evidence retained per §4 | repeated-boot containment in the declared QEMU environment; not hardware behavior and not absence of all races |
| W16-DV06 → W16 closure | failure-classification review | audit a sample of recorded FAIL/BLOCKED verdicts | every verdict classified per §6; no oracle weakening; no silent row deletion | review discipline; not package completeness |
| W16-DV07 → handoff | consumability review | read §4 fields as W17 (environment), §5–§6 as W18, the VG family as W19, the whole matrix as W20 | each consumer can act without inventing policy | handoff readiness; not that downstream packages are done |

W16-DV05 and DV06 require real evidence and are the only rows that can
satisfy P8-V21/V22; no other validation may be reported as doing so. All
matrix runs remain QEMU/TCG-scoped: success never proves AArch64 hardware
semantics, real-board behavior, or machine-ABI correctness beyond matching
the approved contract.

## 8. Error, security, and observability model

- **Failure model:** every execution path ends in a recorded verdict; the
  harness has no silent path. Timeout, marker mismatch, unexpected fault
  class, and environment mismatch are distinct, named oracles.
- **Security position:** W16 adds no privilege boundary. Its security role is
  negative evidence: the no-host-leak rule of
  [01](01-regression-matrix.md) §2 and the fatal-class exclusion keep Host
  facts and Hypervisor-invariant failures out of "passing" results. Containment
  semantics themselves are W18's package.
- **Observability:** the evidence trail (transcripts, run records, fault
  ledgers) is the only accepted proof surface. Duration fields in run records
  are informational; any performance reading of them must go through
  [W17's](../p8-w17-linux-performance-baseline/README.md) method or it is not
  a baseline observation.

## 9. Handoff checklist

Before handing W16 to a reviewer, provide:

- the exact changed-file/artifact list and where each matrix row's content
  authority lives;
- W16-DV01–DV07 evidence paths and run status, including explicit not-run
  entries for every blocked or deferred row;
- the fixture and machine references used by every executed run;
- confirmation that no CI workflow, second QEMU entry, fixture recipe, or
  hypervisor/Guest source change was introduced under W16 authority;
- confirmation that no timing value was used as a pass condition and no retry
  was used to obtain a pass;
- open items: owner-package vocabularies not yet evidenced (list them),
  RAM-class values pending the approved contract (task book §8 `ADR
  Required`, routed via W02), and the P0-W09 parameter-mechanism dependency —
  without resolving any of them here.
