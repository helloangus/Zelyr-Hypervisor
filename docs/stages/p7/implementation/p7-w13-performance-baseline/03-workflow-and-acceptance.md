# P7-W13 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W13 detailed design](README.md).

## 1. Preconditions and failure boundary

Before executing any step, the implementer verifies it has loaded the
documents named in the parent README and confirms, through the P7-W01
recorded input boundary:

- the P7-W09 observability contract is evidenced and its trace/accounting
  surface covers the event pairs of [the method](01-measurement-method.md) §2;
- the P6 timer/timestamp contract identifies the scheduler timestamp source
  W09 consumes;
- the P7-W03 static-pinned configuration contract is evidenced for the basis
  runs;
- a schedulable P7 implementation exists in the environment to be measured,
  with the W10 workload assets available for payloads.

Stop and record a blocker instead of improvising when:

- a metric's event pair is not derivable from the evidenced W09 surface —
  that metric is blocked with the missing observable named; W13 does not add
  trace events or instrumentation;
- the timestamp source is undeclared or ambiguous — measurement is blocked;
  guessing a clock domain would invalidate every number;
- payload or basis configuration is missing — the affected scenario is
  blocked;
- producing numbers appears to require hypervisor source changes, new
  dependencies, or tuning runs — a scope violation; stop and record;
- a floor, metric definition, or scenario row would need to change — follow
  the design-change path of the parent README, not a local shortcut.

## 2. Ordered implementation steps

### Step 1 — bind observables and the clock domain

Target: implementation record
(`docs/stages/p7/implementation/p7-w13-performance-baseline-record.md`,
created in this step).

Work: bind each metric M-01–M-05 to the concrete W09 trace events/fields and
counters, record the declared scheduler timestamp source from the P6/W09
contracts, and state the extraction method (how captured evidence becomes
per-repetition samples). Mark M-03 direct or derived per
[the method](01-measurement-method.md) §2.

**Acceptance:** a complete binding table exists; the clock domain is named;
no metric is left unbound or silently dropped.  
**Failure/blocker:** a missing observable blocks its metric; an undeclared
timestamp source blocks the package.

### Step 2 — verify scenario runnability and fix parameters

Target: implementation record.

Work: for each scenario SC-01–SC-05 and its basis pairing, confirm payloads,
topology, and placement are realizable; fix concrete repetition counts and
window lengths at or above the floors; prepare the environment-declaration
template. Record any blocked scenario with its owner.

**Acceptance:** every scenario has a runnable/blocked status and complete
declared parameters; no floor is violated.  
**Failure/blocker:** a runnable classification that proves wrong during
execution reverts the scenario to blocked with the discovery recorded.

### Step 3 — execute sessions and retain raw data

Target: verification record
(`docs/stages/p7/verification/p7-w13-performance-baseline-verification.md`)
and the raw-data artifact directory.

Work: run each scenario/basis pair per [the comparison conditions](02-scenarios-and-comparison.md)
§4 — one session per pair, interleaved, warm-ups discarded, quiet rule
observed, environment declaration completed, anomalies discarded with
reasons. Retain raw per-repetition data per the retention policy.

**Acceptance:** every retained measurement has a complete environment
declaration, meets the floors, and its raw data is retained; every discard
and anomaly is recorded.  
**Failure/blocker:** a session with an incomplete declaration or an
unrecorded anomaly is invalid evidence and is re-run; infrastructure faults
are recorded, never absorbed into numbers.

### Step 4 — compute summaries, comparison, and the ADR-057 review

Target: verification record.

Work: compute the per-metric summaries and the M-06 comparison per
[the scenarios file](02-scenarios-and-comparison.md) §3–§4 from retained raw
data; write the limitation statements of
[the method](01-measurement-method.md) §4/§6; then perform the required
review that no measurement, summary, or comparison statement selects or
argues for a default scheduler algorithm (ADR-057) and that no number is
stated as a target, threshold, or quality verdict.

**Acceptance:** summaries are recomputable from retained raw data; the
comparison rows meet §4's validity conditions; the review concludes with no
policy selection and records that conclusion.  
**Failure/blocker:** a summary or phrasing that reads as a policy argument
or KPI is rewritten as a factual, conditioned statement — or the row is
dropped; the reviewer records which.

### Step 5 — hand off closeout inputs to W14

Target: implementation record (handoff section).

Work: emit to [P7-W14](../p7-w14-documentation-p8-handoff/README.md): the
baseline record location, the metric list with per-metric limitation
statements, the environment/mode of the runs, blocked metrics or scenarios
with reasons, and the no-KPI/no-gate rules that later consumers (including
P8-W17 through P8-W14) inherit.

**Acceptance:** the handoff lets W14 index the baseline for P7-V30 without
re-deriving anything; it grants consumers nothing beyond the record.  
**Failure/blocker:** a missing item is recorded as an open item with owner.

## 3. Validation matrix

Every row is planned evidence; none asserts a run has happened. Each is
recorded as passed / failed / blocked / not run with command/input,
environment, timestamp, and reason.

| ID | Review or test | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W13-DV01 | Observable and clock binding review | Workflow step 1 against [the method](01-measurement-method.md) §1–§3 | Every metric bound to named W09 observables; timestamp source declared; extraction method stated | The method is grounded in the evidenced contract; not that measurements exist |
| W13-DV02 → P7-V29 | Method-compliant execution review | Workflow steps 2–3; inspect retained raw data and declarations | Floors met; environment declarations complete; raw data retained and recomputable | Comparable, reproducible-in-method measurements exist for the declared scenarios in the declared environment; not hardware behavior, not worst-case latency |
| W13-DV03 | Comparison-validity review | Workflow step 4 against [comparison conditions](02-scenarios-and-comparison.md) §4 | Every reported M-06 row meets all five conditions; invalid comparisons are absent or marked invalid | The static-vs-scheduled delta is factual under equal conditions; not scheduler quality, not algorithm ranking |
| W13-DV04 → P7-V29 | No-KPI / ADR-057 review | Workflow step 4 | No target, threshold, gate, quality verdict, or algorithm selection present in the record | The baseline stays a baseline; not that future consumers will respect the rule (their designs must) |
| W13-DV05 → W14 prerequisite | Closeout-handoff review | Workflow step 5; read as W14 would | Baseline location, limitations, and rules handed off completely | Handoff completeness; not that W14 closure is done |

No validation here satisfies P7-V24–V28 (W11/W12) or P7-V30 (W14), and none
may be reported as doing so.

## 4. Error, security, and observability model

W13 adds no hypervisor error path and touches no security boundary: it
consumes evidence produced under the normal untrusted-Guest rules and grants
the measurement process no authority over the hypervisor. The package's
characteristic failure mode is **method erosion** — dropping a warm-up,
merging clock domains, comparing across modes, quoting a lone median, or
letting a number drift into a target. Each such defect invalidates the
affected evidence, is recorded as failed/not-run-for-evidence with the defect
named, and is never repaired by re-narrating the number.

Observability is the deliverable in a second sense: the baseline quantifies
the scheduler's observable cost, and the record's limitations must state that
QEMU timing reflects the declared environment only and that unmeasured paths
remain unmeasured. Documentation-grade failures — an incomplete declaration,
an unrecorded discard, a summary not recomputable from raw data — fail the
associated review and are recorded as such.

## 5. Handoff checklist

Before handing W13 results to a reviewer, provide:

- the implementation record with the binding table, clock-domain statement,
  extraction method, and per-scenario parameters;
- the verification record with raw-data references, per-metric summaries,
  comparison rows, and run/not-run entries, including blocked items;
- the limitation statements and the concluded ADR-057/no-KPI review;
- confirmation that no hypervisor source, dependency, instrumentation, or
  Guest-code change was made by W13;
- the W14 handoff (step 5) — without resolving W14's closure contract here.
