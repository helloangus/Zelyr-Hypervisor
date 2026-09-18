# P4-W07 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the tree: `git ls-files`,
plus the sibling implementation records for W02–W06 (created when those
packages start), which carry the evidence the assumed contracts M1–M6 in
[01 §2](01-scope-and-foundations.md) wait on. Entry-order rule: driver,
determinism, ledger, and grammar logic with host-side unit tests can proceed
against assumed signatures, but every on-target row (DV01–DV08) requires the
W02/W03/W04/W06 paths delivered; a missing upstream is a recorded blocked
prerequisite per W01 §4.

Stop and obtain direction instead of guessing when any of the following
occurs:

- W03's delivered record shows `init_zeroed` re-init from `Loaded` is not
  supported — drop `RamReuse` to Reserved, record the gap against M2, and
  re-own decision D2 in this design's record; do not improvise a third
  teardown depth;
- W04/W02 sequencing differs from the assumed order — joint design note; the
  driver never reorders;
- W06's diagnostic fields differ — joint review (M4); the ledger never
  synthesizes correlation fields;
- the P0 baseline lacks structured events or build identity (M5) — degrade
  per the baseline's fallback and record the degraded basis with the
  affected evidence;
- an `unsafe` need appears — stop; add a contract here first, with SAFETY
  justification and an inventory entry (D9).

## 2. Ordered implementation steps

### Step 1 — determinism module with host-side proof

Target: `determinism` module ([03 §5](03-code-contracts-repeat-driver.md)).

Work: implement the FNV-1a-64 digest over the W03 host view signature, typed
context equality, sequence equality with the expected-prefix rule, and count
vector equality. Why first: pure functions with no upstream dependencies;
everything else consumes them.

**Acceptance:** known-answer digest tests pass; equality/inequality tests
pass per surface; re-init equivalence test (allocate+init vs retain+re-init)
passes against a stubbed view.  
**Failure/blocker:** none expected; a signature mismatch with W03's view is
a joint design note.  
**Evidence:** `../../verification/p4-w07-repeatability-telemetry-verification.md`
(DV05 rows).

### Step 2 — telemetry ledger

Target: `telemetry-ledger` module ([04 §1–§3](04-code-contracts-telemetry.md)).

Work: implement counters over the §1 event inventory, episode vectors,
correlation records, and the summary; define the fixed capacity from the
plan's episode/fault budgets.

**Acceptance:** vector correctness and monotonicity tests pass; correlation
completeness covers every P4-V12 dimension; overflow surfaces as
`LedgerOverflow`.  
**Failure/blocker:** a missing sibling event set is a blocked prerequisite
recorded against the relevant W01 row; the ledger is not extended with
invented events.  
**Evidence:** verification record DV06.

### Step 3 — repeat driver

Target: `run-episode`/`run-driver` modules
([03 §1–§4](03-code-contracts-repeat-driver.md)).

Work: implement `EpisodePlan`/`RunPlan`, `RunDriver::new`, `run_episode`
(both teardown depths), and `run_all` with the abort/continue rules; wire
accounting restoration checks behind the M6/M2 signatures.

**Acceptance:** host-side state-machine tests pass for both depths, abort
paths, truncation-point recording, and divergence-continue behavior; misuse
paths escalate as documented.  
**Failure/blocker:** sequencing mismatches with W03 §4/W04 §4 stop the step
(joint design note), never a local reorder.  
**Evidence:** verification record DV01/DV02.

### Step 4 — run record and joint grammar review

Target: `run-record` module ([04 §4](04-code-contracts-telemetry.md)); joint
review with W08.

Work: implement `emit_run_record` for P4-RR v1 over bounded buffers with the
emission-failure path; agree the grammar and field list with W08 (open item
O1) and the expected count patterns ([04 §5](04-code-contracts-telemetry.md)).

**Acceptance:** golden-record tests pass (happy/diverged/aborted); W08
acknowledgment recorded; truncation annotated.  
**Failure/blocker:** W08 disagreement re-opens the grammar here; it is never
resolved by emitting a second divergent format.  
**Evidence:** verification record DV09 (review row).

### Step 5 — same-session repeat evidence (gated on M2–M6 delivered)

Target: on-target episodes through the delivered W02/W03/W04/W06 paths.

Work: execute the P4-V10 plan: ≥ 3 episodes (minimum mix: episode 0
`FullRebuild` reference, episode 1 `FullRebuild`, episode 2 `RamReuse`),
scenario sequence fixed in the plan; verify accounting restoration, surface
equality, and expected count patterns per episode.

**Acceptance:** all episodes determinate; `DeterminismSummary::Stable`;
accounting balanced; patterns satisfied.  
**Failure/blocker:** any divergence or imbalance is failed evidence —
record, diagnose, correct, re-run; never declared stable from partial
equality.  
**Evidence:** verification record DV01–DV03/DV07; raw run data under the
W08 evidence layout.

### Step 6 — cold-boot consistency expectations handed to W08

Target: the W08 manifest inputs.

Work: hand over the declared minimums (D7), the P4-RR v1 grammar, the
determinism surfaces with reference-digest semantics, and the count
patterns; confirm W08's manifest encodes ≥ 5 cold-boot iterations of one
declared build with per-iteration record comparison.

**Acceptance:** W08's design/manifest rows cite these inputs without
reinvention.  
**Failure/blocker:** disagreements are joint review items recorded in both
records.  
**Evidence:** verification record DV04 preparation row; W08 record cross-link.

### Step 7 — closure review and handoff

Work: run the review matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md), confirm the
handoff checklist, and record implementation facts (delivered depths, event
set, grammar version, limitations) in
`../p4-w07-repeatability-telemetry-record.md` when work starts. Completion
is claimed only in the verification record, with evidence, only for what
actually ran.

## 3. Validation matrix

See [06-validation-and-handoff.md](06-validation-and-handoff.md) §1
(W07-DV01 through W07-DV10). Each row is recorded as **passed / failed /
blocked / not run** with command or review input, environment, date, and
reason. No row here proves P4-V01–V09 or P4-V13–V16, and none may be
reported as doing so.
