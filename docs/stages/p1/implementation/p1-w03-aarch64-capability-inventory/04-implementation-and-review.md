# P1-W03 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

W03 can start only after the W02 runtime exists in implementable form (the
inventory executes inside it) and the W09 tracker/adapter contracts are
available at their accepted-design level. Before changing any file, the
implementer verifies the mandatory reading (parent README), inspects the
current tree (`git ls-files`; confirm the W02/W09 state and the P0
host-test/unsafe-governance baselines), and records the assumed-contract
states from [01-architecture-and-state.md](01-architecture-and-state.md) §6.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 runtime or the W09 `capabilities` adapter is absent or
  contradicts the §6 seams — raise the conflict per W09's rule; do not
  adapt silently;
- the announced architecture revision cannot be established (needed for the
  recorded decode references) — record the blocker; do not guess bit
  ranges;
- closure appears to require DTB parsing, a `PlatformCapabilities`
  framework, GIC/timer/Stage-2 mechanisms, or control-register writes —
  those are Out of Scope (parent README); stop;
- a required fact proves unimplementable as specified (source register
  inaccessible at EL2, decode ambiguity) — that is a design conflict to
  record, not a local reclassification.

## 2. Ordered implementation steps

### Step 1 — inventory the facts and confirm the reserved boundary

Target: implementation record
(`../p1-w03-aarch64-capability-inventory-record.md`, created in this step).

Work: walk the fact table of
[01-architecture-and-state.md](01-architecture-and-state.md) §2 against the
announced architecture revision; record the exact bit ranges and encodings
used, the classification per fact with its authority quote, and the facts
explicitly reserved (not in the set) with their future owners.

Suggested observation: the architecture reference for the recorded revision;
no repository change.

**Acceptance:** every fact row has source, range, classification, and
rationale; the reserved list is explicit.  
**Failure/blocker:** a fact whose source is ambiguous at the recorded
revision is dropped or reclassified through a design change, not guessed.

### Step 2 — implement extraction

Target: the extraction module.

Work: implement the raw reads (§1 of the extraction contracts — the audited
`unsafe` boundary) and the typed decodes exactly per
[02-code-contracts-fact-extraction.md](02-code-contracts-fact-extraction.md).

Suggested observation: host-side unit evidence of the decode functions
against recorded raw values, where the P0 host-test baseline permits.

**Acceptance:** the `unsafe` inventory contains exactly the six reads; every
decode handles reserved encodings without panic.  
**Failure/blocker:** a decode that cannot be expressed safely is a design
conflict to record.

### Step 3 — implement classification, check, and publication

Target: the classification/report module.

Work: implement `Classification`, `CapabilityRejection`, `verify_required`,
the once-publication report cell, and `build_capability_report` per
[03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md)
§1–§6, including the rejection vocabulary of §4.

**Acceptance:** the check is deterministic in `ALL` order; the publication
discipline matches §4 of the architecture file; the phase body adds
nothing beyond the contracted sequence.  
**Failure/blocker:** a route seam mismatch (panic route cannot carry the
rejection) is a W02/W03 coordination issue; W03 builds no second output
path.

### Step 4 — implement rendering and integrate the route

Target: the render function and the route call.

Work: implement `render_report` per §7; confirm the fail-fast route reaches
W02's panic route with the §4 vocabulary; confirm W09's `console`-phase
wiring can call the render exactly once.

**Acceptance:** lines are bounded, ordered by `ALL`, and channel-agnostic;
the route carries fact label + reason, nothing else.  
**Failure/blocker:** a W06 format conflict is recorded as a coordination
issue (§7); W03 does not adopt channel-internal framing.

### Step 5 — consumer-conduct and negative-path review

Target: implementation record; the verification record.

Work: review that every current consumer (W04's planned baseline mapping is
the named one; W08 consumes transitively through W04 unless its design
names a direct dependency) is specified to query the API rather than
re-read registers or branch on platform names (ADR-044; plan work seq 4).
Review the negative paths: Required-absent rejection carries the named
reason; Optional-absent reports and continues; Unreadable is reported at
full weight.

**Acceptance:** no consumer design point re-derives a fact; the negative
paths match observation rules O1–O4.  
**Failure/blocker:** a consumer-conduct violation is a finding against that
consumer's design, recorded here with a pointer.

### Step 6 — acceptance evidence and handoff

Target: verification record
(`../../verification/p1-w03-aarch64-capability-inventory-verification.md`).

Work: perform the executable reviews of §3; record the deferred executions
(the report appearing in real boots; the NC2 scenario) with their W09/W10/W11
ownership; complete the handoff checklist and record the NC2 variability
finding (parent README decision 8) in its current state.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W03-DV01 → P1-V05 | Fact-set review | walk the implementation against [01-architecture-and-state.md](01-architecture-and-state.md) §2 and the recorded revision | every fact present with source/range/classification/rationale; reserved facts listed with owners | the inventory covers P1's declared needs; not that the platform actually reports each value |
| W03-DV02 → P1-V05/P1-V06 | Taxonomy review | inspect Classification/Observation usage across the code | the two axes are orthogonal; every record carries both; O1–O4 rules hold by construction | supported/optional/future remain distinguishable; not platform behavior |
| W03-DV03 → P1-V06 | Fail-fast and negative-path review | exercise `verify_required` logic against constructed records (host-side where permitted; otherwise inspection) | Required-absent rejects with the named reason in ALL order; Optional-absent continues; nothing partial publishes | the fail-fast policy as designed; not a real CPU's absence behavior (NC2, W11) |
| W03-DV04 → P1-V05 | Extraction-boundary review | read the `unsafe` inventory and decode functions | exactly the six register reads; SAFETY justifications filed; no other privileged access | the audited boundary is complete; not register semantics on real hardware |
| W03-DV05 → P1-V05 | Consumer-conduct review (work seq 4) | search consumer designs/records for register re-reads or platform-name branches | all fact consumption goes through the query API | capability-driven selection as designed; not consumers' eventual compliance (their reviews own that) |
| W03-DV06 → P1-V05/P1-V06 | Executed evidence (deferred) | report lines observed in W10-regressed boots; NC2 executed by W11 where a required fact is variable | report renders once per boot with all facts; NC2 (if unblocked) rejects with the named reason | the report and policy fire in reality; deferred by contracted wiring, not omitted |
| W03-DV07 → W03 closure | Consumability review | read the outputs as W04 (can I map categories to facts?), W08 (are translation limits queryable?), W09 (is the phase body exactly the contracted sequence?), W11 (is NC2's vocabulary fixed?), P2 (is the knowledge boundary clear?) | each consumer can act without inventing W03 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The QEMU-
dependent proofs are deferred to W10/W11 execution by contracted wiring;
until they exist, P1-V05/P1-V06's executed half is unproven and no W03
artifact may report otherwise. No validation here proves P1-V07 through
P1-V21.

## 4. Error, security, and observability model

**Errors.** W03 has one failure class: the required-fact rejection, which is
terminal, named, and routed (W09 matrix `capabilities` row). Optional and
future absences are data, never errors. Misuse of the publication/query API
is an invariant violation routed like any other. There is no retry, no
degradation, no partial report.

**Security.** The inventory reads only self-describing identification
registers; it configures nothing and authorizes nothing. Security-relevant
postures preserved: no register outside the six-read boundary is touched;
the rejection path emits fixed vocabulary only; `unsafe` is confined to the
reads plus the once-publication cell, each with a `SAFETY` justification in
the P0 unsafe inventory; no platform-name branch exists anywhere in W03 or
is made easy for consumers.

**Observability.** The report's render/emit lines are the observable — every
fact with its classification appears in the boot log once the W06 channel is
available, satisfying P1-V05's "classified and reported". The rejection
vocabulary is observable pre-console through the panic route. W03 adds no
counters, prints, or telemetry of its own.

## 5. Handoff checklist

Before handing W03 to a reviewer, provide:

- the exact changed-file list and module locations of every contracted item;
- the recorded architecture revision and per-fact bit-range references;
- W03-DV01..DV07 evidence paths and run status, including the explicit
  deferred/not-run entries (real-boot rendering → W10; NC2 → W11);
- the audited-`unsafe` list (six reads + publication cell) with filed
  `SAFETY` justifications;
- the rejection vocabulary and render line format as recorded before first
  verdict-bearing evidence;
- the NC2 variability finding in its current state (variable required fact
  identified, or the blocked-scenario coordination record);
- confirmation that no DTB parsing, platform framework, GIC/timer/Stage-2
  mechanism, control-register write, allocator, or platform-name branch was
  introduced;
- open items: W06 render-format coordination (if raised), W04/W08 consumer
  contracts (they query, they do not re-derive), P2 knowledge handoff
  boundary — recorded, not resolved here.
