# P7-W08 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
preflight set, and the supporting file for its step; and it re-inspects the
current tracked tree (`git ls-files`). All inputs are assumed contracts
([01-smp-scheduler-architecture.md](01-smp-scheduler-architecture.md) §2):
confirm the W01 reconciliation record
(`../p7-w01-entry-contract-reconciliation-record.md`, when it exists) has not
flagged them as blocked, and that no owner published a contradicting frozen
contract since.

Stop and obtain direction instead of guessing when:

- the P3-W07 notification surface cannot express scheduler-class targeted
  delivery with defined offline-target outcomes — PC-3 gap; record and stop;
- the P6-W05 timer surface cannot express "earliest of folded obligations" —
  PC-5 gap; idle cannot be made wake-complete; stop;
- the arch wait seam does not exist behind the P1/P3 boundaries — do not
  implement WFE in Core; stop (layering violation);
- implementing SMP behavior appears to require hotplug, power management, or
  work stealing — scope violation (parent README exclusions); or
- the bounded re-check bounds cannot absorb observed storm behavior without
  becoming polling — design defect; stop and escalate.

## 2. Ordered implementation steps

### Step 1 — reconcile assumed contracts against the delivered P7 baseline

Target: implementation record
(`../p7-w08-smp-reschedule-idle-record.md`, created in this step).

Work: read the W03/W05/W06/W07 sibling designs and the P3/P6 records actually
present; record per-contract status for L-1..W7-3 as satisfied, changed, or
absent/blocked. A changed contract that invalidates a parent-README decision
stops this design (Architecture Change Request), not a local adaptation.

Acceptance: per-contract status table with deltas and blockers named.

Failure/blocker: contradicting authority — stop per §1.

Evidence: implementation record.

### Step 2 — implement the per-pCPU scheduling loop and isolation rules

Target: scheduler per-pCPU context consuming PC-1's reserved capacity; the
single decision entry of [01](01-smp-scheduler-architecture.md) §3.

Work: build the per-pCPU context (current registration, intent flag, idle
state, deadline-obligation view); implement the decision entry as the only
dispatch path; enforce the mutability rules (§3) and document each shared
structure. Do not implement `select_next` policy — call the S-2/W05 surface.

Suggested observation: host-side unit tests with fake pCPU sets exercising
concurrent decisions (when the P0-W08 host testing baseline exists).

Acceptance: exactly one decision entry exists; no global current-CPU state;
two pCPUs can run distinct vCPUs in tests without shared mutable state beyond
declared structures.

Failure/blocker: PC-1 capacity absent — stop per §1.

Evidence: implementation record.

### Step 3 — implement the reconsideration protocol

Target: R-1..R-4 per
[02-code-contracts-reschedule.md](02-code-contracts-reschedule.md); register
R-2 as the scheduler-class reception hook on the P3-W07 path.

Work: implement the intent flag, requester ordering (effect visible before
signal), handler clear-then-decide bounded loop, self-target fast path, and
the R-3 targeting predicate. Verify coalescing: N concurrent requests produce
one delivery per flag transition.

Acceptance: R-1/R-2 contract tests pass, including spurious-signal and
refused-target cases; storm tests show bounded handler invocations.

Failure/blocker: PC-3/PC-4 surface gaps — stop per §1.

Evidence: implementation record; unit-test inventory.

### Step 4 — implement designed idle

Target: I-1..I-5 per [03-code-contracts-idle.md](03-code-contracts-idle.md);
consume (never implement) the arch wait seam.

Work: implement the two-phase idle entry, the closed wake-source list, the
deadline fold through PC-5, and exit ordering (clear intent before decision).
Verify no Core-side polling loop exists anywhere around the wait seam.

Acceptance: an idle pCPU in tests consumes no scheduling iterations until a
listed wake source fires; request-vs-idle-entry races lose no request; idle
refusal on deadline-fold failure behaves per I-1.

Failure/blocker: absence of the arch seam or PC-5 fold capability — stop
per §1.

Evidence: implementation record.

### Step 5 — integration pass and evidence

Target: verification record
(`../../verification/p7-w08-smp-reschedule-idle-verification.md`).

Work: run the validation matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md) as far as
prerequisites exist. SMP and idle QEMU rows require the P3 SMP baseline and
P1/P4 QEMU baselines; without them record not run with the owning stage
named — never substitute single-core approximations.

Acceptance: every matrix row has a passed/failed/blocked/not-run status with
commands, environment, timestamps, and reasons.

Failure/blocker: a failed row is recorded with diagnosis; no weakening of a
passing condition.

## 3. Evidence destinations

- Implementation decisions, deltas, changed files:
  `../p7-w08-smp-reschedule-idle-record.md`.
- Command output, run/not-run status:
  `../../verification/p7-w08-smp-reschedule-idle-verification.md`.
- Neither file may claim W08 complete; completion evidence lives only in the
  verification record, only for what actually ran.

## 4. Ordering and review constraints

- Locks and ordering: only P3-W06-sanctioned primitives; the intent flag and
  idle intent use the release/acquire pairings stated in the contracts; no
  new lock order beyond P3-W06's baseline.
- IRQ context: R-2 and all wake handlers bounded, allocation-free, no
  blocking; the R-2 loop bound is a named constant with storm justification.
- Layering: no WFE, GIC, or timer register access in Core; the wait seam and
  transport stay behind the P1/P3/P6 boundaries; no board-name conditionals.
- No new `unsafe` is authorized by W08 logic; touched `unsafe` stays within
  existing audited boundaries with `SAFETY` commentary intact.
- New `unsafe`, ABI/public API, or dependency changes: none authorized; their
  appearance is a review failure.
