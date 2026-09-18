# P7-W02 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, the implementer verifies the Coding Guidelines preflight
documents and the P7-W01 register status for the rows this package consumes
(P7-IN-01…05). Every runtime-binding row must be
`contract-mapped (evidence pending)` at most — work proceeds on the assumed
contract — or `available`; a `blocked` row whose failure boundary names W02
stops the affected step.

Stop and obtain direction instead of guessing when:

- a P3/P4/P5/P6 mechanism differs from the assumed contract at the seam
  (lock primitive class, entry-mechanism signature class, fault taxonomy) —
  raise the W01 §5 procedure; do not adapt W02 to redefine the predecessor;
- implementing a contract appears to require a scheduling-policy decision
  (order of picking, slice values) — that is W05/W04 scope; stop the step;
- the gate or engine would need a new `unsafe` block beyond the P4/P3
  boundaries — scheduler logic is expected to be safe Rust; any `unsafe`
  need is a design conflict to record, not a local invention (P0-W10
  governance applies to any that is approved); or
- a reviewer requests a lifecycle edge change — edges change only through a
  new design decision, never in-code.

## 2. Ordered implementation steps

### Step 1 — lifecycle foundations

Target: the `lifecycle` module (types + table), host-testable, `no_std`,
allocation-free.

Work: implement `VcpuRunState`, `LifecycleEvent`, `TransitionContext`,
`TransitionError`, and `successor` exactly per
[contracts](03-code-contracts-lifecycle.md) §1, with the exhaustive
table test. No engine, no locking yet.

Suggested observation: `cargo test -p <crate> lifecycle::table` naming per
the repository's approved layout; command spelling is not the contract.

**Acceptance:** the 63-pair table test passes and encodes
[the matrix](03-code-contracts-lifecycle.md) §Contract 1.3 exactly; no
policy parameter appears.  
**Failure/blocker:** a needed state or edge outside the table is a design
conflict — stop, record, escalate.

### Step 2 — transition engine

Target: `lifecycle` engine (`try_transition`, `try_transition_from_irq`,
cell type, transition record).

Work: implement per Contracts 1.4–1.7 on the P3 lock primitive seam, with
trace-semantics emission marks. Add negative tests for every error variant
and a property test that random legal/illegal event sequences leave the
cell table-reachable.

**Acceptance:** engine-only mutation holds (no other code writes the cell);
leaf-lock rule respected (no lock acquisition inside the critical section);
IRQ entry performs no allocation.  
**Failure/blocker:** needing a second writer or a nested lock is a design
violation — stop and re-examine the calling path's design.

### Step 3 — admission gate and exit boundary

Target: `admission` module (`admit_for_entry`, `complete_exit`,
`EntryPermit`, `ExitOutcome`, `PostExitControl`, `SchedulerMode`).

Work: implement per Contracts 2.1–2.2 and the supporting data contracts,
attaching `current_vcpu` slot storage to the P3 per-CPU reserved area.
Host tests simulate gate/exit interleavings including lost races
(`NotDispatchable` requeue path) and slot-agreement checks.

**Acceptance:** all-or-nothing gate semantics proven by tests; no path
mutates both cell and slot outside these functions.  
**Failure/blocker:** a caller that must gate while holding a queue lock
indicates the W05 discipline is being violated — stop and align with
[P7-W05](../p7-w05-shared-mn-multivm/README.md) design, not with a lock
nesting "fix".

### Step 4 — activation and bypass register

Target: `activate_scheduler`, `SchedulerMode`, static `BYPASS_PATHS`.

Work: implement per Contract 2.3; enumerate the concrete E1/E2 instances
this stage's bring-up actually contains (expected: the P4 first-guest
bring-up path under E2, pre-scheduler boot paths under E1; the list starts
minimal and grows only by review). Wire activation into the boot sequence
seam documented by the P1/P3 plans.

**Acceptance:** every registered bypass is a named code path with a class;
post-activation use of E1/E2 paths is detectable (audit hook or assertion).  
**Failure/blocker:** a bring-up path that cannot fit E1–E3 is a boundary
conflict with W01 — escalate per its §5 procedure.

### Step 5 — invariant checker

Target: `audit` module (`check_scheduler_invariants`, snapshot types).

Work: implement per Contract 3.1 with host property tests (random legal
sequences → all-Pass) and a QEMU assertion hook stub callable at safe
points (the hook's call sites multiply in W04/W05 work, not here).

**Acceptance:** checker is pure, lock-free, and detects seeded violations
for each of INV-1…INV-5 in tests.  
**Failure/blocker:** an invariant needing runtime mutation to check is
mis-specified — return to
[architecture and state](02-architecture-and-state.md) §5, not to a
checker-side workaround.

### Step 6 — integration seams

Target: glue points only.

Work: connect (a) the W03 eligibility predicate interface (a function
parameter until W03 lands — a documented placeholder that rejects
everything is acceptable for compilation but must be replaced before any
QEMU evidence), (b) the P4 entry/exit mechanism calls, (c) the W05 control
loop call sites (stubbed loop is acceptable at this step), (d) trace
emission through the P0 namespace.

**Acceptance:** seams compile against the assumed contracts; every
mismatch found is recorded per the W01 procedure rather than papered over.  
**Failure/blocker:** a seam that cannot bind without changing a predecessor
contract stops the step as blocked.

### Step 7 — records and closure

Work: record decisions/deviations in
`../p7-w02-scheduler-admission-lifecycle-record.md`; run the validation
matrix in [validation and handoff](05-validation-and-handoff.md) and record
evidence in
`../../verification/p7-w02-scheduler-admission-lifecycle-verification.md`;
confirm the handoff checklist. Completion is claimed only in the
verification record, only for what ran.

## 3. Evidence rules

- Planned, run, blocked, and failed evidence remain distinct states in the
  verification record; a skipped test is "not run" with a reason, never
  silently dropped.
- Host table/property evidence satisfies P7-V03 review requirements only
  together with the target-side evidence named in the matrix; host passes
  never substitute for QEMU rows.
- Any new `unsafe`, dependency, or public-surface change discovered during
  implementation is reported per the repository AGENTS.md reporting rules,
  with the expected answer "none" for this package.
