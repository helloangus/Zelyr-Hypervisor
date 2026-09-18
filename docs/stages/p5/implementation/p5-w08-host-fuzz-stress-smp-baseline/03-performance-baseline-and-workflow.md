# P5-W08 Performance Baseline and Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the actual state:

- the W03–W06 detailed designs and implementations exist, and each boundary
  named in [01 §1](01-fuzz-and-property-matrix.md) exposes a host-callable
  seam or a recorded gap;
- the P0 host-test basis ([P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md))
  exists as an evidenced harness convention;
- the P3-evidenced multi-pCPU QEMU foundation ([P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md))
  is available for the SMP rows;
- any fuzzing/property tool to be added is routed through
  [P0-W18](../../../p0/plans/p0-w18-dependency-governance.md) governance
  before it enters the tree.

Stop and obtain direction instead of guessing when: a seam is missing
(blocked prerequisite against the owning boundary — do not reimplement its
logic); the P3 foundation cannot host the two-pCPU rows (record blocked;
the host-side rows still run); a dependency choice exceeds stage-local
freedom (route to P0-W18); or a baseline number is requested as a target or
comparison promise (refuse: the baseline is a record, per README decision 6).

## 2. Performance-baseline method

### 2.1 Categories

Exactly four categories, per the plan and P5-V14:

| Category | What is measured | Where |
|---|---|---|
| PB-01 minimal call | one full dispatch of the W06 minimal permitted operation, decode through result encoding, on the QEMU stack | declared QEMU environment, end to end |
| PB-02 handle lookup | the W04 resolution seam over a populated fake table | host-side seam |
| PB-03 authority check | the W05 check seam over a populated fake capability state | host-side seam |
| PB-04 Guest-data validation | the W03 validation seam over descriptor cases | host-side seam |

### 2.2 Procedure rules

- **Warm-up excluded, measurement declared:** a declared warm-up is excluded
  from the recorded samples; the sample count, iteration structure, and what
  is timed are fixed before the run and recorded with the raw data.
- **Environment recorded:** host model, QEMU version and configuration
  (pCPU count, RAM), toolchain identity from the P0 pin, and the exact
  build profile of the measured binaries.
- **No semantic changes for speed:** tracing/debug must be set to the
  declared measurement configuration, but security-boundary checks are
  never disabled or weakened for measurement (Coding Guidelines
  benchmark-path rule). The measured binary is a security-complete binary.
- **Raw data retained:** per-sample values (or a declared statistically
  faithful summary agreed in review, plus the raw file) are recorded in the
  verification record. Summaries without raw data do not satisfy P5-V14.

### 2.3 Measurement mechanism

The host-side categories (PB-02–PB-04) are timed by the host test harness's
monotonic clock; the mechanism detail is recorded in the implementation
record. PB-01 is measured in the declared QEMU environment end to end,
timed by the harness wall clock around a declared Guest-driven call loop;
it is a coarse, environment-specific figure. If a later approved design
provides an evidenced EL2/Guest counter facility, PB-01's method may be
refined through a record amendment — introducing such a facility is not
W08 scope (a P6-adjacent timer mechanism stays out).

### 2.4 Use and non-use of the baseline

The recorded figures are a **reference point for future regression
discussions only**: same-method, same-environment comparisons may inform
later designs; nothing here sets a requirement, budget, KPI, or
optimization obligation, and no figure may be quoted as hardware
performance. Every publication of these numbers outside the verification
record must carry the method, environment, and the explicit statement that
QEMU timing is not hardware timing.

## 3. Ordered implementation steps

### Step 1 — reconcile prerequisites and seam inventory

Target: implementation record
(`../p5-w08-host-fuzz-stress-smp-baseline-record.md`, created in this
step).

Work: inventory the delivered seams per [01 §1](01-fuzz-and-property-matrix.md);
confirm the P0 harness conventions and P3 two-pCPU foundation; record the
fuzzing/property tooling decision from P0-W18 governance (or record its
pending state and proceed with harness-internal, dependency-free
generators).

**Acceptance:** the record names each seam confirmed/gapped and the tool
decision with its governance reference.  
**Failure/blocker:** a gapped seam blocks its matrix rows only; the block
is recorded against the owning package.

### Step 2 — implement the fuzz and property harness

Target: host test harness (per P0-W08 conventions).

Work: implement the FZ-01–FZ-05 targets and PR-01–PR-05 properties over the
seams, with the oracle set ([01 §4](01-fuzz-and-property-matrix.md)), seed
handling, bounds, and the smoke configuration
([01 §5](01-fuzz-and-property-matrix.md)).

**Acceptance:** a deliberately injected accepted-invalid in a fake
environment is caught by the relevant oracle; identical seeds reproduce
identical results.  
**Failure/blocker:** an oracle that cannot observe its condition through
the delivered contracts is a harness gap — narrow the row and record; do
not add hypervisor-side test hooks the owning designs did not authorize.

### Step 3 — implement the stress and SMP scenarios

Target: harness scenarios of
[02](02-stress-smp-scenarios.md) §2–§3 with the audits of §4.

Work: implement ST-01–ST-05 host-side; implement SMP-01–SMP-05 on the
declared two-pCPU QEMU configuration with rendezvous and watchdog rules.

**Acceptance:** each scenario runs to its declared bound with audits
executing; a knowingly broken audit (test the auditor) fails the run.  
**Failure/blocker:** a missing P3 foundation blocks SMP rows as `blocked`
with reason; host rows proceed.

### Step 4 — implement the baseline measurements

Target: PB-01–PB-04 per §2.

Work: fix sample counts and timing points; record environment; produce raw
data files into the verification area.

**Acceptance:** raw data plus method description exist for all four
categories; the non-use rules of §2.4 appear in the record.  
**Failure/blocker:** PB-01 without the delivered QEMU integration path is
blocked with reason; the seam categories still run.

### Step 5 — run the matrix and record evidence

Target: verification record
(`../../verification/p5-w08-host-fuzz-stress-smp-baseline-verification.md`).

Work: execute the §4 matrix rows in scope; smoke runs and deep runs
recorded separately; every oracle hit recorded with seed and minimal
reproducer, fixed by the owning package, and re-run; explicit not-run and
blocked entries for everything deferred.

**Acceptance:** every executed row meets its pass condition; smoke
configuration is recorded and stable.  
**Failure/blocker:** a security-class finding (O4 accepted-invalid, O6
disclosure) is escalated immediately and blocks suite-level pass claims
until resolved.

### Step 6 — records and handoff

Target: implementation record; handoff section of the verification record.

Work: complete the record (changed files — expected: test harness only, no
hypervisor behavior changes; dependencies with their P0-W18 reference; new
`unsafe` expected none in harness code); deliver the smoke configuration,
oracle classes, and robustness limits to W09; deliver evidence, stress
configuration, and the performance record to W10.

**Acceptance:** handoff artifacts located per the README's downstream
section.  
**Failure/blocker:** a missing artifact blocks closure review, not the
consumer.

## 4. Validation matrix

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, seed, environment, timestamp, and reason. Host
rows prove properties of the delivered logic on the host; QEMU rows prove
behavior in the declared QEMU environment only. Nothing here proves absence
of all bugs, final scalability, real-hardware timing, or any performance
KPI.

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W08-DV01 → P5-V13 | seam inventory review | Step 1 | every seam confirmed or gapped-with-owner; tool decision governed | the evidence basis exists; not boundary correctness |
| W08-DV02 → P5-V13 | FZ-01/FZ-04 runs | Step 5, recorded seeds/bounds | no oracle hit; all outcomes in vocabulary | decoder and range check resist the exercised random space; not exhaustiveness |
| W08-DV03 → P5-V13 | FZ-02/FZ-03 runs | Step 5 | no accepted-invalid; denial classes precise | reference/authority checks resist random probing; not W04/W05 internals |
| W08-DV04 → P5-V13 | FZ-05 pipeline run | Step 5 | one outcome + category per input; no Guest-buffer writes on denials | end-to-end host-side containment; not Guest-side behavior (W07) |
| W08-DV05 → P5-V13 | PR-01–PR-05 runs | Step 5, declared case counts | all properties hold | lifecycle/revoke/range/quota properties over generated space; not concurrency |
| W08-DV06 → P5-V13 | oracle self-test | Step 2 acceptance | injected accepted-invalid caught; same-seed reproduction exact | the oracles actually detect their classes; not hypervisor correctness |
| W08-DV07 → P5-V13 | ST-01–ST-05 runs | Step 5 | audits hold at every boundary; limits recorded | lifecycle invariants under declared churn; not capacity limits or speed |
| W08-DV08 → P5-V14 | SMP-01–SMP-05 runs | Step 5, two-pCPU declared config | serializable outcomes; no hang/corruption | state protection on two pCPUs in QEMU; not final lock strategy, scalability, or hardware SMP |
| W08-DV09 → P5-V14 | baseline record review | Step 4 + §2.4 review | four categories with method, environment, raw data; no KPI language; security checks unmodified | a recorded reference point; not a target, not hardware timing |
| W08-DV10 → W08 closure | handoff and record review | Step 6 checklist | W09/W10 artifacts located; smoke config stable | handoff readiness; not downstream completion |

## 5. Error, security, and observability model

The harness fails loudly and classifiably: oracles O1–O6 and the audit
model are its error detector; findings route to owning packages. Security
rules embedded in the method: measured binaries are security-complete; no
oracle is waivable; evidence contains no Host pointers or Guest buffer
contents (O6 applies to W08's own artifacts). Observability is the recorded
seed/bound/audit trail sufficient for W09 to re-run the smoke set as a
regression row without reinterpretation. No `unsafe` is expected in harness
code; any exception is reported per the Coding Guidelines.

## 6. Handoff checklist

Before handing W08 to a reviewer, provide:

- the exact changed-file list (harness and scenarios only) and the seam
  inventory with per-seam owner confirmation;
- evidence paths and run status for W08-DV01–DV10 with seeds, bounds,
  environments, and explicit blocked/not-run entries;
- every oracle finding with its seed, minimal reproducer, owning package,
  and resolution status;
- the fixed smoke configuration delivered to W09 and the confirmation that
  it re-runs deterministically;
- the performance record with method, environment, raw data, and the
  QEMU-not-hardware statement;
- confirmation that no hypervisor behavior, lock/index/allocator strategy,
  or P6+ mechanism was changed or added for testing;
- open items: gapped seams, blocked rows, Reserved corpora/tooling, and any
  `Architecture Change Request` / `ADR Required` record — without resolving
  them here.
