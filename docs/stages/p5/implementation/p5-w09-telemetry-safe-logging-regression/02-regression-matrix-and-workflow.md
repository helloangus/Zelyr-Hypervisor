# P5-W09 Regression Matrix and Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the actual state:

- W06's category assignment, W07's marker set, and W08's smoke
  configuration exist as implemented and recorded (their packages' records);
- the P4 regression set and automation entry point exist as implemented,
  evidenced artifacts (P4-W08 delivery, reconciled by P4-W09);
- the P0-W12/W13 diagnostics and namespace rules are available in their
  delivered form;
- the QEMU environment from the P4/P0 runner basis is available and its
  declared configuration is known.

Stop and obtain direction instead of guessing when: any prerequisite is
missing or diverges from its plan (blocked prerequisite — record; never
reconstruct P4 cases or markers from plan text alone); composing the
regression would require changing a P4 case, a W07 marker, or a W08 smoke
config (route to the owning package); or wiring the regression appears to
require CI-policy changes, a telemetry backend, or P6+ telemetry (scope
violation — stop).

## 2. Regression set composition

The regression set is the closed composition of three row groups. It is
recorded as a versioned list; adding, amending, or retiring a row follows
the owning package's maintenance rule and is visible in this package's
record.

### 2.1 Inherited P4 rows (content unchanged; P4-W08 ownership)

| Row | Inherited expectation | Source |
|---|---|---|
| R-P4-01 | positive EL1 entry marker | P4-W08 acceptance set |
| R-P4-02 | Stage-2 translation-fault behavior determinate | P4-W08 |
| R-P4-03 | permission-fault behavior determinate | P4-W08 |
| R-P4-04 | defined survival/stop behavior | P4-W08 |
| R-P4-05 | repeated-run stability across the P4 declared iterations | P4-W08 |

These rows run exactly as P4 delivered them. W09 adds collection (verdicts,
environment, artifacts) but no content change; a P4 row that no longer
passes after P5 work is a P5 regression **failure**, not a P4 edit.

### 2.2 P5 scenario rows (from W07's delivered inventory)

| Row group | Expectation | Source |
|---|---|---|
| R-P5-valid | group A scenarios match markers (`Completed` paths) | W07 matrix group A |
| R-P5-negative-structural | group B markers | W07 group B |
| R-P5-handle | group C markers | W07 group C |
| R-P5-authority | groups D and E markers (incl. revoke/re-grant) | W07 groups D, E |
| R-P5-address | group F markers (overflow/unmapped/partial/type) | W07 group F |
| R-P5-lifecycle | group G markers | W07 group G |
| R-P5-isolation | P5VG-100–103 two-context outcomes incl. negative control | W07 [02 §2](../p5-w07-validation-guest-isolation-suite/02-two-context-isolation-and-harness.md) |

Marker content is consumed read-only from W07's delivered set.

### 2.3 Fuzz-smoke row (from W08's delivered smoke config)

| Row | Expectation | Source |
|---|---|---|
| R-P5-fuzz-smoke | W08's fixed smoke seeds/bounds over FZ-01–FZ-05 and PR-01–PR-05 complete with zero oracle hits | W08 [01 §5](../p5-w08-host-fuzz-stress-smp-baseline/01-fuzz-and-property-matrix.md) |

### 2.4 Telemetry correlation rows (W09's own addition)

| Row | Expectation |
|---|---|
| R-P5-tel-01 | for each regression boot, the dispatch-side category totals match the scenario outcomes actually observed (e.g., every denied scenario incremented exactly its category) |
| R-P5-tel-02 | the `invariant` counter is zero in every boot |
| R-P5-tel-03 | all collected artifacts satisfy the redaction table of [01 §4](01-telemetry-and-safe-logging-contract.md) |

## 3. Determinate expectations

Every row has, before any run: a fixed input configuration (which asset
builds, which boot composition, which scenario order), a fixed expected
outcome (marker classes per §2.2; verdicts per §2.1/§2.3; counter
equalities per §2.4), and a declared environment (QEMU version, pCPU
count). Expectations are never renegotiated at run time; a wrong
expectation is amended through the owning package's rule, recorded, and
re-run.

## 4. Non-success classes

| Class | Meaning | Suite effect |
|---|---|---|
| `row-failure` | a row produced a determinate wrong outcome | suite fails; diagnosis recorded |
| `timeout` | a declared wait expired (boot, scenario, or smoke run) | suite fails; recorded with the wait point |
| `incomplete-evidence` | a run ended without producing all expected artifacts (missing marker, missing counter view, truncated capture) | suite fails; recorded |
| `leaked-sensitive-information` | any artifact violates the redaction table | suite fails; security finding; blocks pass claims until resolved |
| `telemetry-mismatch` | R-P5-tel-01/02 equality fails | suite fails; dispatch/telemetry integration defect |
| `environment-unsupported` | declared environment unavailable | suite outcome is `blocked`; never converted to a pass |
| `prerequisite-blocked` | an inherited prerequisite is absent | affected rows `blocked` with reason; rest may run |

## 5. Ordered implementation steps

### Step 1 — reconcile prerequisites and freeze the row list

Target: implementation record
(`../p5-w09-telemetry-safe-logging-regression-record.md`, created in this
step).

Work: confirm the §1 prerequisites; transcribe the delivered W07 markers
and W08 smoke config into the §2 rows verbatim; declare the environment;
record the row-list version.

**Acceptance:** every row has fixed inputs, expectations, and environment;
prerequisite confirmations or blocks recorded.  
**Failure/blocker:** a missing prerequisite blocks its rows per §4.

### Step 2 — integrate counters and events

Target: the dispatch path's S8 step and the VM lifetime code (placements
per the delivered W06/VM designs).

Work: attach the `HypercallResultCounters` set to VM lifetime; emit the
namespace-registered event at S8; wire the read-time aggregation view; add
the rate-limited unusual-condition note path per
[01 §3.2](01-telemetry-and-safe-logging-contract.md).

**Acceptance:** a host-side path shows one category + one event per
synthetic call; no allocation, blocking, or formatting in exit context;
counter failure leaves outcomes unchanged (test-injected).  
**Failure/blocker:** any need to alter dispatch semantics to observe it is
a design conflict — stop and record.

### Step 3 — apply redaction to every visible path

Target: all diagnostics paths W09 touches.

Work: apply the [01 §4](01-telemetry-and-safe-logging-contract.md) table to
the new paths and audit the pre-existing paths now reachable in regression
artifacts; document each path's classification (default vs developer-debug).

**Acceptance:** no default path emits forbidden content (code review
checklist recorded); debug-only additions are gated per P0-W12.  
**Failure/blocker:** forbidden content reachable by default is a security
finding — fix before any suite run.

### Step 4 — wire the regression runner composition

Target: the runner procedure extending the P4-W08 entry point.

Work: compose §2's row groups in the declared order; implement the §4
non-success classes verbatim; implement artifact collection (verdicts,
counter views, captured output) into the verification area.

**Acceptance:** a deliberately broken expectation yields `row-failure`, a
truncated capture yields `incomplete-evidence`, and an unavailable
environment yields `environment-unsupported` — each recorded, none
converted to a pass.  
**Failure/blocker:** any reinterpretation of markers or expectations at
run time is a design violation — fix the runner, never the expectation.

### Step 5 — verify the ABI/security documentation route

Target: review item of W09-DV04 (plan P5-V15's documentation element).

Work: confirm that the factual ABI/security documentation route recorded by
W01's reconciliation points to artifacts that exist where implementation
has reached (authored by W02–W05, published by W10) and that links from
this package's records resolve. W09 verifies the route and links only; it
authors nothing.

**Acceptance:** the route review is recorded; absent not-yet-published
artifacts are listed as pending with owners — never fabricated.  
**Failure/blocker:** a broken or absent route is a blocked item recorded
against W01/W10, not a W09 authoring task.

### Step 6 — run the suite and record evidence

Target: verification record
(`../../verification/p5-w09-telemetry-safe-logging-regression-verification.md`).

Work: execute the §6 matrix rows; record every command, environment value,
verdict, artifact path, and non-success class; keep the inherited P4 rows'
verdicts separate so P4 regression status remains attributable.

**Acceptance:** all executed rows pass with evidence; failures and
non-successes recorded with diagnosis; not-run rows explicit.  
**Failure/blocker:** a failing inherited P4 row is recorded as a P5-caused
regression failure and escalated; it must not be absorbed by editing P4
content.

### Step 7 — records and handoff

Target: implementation record; handoff section of the verification record.

Work: complete the record (changed files, dependencies — expected none new
beyond the P0-governed set; new `unsafe` expected none); deliver the
regression/evidence index, observability-limits record, and redaction-rule
reference to W10; declare the P6+ consumable boundary per the README.

**Acceptance:** handoff artifacts located per the README's downstream
section.  
**Failure/blocker:** a missing artifact blocks closure review, not the
consumer.

## 6. Validation matrix

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. All rows are
declared-environment (QEMU) evidence: they prove determinate behavior of
the delivered mechanisms there; they do not prove a production telemetry
system, architectural or hardware correctness, or any P6+ behavior.

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W09-DV01 → P5-V15 | row-list freeze review | Step 1 | every row fixed; prerequisites confirmed/blocked; environment declared | the regression is well-defined; not that it passes |
| W09-DV02 → P5-V15 | counter integration evidence | Step 2 + synthetic host path | one category per call; exit-context discipline; counter failure harmless | required result categories are observable; not the final telemetry API |
| W09-DV03 → P5-V15 | event/aggregate evidence | Step 2 + regression boot correlation | R-P5-tel-01/02 hold in every run | per-VM accounting works end to end; not production metrics |
| W09-DV04 → P5-V15 | redaction and route review | Step 3 checklist + Step 5 + `leaked-sensitive-information` class clean | no default disclosure; route links resolve or pending-with-owner | safe default diagnostics and a factual documentation route; not document content (W10) |
| W09-DV05 → P5-V16 | inherited P4 rows run | Step 6 | R-P4-01–05 verdicts determinate and unchanged in content | P4 regression preserved; not P4 re-validation beyond its own set |
| W09-DV06 → P5-V16 | P5 scenario rows run | Step 6 | R-P5-* groups match delivered expectations | valid/invalid HVC, reference, authority, revoke, address, overflow, isolation outcomes determinate; not new mechanism correctness (owning packages') |
| W09-DV07 → P5-V16 | fuzz-smoke row run | Step 6 | R-P5-fuzz-smoke: zero oracle hits on the fixed config | the smoke regression is stable; not exhaustive robustness (W08's deep runs) |
| W09-DV08 → P5-V16 | non-success class behavior | Step 4 acceptance injections | each §4 class produced by its trigger and recorded | failures fail loudly; not environment coverage |
| W09-DV09 → W09 closure | handoff and record review | Step 7 checklist | W10/P6-consumer artifacts located; row-list version recorded | handoff readiness; not downstream completion |

## 7. Error, security, and observability model

The suite's error model is the §4 class table — every non-pass is named,
recorded, and suite-failing. Its security property is the redaction table:
safe defaults everywhere, debug-gated exceptions only, unconditional
protection for Guest buffer contents and Host pointers. Its observability
is deliberately self-limiting: counters and one event family under the
P0-W13 namespace, loss-tolerant in exit context, never load-bearing for
correctness. No `unsafe` is expected; any exception is reported per the
Coding Guidelines.

## 8. Handoff checklist

Before handing W09 to a reviewer, provide:

- the exact changed-file list and the diagnostics-path classification
  table (default vs developer-debug) from Step 3;
- evidence paths and run status for W09-DV01–DV09, including explicit
  blocked/not-run entries and every non-success class observed;
- the frozen row-list version with the transcribed W07/W08 inputs and the
  inherited P4 rows' verdicts kept separate;
- the route review result from Step 5 (links resolved or pending with
  owners);
- the observability-limits record (what P5 telemetry provides and does
  not) and the redaction-rule reference for W10's security documentation;
- the declared P6+ consumable boundary (rows, expectations, environment
  constraints, non-success classes) — extension rights only, no weakening;
- open items: blocked prerequisites, pending documentation artifacts, and
  any `Architecture Change Request` / `ADR Required` record — without
  resolving them here.
