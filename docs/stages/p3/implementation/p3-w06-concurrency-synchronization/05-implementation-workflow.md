# P3-W06 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tracked tree
(read-only discovery such as `git ls-files` suffices). Useful
pre-workflow checks: the W02–W05 designs' delivered protocols are
available as agreed designs; the P0 unsafe/diagnostic governance surfaces
named in [01 §1.2](01-scope-and-foundations.md) are present as reviewed
deliverables; the host-side test entry point (P0-W08 baseline) exists.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a delivered W02–W05 protocol contradicts the atomic-policy table —
  raise the cross-design conflict with the owning design; do not reword
  the policy or the protocol silently;
- the P2 allocator contract arrives requiring allocation under arbitrary
  P3-held locks — raise as a cross-stage conflict (README decision 4);
  do not weaken the rule locally;
- a consumer design (W07–W11) needs a new ladder class, a stronger
  ordering, or a reserved primitive — treat it as a W06 design change;
  do not grow the primitives locally;
- implementing the flavors appears to need allocation, blocking, or
  unstable features — that is a design error to raise, not a local
  workaround.

## 2. Ordered implementation steps

### Step 1 — implement the lock flavors

Target: the crate/module the approved build design assigns for
synchronization primitives.

Work: implement `SpinLock<T>`, `InterruptSaveSpinLock<T>`, the guards,
and `LadderClass` per
[03-code-contracts-lock-primitives.md](03-code-contracts-lock-primitives.md).
Keep the `unsafe` surface to the audited flag/payload boundary and the two
interrupt-mask intrinsics; SAFETY-comment per P0-W10 governance. No
allocation, no waiting beyond the acquire loop, no WFE.

Acceptance: the primitives compile on the host target and the AArch64
target (per the approved build design); the `unsafe` inventory records
each block; no consumer-facing API beyond the contracts exists.

Failure/blocker: if the flag encoding cannot meet the stated orderings
without SeqCst or fences, stop and raise the design question — do not
"just make it SeqCst".

Evidence: implementation record (design decisions, `unsafe` inventory
entries).

### Step 2 — primitive unit tests (correctness)

Target: host-side tests per the P0-W08 baseline.

Work: implement the W06-DV01/DV02 tests: mutual exclusion under the L-1
hammer, try_lock accounting (L-2), unlock/reacquire happens-before
observed through the payload, irq-save mask pairing with simulated
intrinsics (L-3).

Acceptance: all limits in
[04 §7](04-code-contracts-atomic-and-ordering-policy.md) pass with exact
accounting; harness global timeout never fires.

Failure/blocker: a flaky failure is evidence — capture it, diagnose
(ordering bug vs test bug), and re-run to the stated limits; do not
weaken the limits to pass.

Evidence: verification record.

### Step 3 — atomic policy and ladder as review artifacts

Target: the normative statements live in this design
([04](04-code-contracts-atomic-and-ordering-policy.md) §3–§4); the
implementation step adds the AP/LOL/BW/MIS ids as code comments at
consumer construction sites *when those consumers land* — W06 itself adds
no instances.

Work: verify consistency of the policy table against each delivered
W02–W05 protocol (a reading review, recorded); confirm the only AP-4
instance is W05's declaration; confirm no SeqCst exists anywhere.

Acceptance: consistency review recorded with a per-protocol verdict;
any conflict raised per §1, not absorbed.

Failure/blocker: a protocol/policy conflict is a stop condition.

Evidence: verification record (review notes).

### Step 4 — busy-wait and misuse rules wiring

Target: the rule ids in
[04 §5–§6](04-code-contracts-atomic-and-ordering-policy.md).

Work: confirm the rule set is complete against the consumer plans
(notification WFE waits, W08 completion polls, W09 handler constraints);
record the cross-reference (BW-5's idle-wait exception ↔ W07) explicitly.

Acceptance: every consumer context named in the plans maps to at least
one rule id; no consumer context is unruly.

Failure/blocker: a gap is a design change to W06 (new rule), recorded —
not a consumer-side local rule.

Evidence: implementation record.

### Step 5 — ladder census and model check

Target: host-side harness (test-side) + review.

Work: build the L-4 model check over the construction-site census; at
W06 closure the census contains W06's own reference consumers' planned
entries (W08 initiation lock = Infrastructure; W09/W11 Diagnostics and
Statistics class notes) as *declared* rows, with real instances arriving
as those packages land.

Acceptance: the model check runs and reports the declared census
acyclic; the check is part of the host test entry.

Failure/blocker: a cycle or violation in a *declared* row is raised with
the declaring design.

Evidence: verification record.

### Step 6 — closure review and handoff record

Work: run the matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md); confirm the
handoff checklist; verify against the plan's work sequence and the
consumer handoff wording. Record implementation decisions in
`../p3-w06-concurrency-synchronization-record.md` and evidence in
`../../verification/p3-w06-concurrency-synchronization-verification.md`
only for what was actually performed. Completion is claimed only in the
verification record, only for what was run.

## 3. Deferred-to-consumer obligations (recorded, not performed here)

- W07 must declare its WFE/SEV bounds and its idle-wait status (BW-3/5)
  in its design; W06's policy already reserves the semantics.
- W08 must construct its initiation lock with
  `LadderClass::Infrastructure` and satisfy BW-4.
- W09/W11 must cite CR-4/CR-5 and LOL ranks in their designs.
- W10 must consume MIS-1..MIS-12 and LOL-R4 as audit criteria.
These obligations are part of the handoff checklist; W06 performs none of
them.
