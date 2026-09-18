# P8-W17 Baseline Record, Workflow, and Comparison Rules

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W17 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any measurement, the implementer verifies it has loaded the documents
named in the parent README and inspects the tracked tree. Read-only discovery:
`git ls-files` (confirm no baseline record, raw-data asset, or measurement
tool exists under P8). The following are **assumed contracts**; if any
delivers differently, the affected metric or run is recorded as a blocked
prerequisite and never locally substituted:

- **W16 envelope** (`p8-w16-automated-linux-regression`): fixed scenario
  rows, run-record environment/identity fields, harness oracles. *Failure
  boundary:* no executed W16 rows ⇒ W17 records blocked; W17 must not build a
  second execution path.
- **W15 fixture** and **W02-governed approved machine values**: measurement
  requires the same pinned inputs as regression. *Failure boundary:* missing
  references ⇒ blocked, never approximated.
- **P0-W12/W13 telemetry governance** and the owning stages' accounting
  evidence for M2–M5; **P6-W13 latency method** for M7 (see
  [01](01-measurement-method.md) §2 failure boundaries).
- **P7-W13 baseline terminology** (`p7-w13-performance-baseline`): W17 reuses
  its environment/limitation vocabulary where applicable so that scheduler
  and Linux baselines remain mutually readable; divergent terminology is
  raised as a conflict, not papered over.
- **P0 quality gates** (P0-W07): measurement tooling, if it lives in the
  repository, is subject to the same gates — measurement code is not
  gate-exempt.

Stop and obtain direction instead of guessing when a required decision is
missing, when an upstream contract contradicts this method, or when producing
a number would require weakening a declaration (dropping the environment
block, aggregating across clock domains, or quoting a blocked metric).

## 2. Environment capture procedure

Target: the environment block of every baseline record.

Work: capture the [01](01-measurement-method.md) §3 fields immediately before
the measured runs, from the same host state that will serve the runs; embed
the captured block verbatim in each record. Record the capture commands and
outputs in the verification record so the declaration is auditable. Extend
the W16 run-record environment block; do not maintain a divergent copy.

**Acceptance:** every record's environment block is complete, verbatim from a
captured state, and identifies the host, QEMU, build, fixture, machine
reference, telemetry configuration, and method revision.  
**Failure/blocker:** an undeclarable field is recorded as unknown with the
affected metrics marked limited-comparability; omitting the field is
prohibited.

## 3. Warm-up and repetition policy

The policy is fixed by design; per-run values are recorded, never tuned
post hoc:

1. **Warm-up:** for each measured row, the harness executes warm-up
   repetitions that are excluded from summaries but retained in raw data.
   The warm-up count is declared in the record before the measured runs
   (default when nothing else is justified: 1; the declaration, not the
   default, is normative for the record).
2. **Measured repetitions:** each measured row executes the repetition count
   declared in the record (minimum 3; the declaration may set more, never
   fewer). Every repetition produces its own raw record; no repetition is
   discarded.
3. **Outlier policy: none.** All measured repetitions are retained and
   summarized. Discarding or re-running individual repetitions until a
   preferred value appears is prohibited; a suspected anomalous repetition is
   annotated, not removed.
4. **Interleaving:** when a baseline cycle measures multiple rows, the cycle
   executes row repetitions in a declared order recorded in the record, so
   that ordering effects are inspectable rather than invisible.
5. **No silent re-baselining:** a new baseline cycle is a new record. Old
   records are never edited or deleted.

## 4. Raw-data retention

Targets: `../../verification/assets/p8-w17/<cycle-id>/` (created when data
exists).

Work: retain, per cycle: every repetition's raw M1–M7 values with their
timestamps and ordering index; the environment capture output; the harness
run records (consumed from W16) for every repetition including warm-ups; and
the exact method revision used. Raw values are retained unaggregated; the
summary block of the baseline record references them by path.

**Acceptance:** for any summary number in a baseline record, a reviewer can
reconstruct it from retained raw data without access to the original
session.  
**Failure/blocker:** a summary number whose raw inputs were not retained is
invalid and must be removed from the record; the cycle is re-run or recorded
as not run.

## 5. Baseline-record contract

Target:
`../../verification/p8-w17-linux-performance-baseline-verification.md`
(evidence and run/not-run status) and the factual baseline record under the
same `verification/` area, indexed from it (created when work starts).

Required record sections:

```text
identity        cycle ID, date, method revision, author
environment     verbatim §2 block
scenarios       measured rows with repetition/warm-up declarations
per-metric M1–M7
                raw-data reference, declared summary statistic (per cycle,
                before runs: median with min–max spread; no other statistic
                without a reviewed declaration), and per-metric limitation
blocked_metrics metrics not measurable and why (assumed-contract failures)
limits          the [01] §4 platform-dependence and overhead disclosures
non_kpi_note    the §6 statement of this record, verbatim
```

Rules: the summary statistic is declared before the runs that produce it;
records contain no optimization language; no record may be edited after
publication within a review cycle — corrections are appended records.

## 6. Comparison-validity rules

A comparison between two baseline records is valid **only when all** of the
following hold, and the comparison must state them:

1. same metric set and method revision (or an explicit per-metric mapping
   with justification);
2. same fixture reference and approved machine-contract identity;
3. same telemetry configuration, or an explicit statement of the
   configuration delta and which metrics it may bias;
4. same declared host-load policy and comparable host identity (different
   hosts ⇒ cross-host comparison is prohibited; two records on different
   hosts may both exist but never be subtracted);
5. same acceleration/emulation mode of QEMU.

An invalid comparison is not "weak evidence" — it is prohibited output.
Comparisons describe direction and magnitude of raw/summary movement under
the declared differences; they assign no blame, select no policy, and gate
nothing.

## 7. Ordered implementation workflow

### Step 1 — verify prerequisites and record blocked metrics

Target: implementation record (`../p8-w17-linux-performance-baseline-record.md`,
created in this step).

Work: verify each §1 assumed contract; list which of M1–M7 are measurable
today and which are blocked, with the owning package named.

**Acceptance:** a written measurability statement with owner attribution.  
**Failure/blocker:** a blocked metric stays blocked; no substitute method.

### Step 2 — fix the cycle declarations

Target: baseline-record skeleton.

Work: declare warm-up/measured repetition counts, summary statistic, row
order, and timeout classes per row, per [01](01-measurement-method.md) §5.

**Acceptance:** declarations complete before any measured run.  
**Failure/blocker:** a declaration changed after runs are a new cycle.

### Step 3 — execute the measurement cycle

Target: raw data under `../../verification/assets/p8-w17/`.

Work: run the bound W16 rows through the W16 envelope; collect M1–M7 per
repetition; capture the environment block; retain everything per §4.

**Acceptance:** every repetition has a raw record; environment block captured
from the run state; no repetition discarded.  
**Failure/blocker:** a failed W16 row is reported as failed; W17 does not
re-run it into passing and does not measure a broken scenario.

### Step 4 — author the baseline record

Target: baseline record per §5.

Work: aggregate per the pre-declared statistic; write limits, blocked
metrics, and the non-KPI note; link raw data.

**Acceptance:** record complete per §5; every number reconstructible from
retained raw data.  
**Failure/blocker:** missing raw data ⇒ remove the number or mark not run.

### Step 5 — policy and comparison review

Work: run the validation matrix below; audit the record against the §6
comparison rules and the [01](01-measurement-method.md) §6 prohibitions.

**Acceptance:** W17-DV05 passes; no prohibited statement present.  
**Failure/blocker:** a prohibited statement is removed by appending a
correcting record; the original stays.

## 8. Validation matrix

All rows are planned evidence; none asserts a measurement exists. Record
each as **passed**, **failed**, **blocked**, or **not run** with command,
input, environment, timestamp, and reason.

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W17-DV01 → prerequisites | contract review | inspect §1 assumed contracts against tracked state | every consumed contract named with owner and failure boundary | method coherence; not that prerequisites delivered |
| W17-DV02 → P8-V23 | method completeness review | review [01](01-measurement-method.md) §2/§5 | M1–M7 each declare domain, point, unit, aggregation, cannot-prove; binding table covers all metric groups | measurable definitions; not that values exist |
| W17-DV03 → P8-V23 | environment/overhead review | review §2–§4 of [01](01-measurement-method.md) and a produced environment block | all required fields present; telemetry configuration disclosed; platform-dependence statement present | declared environment; not host excellence or coverage of all environments |
| W17-DV04 → P8-V23 | reproducibility and retention audit | re-run one declared row cycle per §3; attempt reconstruction of summaries from raw data | same declared procedure reproduces raw records of the same kind (values will vary; determinism is not claimed); every summary reconstructible | procedural reproducibility; not value reproducibility and not hardware behavior |
| W17-DV05 → P8-V23 | non-KPI compliance review | audit every W17 artifact for [01](01-measurement-method.md) §6 prohibitions | zero prohibited statements; comparisons satisfy [02] §6 | observation/judgment separation; not future compliance by later users |
| W17-DV06 → handoff | consumability review | read the record as W20 (evidence index) and as P9 (comparison inputs) | each consumer can act without inventing policy or crossing the non-KPI boundary | handoff readiness; not that downstream work is done |

Only W17-DV04 with real runs, plus DV05's audit of that record, can satisfy
P8-V23; all values remain QEMU-environment baselines with explicit limits and
never KPIs or hardware predictions.

## 9. Error, security, and observability model

- **Failure model:** W17's failure modes are methodological: missing
  environment fields, undeclared statistics, unretained raw data, blocked
  metrics silently measured, prohibited language. Each is a named review
  failure with a recorded remedy (append, re-run, or mark not run).
- **Security position:** W17 adds no privilege boundary and no Guest-facing
  surface. Metric collection must not weaken Guest containment or read
  memory outside declared accounting paths; instrumentation disclosure is
  also a security-relevant fact (what was observable during the run).
- **Observability:** the baseline record plus retained raw data is the proof
  surface. The W16 run records and transcripts remain the execution evidence;
  W17 never duplicates or amends them.

## 10. Handoff checklist

Before handing W17 to a reviewer, provide:

- the exact artifact list (method files, records, raw-data assets) and the
  method revision identity;
- W17-DV01–DV06 evidence paths and run status, including explicit not-run
  and blocked-metric entries with owning packages named;
- the environment capture outputs for every cycle;
- confirmation that no threshold, target, optimization conclusion,
  cross-host comparison, or hardware-transfer statement exists in any
  artifact;
- confirmation that M7 (if measured) used the P6-W13 method unchanged, or is
  recorded as blocked;
- open items: counter-namespace dependencies (P0-W13), latency-method
  dependency (P6-W13), approved machine values (task book §8 `ADR Required`
  via W02), and any P7-W13 terminology alignment — without resolving any of
  them here.
