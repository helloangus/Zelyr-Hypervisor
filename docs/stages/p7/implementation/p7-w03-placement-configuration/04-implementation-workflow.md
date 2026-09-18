# P7-W03 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, the implementer verifies the Coding Guidelines preflight and
the W01 register rows W03 consumes (P7-IN-03, P7-IN-04, P7-IN-06); a
`blocked` status on any of them stops the dependent step per the W01 §5
procedure. The W02 lifecycle design is the attach-point authority; its
`Offline`-only rule and the gate's eligibility seam are assumed stable.

Stop and obtain direction instead of guessing when:

- the P2/P3 topology facts do not yield a usable capacity constant or a
  stable logical id space — blocked prerequisite, record per procedure; do
  not invent a capacity number;
- the P5 rights model cannot express the scheduling-policy control right —
  prerequisite mismatch, record; do not mint a right or weaken the hook;
- avoiding a typed rejection appears to require a "reasonable default"
  (e.g., clamping an offline pCPU out of a Shared set silently) — that is
  exactly the forbidden silent fallback; the design rejects instead;
- placement logic would need to read timers, GIC state, or architecture
  registers — layering conflict, stop and record.

## 2. Ordered implementation steps

### Step 1 — prerequisite inspection

Target: no code; the implementation record.

Work: read the W01 register rows and the cited P2-W10, P3-W14, P5-W10 plan
sections; record the assumed topology-capacity source, registry states, and
rights seam in
`../p7-w03-placement-configuration-record.md` (created in this step).

**Acceptance:** the record names each assumed contract with its plan path
and the failure boundary if delivered differently.  
**Failure/blocker:** a blocked row naming W03 stops dependent steps; record
and stop.

### Step 2 — `CpuSet`

Target: `cpuset` module.

Work: implement per [Contract 1.1](03-code-contracts-placement.md) with the
capacity constant wired to the topology facts; full operation unit tests
including boundary ids and `CapacityExceeded`.

**Acceptance:** all operations total; no panic path; capacity constant
traceable to the platform facts.  
**Failure/blocker:** an id space larger than the representable capacity is
a prerequisite mismatch (Step 1 record), not a local constant to bump
silently — escalate.

### Step 3 — validation and resolved placement

Target: `placement` module.

Work: implement `PlacementSpec`/`PlacementMode`/`ResolvedPlacement` and
`validate_placement` per Contracts 1.2/2.1; exhaustive negative tests (one
per error variant) plus valid-combination matrix tests.

**Acceptance:** purity holds (no hidden reads); the error set is closed;
`ReconfigurationNotSupported` fires for every non-`Offline` attempt.  
**Failure/blocker:** a needed validation rule outside the closed error set
is a design change — stop, record, amend the design first.

### Step 4 — ledger

Target: `ledger` module.

Work: implement attach/query per Contracts 2.2/3.2 with the exclusivity
index, receipt reference, and sequence numbers; concurrency test with
conflicting pinned claims.

**Acceptance:** attach atomicity proven under concurrency; snapshots
consistent; append-only within the stage.  
**Failure/blocker:** a race that lets two pinned vCPUs claim one pCPU is a
correctness failure — fix the locking, never by weakening validation.

### Step 5 — eligibility predicate and default rule

Target: `eligibility` module; configuration entry point default.

Work: implement `is_eligible` (Contract 3.1) and the default-placement
rule (§6 of the contracts file); wire the predicate as the W02 gate's
`eligibility` parameter and the W05 picker's filter seam (stub consumers
acceptable at this step; real integration lands with W02/W05 work).

**Acceptance:** a single eligibility function serves gate and picker (no
duplicate rule); default rule produces the all-online shared set.  
**Failure/blocker:** a consumer that needs a different eligibility rule is
a design conflict — escalate; do not fork the predicate.

### Step 6 — telemetry semantics and authorization hook

Target: configuration entry point; event emission.

Work: implement `configure_placement` per Contract 2.3 with the P5 receipt
seam (a real check once P5 lands; until then the seam takes the receipt
reference and records it, with the rights check marked as the documented
upstream dependency) and both event emissions.

**Acceptance:** unauthorized and rejected configurations are both visible
as events; receipts are recorded, never interpreted locally.  
**Failure/blocker:** interpreting rights bits inside placement code is a
layering violation — stop.

### Step 7 — records, validation, closure

Work: complete the implementation record; run the
[validation matrix](05-validation-and-handoff.md); write evidence to
`../../verification/p7-w03-placement-configuration-verification.md`;
confirm the handoff checklist. Completion is claimed only in the
verification record, only for what ran.

## 3. Evidence rules

- Host unit/property evidence is necessary but not sufficient: P7-V05–V07
  pass only with the target-side rows in the matrix.
- Planned, run, blocked, failed remain distinct; skipped scenarios are
  "not run" with reasons.
- Expected implementation profile: no new `unsafe`, no new dependencies, no
  public API, no external encoding of `CpuSet`; any deviation is reported
  per the repository reporting rules.
