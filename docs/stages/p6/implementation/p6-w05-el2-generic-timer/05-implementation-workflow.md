# P6-W05 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W05 design entry](README.md).

## 1. Preconditions and failure boundary

Before changing any file the implementer completes the Coding-Guidelines
preflight (repository `AGENTS.md`, documentation index, ADR baseline, P6 task
book, P6-W05 plan) and loads [01](01-scope-and-foundations.md),
[02](02-architecture-and-state.md), and the contract file for the step at
hand. Because P1–P5 are unimplemented at design time, workflow step 1 is a
real entry review: it inspects whatever upstream evidence exists at
implementation time and stops on gaps, per the failure boundaries in
[01](01-scope-and-foundations.md) §5. Stop and record a blocker instead of
guessing when: an upstream contract is missing or contradictory (owner: the
conflicting package; W05 never repairs it); the delivered timer baseline
differs from D1 (design amendment required); or a step appears to require
Guest-visible, scheduler, or wall-clock behavior (scope violation — those are
W06/P7/out of scope).

## 2. Ordered implementation steps

### Step 1 — entry review and prerequisite reconciliation

Target: implementation record (`../p6-w05-el2-generic-timer-record.md`,
created in this step).

Work: inspect the actually delivered P1–P5 evidence and the W01–W03 design
and implementation records for the contracts in [01](01-scope-and-foundations.md)
§5 (timer-access baseline, per-pCPU ownership, synchronization, W03
classified dispatch, W02 readiness, platform timer facts). Record each
contract as available / available-with-differences / missing.

Suggested observation: read the upstream implementation and verification
records; `git ls-files` confirms what exists.

**Acceptance:** every §5 contract has a recorded status; no contract is
assumed silently. **Failure/blocker:** a missing or contradictory contract
stops W05 at this step with a recorded blocker naming the owning package;
W05 does not proceed to code changes on an unmet prerequisite.

### Step 2 — place the logical modules and declare the unsafe boundary

Target: the workspace layout chosen by the workspace-owning packages
(logical modules `time-core`, `deadline-timer`, `timer-consumer-dispatch`
per [02](02-architecture-and-state.md) §2).

Work: map the logical modules onto the actual crate/module tree, respecting
ADR layering (Arch-domain module owns all system-register access; Core sees
only typed values). Declare the W05 `unsafe` inventory entries
(register accessors) with draft `SAFETY` justifications.

**Acceptance:** no system-register access outside the Arch-domain module;
the unsafe inventory lists every W05 register accessor; no crate dependency
was added. **Failure/blocker:** if the workspace cannot express the layering,
record a workspace-package blocker; do not inline register access elsewhere.

### Step 3 — implement the time-value and clock module

Target: `time-core` (contracts in
[03](03-code-contracts-time-core.md)).

Work: implement `TimerCounter`/`TimerDelta`/`TimerDeadline`/
`TimerFrequency` with checked conversion, `clock_read()` with the ordering
discipline, and `monotonic_pair_check()`, plus host-side unit tests
(conversion boundaries, overflow, monotonic-pair semantics).

**Acceptance:** all raw counter/deadline arithmetic is inside the module
behind newtypes; checked arithmetic throughout; unit tests pass under the
pinned toolchain. **Failure/blocker:** a discovered need for untyped
arithmetic is a design conflict, not a convenience — stop and record it.

### Step 4 — timer intake and per-pCPU construction

Target: `deadline-timer` init path (contract
[04](04-code-contracts-deadline-timer.md) §2).

Work: implement `intake_and_init()` including the frequency cross-check,
INTID intake from W02/W03 data, and trap-posture verification against the P1
baseline; leave the timer disabled on every path.

**Acceptance:** a pCPU with a failing check has no usable timer and a
precise diagnosis; no Core-level timer constant exists (D6).
**Failure/blocker:** an unfunded intake input (e.g. no W02 INTID binding) is
a step-1-class blocker for timer use.

### Step 5 — arm/cancel/rearm and the register sequences

Target: `deadline-timer` consumer operations (contract
[04](04-code-contracts-deadline-timer.md) §3).

Work: implement `arm`, `cancel`, `rearm` (both modes with bounded catch-up),
with the §5 barrier placements, generation discipline, and diagnostics
counters.

**Acceptance:** state-machine transitions are exactly those of
[02](02-architecture-and-state.md) §4.1; every register sequence matches the
§5 ordering table; no operation loops or allocates. **Failure/blocker:** an
observed need for a new transition is a design amendment, not a local state
field.

### Step 6 — expiry service and W03 integration

Target: `expiry_service()`, record handoff, consumer dispatch; the W03
handler registration (contract [04](04-code-contracts-deadline-timer.md)
§3.3, §3.5, §4).

Work: implement the IRQ-context service path, the single-slot handoff, and
bind the handler through the W03 classified-dispatch integration; exercise
the cancel/fire race paths against the W03 classification outcomes.

**Acceptance:** W05 performs only the bounded §3.3 work in IRQ context;
completion responsibility stays with W03; a `CancelledFire` is classified
safely by the delivered W03 lifecycle. **Failure/blocker:** if W03 cannot
classify condition-cleared deliveries, raise the cross-design conflict
recorded in [04](04-code-contracts-deadline-timer.md) §4 — Architecture
Change Request path, no local workaround.

### Step 7 — acceptance scenarios and evidence

Target: verification record
(`../../verification/p6-w05-el2-generic-timer-verification.md`).

Work: run the validation matrix ([06](06-validation-and-handoff.md) §1):
one-shot attribution, cancel-before-fire, rearm stability and bounded
catch-up, monotonicity across Guest entry/exit windows, multi-pCPU
independence. Record commands, environment, outputs, timestamps, and
explicit not-run entries; complete the implementation record with changed
files, new `unsafe` entries, and deviations.

**Acceptance:** every W05-DV row has a run status with evidence or an
explicit reason it was not run; P6-V07/P6-V08 are claimable only from the
verification record, never from this design.

### Step 8 — closure review and handoff

Work: run the [06](06-validation-and-handoff.md) §3 handoff checklist;
verify the mechanism-not-policy property (D5) by reviewing the public
surface; confirm the W06/W10/W11/W13/P7 handoff statements are still
accurate.

**Acceptance:** checklist complete; handoff wording matches delivered
evidence.

## 3. Evidence destinations

- Implementation record: `../p6-w05-el2-generic-timer-record.md` (created at
  step 1; holds decisions taken, changed files, unsafe inventory deltas,
  deviations).
- Verification record:
  `../../verification/p6-w05-el2-generic-timer-verification.md` (created
  when evidence exists; holds run/not-run status per matrix row).
- Neither file may be created by this design, and no completion claim may
  appear in any design file.

The validation matrix, error/security/observability model, and handoff
checklist that close this workflow are in
[06-validation-and-handoff.md](06-validation-and-handoff.md).
