# P7-W04 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W04 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, verify the Coding Guidelines preflight and the W01 register
rows W04 consumes (P7-IN-01/02/05/07/08). A `blocked` row whose failure
boundary names W04 stops the dependent step per the W01 §5 procedure. The
P7-W02 lifecycle design is assumed stable (engine, gate, exit boundary,
invariants).

Stop and obtain direction instead of guessing when:

- the P6 mechanism cannot arm per-pCPU one-shot deadlines (tick-only or
  shared timer) — the on-demand deadline model cannot bind; record an ACR
  per the procedure; do not build a tick emulator in P7;
- the P4 entry path cannot be invoked per dispatch, or context save at
  exit cannot be assumed — the switch sequence's preconditions fail;
  record and stop; do not add register manipulation to scheduler code;
- preserving a state class (timer, vIRQ, events) across switches appears
  to require P6/P4 changes — that is an upstream contract change; record;
- implementing the slice default appears to require a performance
  measurement to justify — not required: the value is a recorded policy
  constant with the decision-3 selection rule, not a KPI;
- any step would need new `unsafe` outside the P4/P6/P1 boundaries —
  scheduler logic is safe Rust; an `unsafe` need is a design conflict to
  record (P0-W10 governance for any approved exception).

## 2. Ordered implementation steps

### Step 1 — prerequisite inspection

Target: the implementation record (created in this step).

Work: read the W01 register rows and the cited P6-W05, P6-W13, P4-W04,
P4-W09 plan sections plus the P7-W02 contracts; record the assumed seam
signatures (deadline arm/cancel/IRQ, entry/exit, address-space activation,
timer restore) and failure boundaries in
`../p7-w04-preemption-context-switch-record.md`.

**Acceptance:** each assumed seam is named with its plan path and mismatch
handling.  
**Failure/blocker:** a blocked row naming W04 stops dependent steps.

### Step 2 — slice type and policy source

Target: `slice` module.

Work: implement `TimeSlice` with validation and the policy-source seam per
[Contracts 1.1/1.2](03-code-contracts-preemption.md); wire the v0 default
constant (selected now per the decision-3 rule; record value, rationale,
and date in the implementation record).

**Acceptance:** boundary tests pass; the seam compiles against a stub
policy; the default is recorded.  
**Failure/blocker:** a policy answer failing validation aborts dispatch by
design — implement that path, not a clamp.

### Step 3 — deadline discipline and intent

Target: `deadline` + `intent` modules.

Work: implement Contracts 2.1–2.4 against a P6-contract-faithful fake
first (host tests): arm/cancel state machine, spurious-expiry rule, IRQ
boundedness, intent atomics per the P3 ordering baseline.

**Acceptance:** host tests cover arm/expire/cancel interleavings and the
shared-vs-pinned arm-failure asymmetry; handler performs the three
permitted operations only.  
**Failure/blocker:** a P6 seam mismatch found here is recorded, not
adapted around.

### Step 4 — trigger set and loop hook

Target: trigger types + the reconsideration hook signature.

Work: implement `PreemptionTrigger`/`DescheduleReason` (Contracts 3.1/3.2)
and the loop hook consumed by W05's control loop (stub loop acceptable);
property-test reason/state agreement.

**Acceptance:** the trigger set is closed; producers are exactly the three
named paths; agreement property holds.  
**Failure/blocker:** a fourth producer discovered is a design change —
stop and amend.

### Step 5 — switch sequence

Target: `switchseq` module.

Work: implement `switch_to` per
[the switch contracts](04-code-contracts-context-switch.md) against
contract-faithful fakes for P4/P6 seams; host-test the ordering (steps
0–10), the failure containment of Contract §3, and the isolation trace
marks.

**Acceptance:** host tests prove: quiesce-before-gate ordering; single
`Running` handover; all-or-nothing candidate activation; per-class trace
marks; no step writes an unowned class (review checklist).  
**Failure/blocker:** needing to write an unowned class means the sequence
or a predecessor seam is wrong — stop and redesign; never take a shortcut
around an owner.

### Step 6 — target integration

Target: real P4/P6 seams; W02 gate/exit; W05 loop call sites.

Work: bind the fakes to the real mechanisms per the assumed contracts;
integrate the deadline IRQ registration with the P6 delivery path and the
sequence with the W05 loop. Isolation instrumentation (trace marks) lives
behind the P0 trace namespace.

**Acceptance:** the QEMU scenarios of
[the validation matrix](06-validation-and-handoff.md) DV03/DV03b run;
mismatches are recorded per the W01 procedure.  
**Failure/blocker:** a real-mechanism divergence (e.g., lazy LR behavior
differing from the assumed contract) is recorded and, if it changes the
isolation obligations, escalated as an ACR — not absorbed.

### Step 7 — records, validation, closure

Work: complete the implementation record; run the
[validation matrix](06-validation-and-handoff.md); write evidence to
`../../verification/p7-w04-preemption-context-switch-verification.md`;
confirm the handoff checklist. Completion is claimed only in the
verification record, only for what ran.

## 3. Evidence rules

- Host fake-based evidence is necessary, never sufficient: P7-V08/V09 pass
  only on target rows.
- Planned, run, blocked, failed remain distinct; skipped scenarios are
  "not run" with reasons.
- Expected implementation profile: no new `unsafe` (assembly remains in
  the P4/P1 boundaries), no new dependencies, no public API, no timer
  register access; deviations are reported per the repository rules.
